//! Test Helper Functions
//!
//! This module provides helper functions and utilities for testing.

use crate::ax::AgentExchangeDocument;
use crate::dns::DnsDiscovery;
use crate::http::AxHttpClient;
use crate::test_utils::{MockDnsResolver, MockHttpServer};
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

/// Test environment that combines all test utilities
pub struct TestEnvironment {
    pub mock_dns: MockDnsResolver,
    pub mock_http: MockHttpServer,
    pub temp_dir: TempDir,
    pub http_client: AxHttpClient,
}

impl TestEnvironment {
    /// Create a new test environment with all utilities initialized
    pub async fn new() -> Self {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpServer::new().await;
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let http_client = AxHttpClient::with_timeout(Duration::from_secs(5))
            .expect("Failed to create HTTP client");

        Self {
            mock_dns,
            mock_http,
            temp_dir,
            http_client,
        }
    }

    /// Get the temporary directory path
    pub fn temp_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Reset all mocks to clean state
    pub async fn reset(&self) {
        self.mock_dns.clear();
        self.mock_http.reset().await;
    }

    /// Setup a complete mock scenario for successful discovery
    pub async fn setup_successful_discovery(&self, domain: &str, ax_record_json: &str) {
        // Setup DNS to return cache endpoint
        let cache_url = self.mock_http.uri();
        self.mock_dns.add_cache_endpoint(domain, &cache_url);

        // Setup HTTP server to return AX record
        self.mock_http
            .mock_cache_discovery_success(domain, ax_record_json)
            .await;
    }

    /// Setup a mock scenario for direct discovery (no cache)
    pub async fn setup_direct_discovery(&self, domain: &str, ax_record_json: &str) {
        // No DNS cache endpoint configured
        // Setup HTTP server to return AX record for direct endpoint
        let path = format!("/_agent.{}/.well-known/agent-exchange.json", domain);
        self.mock_http
            .mock_get_regex(&path, 200, ax_record_json)
            .await;
    }

    /// Setup a mock scenario for DNS failure
    pub async fn setup_dns_failure(&self, _domain: &str) {
        // Don't add any DNS records - this will cause lookup failure
        // The mock DNS resolver will return an error for unknown domains
    }

    /// Setup a mock scenario for HTTP failure
    pub async fn setup_http_failure(&self, domain: &str, status_code: u16, error_message: &str) {
        let cache_url = self.mock_http.uri();
        self.mock_dns.add_cache_endpoint(domain, &cache_url);

        let path = format!("/discover/{}", domain);
        self.mock_http
            .mock_http_error(&path, status_code, error_message)
            .await;
    }
}

/// Helper function to run async tests with timeout
pub async fn run_with_timeout<F, T>(future: F, timeout_duration: Duration) -> Result<T, String>
where
    F: std::future::Future<Output = T>,
{
    match timeout(timeout_duration, future).await {
        Ok(result) => Ok(result),
        Err(_) => Err(format!("Test timed out after {:?}", timeout_duration)),
    }
}

/// Helper function to assert that two AX documents are equivalent
pub fn assert_ax_documents_equal(actual: &AgentExchangeDocument, expected: &AgentExchangeDocument) {
    assert_eq!(
        actual.records.len(),
        expected.records.len(),
        "Documents have different number of records"
    );

    for (i, (actual_record, expected_record)) in actual
        .records
        .iter()
        .zip(expected.records.iter())
        .enumerate()
    {
        assert_eq!(
            actual_record.record_type, expected_record.record_type,
            "Record {} has different record_type",
            i
        );
        assert_eq!(
            actual_record.version, expected_record.version,
            "Record {} has different version",
            i
        );
        assert_eq!(
            actual_record.agent.name, expected_record.agent.name,
            "Record {} has different agent name",
            i
        );
        assert_eq!(
            actual_record.agent.description, expected_record.agent.description,
            "Record {} has different agent description",
            i
        );
        assert_eq!(
            actual_record.agent.provider, expected_record.agent.provider,
            "Record {} has different agent provider",
            i
        );
        assert_eq!(
            actual_record.endpoints.len(),
            expected_record.endpoints.len(),
            "Record {} has different number of endpoints",
            i
        );
    }
}

/// Helper function to create a test DNS discovery instance with mock resolver
pub fn create_test_dns_discovery(_mock_resolver: &MockDnsResolver) -> DnsDiscovery {
    // Note: This would require modifying DnsDiscovery to accept a custom resolver
    // For now, we'll use the default and rely on integration testing
    DnsDiscovery::default()
}

/// Helper function to validate AX record structure
pub fn validate_ax_record_structure(document: &AgentExchangeDocument) -> Result<(), String> {
    if document.records.is_empty() {
        return Err("Document has no records".to_string());
    }

    for (i, record) in document.records.iter().enumerate() {
        if record.record_type != "AX" {
            return Err(format!(
                "Record {} has invalid record_type: {}",
                i, record.record_type
            ));
        }

        if record.version != "1.0" {
            return Err(format!(
                "Record {} has invalid version: {}",
                i, record.version
            ));
        }

        if record.agent.name.is_empty() {
            return Err(format!("Record {} has empty agent name", i));
        }

        if record.agent.description.is_empty() {
            return Err(format!("Record {} has empty agent description", i));
        }

        if record.agent.provider.is_empty() {
            return Err(format!("Record {} has empty agent provider", i));
        }

        if record.endpoints.is_empty() {
            return Err(format!("Record {} has no endpoints", i));
        }

        for (j, endpoint) in record.endpoints.iter().enumerate() {
            if endpoint.url.is_empty() {
                return Err(format!("Record {} endpoint {} has empty URL", i, j));
            }

            if endpoint.auth.is_empty() {
                return Err(format!("Record {} endpoint {} has no auth methods", i, j));
            }
        }
    }

    Ok(())
}

