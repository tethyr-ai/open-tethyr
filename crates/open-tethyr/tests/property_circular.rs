//! Property 12: Circular Dependency Prevention

use open_tethyr::cache::coordinator::CacheCoordinator;

#[test]
fn empty_root_url_detected() {
    let result = CacheCoordinator::new(100, 3600, Some("".into()));
    assert!(result.is_err());
}
