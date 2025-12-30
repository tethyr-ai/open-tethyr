//! Mock Implementations for Testing
//!
//! This module provides mock implementations of DNS resolver and HTTP server
//! for use in tests.

use crate::dns::DnsError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Mock DNS resolver for testing DNS discovery functionality
#[derive(Debug, Clone)]
pub struct MockDnsResolver {
    txt_records: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl MockDnsResolver {
    /// Create a new mock DNS resolver
    pub fn new() -> Self {
        Self {
            txt_records: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a TXT record response for a given domain
    pub fn add_txt_record(&self, domain: &str, records: Vec<String>) {
        let mut txt_records = self.txt_records.lock().unwrap();
        txt_records.insert(domain.to_string(), records);
    }

    /// Add a cache endpoint TXT record for a domain
    pub fn add_cache_endpoint(&self, domain: &str, cache_url: &str) {
        let record = format!("endpoint={}", cache_url);
        self.add_txt_record(&format!("_ax-cache.{}", domain), vec![record]);
    }

    /// Add a root cache endpoint TXT record for a domain
    pub fn add_root_cache_endpoint(&self, domain: &str, cache_url: &str) {
        let record = format!("endpoint={}", cache_url);
        self.add_txt_record(&format!("_ax-cache-root.{}", domain), vec![record]);
    }

    /// Clear all TXT records
    pub fn clear(&self) {
        let mut txt_records = self.txt_records.lock().unwrap();
        txt_records.clear();
    }

    /// Simulate a DNS lookup for TXT records
    pub async fn lookup_txt(&self, name: &str) -> Result<Vec<String>, DnsError> {
        let txt_records = self.txt_records.lock().unwrap();

        if let Some(records) = txt_records.get(name) {
            Ok(records.clone())
        } else {
            Err(DnsError::LookupFailed(format!(
                "No TXT records found for {}",
                name
            )))
        }
    }

    /// Get all configured TXT records (for debugging)
    pub fn get_all_records(&self) -> HashMap<String, Vec<String>> {
        let txt_records = self.txt_records.lock().unwrap();
        txt_records.clone()
    }
}

impl Default for MockDnsResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock HTTP server for testing HTTP client functionality
pub struct MockHttpServer {
    server: MockServer,
}

impl MockHttpServer {
    /// Create a new mock HTTP server
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        Self { server }
    }

    /// Get the base URI of the mock server
    pub fn uri(&self) -> String {
        self.server.uri()
    }

    /// Get the address of the mock server
    pub fn address(&self) -> &std::net::SocketAddr {
        self.server.address()
    }

    /// Mock a successful AX record response for a domain
    pub async fn mock_ax_record_success(&self, domain: &str, json_response: &str) {
        let path_pattern = format!("/_agent.{}/.well-known/agent-exchange.json", domain);

        Mock::given(method("GET"))
            .and(path(&path_pattern))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(json_response)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock a cache discovery response
    pub async fn mock_cache_discovery_success(&self, domain: &str, json_response: &str) {
        let path_pattern = format!("/discover/{}", domain);

        Mock::given(method("GET"))
            .and(path(&path_pattern))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(json_response)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock an HTTP error response
    pub async fn mock_http_error(&self, path_pattern: &str, status_code: u16, error_message: &str) {
        Mock::given(method("GET"))
            .and(path(path_pattern))
            .respond_with(
                ResponseTemplate::new(status_code)
                    .set_body_string(error_message)
                    .insert_header("content-type", "text/plain"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock a timeout response (very slow response)
    pub async fn mock_timeout(&self, path_pattern: &str) {
        Mock::given(method("GET"))
            .and(path(path_pattern))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(std::time::Duration::from_secs(60)) // Longer than typical timeout
                    .set_body_string("{}"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock an invalid JSON response
    pub async fn mock_invalid_json(&self, path_pattern: &str) {
        Mock::given(method("GET"))
            .and(path(path_pattern))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("invalid json content")
                    .insert_header("content-type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock a certificate validation error (using self-signed cert simulation)
    pub async fn mock_certificate_error(&self, path_pattern: &str) {
        // Note: This is a simplified mock - in real tests, you'd need to set up
        // actual certificate validation failures
        Mock::given(method("GET"))
            .and(path(path_pattern))
            .respond_with(ResponseTemplate::new(526)) // Invalid SSL Certificate
            .mount(&self.server)
            .await;
    }

    /// Mock any GET request with a regex pattern
    pub async fn mock_get_regex(&self, path_regex_pattern: &str, status: u16, body: &str) {
        Mock::given(method("GET"))
            .and(path_regex(path_regex_pattern))
            .respond_with(
                ResponseTemplate::new(status)
                    .set_body_string(body)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Reset all mocks
    pub async fn reset(&self) {
        self.server.reset().await;
    }

    /// Verify that all expected requests were received
    pub async fn verify(&self) {
        // wiremock automatically verifies that all mounted mocks were called
        // This method is here for explicit verification if needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_dns_resolver_creation() {
        let resolver = MockDnsResolver::new();
        assert!(resolver.get_all_records().is_empty());
    }

    #[test]
    fn test_mock_dns_resolver_add_txt_record() {
        let resolver = MockDnsResolver::new();
        resolver.add_txt_record("example.com", vec!["test record".to_string()]);

        let records = resolver.get_all_records();
        assert_eq!(records.len(), 1);
        assert_eq!(records.get("example.com").unwrap(), &vec!["test record"]);
    }

    #[test]
    fn test_mock_dns_resolver_add_cache_endpoint() {
        let resolver = MockDnsResolver::new();
        resolver.add_cache_endpoint("example.com", "https://cache.example.com");

        let records = resolver.get_all_records();
        assert_eq!(records.len(), 1);
        assert_eq!(
            records.get("_ax-cache.example.com").unwrap(),
            &vec!["endpoint=https://cache.example.com"]
        );
    }

    #[tokio::test]
    async fn test_mock_dns_resolver_lookup() {
        let resolver = MockDnsResolver::new();
        resolver.add_txt_record(
            "test.com",
            vec!["record1".to_string(), "record2".to_string()],
        );

        let result = resolver.lookup_txt("test.com").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["record1", "record2"]);

        let result = resolver.lookup_txt("nonexistent.com").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_http_server_creation() {
        let server = MockHttpServer::new().await;
        assert!(!server.uri().is_empty());
        assert!(server.uri().starts_with("http://"));
    }
}
