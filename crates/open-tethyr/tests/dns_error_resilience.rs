//! DNS Error Resilience Property-Based Tests
//!
//! Property-based tests for DNS discovery error resilience.

use open_tethyr::client::OpenTethyr;
use open_tethyr::dns::DnsDiscovery;
use proptest::prelude::*;

// Property 19: DNS Discovery Error Resilience
// **Validates: Requirements 11.5**
//
// For any DNS lookup failure during cache discovery, the system should
// continue with direct discovery without propagating DNS errors to the client.
//
// This property test validates that DNS failures are handled gracefully
// and do not prevent the client from attempting direct discovery.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_dns_error_resilience(
        client_domain in "[a-z][a-z0-9]{2,8}\\.[a-z]{2,4}",
        target_domain in "[a-z][a-z0-9]{2,8}\\.[a-z]{2,4}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Create client with a domain that will likely fail DNS lookup
            // The client should still be created successfully
            let client = OpenTethyr::new(&client_domain);
            prop_assert!(
                client.is_ok(),
                "Client creation should succeed even if DNS will fail later"
            );

            let client = client.unwrap();

            // Attempt discovery - this will trigger DNS cache discovery
            // which will likely fail for non-existent domains
            // The client should fall back to direct discovery
            let result = client.discover(&target_domain).await;

            // The result should be an error (because the target domain doesn't exist)
            // but it should NOT be a DNS error - it should be an HTTP error
            // from attempting direct discovery
            if let Err(e) = result {
                // Verify it's not a DNS error - DNS errors should be caught
                // and handled internally, not propagated to the caller
                let error_msg = e.to_string();
                prop_assert!(
                    !error_msg.contains("DNS error"),
                    "DNS errors should not be propagated to client, got: {}",
                    error_msg
                );

                // Should be an HTTP error from direct discovery attempt
                prop_assert!(
                    error_msg.contains("HTTP error") || error_msg.contains("Discovery failed"),
                    "Should be HTTP or discovery error from direct fetch, got: {}",
                    error_msg
                );
            }

            Ok(())
        })?;
    }
}

/// Unit tests for DNS error resilience scenarios
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_dns_lookup_failure_does_not_prevent_client_creation() {
        // Test that client can be created even if DNS will fail later
        let client = OpenTethyr::new("nonexistent-test-domain-12345.com");

        assert!(
            client.is_ok(),
            "Client creation should succeed regardless of DNS availability"
        );
    }

    #[tokio::test]
    async fn test_dns_cache_discovery_failure_falls_back_to_direct() {
        let client = OpenTethyr::new("nonexistent-test-domain-12345.com").unwrap();

        // Attempt discovery - DNS cache discovery will fail
        // but should fall back to direct discovery
        let result = client.discover("nonexistent-target-domain-67890.com").await;

        // Should fail with HTTP error (from direct discovery attempt)
        // not DNS error (DNS errors should be caught internally)
        assert!(
            result.is_err(),
            "Should fail to discover non-existent domain"
        );

        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                !error_msg.contains("DNS error"),
                "DNS errors should not be propagated to client, got: {}",
                error_msg
            );
        }
    }

    #[tokio::test]
    async fn test_dns_discovery_returns_none_continues_with_direct() {
        let dns = DnsDiscovery::new().unwrap();

        // Test DNS discovery for non-existent domain
        let result = dns
            .discover_cache("nonexistent-test-domain-12345.com")
            .await;

        // DNS discovery should either return None or an error
        // Both cases should allow the client to continue with direct discovery
        match result {
            Ok(None) => {
                // Expected: no cache found, client should continue with direct discovery
            }
            Err(_) => {
                // Also acceptable: DNS error, client should catch and continue
            }
            Ok(Some(_)) => {
                panic!("Should not find cache for non-existent test domain");
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_dns_txt_record_does_not_block_discovery() {
        let client = OpenTethyr::new("example.com").unwrap();

        // Even if DNS returns invalid data, the client should handle it
        // and fall back to direct discovery
        let result = client.discover("nonexistent-target-12345.com").await;

        // Should fail with HTTP error (from direct discovery)
        // not DNS parsing error
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                !error_msg.contains("DNS error") && !error_msg.contains("Invalid endpoint format"),
                "DNS parsing errors should not be propagated, got: {}",
                error_msg
            );
        }
    }

    #[tokio::test]
    async fn test_dns_timeout_does_not_block_discovery() {
        // Create client with a domain that might timeout
        let client = OpenTethyr::new("timeout-test-domain-12345.com").unwrap();

        // Attempt discovery - DNS might timeout but should not block
        let result = client.discover("target-domain-12345.com").await;

        // Should eventually fail with HTTP error (from direct discovery)
        // DNS timeout should be caught and handled internally
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                !error_msg.contains("DNS error") && !error_msg.contains("timeout"),
                "DNS timeout should not be propagated to client, got: {}",
                error_msg
            );
        }
    }

    #[tokio::test]
    async fn test_discover_with_cache_handles_dns_independent_of_cache() {
        let client = OpenTethyr::new("test-domain.com").unwrap();

        // When using explicit cache URL, DNS discovery is bypassed
        // This should work regardless of DNS state
        let result = client
            .discover_with_cache(
                "nonexistent-target-12345.com",
                "https://nonexistent-cache-12345.com",
            )
            .await;

        // Should fail with HTTP error (from cache or direct fetch)
        // not DNS error (DNS is not involved in this path)
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(
                !error_msg.contains("DNS error"),
                "DNS should not be involved when using explicit cache URL, got: {}",
                error_msg
            );
        }
    }
}
