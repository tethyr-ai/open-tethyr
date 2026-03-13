//! Integration test: policy enforcement

#[cfg(feature = "server")]
mod policy_tests {
    use open_tethyr::server::PolicyEngine;

    #[test]
    fn domain_locking_full_scenario() {
        let policy = PolicyEngine::new(
            true,
            Some("acme.com".into()),
            vec!["partner.org".into(), "trusted.io".into()],
        );

        // Home domain allowed
        assert!(policy.check_discovery_allowed("acme.com").is_ok());
        // Subdomain of home allowed
        assert!(policy.check_discovery_allowed("api.acme.com").is_ok());
        // Allowlisted domains allowed
        assert!(policy.check_discovery_allowed("partner.org").is_ok());
        assert!(policy.check_discovery_allowed("trusted.io").is_ok());
        // External domain blocked
        assert!(policy.check_discovery_allowed("evil.com").is_err());
        assert!(policy.check_discovery_allowed("random.net").is_err());
    }

    #[test]
    fn no_locking_allows_everything() {
        let policy = PolicyEngine::new(false, None, vec![]);
        assert!(policy.check_discovery_allowed("anything.com").is_ok());
        assert!(policy.check_discovery_allowed("evil.com").is_ok());
    }
}
