//! In-Memory Cache Implementation

use crate::error::CacheError;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Cache entry with TTL tracking
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub data: String, // Serialized AX document JSON
    pub created_at: Instant,
    pub ttl: Duration,
    pub no_cache: bool,
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// In-memory cache with LRU eviction
pub struct MemoryCache {
    cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    max_entries: usize,
}

impl MemoryCache {
    pub fn new(max_entries: usize) -> Self {
        let cap = NonZeroUsize::new(max_entries.max(1)).unwrap();
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(cap))),
            max_entries,
        }
    }

    /// Get a cached entry, checking TTL
    pub fn get(&self, domain: &str) -> Option<CacheEntry> {
        let mut cache = self.cache.write().ok()?;
        if let Some(entry) = cache.get(domain) {
            if entry.is_expired() {
                cache.pop(domain);
                return None;
            }
            return Some(entry.clone());
        }
        None
    }

    /// Put an entry into the cache
    pub fn put(&self, domain: String, entry: CacheEntry) -> Result<(), CacheError> {
        if entry.no_cache {
            return Ok(()); // Respect Cache-Control: no-cache
        }
        let mut cache = self
            .cache
            .write()
            .map_err(|e| CacheError::OperationFailed(e.to_string()))?;
        cache.put(domain, entry);
        Ok(())
    }

    /// Invalidate a specific domain
    pub fn invalidate(&self, domain: &str) -> bool {
        if let Ok(mut cache) = self.cache.write() {
            return cache.pop(domain).is_some();
        }
        false
    }

    /// Clear entire cache
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Current cache size
    pub fn size(&self) -> usize {
        self.cache.read().map(|c| c.len()).unwrap_or(0)
    }

    pub fn max_entries(&self) -> usize {
        self.max_entries
    }
}
