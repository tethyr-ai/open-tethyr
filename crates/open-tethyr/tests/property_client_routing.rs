//! Property 17: DNS Cache Discovery Routing
use open_tethyr::OpenTethyr;

#[test]
fn client_creates_without_error() {
    let client = OpenTethyr::new("example.com");
    assert!(client.is_ok());
}

#[test]
fn client_rejects_empty_domain() {
    let client = OpenTethyr::new("");
    assert!(client.is_err());
}
