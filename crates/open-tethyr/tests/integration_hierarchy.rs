//! Integration test: hierarchical cache structure

use open_tethyr::cache::coordinator::CacheCoordinator;
use open_tethyr::cache::memory::{CacheEntry, MemoryCache};
use std::time::{Duration, Instant};

#[test]
fn local_cache_serves_without_upstream() {
    let cache = MemoryCache::new(100);
    cache
        .put(
            "example.com".into(),
            CacheEntry {
                data: r#"{"records":[]}"#.into(),
                created_at: Instant::now(),
                ttl: Duration::from_secs(3600),
                no_cache: false,
            },
        )
        .unwrap();
    assert!(cache.get("example.com").is_some());
}

#[test]
fn coordinator_without_root_creates_successfully() {
    let coord = CacheCoordinator::new(100, 3600, None);
    assert!(coord.is_ok());
}

#[test]
fn coordinator_with_root_creates_successfully() {
    let coord = CacheCoordinator::new(100, 3600, Some("https://root.cache.example.com".into()));
    assert!(coord.is_ok());
}
