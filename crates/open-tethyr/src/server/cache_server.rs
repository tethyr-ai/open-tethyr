//! Main Cache Server

use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;

use super::handlers;
use super::middleware::CorrelationIdLayer;
use super::policy::PolicyEngine;
use crate::cache::coordinator::CacheCoordinator;
use crate::cache::stats::CacheStats;
use crate::config::ServerConfig;
use crate::error::ServerError;

/// Shared application state
pub struct AppState {
    pub coordinator: CacheCoordinator,
    pub policy: PolicyEngine,
    pub stats: CacheStats,
}

/// Main cache server
pub struct CacheServer {
    config: ServerConfig,
    state: Arc<AppState>,
}

impl CacheServer {
    pub async fn new(config: ServerConfig) -> Result<Self, ServerError> {
        let policy = PolicyEngine::new(
            config.policy.domain_locking,
            config.policy.home_domain.clone(),
            config.policy.allowlist.clone(),
        );

        let coordinator = CacheCoordinator::new(
            config.cache.max_entries,
            config.cache.default_ttl,
            None, // Root cache URL discovered via DNS at runtime
        )
        .map_err(|e| ServerError::StartupFailed(e.to_string()))?;

        let state = Arc::new(AppState {
            coordinator,
            policy,
            stats: CacheStats::new(),
        });

        Ok(Self { config, state })
    }

    /// Build the router
    pub fn build_routes(state: Arc<AppState>) -> Router {
        Router::new()
            .route("/discover/{domain}", get(handlers::handle_discover))
            .route("/health", get(handlers::handle_health))
            .route("/metrics", get(handlers::handle_metrics))
            .layer(CorrelationIdLayer)
            .with_state(state)
    }

    /// Start the cache server
    pub async fn start(&self) -> Result<(), ServerError> {
        let router = Self::build_routes(self.state.clone());
        let addr: SocketAddr = format!("0.0.0.0:{}", self.config.port)
            .parse()
            .map_err(|e| ServerError::StartupFailed(format!("Invalid address: {}", e)))?;

        tracing::info!("Cache server starting on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| ServerError::StartupFailed(e.to_string()))?;

        axum::serve(listener, router)
            .await
            .map_err(|e| ServerError::StartupFailed(e.to_string()))?;

        Ok(())
    }
}
