//! Property 5: Cache Miss Fallback
use open_tethyr::cache::memory::MemoryCache;

#[test]
fn miss_on_empty_cache() {
    let cache = MemoryCache::new(100);
    assert!(cache.get("example.com").is_none());
    assert_eq!(cache.size(), 0);
}
