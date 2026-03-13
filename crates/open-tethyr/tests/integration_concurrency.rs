//! Concurrency load test: 1000 concurrent cache operations

use open_tethyr::cache::memory::{CacheEntry, MemoryCache};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::test]
async fn concurrent_cache_operations() {
    let cache = Arc::new(MemoryCache::new(10_000));
    let mut handles = vec![];

    // Spawn 1000 concurrent tasks
    for i in 0..1000 {
        let cache = cache.clone();
        handles.push(tokio::spawn(async move {
            let domain = format!("domain-{}.com", i);
            cache
                .put(
                    domain.clone(),
                    CacheEntry {
                        data: format!(r#"{{"records":[{{"id":{}}}]}}"#, i),
                        created_at: Instant::now(),
                        ttl: Duration::from_secs(3600),
                        no_cache: false,
                    },
                )
                .unwrap();
            // Read back
            let _ = cache.get(&domain);
        }));
    }

    // All should complete without panic
    for handle in handles {
        handle.await.unwrap();
    }

    // Cache should have entries (may be less than 1000 due to LRU if max < 1000)
    assert!(cache.size() > 0);
    assert!(cache.size() <= 10_000);
}
