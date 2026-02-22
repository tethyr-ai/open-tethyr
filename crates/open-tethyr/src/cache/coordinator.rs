//! Cache Coordination
//!
//! Hierarchical cache coordination with DNS-based discovery and fallback chains.

use crate::ax::AgentExchangeDocument;
use crate::cache::{CacheConfig, MemoryCache};
use crate::dns::{DnsDiscovery, DnsError};
use crate::http::{AxHttpClient, HttpError};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Cache coordination errors
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("DNS discovery failed: {0}")]
    DnsError(#[from] DnsError),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] HttpError),

    #[error("Circular dependency detected in cache configuration: {0}")]
    CircularDependency(String),

    #[error("Cache configuration validation failed: {0}")]
    ConfigurationError(String),

    #[error("All cache sources failed for domain: {0}")]
    AllSourcesFailed(String),

    #[error("Cache coordinator not initialized")]
    NotInitialized,
}

/// Cache coordinator configuration
#[derive(Debug, Clone)]
pub struct CacheCoordinatorConfig {
    /// Local cache configuration
    pub cache_config: CacheConfig,
    /// HTTP client timeout
    pub http_timeout: Duration,
    /// Whether to use DNS-based cache discovery
    pub enable_dns_discovery: bool,
    /// Manual cache endpoints (domain -> cache_url)
    pub manual_cache_endpoints: HashMap<String, String>,
    /// Root cache URL (if not using DNS discovery)
    pub root_cache_url: Option<String>,
    /// Maximum depth for cache fallback chain
    pub max_fallback_depth: usize,
}

impl Default for CacheCoordinatorConfig {
    fn default() -> Self {
        Self {
            cache_config: CacheConfig::default(),
            http_timeout: Duration::from_secs(30),
            enable_dns_discovery: true,
            manual_cache_endpoints: HashMap::new(),
            root_cache_url: None,
            max_fallback_depth: 3,
        }
    }
}

/// Cache coordinator for hierarchical caching with DNS-based discovery
pub struct CacheCoordinator {
    /// Local in-memory cache
    local_cache: Arc<MemoryCache>,
    /// DNS discovery service
    dns_discovery: Arc<DnsDiscovery>,
    /// HTTP client for fetching from upstream caches and direct endpoints
    http_client: Arc<AxHttpClient>,
    /// Configuration
    config: CacheCoordinatorConfig,
    /// Discovered cache endpoints (domain -> cache_url)
    discovered_caches: Arc<std::sync::RwLock<HashMap<String, String>>>,
    /// Root cache URL (discovered or configured)
    root_cache_url: Arc<std::sync::RwLock<Option<String>>>,
}

impl CacheCoordinator {
    /// Create a new cache coordinator with default configuration
    pub fn new() -> Result<Self, CacheError> {
        Self::with_config(CacheCoordinatorConfig::default())
    }

    /// Create a new cache coordinator with custom configuration
    pub fn with_config(config: CacheCoordinatorConfig) -> Result<Self, CacheError> {
        let local_cache = Arc::new(MemoryCache::new(config.cache_config.clone()));
        let dns_discovery = Arc::new(
            DnsDiscovery::new().map_err(|e| CacheError::ConfigurationError(e.to_string()))?,
        );
        let http_client = Arc::new(
            AxHttpClient::with_timeout(config.http_timeout)
                .map_err(|e| CacheError::ConfigurationError(e.to_string()))?,
        );

        Ok(Self {
            local_cache,
            dns_discovery,
            http_client,
            config,
            discovered_caches: Arc::new(std::sync::RwLock::new(HashMap::new())),
            root_cache_url: Arc::new(std::sync::RwLock::new(None)),
        })
    }

