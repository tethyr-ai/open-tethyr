//! In-Memory Cache Implementation
//!
//! Provides in-memory caching with LRU eviction, TTL expiration, and Cache-Control header support.

use crate::ax::AgentExchangeDocument;
use crate::cache::{CacheConfig, CacheControl, CacheEntry, CacheStats};
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

/// In-memory cache with LRU eviction and TTL support
#[derive(Debug)]
pub struct MemoryCache {
    /// Cache entries storage
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// LRU tracker for eviction policy
    lru: Arc<Mutex<LruCache<String, ()>>>,
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics
    stats: Arc<CacheStats>,
}

impl MemoryCache {
    /// Create a new memory cache with the given configuration
    pub fn new(config: CacheConfig) -> Self {
        let max_entries = NonZeroUsize::new(config.max_entries.max(1))
            .unwrap_or(NonZeroUsize::new(1000).unwrap());

        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            lru: Arc::new(Mutex::new(LruCache::new(max_entries))),
            config,
            stats: Arc::new(CacheStats::new()),
        }
    }

    /// Create a new memory cache with default configuration
    pub fn with_default_config() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Get a cached document by domain
    pub async fn get(&self, domain: &str) -> Option<AgentExchangeDocument> {
        // First check if entry exists and is not expired
        let (should_invalidate, document) = {
            let entries = self.entries.read().ok()?;
            if let Some(entry) = entries.get(domain) {
                let is_expired = entry.is_expired();
                let should_invalidate = if let Some(cache_control) = &entry.cache_control {
                    !cache_control.allows_caching()
                } else {
                    false
                };

                if is_expired || should_invalidate {
                    (true, None)
                } else {
                    (false, Some(entry.document.clone()))
                }
            } else {
                (false, None)
            }
        };

        if should_invalidate {
            self.invalidate(domain).await;
            self.stats.record_miss();
            return None;
        }

        if let Some(document) = document {
            // Update LRU order
            if let Ok(mut lru) = self.lru.lock() {
                lru.get(domain);
            }

            self.stats.record_hit();
            return Some(document);
        }

        self.stats.record_miss();
        None
    }

    /// Store a document in the cache
    pub async fn put(
        &self,
        domain: &str,
        document: AgentExchangeDocument,
        ttl: Duration,
        cache_control_header: Option<&str>,
    ) {
        let cache_control = cache_control_header.map(CacheControl::parse);

        // Check if caching is allowed by Cache-Control
        if let Some(ref cc) = cache_control {
            if !cc.allows_caching() {
                return; // Don't cache if no-cache or no-store
            }
        }

        // Determine effective TTL
        let effective_ttl = self.calculate_effective_ttl(ttl, &cache_control);

        let entry = CacheEntry::new(document, domain.to_string(), effective_ttl, cache_control);

        // Ensure we don't exceed max entries
        self.ensure_capacity().await;

        // Store the entry
        {
            let mut entries = match self.entries.write() {
                Ok(entries) => entries,
                Err(_) => return, // Lock poisoned, skip caching
            };

            entries.insert(domain.to_string(), entry);
        }

        // Update LRU
        if let Ok(mut lru) = self.lru.lock() {
            lru.put(domain.to_string(), ());
        }

        self.update_stats().await;
    }

    /// Remove a specific entry from the cache
    pub async fn invalidate(&self, domain: &str) -> bool {
        let removed = {
            let mut entries = match self.entries.write() {
                Ok(entries) => entries,
                Err(_) => return false,
            };
            entries.remove(domain).is_some()
        };

        if removed {
            if let Ok(mut lru) = self.lru.lock() {
                lru.pop(domain);
            }
            self.update_stats().await;
        }

        removed
    }

    /// Clear all entries from the cache
    pub async fn clear(&self) {
        {
            let mut entries = match self.entries.write() {
                Ok(entries) => entries,
                Err(_) => return,
            };
            entries.clear();
        }

        if let Ok(mut lru) = self.lru.lock() {
            lru.clear();
        }

        self.update_stats().await;
    }

    /// Get the current number of entries in the cache
    pub fn size(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }

    /// Get cache statistics
    pub fn stats(&self) -> Arc<CacheStats> {
        self.stats.clone()
    }

    /// Get cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let expired_keys: Vec<String> = {
            let entries = match self.entries.read() {
                Ok(entries) => entries,
                Err(_) => return,
            };

            entries
                .iter()
                .filter_map(|(key, entry)| {
                    if entry.is_expired() {
                        Some(key.clone())
                    } else {
                        None
                    }
                })
                .collect()
        };

        for key in expired_keys {
            self.invalidate(&key).await;
        }
    }

    /// Calculate effective TTL considering Cache-Control headers and config limits
    fn calculate_effective_ttl(
        &self,
        requested_ttl: Duration,
        cache_control: &Option<CacheControl>,
    ) -> Duration {
        let mut effective_ttl = requested_ttl;

        // Apply Cache-Control max-age if present and cache control is respected
        if self.config.respect_cache_control {
            if let Some(cache_control) = cache_control {
                if let Some(cc_ttl) = cache_control.effective_ttl() {
                    effective_ttl = cc_ttl;
                }
            }
        }

        // Apply configuration limits
        effective_ttl = effective_ttl
            .max(self.config.min_ttl)
            .min(self.config.max_ttl);

        effective_ttl
    }

    /// Ensure cache doesn't exceed maximum capacity
    async fn ensure_capacity(&self) {
        let current_size = self.size();
        if current_size >= self.config.max_entries {
            // Need to evict entries using LRU policy
            let entries_to_evict = current_size - self.config.max_entries + 1;

            for _ in 0..entries_to_evict {
                if let Some(key_to_evict) = self.get_lru_key() {
                    self.invalidate(&key_to_evict).await;
                    self.stats.record_eviction();
                } else {
                    break; // No more entries to evict
                }
            }
        }
    }

    /// Get the least recently used key
    fn get_lru_key(&self) -> Option<String> {
        if let Ok(mut lru) = self.lru.lock() {
            lru.pop_lru().map(|(key, _)| key)
        } else {
            None
        }
    }

    /// Update cache statistics
    async fn update_stats(&self) {
        let size = self.size();
        self.stats.update_entry_count(size);

        // Estimate memory usage (rough calculation)
        let estimated_memory = size * 1024; // Rough estimate: 1KB per entry
        self.stats.update_memory_usage(estimated_memory);
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::with_default_config()
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
    async fn test_cache_put_and_get() {
        let cache = MemoryCache::with_default_config();
        let document = create_test_document();
        let domain = "test.com";

        // Put document in cache
        cache
            .put(domain, document.clone(), Duration::from_secs(300), None)
            .await;

        // Get document from cache
        let cached_doc = cache.get(domain).await;
        assert!(cached_doc.is_some());
        assert_eq!(cached_doc.unwrap().records.len(), document.records.len());
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = CacheConfig {
            min_ttl: Duration::from_millis(1), // Allow very short TTL for testing
            ..Default::default()
        };
        let cache = MemoryCache::new(config);
        let document = create_test_document();
        let domain = "test.com";

        // Put document with very short TTL
        cache
            .put(domain, document, Duration::from_millis(10), None)
            .await;

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Should return None due to expiration
        let cached_doc = cache.get(domain).await;
        assert!(cached_doc.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = MemoryCache::with_default_config();
        let document = create_test_document();
        let domain = "test.com";

        // Put document in cache
        cache
            .put(domain, document, Duration::from_secs(300), None)
            .await;

        // Verify it's cached
        assert!(cache.get(domain).await.is_some());

        // Invalidate
        let removed = cache.invalidate(domain).await;
        assert!(removed);

        // Should return None after invalidation
        assert!(cache.get(domain).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_control_no_cache() {
        let cache = MemoryCache::with_default_config();
        let document = create_test_document();
        let domain = "test.com";

        // Put document with no-cache directive
        cache
            .put(domain, document, Duration::from_secs(300), Some("no-cache"))
            .await;

        // Should not be cached due to no-cache directive
        assert!(cache.get(domain).await.is_none());
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let config = CacheConfig {
            max_entries: 2,
            ..Default::default()
        };
        let cache = MemoryCache::new(config);

        // Add entries up to capacity
        cache
            .put(
                "domain1.com",
                create_test_document(),
                Duration::from_secs(300),
                None,
            )
            .await;
        cache
            .put(
                "domain2.com",
                create_test_document(),
                Duration::from_secs(300),
                None,
            )
            .await;

        // Both should be cached
        assert!(cache.get("domain1.com").await.is_some());
        assert!(cache.get("domain2.com").await.is_some());

        // Add third entry, should evict least recently used
        cache
            .put(
                "domain3.com",
                create_test_document(),
                Duration::from_secs(300),
                None,
            )
            .await;

        // domain1.com should be evicted (least recently used)
        assert!(cache.get("domain1.com").await.is_none());
        assert!(cache.get("domain2.com").await.is_some());
        assert!(cache.get("domain3.com").await.is_some());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = MemoryCache::with_default_config();
        let document = create_test_document();
        let domain = "test.com";

        // Initial stats
        let stats = cache.stats();
        assert_eq!(
            stats.hit_count.load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        assert_eq!(
            stats.miss_count.load(std::sync::atomic::Ordering::Relaxed),
            0
        );

        // Cache miss
        cache.get(domain).await;
        assert_eq!(
            stats.miss_count.load(std::sync::atomic::Ordering::Relaxed),
            1
        );

        // Cache put and hit
        cache
            .put(domain, document, Duration::from_secs(300), None)
            .await;
        cache.get(domain).await;
        assert_eq!(
            stats.hit_count.load(std::sync::atomic::Ordering::Relaxed),
            1
        );
    }
}
