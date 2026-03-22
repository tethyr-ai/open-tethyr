//! Property 22: Request Timeout Handling
use open_tethyr::http::AxHttpClient;

#[test]
fn client_created_with_custom_timeout() {
    let client = AxHttpClient::new(Some(5));
    assert!(client.is_ok());
}

#[test]
fn client_created_with_default_timeout() {
    let client = AxHttpClient::new(None);
    assert!(client.is_ok());
}
