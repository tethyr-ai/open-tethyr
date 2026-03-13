//! Property 13: AX Subdomain URL Construction

use open_tethyr::http::AxHttpClient;
use proptest::prelude::*;

#[test]
fn url_construction_basic() {
    assert_eq!(
        AxHttpClient::build_ax_url("example.com"),
        "https://_agent.example.com/.well-known/agent-exchange.json"
    );
}

#[test]
fn url_construction_subdomain() {
    assert_eq!(
        AxHttpClient::build_ax_url("api.partner.com"),
        "https://_agent.api.partner.com/.well-known/agent-exchange.json"
    );
}

#[test]
fn well_known_path_validation() {
    assert!(AxHttpClient::validate_well_known_path("/.well-known/agent-exchange.json").is_ok());
    assert!(AxHttpClient::validate_well_known_path("/other/path").is_err());
}

proptest! {
    #[test]
    fn url_always_has_well_known_path(domain in "[a-z]{3,10}\\.[a-z]{2,5}") {
        let url = AxHttpClient::build_ax_url(&domain);
        prop_assert!(url.starts_with("https://_agent."));
        prop_assert!(url.ends_with("/.well-known/agent-exchange.json"));
        prop_assert!(url.contains(&domain));
    }
}
