//! Property 18: DNS TXT Record Parsing Correctness

use open_tethyr::dns::DnsDiscovery;

#[test]
fn valid_endpoint_format_parsed() {
    assert_eq!(
        DnsDiscovery::parse_cache_endpoint("endpoint=https://cache.example.com"),
        Some("https://cache.example.com".to_string())
    );
}

#[test]
fn quoted_endpoint_parsed() {
    assert_eq!(
        DnsDiscovery::parse_cache_endpoint("\"endpoint=https://cache.example.com\""),
        Some("https://cache.example.com".to_string())
    );
}

#[test]
fn malformed_record_rejected() {
    assert_eq!(DnsDiscovery::parse_cache_endpoint("not-an-endpoint"), None);
    assert_eq!(DnsDiscovery::parse_cache_endpoint("endpoint="), None);
    assert_eq!(DnsDiscovery::parse_cache_endpoint(""), None);
}

#[test]
fn whitespace_handled() {
    assert_eq!(
        DnsDiscovery::parse_cache_endpoint("  endpoint=https://cache.example.com  "),
        Some("https://cache.example.com".to_string())
    );
}
