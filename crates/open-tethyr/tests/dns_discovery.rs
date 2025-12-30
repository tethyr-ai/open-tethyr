//! DNS Discovery Property-Based Tests
//!
//! Property-based tests for DNS TXT record parsing correctness.

use open_tethyr::dns::{DnsDiscovery, DnsError};
use proptest::prelude::*;

/// Property 18: DNS TXT Record Parsing Correctness
/// **Validates: Requirements 11.4, 11.6**
///
/// For any valid TXT record in the format "endpoint=<https_url>",
/// parsing should succeed and return the correct URL.
/// For any invalid TXT record format, parsing should fail with appropriate error.
#[test]
fn prop_dns_txt_record_parsing_correctness() {
    proptest!(|(
        valid_domain in "[a-z][a-z0-9-]{0,10}\\.[a-z]{2,4}",
        valid_path in "[a-z0-9/-]{0,20}",
        invalid_format in "[^e][a-z0-9=:/.-]{0,50}",
        non_https_scheme in "(http|ftp|ws)://[a-z0-9.-]+",
    )| {
        let dns = DnsDiscovery::default();

        // Test valid endpoint format
        let valid_record = format!("endpoint=https://{}{}", valid_domain, valid_path);
        let result = dns.parse_cache_endpoint(&valid_record);
        prop_assert!(result.is_ok(), "Valid TXT record should parse successfully: {}", valid_record);

        if let Ok(url) = result {
            prop_assert!(url.starts_with("https://"), "Parsed URL should be HTTPS: {}", url);
            prop_assert!(url.contains(&valid_domain), "Parsed URL should contain domain: {}", url);
        }

        // Test invalid format (doesn't start with "endpoint=")
        if !invalid_format.starts_with("endpoint=") {
            let result = dns.parse_cache_endpoint(&invalid_format);
            prop_assert!(result.is_err(), "Invalid format should fail: {}", invalid_format);
            prop_assert!(matches!(result.unwrap_err(), DnsError::InvalidFormat(_)));
        }

        // Test non-HTTPS schemes
        let non_https_record = format!("endpoint={}", non_https_scheme);
        let result = dns.parse_cache_endpoint(&non_https_record);
        prop_assert!(result.is_err(), "Non-HTTPS endpoint should fail: {}", non_https_record);
        prop_assert!(matches!(result.unwrap_err(), DnsError::InvalidFormat(_)));
    });
}

/// Unit tests for specific DNS TXT record parsing scenarios
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_valid_txt_record_formats() {
        let dns = DnsDiscovery::default();

        let test_cases = vec![
            "endpoint=https://cache.example.com",
            "endpoint=https://cache.example.com/ax",
            "endpoint=https://cache.example.com:8443/discover",
            "endpoint=https://ax-cache.internal.corp.com/api/v1",
        ];

        for record in test_cases {
            let result = dns.parse_cache_endpoint(record);
            assert!(result.is_ok(), "Should parse valid record: {}", record);

            let url = result.unwrap();
            assert!(url.starts_with("https://"), "URL should be HTTPS: {}", url);
        }
    }

    #[test]
    fn test_invalid_txt_record_formats() {
        let dns = DnsDiscovery::default();

        let test_cases = vec![
            "cache=https://cache.example.com",   // Wrong prefix
            "endpoint=http://cache.example.com", // HTTP instead of HTTPS
            "endpoint=ftp://cache.example.com",  // FTP protocol
            "endpoint=not-a-url",                // Invalid URL
            "endpoint=",                         // Empty URL
            "random text",                       // No endpoint prefix
            "",                                  // Empty string
        ];

        for record in test_cases {
            let result = dns.parse_cache_endpoint(record);
            assert!(result.is_err(), "Should reject invalid record: {}", record);
            assert!(matches!(result.unwrap_err(), DnsError::InvalidFormat(_)));
        }
    }

    #[test]
    fn test_edge_case_txt_records() {
        let dns = DnsDiscovery::default();

        // Test with extra whitespace (should be handled by caller, but test anyway)
        let result = dns.parse_cache_endpoint("endpoint=https://cache.example.com ");
        assert!(result.is_ok(), "Should handle trailing space in URL");

        // Test with port numbers
        let result = dns.parse_cache_endpoint("endpoint=https://cache.example.com:443");
        assert!(result.is_ok(), "Should handle explicit port numbers");

        // Test with query parameters
        let result = dns.parse_cache_endpoint("endpoint=https://cache.example.com/ax?version=1.0");
        assert!(result.is_ok(), "Should handle query parameters");

        // Test with fragments
        let result = dns.parse_cache_endpoint("endpoint=https://cache.example.com/ax#section");
        assert!(result.is_ok(), "Should handle URL fragments");
    }
}
