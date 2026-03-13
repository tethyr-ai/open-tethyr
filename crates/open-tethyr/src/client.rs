//! Client SDK for agent discovery

use crate::ax::AgentExchangeDocument;
use crate::dns::DnsDiscovery;
use crate::error::ClientError;
use crate::http::AxHttpClient;

/// Client SDK for agent discovery
#[allow(dead_code)]
pub struct OpenTethyr {
    domain: String,
    http_client: AxHttpClient,
    cache_url: Option<String>,
}

impl OpenTethyr {
    /// Create a new client for the given domain with automatic DNS cache discovery
    pub fn new(domain: &str) -> Result<Self, ClientError> {
        if domain.is_empty() {
            return Err(ClientError::InvalidDomain("Domain cannot be empty".into()));
        }
        let http_client = AxHttpClient::new(Some(30))
            .map_err(|e| ClientError::DiscoveryFailed(domain.into(), e.to_string()))?;

        Ok(Self {
            domain: domain.to_string(),
            http_client,
            cache_url: None,
        })
    }

    /// Initialize with DNS cache discovery
    pub async fn with_dns_discovery(domain: &str) -> Result<Self, ClientError> {
        let mut client = Self::new(domain)?;

        // Try DNS-based cache discovery
        if let Ok(dns) = DnsDiscovery::new() {
            if let Some(cache_url) = dns.discover_cache(domain).await {
                tracing::info!("Discovered cache at {} for domain {}", cache_url, domain);
                client.cache_url = Some(cache_url);
            }
        }

        Ok(client)
    }

    /// Discover agents from target domain (cache-first, then direct)
    pub async fn discover(&self, target_domain: &str) -> Result<AgentExchangeDocument, ClientError> {
        // Try cache first
        if let Some(ref cache_url) = self.cache_url {
            match self.http_client.fetch_from_cache(cache_url, target_domain).await {
                Ok(doc) => return Ok(doc),
                Err(e) => {
                    tracing::warn!("Cache fetch failed, falling back to direct: {}", e);
                }
            }
        }

        // Direct fetch
        self.http_client.fetch_ax_record(target_domain).await
            .map_err(|e| ClientError::DiscoveryFailed(target_domain.into(), e.to_string()))
    }

    /// Discover agents using explicit cache URL
    pub async fn discover_with_cache(
        &self,
        target_domain: &str,
        cache_url: &str,
    ) -> Result<AgentExchangeDocument, ClientError> {
        self.http_client.fetch_from_cache(cache_url, target_domain).await
            .map_err(|e| ClientError::DiscoveryFailed(target_domain.into(), e.to_string()))
    }
}
