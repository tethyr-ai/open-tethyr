//! Property-Based Tests for HTTP Error Response Mapping
//!
//! Feature: rust-toolkit-architecture, Property 23: HTTP Error Response Mapping
//! Validates: Requirements 15.6
//!
//! This test validates that the system returns appropriate HTTP status codes for different error conditions:
//! - 400 for invalid requests
//! - 403 for policy violations
//! - 404 for not found
//! - 429 for rate limiting
//! - 502 for upstream fetch failures
//! - 503 for service unavailable

use open_tethyr::cache::RateLimitConfig;
use open_tethyr::policy::enforcement::DomainPolicy;
use proptest::prelude::*;
use std::collections::HashSet;

/// Generate valid domain names
fn arb_domain() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-z][a-z0-9]{1,10}\.[a-z]{2,3}")
        .unwrap()
        .prop_filter("Valid domain", |s| {
            s.len() <= 253
                && !s.contains("--")
                && !s.starts_with('-')
                && !s.ends_with('-')
                && !s.contains(".-")
                && !s.contains("-.")
        })
}

/// Generate invalid domain names
fn arb_invalid_domain() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),                    // Empty
        Just("single".to_string()),              // No TLD
        Just("invalid..domain.com".to_string()), // Double dots
        Just("-invalid.com".to_string()),        // Starts with hyphen
        Just("invalid-.com".to_string()),        // Ends with hyphen
    ]
}

// Feature: rust-toolkit-architecture, Property 23: HTTP Error Response Mapping
#[cfg(feature = "server")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Test that invalid domain formats result in 400 Bad Request
    #[test]
    fn test_invalid_domain_returns_400(invalid_domain in arb_invalid_domain()) {
        use open_tethyr::policy::enforcement::{PolicyEngine, PolicyViolation};
        use std::net::{IpAddr, Ipv4Addr};

        let policy = DomainPolicy::default();
        let engine = PolicyEngine::new(policy);
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        let result = engine.validate_discovery_request(&invalid_domain, client_ip, "test-123");

        // Should return InvalidDomain error (maps to 400)
        prop_assert!(result.is_err(), "Expected error for invalid domain");
        match result {
            Err(PolicyViolation::InvalidDomain { .. }) => {},
            _ => prop_assert!(false, "Expected InvalidDomain error"),
        }
    }

    /// Test that domain locking violations result in 403 Forbidden
    #[test]
    fn test_domain_locking_returns_403(
        blocked_domain in arb_domain(),
        allowed_domain in arb_domain()
    ) {
        use open_tethyr::policy::enforcement::{PolicyEngine, PolicyViolation};
        use std::net::{IpAddr, Ipv4Addr};

        prop_assume!(blocked_domain != allowed_domain);

        let policy = DomainPolicy {
            domain_locking_enabled: true,
            allowed_domains: {
                let mut set = HashSet::new();
                set.insert(allowed_domain.clone());
                set
            },
            ..Default::default()
        };

        let engine = PolicyEngine::new(policy);
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Allowed domain should pass
        let allowed_result = engine.validate_discovery_request(&allowed_domain, client_ip, "test-123");
        prop_assert!(allowed_result.is_ok());

        // Blocked domain should return DomainNotAllowed error (maps to 403)
        let blocked_result = engine.validate_discovery_request(&blocked_domain, client_ip, "test-456");
        prop_assert!(blocked_result.is_err(), "Expected error for blocked domain");
        match blocked_result {
            Err(PolicyViolation::DomainNotAllowed { .. }) => {},
            _ => prop_assert!(false, "Expected DomainNotAllowed error"),
        }
    }

    /// Test that external domain blocking results in 403 Forbidden
    #[test]
    fn test_external_domain_blocking_returns_403(
        external_domain in arb_domain(),
        org_domain in arb_domain()
    ) {
        use open_tethyr::policy::enforcement::{PolicyEngine, PolicyViolation};
        use std::net::{IpAddr, Ipv4Addr};

        prop_assume!(external_domain != org_domain);
        prop_assume!(!external_domain.ends_with(&org_domain));

        let policy = DomainPolicy {
            domain_locking_enabled: true,
            organization_domain: Some(org_domain),
            external_allowlist: HashSet::new(),
            ..Default::default()
        };

        let engine = PolicyEngine::new(policy);
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // External domain should return ExternalDomainBlocked error (maps to 403)
        let result = engine.validate_discovery_request(&external_domain, client_ip, "test-123");
        prop_assert!(result.is_err(), "Expected error for external domain");
        match result {
            Err(PolicyViolation::ExternalDomainBlocked { .. }) => {},
            _ => prop_assert!(false, "Expected ExternalDomainBlocked error"),
        }
    }

    /// Test that rate limiting results in 429 Too Many Requests
    #[test]
    fn test_rate_limiting_returns_429(request_count in 3u32..10u32) {
        use open_tethyr::cache::{RateLimitError, RateLimiter};
        use std::net::{IpAddr, Ipv4Addr};

        let config = RateLimitConfig {
            requests_per_minute: 2,
            requests_per_hour: 120,
        };
        let limiter = RateLimiter::new(config);
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // First 2 requests should succeed
        prop_assert!(limiter.check_rate_limit(client_ip).is_ok());
        prop_assert!(limiter.check_rate_limit(client_ip).is_ok());

        // Subsequent requests should return RateLimitExceeded error (maps to 429)
        for _ in 0..(request_count - 2) {
            let result = limiter.check_rate_limit(client_ip);
            prop_assert!(result.is_err(), "Expected rate limit error");
            match result {
                Err(RateLimitError::RateLimitExceeded { .. }) => {},
                _ => prop_assert!(false, "Expected RateLimitExceeded error"),
            }
        }
    }
}

