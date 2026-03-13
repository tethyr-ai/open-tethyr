//! Client SDK - returns flat AgentExchangeRecord per spec

use crate::ax::AgentExchangeRecord;
use crate::dns::DnsDiscovery;
use crate::error::ClientError;
use crate::http::AxHttpClient;

#[allow(dead_code)]
pub struct OpenTethyr {
    domain: String,
    http_client: AxHttpClient,
    cache_url: Option<String>,
}

impl OpenTethyr {
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
    pub async fn with_dns_discovery(domain: &str) -> Result<Self, ClientError> {
        let mut client = Self::new(domain)?;
        if let Ok(dns) = DnsDiscovery::new() {
            if let Some(url) = dns.discover_cache(domain).await {
                client.cache_url = Some(url);
            }
        }
        Ok(client)
    }
    pub async fn discover(&self, target: &str) -> Result<AgentExchangeRecord, ClientError> {
        if let Some(ref cache_url) = self.cache_url {
            if let Ok(rec) = self.http_client.fetch_from_cache(cache_url, target).await {
                return Ok(rec);
            }
        }
        self.http_client
            .fetch_ax_record(target)
            .await
            .map_err(|e| ClientError::DiscoveryFailed(target.into(), e.to_string()))
    }
    pub async fn discover_with_cache(
        &self,
        target: &str,
        cache_url: &str,
    ) -> Result<AgentExchangeRecord, ClientError> {
        self.http_client
            .fetch_from_cache(cache_url, target)
            .await
            .map_err(|e| ClientError::DiscoveryFailed(target.into(), e.to_string()))
    }
}