    /// Initialize the cache coordinator by discovering root cache
    pub async fn initialize(&self, home_domain: &str) -> Result<(), CacheError> {
        info!("Initializing cache coordinator for domain: {}", home_domain);

        // Discover root cache if DNS discovery is enabled
        if self.config.enable_dns_discovery {
            if let Ok(Some(root_cache)) = self.dns_discovery.discover_root_cache(home_domain).await
            {
                info!("Discovered root cache: {}", root_cache);
                if let Ok(mut root_url) = self.root_cache_url.write() {
                    *root_url = Some(root_cache);
                }
            } else if let Some(ref manual_root) = self.config.root_cache_url {
                info!("Using configured root cache: {}", manual_root);
                if let Ok(mut root_url) = self.root_cache_url.write() {
                    *root_url = Some(manual_root.clone());
                }
            }
        }

        // Validate configuration for circular dependencies
        self.validate_cache_configuration().await?;

        info!("Cache coordinator initialized successfully");
        Ok(())
    }

    /// Discover agents for a domain using hierarchical cache fallback
    pub async fn discover(&self, domain: &str) -> Result<AgentExchangeDocument, CacheError> {
        debug!("Starting discovery for domain: {}", domain);

        // 1. Check local cache first
        if let Some(document) = self.local_cache.get(domain).await {
            debug!("Cache hit in local cache for domain: {}", domain);
            return Ok(document);
        }

        debug!("Cache miss in local cache for domain: {}", domain);

        // 2. Try domain-specific cache (if configured or discovered)
        if let Some(document) = self.try_domain_cache(domain).await {
            debug!("Found document in domain-specific cache for: {}", domain);
            // Cache locally for future requests
            self.cache_locally(domain, &document).await;
            return Ok(document);
        }

        // 3. Try root cache (if available)
        if let Some(document) = self.try_root_cache(domain).await {
            debug!("Found document in root cache for: {}", domain);
            // Cache locally for future requests
            self.cache_locally(domain, &document).await;
            return Ok(document);
        }

        // 4. Fall back to direct fetch from domain
        match self.fetch_direct(domain).await {
            Ok(document) => {
                debug!("Successfully fetched directly from domain: {}", domain);
                // Cache locally for future requests
                self.cache_locally(domain, &document).await;
                Ok(document)
            }
            Err(e) => {
                error!("All discovery methods failed for domain {}: {}", domain, e);
                Err(CacheError::AllSourcesFailed(format!(
                    "Domain: {}, Last error: {}",
                    domain, e
                )))
            }
        }
    }

    /// Try to fetch from domain-specific cache
    async fn try_domain_cache(&self, domain: &str) -> Option<AgentExchangeDocument> {
        // Check if we have a manual cache endpoint configured
        if let Some(cache_url) = self.config.manual_cache_endpoints.get(domain) {
            debug!("Using manual cache endpoint for {}: {}", domain, cache_url);
            return self.fetch_from_cache(cache_url, domain).await.ok();
        }

        // Check if we've already discovered a cache for this domain
        let discovered_cache_url = {
            if let Ok(discovered) = self.discovered_caches.read() {
                discovered.get(domain).cloned()
            } else {
                None
            }
        };

        if let Some(cache_url) = discovered_cache_url {
            debug!(
                "Using discovered cache endpoint for {}: {}",
                domain, cache_url
            );
            return self.fetch_from_cache(&cache_url, domain).await.ok();
        }

        // Try to discover cache endpoint via DNS
        if self.config.enable_dns_discovery {
            if let Ok(Some(cache_url)) = self.dns_discovery.discover_cache(domain).await {
                debug!(
                    "Discovered new cache endpoint for {}: {}",
                    domain, cache_url
                );

                // Store discovered cache endpoint
                if let Ok(mut discovered) = self.discovered_caches.write() {
                    discovered.insert(domain.to_string(), cache_url.clone());
                }

                return self.fetch_from_cache(&cache_url, domain).await.ok();
            }
        }

        None
    }

    /// Try to fetch from root cache
    async fn try_root_cache(&self, domain: &str) -> Option<AgentExchangeDocument> {
        let root_cache_url = {
            if let Ok(root_cache) = self.root_cache_url.read() {
                root_cache.clone()
            } else {
                None
            }
        };

        if let Some(cache_url) = root_cache_url {
            debug!("Trying root cache for {}: {}", domain, cache_url);
            return self.fetch_from_cache(&cache_url, domain).await.ok();
        }
        None
    }

