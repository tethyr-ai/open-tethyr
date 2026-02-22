//! Main cache server implementation

use crate::cache::{
    CacheCoordinator, CacheCoordinatorConfig, CacheError, CacheStats, RateLimitConfig, RateLimiter,
};
use crate::policy::enforcement::{DomainPolicy, PolicyEngine, PolicyViolation};
use crate::server::handlers::{handle_discover, handle_health, handle_metrics};
use crate::server::middleware::rate_limit_middleware;
use axum::{extract::FromRef, middleware, routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tower_http::trace::TraceLayer;
use tracing::info;

/// Cache server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind the server to (e.g., "0.0.0.0:8080")
    pub bind_address: String,
    /// Home domain for this cache server
    pub domain: String,
    /// Cache coordinator configuration
    pub cache_config: CacheCoordinatorConfig,
    /// Domain policy for access control
    pub domain_policy: DomainPolicy,
    /// Rate limiting configuration
    pub rate_limit_config: RateLimitConfig,
    /// Request timeout duration
    pub request_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".to_string(),
            domain: "localhost".to_string(),
            cache_config: CacheCoordinatorConfig::default(),
            domain_policy: DomainPolicy::default(),
            rate_limit_config: RateLimitConfig::default(),
            request_timeout: Duration::from_secs(30),
        }
    }
}

/// Cache server errors
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Server startup failed: {0}")]
    StartupFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Cache error: {0}")]
    CacheError(#[from] CacheError),

    #[error("Policy violation: {0}")]
    PolicyViolation(#[from] PolicyViolation),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid bind address: {0}")]
    InvalidBindAddress(String),
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub coordinator: Arc<CacheCoordinator>,
    pub policy_engine: Arc<PolicyEngine>,
    pub metrics: Arc<CacheMetrics>,
    pub rate_limiter: Arc<RateLimiter>,
}

impl FromRef<AppState> for Arc<CacheCoordinator> {
    fn from_ref(state: &AppState) -> Self {
        state.coordinator.clone()
    }
}

impl FromRef<AppState> for Arc<PolicyEngine> {
    fn from_ref(state: &AppState) -> Self {
        state.policy_engine.clone()
    }
}

impl FromRef<AppState> for Arc<CacheMetrics> {
    fn from_ref(state: &AppState) -> Self {
        state.metrics.clone()
    }
}

impl FromRef<AppState> for Arc<RateLimiter> {
    fn from_ref(state: &AppState) -> Self {
        state.rate_limiter.clone()
    }
}

/// Cache metrics for observability
#[derive(Debug)]
pub struct CacheMetrics {
    /// Cache statistics
    cache_stats: Arc<CacheStats>,
    /// Active connections counter
    active_connections: std::sync::atomic::AtomicU32,
    /// Request duration histogram
    request_duration: Arc<std::sync::Mutex<crate::cache::SimpleHistogram>>,
}

impl CacheMetrics {
    /// Create new cache metrics
    pub fn new(cache_stats: Arc<CacheStats>) -> Self {
        Self {
            cache_stats,
            active_connections: std::sync::atomic::AtomicU32::new(0),
            request_duration: Arc::new(std::sync::Mutex::new(crate::cache::SimpleHistogram::new())),
        }
    }

    /// Record a cache hit
    pub fn record_hit(&self) {
        self.cache_stats.record_hit();
    }

    /// Record a cache miss
    pub fn record_miss(&self) {
        self.cache_stats.record_miss();
    }

    /// Record request duration
    pub fn record_request_duration(&self, duration: Duration) {
        if let Ok(mut histogram) = self.request_duration.lock() {
            histogram.record(duration);
        }
    }

    /// Increment active connections
    pub fn increment_connections(&self) {
        self.active_connections
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Decrement active connections
    pub fn decrement_connections(&self) {
        self.active_connections
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> Arc<CacheStats> {
        self.cache_stats.clone()
    }

    /// Get active connections count
    pub fn active_connections(&self) -> u32 {
        self.active_connections
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get request duration histogram snapshot
    pub fn request_duration_snapshot(&self) -> Option<crate::cache::SimpleHistogram> {
        self.request_duration
            .lock()
            .ok()
            .map(|h| crate::cache::SimpleHistogram {
                buckets: h.buckets().to_vec(),
                total_count: h.total_count(),
                sum: h.sum(),
            })
    }
}

/// Main cache server
pub struct CacheServer {
    config: ServerConfig,
    coordinator: Arc<CacheCoordinator>,
    policy_engine: Arc<PolicyEngine>,
    metrics: Arc<CacheMetrics>,
    rate_limiter: Arc<RateLimiter>,
}

impl CacheServer {
    /// Create a new cache server
    pub async fn new(config: ServerConfig) -> Result<Self, ServerError> {
        info!("Initializing cache server for domain: {}", config.domain);

        // Create cache coordinator
        let coordinator = Arc::new(
            CacheCoordinator::with_config(config.cache_config.clone())
                .map_err(|e| ServerError::ConfigError(e.to_string()))?,
        );

        // Initialize coordinator with home domain
        coordinator
            .initialize(&config.domain)
            .await
            .map_err(|e| ServerError::ConfigError(e.to_string()))?;

        // Create policy engine
        let policy_engine = Arc::new(PolicyEngine::new(config.domain_policy.clone()));

        // Create rate limiter
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_config.clone()));

        // Create metrics
        let metrics = Arc::new(CacheMetrics::new(coordinator.stats()));

        info!("Cache server initialized successfully");

        Ok(Self {
            config,
            coordinator,
            policy_engine,
            metrics,
            rate_limiter,
        })
    }

    /// Start the cache server
    pub async fn start(&self) -> Result<(), ServerError> {
        let addr: SocketAddr = self.config.bind_address.parse().map_err(|e| {
            ServerError::InvalidBindAddress(format!("{}: {}", e, self.config.bind_address))
        })?;

        info!("Starting cache server on {}", addr);

        // Build application state
        let state = AppState {
            coordinator: self.coordinator.clone(),
            policy_engine: self.policy_engine.clone(),
            metrics: self.metrics.clone(),
            rate_limiter: self.rate_limiter.clone(),
        };

        // Build router with routes and middleware
        let app = Router::new()
            .route("/discover/:domain", get(handle_discover))
            .route("/health", get(handle_health))
            .route("/metrics", get(handle_metrics))
            .layer(TraceLayer::new_for_http())
            .layer(middleware::from_fn_with_state(
                state.rate_limiter.clone(),
                rate_limit_middleware,
            ))
            .with_state(state);

        // Create TCP listener
        let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
            ServerError::StartupFailed(format!("Failed to bind to {}: {}", addr, e))
        })?;

        info!("Cache server listening on {}", addr);

        // Start serving
        axum::serve(listener, app)
            .await
            .map_err(|e| ServerError::StartupFailed(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Get server configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Get cache coordinator
    pub fn coordinator(&self) -> Arc<CacheCoordinator> {
        self.coordinator.clone()
    }

    /// Get policy engine
    pub fn policy_engine(&self) -> Arc<PolicyEngine> {
        self.policy_engine.clone()
    }

    /// Get metrics
    pub fn metrics(&self) -> Arc<CacheMetrics> {
        self.metrics.clone()
    }

    /// Get rate limiter
    pub fn rate_limiter(&self) -> Arc<RateLimiter> {
        self.rate_limiter.clone()
    }
}
