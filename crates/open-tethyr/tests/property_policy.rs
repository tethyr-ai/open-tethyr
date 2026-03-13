//! Property 6: Domain Locking Policy Enforcement
use open_tethyr::server::PolicyEngine;

#[test]
fn domain_locking_rejects_external() {
    let policy = PolicyEngine::new(true, Some("home.com".into()), vec![]);
    assert!(policy.check_discovery_allowed("external.com").is_err());
}

#[test]
fn domain_locking_allows_home() {
    let policy = PolicyEngine::new(true, Some("home.com".into()), vec![]);
    assert!(policy.check_discovery_allowed("home.com").is_ok());
}

#[test]
fn domain_locking_allows_subdomain() {
    let policy = PolicyEngine::new(true, Some("home.com".into()), vec![]);
    assert!(policy.check_discovery_allowed("api.home.com").is_ok());
}

#[test]
fn allowlist_permits_domain() {
    let policy = PolicyEngine::new(true, Some("home.com".into()), vec!["partner.com".into()]);
    assert!(policy.check_discovery_allowed("partner.com").is_ok());
}

#[test]
fn no_locking_allows_all() {
    let policy = PolicyEngine::new(false, None, vec![]);
    assert!(policy.check_discovery_allowed("anything.com").is_ok());
}
