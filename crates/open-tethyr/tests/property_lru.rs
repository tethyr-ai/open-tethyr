//! Property 15: LRU Eviction
use open_tethyr::cache::memory::{CacheEntry, MemoryCache};
use std::time::{Duration, Instant};

fn entry(data: &str) -> CacheEntry {
    CacheEntry {
        data: data.into(),
        created_at: Instant::now(),
        ttl: Duration::from_secs(3600),
        no_cache: false,
    }
}

#[test]
fn lru_evicts_oldest() {
    let cache = MemoryCache::new(2);
    cache.put("a.com".into(), entry("a")).unwrap();
    cache.put("b.com".into(), entry("b")).unwrap();
    cache.put("c.com".into(), entry("c")).unwrap(); // evicts a.com
    assert!(cache.get("a.com").is_none());
    assert!(cache.get("b.com").is_some());
    assert!(cache.get("c.com").is_some());
}

#[test]
fn recently_accessed_preserved() {
    let cache = MemoryCache::new(2);
    cache.put("a.com".into(), entry("a")).unwrap();
    cache.put("b.com".into(), entry("b")).unwrap();
    let _ = cache.get("a.com"); // touch a.com
    cache.put("c.com".into(), entry("c")).unwrap(); // evicts b.com (LRU)
    assert!(cache.get("a.com").is_some());
    assert!(cache.get("b.com").is_none());
    assert!(cache.get("c.com").is_some());
}
