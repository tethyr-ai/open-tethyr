//! HTTP Client Implementation
//!
//! HTTP client for fetching AX records from endpoints and cache servers.

use crate::ax::AgentExchangeDocument;
use reqwest::{Client, ClientBuilder};
use std::time::Duration;
use thiserror::Error;
use tracing::debug;
use url::Url;

/// HTTP client errors
#[derive(Debug, Error)]
pub enum HttpError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid well-known path: expected '/.well-known/agent-exchange.json', got '{0}'")]
    InvalidWellKnownPath(String),

    #[error("JSON parsing failed: {0}")]
    JsonParsingFailed(String),

    #[error("HTTPS certificate validation failed: {0}")]
    CertificateValidationFailed(String),

    #[error("Request timeout after {0:?}")]
    Timeout(Duration),

    #[error("HTTP error {status}: {message}")]
    HttpStatus { status: u16, message: String },
}

/// HTTP client for fetching AX records
pub struct AxHttpClient {
    client: Client,
    timeout: Duration,
}

impl AxHttpClient {
    /// Create a new HTTP client with default timeout (30 seconds)
    pub fn new() -> Result<Self, HttpError> {
        Self::with_timeout(Duration::from_secs(30))
    }

    /// Create a new HTTP client with custom timeout
    pub fn with_timeout(timeout: Duration) -> Result<Self, HttpError> {
        let client = ClientBuilder::new()
            .timeout(timeout)
            .use_rustls_tls() // Use rustls for HTTPS certificate validation
            .build()
            .map_err(HttpError::RequestFailed)?;

        Ok(Self { client, timeout })
    }

    /// Fetch AX record directly from domain's well-known endpoint
    pub async fn fetch_ax_record(&self, domain: &str) -> Result<AgentExchangeDocument, HttpError> {
        let url = self.build_ax_url(domain)?;
        debug!("Fetching AX record from: {}", url);

        self.fetch_from_url(&url).await
    }

    /// Fetch AX record from cache server
    pub async fn fetch_from_cache(
        &self,
        cache_url: &str,
        domain: &str,
    ) -> Result<AgentExchangeDocument, HttpError> {
        // Cache URL format: https://cache.example.com/discover/{domain}
        let url = format!("{}/discover/{}", cache_url.trim_end_matches('/'), domain);
        debug!("Fetching AX record from cache: {}", url);

        self.fetch_from_url(&url).await
    }

    /// Build AX URL for direct discovery: https://_agent.<domain>/.well-known/agent-exchange.json
    pub fn build_ax_url(&self, domain: &str) -> Result<String, HttpError> {
        // Validate domain format (basic validation)
        if domain.is_empty()
            || domain.contains("..")
            || domain.starts_with('.')
            || domain.ends_with('.')
        {
            return Err(HttpError::InvalidUrl(format!("Invalid domain: {}", domain)));
        }

        let url = format!("https://_agent.{}/.well-known/agent-exchange.json", domain);
        debug!("Built AX URL: {}", url);
        Ok(url)
    }

    /// Fetch and parse AX document from URL
    async fn fetch_from_url(&self, url: &str) -> Result<AgentExchangeDocument, HttpError> {
        // Parse and validate URL
        let parsed_url =
            Url::parse(url).map_err(|_| HttpError::InvalidUrl(format!("Invalid URL: {}", url)))?;

        // Validate HTTPS
        if parsed_url.scheme() != "https" {
            return Err(HttpError::InvalidUrl(format!(
                "URL must use HTTPS: {}",
                url
            )));
        }

        // For direct AX endpoints, validate well-known path
        if url.contains("_agent.") {
            self.validate_well_known_path(&parsed_url)?;
        }

        debug!("Making HTTP request to: {}", url);

        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .header("User-Agent", "open-tethyr/0.1.0")
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    HttpError::Timeout(self.timeout)
                } else if e.is_connect() || e.is_request() {
                    HttpError::CertificateValidationFailed(e.to_string())
                } else {
                    HttpError::RequestFailed(e)
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text: String = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HttpError::HttpStatus {
                status: status.as_u16(),
                message: error_text,
            });
        }

        debug!("HTTP request successful, status: {}", status);

        let text: String = response.text().await.map_err(HttpError::RequestFailed)?;
        debug!("Response body length: {} bytes", text.len());

        // Parse JSON response
        let document: AgentExchangeDocument = serde_json::from_str(&text)
            .map_err(|e| HttpError::JsonParsingFailed(format!("Failed to parse JSON: {}", e)))?;

        debug!(
            "Successfully parsed AX document with {} records",
            document.records.len()
        );
        Ok(document)
    }

    /// Validate that the URL uses the correct well-known path
    fn validate_well_known_path(&self, url: &Url) -> Result<(), HttpError> {
        if url.path() != "/.well-known/agent-exchange.json" {
            return Err(HttpError::InvalidWellKnownPath(url.path().to_string()));
        }
        Ok(())
    }
}

impl Default for AxHttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ax_url_valid() {
        let client = AxHttpClient::default();
        let result = client.build_ax_url("example.com");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "https://_agent.example.com/.well-known/agent-exchange.json"
        );
    }

    #[test]
    fn test_build_ax_url_invalid_domain() {
        let client = AxHttpClient::default();

        // Test empty domain
        let result = client.build_ax_url("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HttpError::InvalidUrl(_)));

        // Test domain with double dots
        let result = client.build_ax_url("example..com");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HttpError::InvalidUrl(_)));

        // Test domain starting with dot
        let result = client.build_ax_url(".example.com");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HttpError::InvalidUrl(_)));

        // Test domain ending with dot
        let result = client.build_ax_url("example.com.");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HttpError::InvalidUrl(_)));
    }

    #[test]
    fn test_validate_well_known_path_valid() {
        let client = AxHttpClient::default();
        let url = Url::parse("https://_agent.example.com/.well-known/agent-exchange.json").unwrap();
        let result = client.validate_well_known_path(&url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_well_known_path_invalid() {
        let client = AxHttpClient::default();
        let url = Url::parse("https://_agent.example.com/invalid/path").unwrap();
        let result = client.validate_well_known_path(&url);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HttpError::InvalidWellKnownPath(_)
        ));
    }

    #[test]
    fn test_timeout_configuration() {
        let timeout = Duration::from_secs(10);
        let client = AxHttpClient::with_timeout(timeout);
        assert!(client.is_ok());
        assert_eq!(client.unwrap().timeout, timeout);
    }
}
