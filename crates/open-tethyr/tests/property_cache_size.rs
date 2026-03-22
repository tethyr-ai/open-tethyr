//! Property 26: Cache Size Limit Enforcement
use open_tethyr::cache::memory::{CacheEntry, MemoryCache};
use std::time::{Duration, Instant};

#[test]
fn cache_never_exceeds_max() {
    let max = 5;
    let cache = MemoryCache::new(max);
    for i in 0..20 {
        cache
            .put(
                format!("domain-{}.com", i),
                CacheEntry {
                    data: format!("data-{}", i),
                    created_at: Instant::now(),
                    ttl: Duration::from_secs(3600),
                    no_cache: false,
                },
            )
            .unwrap();
        assert!(
            cache.size() <= max,
            "Cache size {} exceeded max {}",
            cache.size(),
            max
        );
    }
}
