//! Property 19: DNS Discovery Error Resilience
use open_tethyr::OpenTethyr;

#[test]
fn client_new_doesnt_require_dns() {
    // new() should succeed without DNS - it's non-async
    let client = OpenTethyr::new("nonexistent.invalid");
    assert!(client.is_ok());
}