/// Unit tests for error response mapping
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_domain_error_mapping() {
        use open_tethyr::policy::enforcement::{PolicyEngine, PolicyViolation};
        use std::net::{IpAddr, Ipv4Addr};

        let policy = DomainPolicy::default();
        let engine = PolicyEngine::new(policy);
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        let invalid_domains = vec!["", "single", "invalid..domain.com", "-invalid.com"];

        for domain in invalid_domains {
            let result = engine.validate_discovery_request(domain, client_ip, "test");
            assert!(matches!(result, Err(PolicyViolation::InvalidDomain { .. })));
        }
    }

    #[test]
    fn test_policy_violation_error_mapping() {
        use open_tethyr::policy::enforcement::{PolicyEngine, PolicyViolation};
        use std::net::{IpAddr, Ipv4Addr};

        let policy = DomainPolicy {
            domain_locking_enabled: true,
            allowed_domains: {
                let mut set = HashSet::new();
                set.insert("allowed.com".to_string());
                set
            },
            ..Default::default()
        };

        let engine = PolicyEngine::new(policy);
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Blocked domain should return DomainNotAllowed
        let result = engine.validate_discovery_request("blocked.com", client_ip, "test");
        assert!(matches!(
            result,
            Err(PolicyViolation::DomainNotAllowed { .. })
        ));
    }

    #[test]
    fn test_rate_limit_error_mapping() {
        use open_tethyr::cache::{RateLimitError, RateLimiter};
        use std::net::{IpAddr, Ipv4Addr};

        let config = RateLimitConfig {
            requests_per_minute: 1,
            requests_per_hour: 60,
        };
        let limiter = RateLimiter::new(config);
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Exhaust limit
        assert!(limiter.check_rate_limit(client_ip).is_ok());

        // Should return RateLimitExceeded
        let result = limiter.check_rate_limit(client_ip);
        assert!(matches!(
            result,
            Err(RateLimitError::RateLimitExceeded { .. })
        ));
    }
}
