//! Property 16: Cache-Control Header Compliance
use open_tethyr::cache::memory::{CacheEntry, MemoryCache};
use std::time::{Duration, Instant};

#[test]
fn no_cache_directive_prevents_caching() {
    let cache = MemoryCache::new(100);
    cache
        .put(
            "example.com".into(),
            CacheEntry {
                data: "test".into(),
                created_at: Instant::now(),
                ttl: Duration::from_secs(3600),
                no_cache: true, // Cache-Control: no-cache
            },
        )
        .unwrap();
    assert!(cache.get("example.com").is_none());
}
