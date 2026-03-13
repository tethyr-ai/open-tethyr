//! Property 4: Cache-First Discovery
use open_tethyr::cache::memory::{CacheEntry, MemoryCache};
use std::time::{Duration, Instant};

#[test]
fn cache_hit_returns_data() {
    let cache = MemoryCache::new(100);
    cache.put("example.com".into(), CacheEntry {
        data: r#"{"records":[]}"#.into(),
        created_at: Instant::now(),
        ttl: Duration::from_secs(3600),
        no_cache: false,
    }).unwrap();
    assert!(cache.get("example.com").is_some());
}

#[test]
fn cache_miss_returns_none() {
    let cache = MemoryCache::new(100);
    assert!(cache.get("unknown.com").is_none());
}
