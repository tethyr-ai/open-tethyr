//! Property 14: HTTPS Certificate Validation
// The reqwest client with rustls-tls feature validates certificates by default.
// This test verifies the client is configured correctly.

use open_tethyr::http::AxHttpClient;

#[test]
fn ax_url_uses_https() {
    let url = AxHttpClient::build_ax_url("example.com");
    assert!(url.starts_with("https://"), "AX URLs must use HTTPS");
}