    /// Fetch from upstream cache
    async fn fetch_from_cache(
        &self,
        cache_url: &str,
        domain: &str,
    ) -> Result<AgentExchangeDocument, CacheError> {
        debug!("Fetching from cache {} for domain: {}", cache_url, domain);

        match self.http_client.fetch_from_cache(cache_url, domain).await {
            Ok(document) => {
                debug!("Successfully fetched from cache: {}", cache_url);
                Ok(document)
            }
            Err(e) => {
                warn!("Failed to fetch from cache {}: {}", cache_url, e);
                Err(CacheError::HttpError(e))
            }
        }
    }

    /// Fetch directly from domain's AX endpoint
    async fn fetch_direct(&self, domain: &str) -> Result<AgentExchangeDocument, CacheError> {
        debug!("Fetching directly from domain: {}", domain);

        match self.http_client.fetch_ax_record(domain).await {
            Ok(document) => {
                debug!("Successfully fetched directly from: {}", domain);
                Ok(document)
            }
            Err(e) => {
                warn!("Failed to fetch directly from {}: {}", domain, e);
                Err(CacheError::HttpError(e))
            }
        }
    }

    /// Cache document locally
    pub async fn cache_locally(&self, domain: &str, document: &AgentExchangeDocument) {
        let ttl = self.config.cache_config.default_ttl;
        self.local_cache
            .put(domain, document.clone(), ttl, None)
            .await;
        debug!("Cached document locally for domain: {}", domain);
    }

    /// Validate cache configuration for circular dependencies
    pub async fn validate_cache_configuration(&self) -> Result<(), CacheError> {
        debug!("Validating cache configuration for circular dependencies");

        // Build dependency graph
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // Add manual cache endpoints
        for (domain, cache_url) in &self.config.manual_cache_endpoints {
            // Extract domain from cache URL for dependency tracking
            let url = url::Url::parse(cache_url).map_err(|e| {
                CacheError::ConfigurationError(format!(
                    "Invalid cache URL '{}' for domain '{}': {}",
                    cache_url, domain, e
                ))
            })?;

            let cache_domain = url.host_str().ok_or_else(|| {
                CacheError::ConfigurationError(format!(
                    "Cache URL '{}' for domain '{}' has no host",
                    cache_url, domain
                ))
            })?;

            graph
                .entry(domain.clone())
                .or_default()
                .push(cache_domain.to_string());
        }

        // Add root cache dependency if configured
        if let Some(ref root_cache_url) = self.config.root_cache_url {
            let url = url::Url::parse(root_cache_url).map_err(|e| {
                CacheError::ConfigurationError(format!(
                    "Invalid root cache URL '{}': {}",
                    root_cache_url, e
                ))
            })?;

            let root_domain = url.host_str().ok_or_else(|| {
                CacheError::ConfigurationError(format!(
                    "Root cache URL '{}' has no host",
                    root_cache_url
                ))
            })?;

            // All domains depend on root cache
            let domains: Vec<String> = graph.keys().cloned().collect();
            for domain in domains {
                graph
                    .entry(domain)
                    .or_default()
                    .push(root_domain.to_string());
            }
        }

        // Check for circular dependencies using DFS
        for domain in graph.keys() {
            if self.has_circular_dependency(&graph, domain, &mut HashSet::new())? {
                return Err(CacheError::CircularDependency(format!(
                    "Circular dependency detected starting from domain: {}",
                    domain
                )));
            }
        }

        debug!("Cache configuration validation passed");
        Ok(())
    }

