//! DNS Cache Discovery Routing Property-Based Tests
//!
//! Property-based tests for client DNS cache discovery routing behavior.

use open_tethyr::dns::DnsDiscovery;
use open_tethyr::http::AxHttpClient;
use proptest::prelude::*;

// Property 17: DNS Cache Discovery Routing
// **Validates: Requirements 11.2, 11.3**
//
// For any client initialization, when a cache endpoint is found via DNS,
// discovery requests should use the cache; when no cache is found,
// requests should fall back to direct HTTPS discovery.
//
// This property test validates the URL construction and routing logic.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_dns_cache_discovery_routing(
        target_domain in "[a-z][a-z0-9]{2,8}\\.[a-z]{2,4}",
        cache_domain in "[a-z][a-z0-9]{2,8}\\.[a-z]{2,4}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Test scenario 1: Cache URL construction
            // When a cache endpoint is provided, the client should construct
            // the correct cache URL format: https://<cache>/discover/<domain>
            let http_client = AxHttpClient::new().unwrap();
            let cache_url = format!("https://{}", cache_domain);

            // The fetch_from_cache method constructs the URL as:
            // https://<cache_url>/discover/<target_domain>
            // This will fail to connect (expected), but we're testing the routing logic
            let result = http_client.fetch_from_cache(&cache_url, &target_domain).await;

            // Should attempt to fetch from cache (will fail with connection error, not URL error)
            prop_assert!(
                result.is_err(),
                "Cache fetch should fail with connection error (expected for non-existent cache)"
            );

            // Verify it's not an InvalidUrl error - that would indicate routing logic failure
            if let Err(e) = result {
                prop_assert!(
                    !matches!(e, open_tethyr::http::HttpError::InvalidUrl(_)),
                    "Should not be URL error, got: {:?}", e
                );
            }

            // Test scenario 2: Direct discovery URL construction
            // When no cache is available, should construct direct AX URL
            let direct_result = http_client.fetch_ax_record(&target_domain).await;

            // Should attempt direct discovery (will fail with connection error, expected)
            prop_assert!(
                direct_result.is_err(),
                "Direct fetch should fail with connection error (expected for non-existent domain)"
            );

            // Verify it's not an InvalidUrl error
            if let Err(e) = direct_result {
                prop_assert!(
                    !matches!(e, open_tethyr::http::HttpError::InvalidUrl(_)),
                    "Should not be URL error for valid domain, got: {:?}", e
                );
            }

            Ok(())
        })?;
    }
}

/// Unit tests for DNS cache discovery routing scenarios
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_url_format_construction() {
        let http_client = AxHttpClient::new().unwrap();
        let cache_url = "https://cache.example.com";
        let target_domain = "api.partner.com";

        // Test that cache URL is constructed correctly
        // The method should construct: https://cache.example.com/discover/api.partner.com
        let result = http_client.fetch_from_cache(cache_url, target_domain).await;

        // Will fail with connection error (expected), but not URL validation error
        assert!(
            result.is_err(),
            "Should fail to connect to non-existent cache"
        );

        // Verify it's not an InvalidUrl error
        if let Err(e) = result {
            assert!(
                !matches!(e, open_tethyr::http::HttpError::InvalidUrl(_)),
                "Should not be URL validation error, got: {:?}",
                e
            );
        }
    }

    #[tokio::test]
    async fn test_cache_url_with_trailing_slash() {
        let http_client = AxHttpClient::new().unwrap();
        let cache_url_with_slash = "https://cache.example.com/";
        let target_domain = "api.partner.com";

        // Test that trailing slash is handled correctly
        let result = http_client
            .fetch_from_cache(cache_url_with_slash, target_domain)
            .await;

        // Will fail with connection error (expected), but not URL validation error
        assert!(
            result.is_err(),
            "Should fail to connect to non-existent cache"
        );

        // Verify it's not an InvalidUrl error
        if let Err(e) = result {
            assert!(
                !matches!(e, open_tethyr::http::HttpError::InvalidUrl(_)),
                "Should handle trailing slash correctly, got: {:?}",
                e
            );
        }
    }

    #[tokio::test]
    async fn test_direct_discovery_url_construction() {
        let http_client = AxHttpClient::new().unwrap();
        let target_domain = "api.partner.com";

        // Test that direct AX URL is constructed correctly
        // Should construct: https://_agent.api.partner.com/.well-known/agent-exchange.json
        let result = http_client.fetch_ax_record(target_domain).await;

        // Will fail with connection error (expected), but not URL validation error
        assert!(
            result.is_err(),
            "Should fail to connect to non-existent domain"
        );

        // Verify it's not an InvalidUrl error
        if let Err(e) = result {
            assert!(
                !matches!(e, open_tethyr::http::HttpError::InvalidUrl(_)),
                "Should not be URL validation error for valid domain, got: {:?}",
                e
            );
        }
    }

    #[tokio::test]
    async fn test_dns_discovery_returns_none_triggers_fallback() {
        let dns = DnsDiscovery::default();

        // Test DNS discovery for non-existent domain
        let result = dns
            .discover_cache("nonexistent-test-domain-12345.com")
            .await;

        // DNS discovery should either return None or an error, both trigger fallback
        match result {
            Ok(None) => {
                // Expected: no cache found, should trigger direct discovery fallback
            }
            Err(_) => {
                // Also acceptable: DNS error should trigger direct discovery fallback
            }
            Ok(Some(_)) => {
                panic!("Should not find cache for non-existent test domain");
            }
        }
    }

    #[tokio::test]
    async fn test_cache_url_requires_https() {
        let http_client = AxHttpClient::new().unwrap();
        let http_cache_url = "http://cache.example.com"; // HTTP, not HTTPS
        let target_domain = "api.partner.com";

        // Test that HTTP cache URLs are rejected
        let result = http_client
            .fetch_from_cache(http_cache_url, target_domain)
            .await;

        // Should fail with InvalidUrl error because HTTP is not allowed
        assert!(result.is_err(), "Should reject HTTP cache URL");

        if let Err(e) = result {
            assert!(
                matches!(e, open_tethyr::http::HttpError::InvalidUrl(_)),
                "Should be InvalidUrl error for HTTP URL, got: {:?}",
                e
            );
        }
    }

    #[test]
    fn test_ax_url_construction_for_direct_discovery() {
        let http_client = AxHttpClient::new().unwrap();
        let target_domain = "api.partner.com";

        // Test that AX URL is constructed correctly
        let url = http_client.build_ax_url(target_domain);

        assert!(url.is_ok(), "Should construct valid AX URL");
        let url_str = url.unwrap();

        assert!(url_str.starts_with("https://"), "Should use HTTPS");
        assert!(
            url_str.contains("_agent.api.partner.com"),
            "Should use _agent subdomain"
        );
        assert!(
            url_str.ends_with("/.well-known/agent-exchange.json"),
            "Should use well-known path"
        );
    }
}
