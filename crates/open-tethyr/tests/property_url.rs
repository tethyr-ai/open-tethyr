use open_tethyr::http::AxHttpClient;
use proptest::prelude::*;

#[test]
fn url_construction_basic() {
    assert_eq!(
        AxHttpClient::build_ax_url("example.com"),
        "https://example.com/.well-known/agent-exchange"
    );
}

#[test]
fn url_construction_subdomain() {
    assert_eq!(
        AxHttpClient::build_ax_url("api.partner.com"),
        "https://api.partner.com/.well-known/agent-exchange"
    );
}

#[test]
fn well_known_path_validation() {
    assert!(AxHttpClient::validate_well_known_path("/.well-known/agent-exchange").is_ok());
    assert!(AxHttpClient::validate_well_known_path("/other/path").is_err());
}

proptest! {
    #[test]
    fn url_always_correct(domain in "[a-z]{3,10}\\.[a-z]{2,5}") {
        let url = AxHttpClient::build_ax_url(&domain);
        prop_assert!(url.starts_with("https://"));
        prop_assert!(url.ends_with("/.well-known/agent-exchange"));
        prop_assert!(url.contains(&domain));
        prop_assert!(!url.contains("_agent."));
    }
}
