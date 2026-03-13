//! Cache Coordination for Hierarchical Caching

use crate::cache::memory::{CacheEntry, MemoryCache};
use crate::error::CacheError;
use crate::http::AxHttpClient;
use std::time::{Duration, Instant};

/// Cache coordinator with fallback chain
pub struct CacheCoordinator {
    local_cache: MemoryCache,
    root_cache_url: Option<String>,
    http_client: AxHttpClient,
    default_ttl: Duration,
}

impl CacheCoordinator {
    pub fn new(
        max_entries: usize,
        default_ttl_secs: u64,
        root_cache_url: Option<String>,
    ) -> Result<Self, CacheError> {
        // Detect circular dependencies (simple self-reference check)
        if let Some(ref url) = root_cache_url {
            if url.is_empty() {
                return Err(CacheError::CircularDependency("Empty root cache URL".into()));
            }
        }

        let http_client = AxHttpClient::new(Some(30))
            .map_err(|e| CacheError::OperationFailed(e.to_string()))?;

        Ok(Self {
            local_cache: MemoryCache::new(max_entries),
            root_cache_url,
            http_client,
            default_ttl: Duration::from_secs(default_ttl_secs),
        })
    }

    /// Discover agents for a domain using fallback chain:
    /// local cache -> root cache -> direct fetch
    pub async fn discover(&self, domain: &str) -> Result<String, CacheError> {
        // 1. Check local cache
        if let Some(entry) = self.local_cache.get(domain) {
            return Ok(entry.data);
        }

        // 2. Try root cache if configured
        if let Some(ref root_url) = self.root_cache_url {
            match self.http_client.fetch_from_cache(root_url, domain).await {
                Ok(doc) => {
                    let json = serde_json::to_string(&doc)
                        .map_err(|e| CacheError::OperationFailed(e.to_string()))?;
                    let _ = self.local_cache.put(domain.to_string(), CacheEntry {
                        data: json.clone(),
                        created_at: Instant::now(),
                        ttl: self.default_ttl,
                        no_cache: false,
                    });
                    return Ok(json);
                }
                Err(e) => {
                    tracing::warn!("Root cache fetch failed for {}: {}, falling back to direct", domain, e);
                }
            }
        }

        // 3. Direct fetch
        let doc = self.http_client.fetch_ax_record(domain).await
            .map_err(|e| CacheError::NotFound(format!("{}: {}", domain, e)))?;
        let json = serde_json::to_string(&doc)
            .map_err(|e| CacheError::OperationFailed(e.to_string()))?;
        let _ = self.local_cache.put(domain.to_string(), CacheEntry {
            data: json.clone(),
            created_at: Instant::now(),
            ttl: self.default_ttl,
            no_cache: false,
        });
        Ok(json)
    }

    pub fn local_cache(&self) -> &MemoryCache {
        &self.local_cache
    }
}