    /// Check for circular dependencies using depth-first search
    fn has_circular_dependency(
        &self,
        graph: &HashMap<String, Vec<String>>,
        current: &str,
        visited: &mut HashSet<String>,
    ) -> Result<bool, CacheError> {
        if visited.contains(current) {
            return Ok(true); // Circular dependency found
        }

        if visited.len() > self.config.max_fallback_depth {
            return Err(CacheError::ConfigurationError(format!(
                "Maximum fallback depth ({}) exceeded",
                self.config.max_fallback_depth
            )));
        }

        visited.insert(current.to_string());

        if let Some(dependencies) = graph.get(current) {
            for dependency in dependencies {
                if self.has_circular_dependency(graph, dependency, visited)? {
                    return Ok(true);
                }
            }
        }

        visited.remove(current);
        Ok(false)
    }

    /// Get local cache reference
    pub fn local_cache(&self) -> Arc<MemoryCache> {
        self.local_cache.clone()
    }

    /// Get cache statistics
    pub fn stats(&self) -> Arc<crate::cache::CacheStats> {
        self.local_cache.stats()
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        self.local_cache.clear().await;
        if let Ok(mut discovered) = self.discovered_caches.write() {
            discovered.clear();
        }
        debug!("Cleared all caches");
    }

    /// Invalidate cache for specific domain
    pub async fn invalidate_domain(&self, domain: &str) -> bool {
        self.local_cache.invalidate(domain).await
    }
}

impl Default for CacheCoordinator {
    fn default() -> Self {
        Self::new().expect("Failed to create default cache coordinator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ax::{Agent, AgentExchangeRecord};

    fn create_test_document() -> AgentExchangeDocument {
        AgentExchangeDocument {
            records: vec![AgentExchangeRecord {
                record_type: "AX".to_string(),
                version: "1.0".to_string(),
                agent: Agent {
                    name: "Test Agent".to_string(),
                    description: "Test Description".to_string(),
                    provider: "Test Provider".to_string(),
                },
                endpoints: vec![],
                capabilities: None,
                schema: None,
                limits: None,
                security: None,
                extensions: None,
            }],
        }
    }

    #[tokio::test]
    async fn test_cache_coordinator_creation() {
        let coordinator = CacheCoordinator::new();
        assert!(coordinator.is_ok());
    }

    #[tokio::test]
    async fn test_local_cache_integration() {
        let coordinator = CacheCoordinator::new().unwrap();
        let domain = "test.com";
        let document = create_test_document();

        // Cache document locally
        coordinator.cache_locally(domain, &document).await;

        // Should find it in local cache
        let cached = coordinator.local_cache.get(domain).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().records.len(), document.records.len());
    }

    #[tokio::test]
    async fn test_circular_dependency_detection() {
        let mut config = CacheCoordinatorConfig::default();

        // Create circular dependency: domain1 -> cache1, cache1 -> domain1
        config.manual_cache_endpoints.insert(
            "domain1.com".to_string(),
            "https://cache1.com/ax".to_string(),
        );
        config.manual_cache_endpoints.insert(
            "cache1.com".to_string(),
            "https://domain1.com/ax".to_string(),
        );

        let coordinator = CacheCoordinator::with_config(config).unwrap();
        let result = coordinator.validate_cache_configuration().await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CacheError::CircularDependency(_)
        ));
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let coordinator = CacheCoordinator::new().unwrap();
        let domain = "test.com";
        let document = create_test_document();

        // Cache document
        coordinator.cache_locally(domain, &document).await;
        assert!(coordinator.local_cache.get(domain).await.is_some());

        // Invalidate
        let removed = coordinator.invalidate_domain(domain).await;
        assert!(removed);
        assert!(coordinator.local_cache.get(domain).await.is_none());
    }

    #[tokio::test]
    async fn test_clear_all_caches() {
        let coordinator = CacheCoordinator::new().unwrap();
        let document = create_test_document();

        // Cache multiple documents
        coordinator.cache_locally("domain1.com", &document).await;
        coordinator.cache_locally("domain2.com", &document).await;

        assert!(coordinator.local_cache.get("domain1.com").await.is_some());
        assert!(coordinator.local_cache.get("domain2.com").await.is_some());

        // Clear all
        coordinator.clear_all().await;

        assert!(coordinator.local_cache.get("domain1.com").await.is_none());
        assert!(coordinator.local_cache.get("domain2.com").await.is_none());
    }
}
