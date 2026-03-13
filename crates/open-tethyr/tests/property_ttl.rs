//! Property 7: TTL Expiration
use open_tethyr::cache::memory::{CacheEntry, MemoryCache};
use std::time::{Duration, Instant};

#[test]
fn expired_entry_not_returned() {
    let cache = MemoryCache::new(100);
    cache.put("example.com".into(), CacheEntry {
        data: "test".into(),
        created_at: Instant::now() - Duration::from_secs(100),
        ttl: Duration::from_secs(10),
        no_cache: false,
    }).unwrap();
    assert!(cache.get("example.com").is_none());
}

#[test]
fn non_expired_entry_returned() {
    let cache = MemoryCache::new(100);
    cache.put("example.com".into(), CacheEntry {
        data: "test".into(),
        created_at: Instant::now(),
        ttl: Duration::from_secs(3600),
        no_cache: false,
    }).unwrap();
    assert!(cache.get("example.com").is_some());
}
