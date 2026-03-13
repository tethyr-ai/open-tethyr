//! HTTP Client for AX endpoint fetching

use crate::ax::AgentExchangeDocument;
use crate::error::HttpError;
use std::time::Duration;

/// HTTP client for fetching AX records
pub struct AxHttpClient {
    client: reqwest::Client,
    timeout: Duration,
}

impl AxHttpClient {
    /// Create a new HTTP client with configurable timeout (default 30s)
    pub fn new(timeout_secs: Option<u64>) -> Result<Self, HttpError> {
        let timeout = Duration::from_secs(timeout_secs.unwrap_or(30));
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| HttpError::RequestFailed(e.to_string()))?;
        Ok(Self { client, timeout })
    }

    /// Build the AX URL for a domain
    pub fn build_ax_url(domain: &str) -> String {
        format!("https://_agent.{}/.well-known/agent-exchange.json", domain)
    }

    /// Validate that a path is the well-known AX path
    pub fn validate_well_known_path(path: &str) -> Result<(), HttpError> {
        if !path.ends_with("/.well-known/agent-exchange.json") {
            return Err(HttpError::InvalidWellKnownPath(path.to_string()));
        }
        Ok(())
    }

    /// Fetch AX record from a domain
    pub async fn fetch_ax_record(&self, domain: &str) -> Result<AgentExchangeDocument, HttpError> {
        let url = Self::build_ax_url(domain);
        self.fetch_from_url(&url).await
    }

    /// Fetch AX record from a cache server
    pub async fn fetch_from_cache(
        &self,
        cache_url: &str,
        domain: &str,
    ) -> Result<AgentExchangeDocument, HttpError> {
        let url = format!("{}/discover/{}", cache_url.trim_end_matches('/'), domain);
        self.fetch_from_url(&url).await
    }

    async fn fetch_from_url(&self, url: &str) -> Result<AgentExchangeDocument, HttpError> {
        let response = self.client.get(url).send().await.map_err(|e| {
            if e.is_timeout() {
                HttpError::Timeout(self.timeout.as_secs())
            } else {
                HttpError::RequestFailed(e.to_string())
            }
        })?;

        if !response.status().is_success() {
            return Err(HttpError::InvalidResponse(
                url.to_string(),
                format!("HTTP {}", response.status()),
            ));
        }

        response
            .json::<AgentExchangeDocument>()
            .await
            .map_err(|e| HttpError::InvalidResponse(url.to_string(), e.to_string()))
    }
}
