//! HTTP Client Property-Based Tests
//!
//! Property-based tests for AX URL construction correctness.

use open_tethyr::http::{AxHttpClient, HttpError};
use proptest::prelude::*;

/// Property 13: AX Subdomain URL Construction
/// **Validates: Requirements 9.4**
///
/// For any valid domain name, the AX URL construction should:
/// 1. Create a URL with the _agent subdomain
/// 2. Use HTTPS protocol
/// 3. Use the correct well-known path /.well-known/agent-exchange.json
/// 4. Reject invalid domain names with appropriate errors
#[test]
fn prop_ax_url_construction_correctness() {
    proptest!(|(
        valid_domain in "[a-z][a-z0-9-]{1,10}\\.[a-z]{2,4}",
        subdomain in "[a-z][a-z0-9-]{1,8}",
        invalid_chars in "[^a-z0-9.-]",
    )| {
        let client = AxHttpClient::default();

        // Test valid domain construction
        let result = client.build_ax_url(&valid_domain);
        prop_assert!(result.is_ok(), "Valid domain should construct URL successfully: {}", valid_domain);

        if let Ok(url) = result {
            prop_assert!(url.starts_with("https://"), "URL should use HTTPS: {}", url);
            prop_assert!(url.contains(&format!("_agent.{}", valid_domain)), "URL should contain _agent subdomain: {}", url);
            prop_assert!(url.ends_with("/.well-known/agent-exchange.json"), "URL should end with well-known path: {}", url);

            // Verify URL is parseable
            let parsed = url::Url::parse(&url);
            prop_assert!(parsed.is_ok(), "Constructed URL should be valid: {}", url);
        }

        // Test valid subdomain construction
        let subdomain_domain = format!("{}.{}", subdomain, valid_domain);
        let result = client.build_ax_url(&subdomain_domain);
        prop_assert!(result.is_ok(), "Valid subdomain should construct URL successfully: {}", subdomain_domain);

        if let Ok(url) = result {
            prop_assert!(url.contains(&format!("_agent.{}", subdomain_domain)), "URL should contain _agent with subdomain: {}", url);
        }

        // Test some invalid domain patterns
        let invalid_domain = format!("invalid{}", invalid_chars);
        if invalid_domain.contains("..") || invalid_domain.starts_with('.') || invalid_domain.ends_with('.') {
            let result = client.build_ax_url(&invalid_domain);
            prop_assert!(result.is_err(), "Invalid domain should be rejected: '{}'", invalid_domain);
            prop_assert!(matches!(result.unwrap_err(), HttpError::InvalidUrl(_)));
        }
    });
}

/// Unit tests for specific AX URL construction scenarios
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_valid_ax_url_construction() {
        let client = AxHttpClient::default();

        let test_cases = vec![
            (
                "example.com",
                "https://_agent.example.com/.well-known/agent-exchange.json",
            ),
            (
                "api.example.com",
                "https://_agent.api.example.com/.well-known/agent-exchange.json",
            ),
            (
                "test-domain.co.uk",
                "https://_agent.test-domain.co.uk/.well-known/agent-exchange.json",
            ),
            ("a.b", "https://_agent.a.b/.well-known/agent-exchange.json"),
        ];

        for (domain, expected_url) in test_cases {
            let result = client.build_ax_url(domain);
            assert!(
                result.is_ok(),
                "Should construct URL for valid domain: {}",
                domain
            );
            assert_eq!(
                result.unwrap(),
                expected_url,
                "URL should match expected format for domain: {}",
                domain
            );
        }
    }

    #[test]
    fn test_invalid_ax_url_construction() {
        let client = AxHttpClient::default();

        let test_cases = vec![
            "",              // Empty domain
            ".",             // Single dot
            ".example.com",  // Leading dot
            "example.com.",  // Trailing dot
            "example..com",  // Double dots
            "example.com..", // Multiple trailing dots
            "..example.com", // Multiple leading dots
            "example...com", // Multiple consecutive dots
        ];

        for domain in test_cases {
            let result = client.build_ax_url(domain);
            assert!(
                result.is_err(),
                "Should reject invalid domain: '{}'",
                domain
            );
            assert!(matches!(result.unwrap_err(), HttpError::InvalidUrl(_)));
        }
    }

    #[test]
    fn test_ax_url_components() {
        let client = AxHttpClient::default();
        let domain = "example.com";

        let result = client.build_ax_url(domain);
        assert!(result.is_ok());

        let url = result.unwrap();
        let parsed = url::Url::parse(&url).expect("URL should be parseable");

        // Verify URL components
        assert_eq!(parsed.scheme(), "https", "Should use HTTPS scheme");
        assert_eq!(
            parsed.host_str().unwrap(),
            "_agent.example.com",
            "Should use _agent subdomain"
        );
        assert_eq!(
            parsed.path(),
            "/.well-known/agent-exchange.json",
            "Should use correct well-known path"
        );
        assert_eq!(
            parsed.port(),
            None,
            "Should not specify explicit port for HTTPS"
        );
    }

    #[test]
    fn test_complex_domain_names() {
        let client = AxHttpClient::default();

        let test_cases = vec![
            "sub.example.com",
            "api-v2.service.example.com",
            "test123.example.org",
            "a-b-c.example.net",
        ];

        for domain in test_cases {
            let result = client.build_ax_url(domain);
            assert!(result.is_ok(), "Should handle complex domain: {}", domain);

            let url = result.unwrap();
            assert!(
                url.contains(&format!("_agent.{}", domain)),
                "Should preserve full domain in _agent subdomain"
            );
            assert!(url.starts_with("https://"), "Should use HTTPS");
            assert!(
                url.ends_with("/.well-known/agent-exchange.json"),
                "Should use well-known path"
            );
        }
    }
}