/// Helper function to create test file in temp directory
pub fn create_test_file(temp_dir: &TempDir, filename: &str, content: &str) -> std::path::PathBuf {
    let file_path = temp_dir.path().join(filename);
    std::fs::write(&file_path, content).expect("Failed to write test file");
    file_path
}

/// Helper function to read file content as string
pub fn read_file_content(path: &std::path::Path) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

/// Helper function to create a directory structure for testing
pub fn create_test_directory_structure(
    temp_dir: &TempDir,
    structure: &[&str],
) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();

    for path_str in structure {
        let path = temp_dir.path().join(path_str);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent directories");
        }

        if path_str.ends_with('/') {
            // It's a directory
            std::fs::create_dir_all(&path).expect("Failed to create directory");
        } else {
            // It's a file
            std::fs::write(&path, "").expect("Failed to create file");
        }

        paths.push(path);
    }

    paths
}

/// Helper function to assert that a file exists and has expected content
pub fn assert_file_content(path: &std::path::Path, expected_content: &str) {
    assert!(path.exists(), "File does not exist: {:?}", path);
    let actual_content = read_file_content(path).expect("Failed to read file");
    assert_eq!(actual_content.trim(), expected_content.trim());
}

/// Helper function to assert that a directory exists and has expected structure
pub fn assert_directory_structure(base_path: &std::path::Path, expected_files: &[&str]) {
    assert!(
        base_path.exists(),
        "Base directory does not exist: {:?}",
        base_path
    );
    assert!(
        base_path.is_dir(),
        "Path is not a directory: {:?}",
        base_path
    );

    for expected_file in expected_files {
        let file_path = base_path.join(expected_file);
        assert!(
            file_path.exists(),
            "Expected file does not exist: {:?}",
            file_path
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::fixtures::*;

    #[tokio::test]
    async fn test_test_environment_creation() {
        let env = TestEnvironment::new().await;
        assert!(!env.mock_http.uri().is_empty());
        assert!(env.temp_path().exists());
    }

    #[tokio::test]
    async fn test_test_environment_reset() {
        let env = TestEnvironment::new().await;

        // Add some data
        env.mock_dns
            .add_cache_endpoint("example.com", "https://cache.example.com");
        env.mock_http
            .mock_ax_record_success("example.com", &minimal_ax_record_json())
            .await;

        // Reset should clear everything
        env.reset().await;

        // Verify DNS is cleared
        assert!(env.mock_dns.get_all_records().is_empty());
    }

    #[tokio::test]
    async fn test_run_with_timeout_success() {
        let result = run_with_timeout(async { "success" }, Duration::from_secs(1)).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_run_with_timeout_failure() {
        let result = run_with_timeout(
            async {
                tokio::time::sleep(Duration::from_secs(2)).await;
                "should not reach here"
            },
            Duration::from_millis(100),
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timed out"));
    }

    #[test]
    fn test_assert_ax_documents_equal() {
        let doc1 = single_record_document();
        let doc2 = single_record_document();

        // Should not panic
        assert_ax_documents_equal(&doc1, &doc2);
    }

    #[test]
    fn test_validate_ax_record_structure_valid() {
        let document = single_record_document();
        let result = validate_ax_record_structure(&document);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_ax_record_structure_empty() {
        let document = empty_document();
        let result = validate_ax_record_structure(&document);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no records"));
    }

    #[test]
    fn test_create_test_file() {
        let temp_dir = TempDir::new().unwrap();
        let content = "test content";
        let file_path = create_test_file(&temp_dir, "test.txt", content);

        assert!(file_path.exists());
        let read_content = read_file_content(&file_path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_create_test_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let structure = &[
            "dir1/",
            "dir1/file1.txt",
            "dir2/",
            "dir2/subdir/",
            "dir2/subdir/file2.txt",
            "root_file.txt",
        ];

        let paths = create_test_directory_structure(&temp_dir, structure);
        assert_eq!(paths.len(), structure.len());

        for path in paths {
            assert!(path.exists());
        }
    }

    #[test]
    fn test_assert_file_content() {
        let temp_dir = TempDir::new().unwrap();
        let content = "expected content";
        let file_path = create_test_file(&temp_dir, "test.txt", content);

        // Should not panic
        assert_file_content(&file_path, content);
    }

    #[test]
    fn test_assert_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let structure = &["file1.txt", "subdir/", "subdir/file2.txt"];
        create_test_directory_structure(&temp_dir, structure);

        // Should not panic
        assert_directory_structure(temp_dir.path(), &["file1.txt", "subdir"]);
    }
}
