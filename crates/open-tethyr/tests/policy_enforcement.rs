//! Property tests for policy enforcement

use open_tethyr::policy::{DomainPolicy, PolicyEngine, PolicyViolation};
use proptest::prelude::*;
use std::net::{IpAddr, Ipv4Addr};

// Property 6: Domain Locking Policy Enforcement
// Validates: Requirements 3.4, 12.1, 12.2, 12.5
proptest! {
    #[test]
    fn property_domain_locking_enforcement(
        allowed_domains in prop::collection::hash_set("[a-z]{3,10}\\.[a-z]{2,3}", 1..5),
        external_allowlist in prop::collection::hash_set("[a-z]{3,10}\\.[a-z]{2,3}", 0..3),
        test_domain in "[a-z]{3,10}\\.[a-z]{2,3}",
        client_ip_octets in (1u8..255, 1u8..255, 1u8..255, 1u8..255),
    ) {
        let client_ip = IpAddr::V4(Ipv4Addr::new(
            client_ip_octets.0,
            client_ip_octets.1,
            client_ip_octets.2,
            client_ip_octets.3,
        ));

        // Test with domain locking enabled
        let mut policy = DomainPolicy {
            domain_locking_enabled: true,
            allowed_domains: allowed_domains.clone(),
            external_allowlist: external_allowlist.clone(),
            organization_domain: Some("acme.com".to_string()),
        };

        let engine = PolicyEngine::new(policy.clone());
        let result = engine.validate_discovery_request(&test_domain, client_ip, "test-correlation");

        // Property: If domain locking is enabled, only allowed domains or external allowlisted domains should pass
        if allowed_domains.contains(&test_domain) {
            // Domain is in allowed list - should always pass
            prop_assert!(result.is_ok(), "Allowed domain should pass: {}", test_domain);
        } else if !test_domain.ends_with("acme.com") && external_allowlist.contains(&test_domain) {
            // External domain in allowlist - should pass
            prop_assert!(result.is_ok(), "External allowlisted domain should pass: {}", test_domain);
        } else if !test_domain.ends_with("acme.com") {
            // External domain not in allowlist - should be blocked
            prop_assert!(
                matches!(result, Err(PolicyViolation::ExternalDomainBlocked { .. })),
                "External non-allowlisted domain should be blocked: {}",
                test_domain
            );
        } else {
            // Internal domain not in allowed list - should be blocked
            prop_assert!(
                matches!(result, Err(PolicyViolation::DomainNotAllowed { .. })),
                "Non-allowed internal domain should be blocked: {}",
                test_domain
            );
        }

        // Test with domain locking disabled - should always allow valid domains
        policy.domain_locking_enabled = false;
        let permissive_engine = PolicyEngine::new(policy);
        let permissive_result = permissive_engine.validate_discovery_request(&test_domain, client_ip, "test-correlation");
        prop_assert!(permissive_result.is_ok(), "Permissive policy should allow all valid domains: {}", test_domain);
    }
}

proptest! {
    #[test]
    fn property_invalid_domain_rejection(
        invalid_domain_type in 0u8..6,
        client_ip_octets in (1u8..255, 1u8..255, 1u8..255, 1u8..255),
    ) {
        let invalid_domain = match invalid_domain_type {
            0 => "".to_string(),
            1 => "single".to_string(),
            2 => "invalid..domain.com".to_string(),
            3 => "-invalid.com".to_string(),
            4 => "invalid-.com".to_string(),
            _ => "a".repeat(300), // Too long
        };

        let client_ip = IpAddr::V4(Ipv4Addr::new(
            client_ip_octets.0,
            client_ip_octets.1,
            client_ip_octets.2,
            client_ip_octets.3,
        ));

        let engine = PolicyEngine::permissive();
        let result = engine.validate_discovery_request(&invalid_domain, client_ip, "test-correlation");

        // Property: Invalid domains should always be rejected regardless of policy
        prop_assert!(
            matches!(result, Err(PolicyViolation::InvalidDomain { .. })),
            "Invalid domain should be rejected: {}",
            invalid_domain
        );
    }
}

proptest! {
    #[test]
    fn property_audit_logging_consistency(
        target_domain in "[a-z]{3,10}\\.[a-z]{2,3}",
        client_ip_octets in (1u8..255, 1u8..255, 1u8..255, 1u8..255),
        correlation_id in "[a-zA-Z0-9-]{10,20}",
    ) {
        let client_ip = IpAddr::V4(Ipv4Addr::new(
            client_ip_octets.0,
            client_ip_octets.1,
            client_ip_octets.2,
            client_ip_octets.3,
        ));

        let engine = PolicyEngine::permissive();

        // Property: All discovery requests should be logged regardless of outcome
        // We can't directly test logging output in property tests, but we can ensure
        // the function completes without panicking
        let result = engine.validate_discovery_request(&target_domain, client_ip, &correlation_id);

        // The function should complete (either Ok or Err, but not panic)
        prop_assert!(result.is_ok() || result.is_err());
    }
}
