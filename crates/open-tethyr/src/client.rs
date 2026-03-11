//! Client SDK for agent discovery
//!
//! This module provides the main client interface for discovering agents.
//! The client automatically discovers cache endpoints via DNS and falls back
//! to direct discovery if no cache is available.

use crate::ax::Agent;
use crate::dns::DnsDiscovery;
use crate::http::AxHttpClient;
use tracing::{debug, info, warn};

/// Client SDK for agent discovery
pub struct OpenTethyr {
    domain: String,
    cache_url: Option<String>,
    http_client: AxHttpClient,
    dns_discovery: DnsDiscovery,
}

/// Client errors
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Discovery failed: {0}")]
    DiscoveryFailed(String),

    #[error("Invalid domain: {0}")]
    InvalidDomain(String),

    #[error("HTTP error: {0}")]
    HttpError(#[from] crate::http::HttpError),

    #[error("DNS error: {0}")]
    DnsError(#[from] crate::dns::DnsError),
}

impl OpenTethyr {
    /// Create a new client for the given domain with automatic cache discovery
    ///
    /// This will perform DNS lookup for _ax-cache.<domain> to discover
    /// organizational cache endpoints. If no cache is found, the client
    /// will fall back to direct HTTPS discovery.
    ///
    /// # Arguments
    ///
    /// * `domain` - The client's home domain (e.g., "acme.com")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use open_tethyr::OpenTethyr;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenTethyr::new("acme.com")?;
    ///     let agents = client.discover("api.partner.com").await?;
    ///     println!("Found {} agents", agents.len());
    ///     Ok(())
    /// }
    /// ```
    pub fn new(domain: &str) -> Result<Self, ClientError> {
        if domain.is_empty() {
            return Err(ClientError::InvalidDomain(
                "Domain cannot be empty".to_string(),
            ));
        }

        let http_client = AxHttpClient::new()?;
        let dns_discovery = DnsDiscovery::new()?;

        Ok(Self {
            domain: domain.to_string(),
            cache_url: None,
            http_client,
            dns_discovery,
        })
    }

    /// Discover agents from target domain using automatic cache discovery
    ///
    /// This method will:
    /// 1. Check if a cache URL was discovered during initialization
    /// 2. If cache is available, query the cache for the target domain
    /// 3. If no cache or cache fails, fall back to direct HTTPS discovery
    ///
    /// # Arguments
    ///
    /// * `target_domain` - The domain to discover agents from (e.g., "api.partner.com")
    ///
    /// # Returns
    ///
    /// A vector of discovered agents
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use open_tethyr::OpenTethyr;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OpenTethyr::new("acme.com")?;
    /// let agents = client.discover("api.partner.com").await?;
    /// for agent in agents {
    ///     println!("Agent: {} - {}", agent.name, agent.description);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover(&self, target_domain: &str) -> Result<Vec<Agent>, ClientError> {
        info!("Discovering agents for domain: {}", target_domain);

        // First, try to discover cache endpoint via DNS if not already cached
        let cache_url = if self.cache_url.is_some() {
            self.cache_url.clone()
        } else {
            debug!("Attempting DNS cache discovery for domain: {}", self.domain);
            match self.dns_discovery.discover_cache(&self.domain).await {
                Ok(Some(url)) => {
                    info!("Discovered cache endpoint via DNS: {}", url);
                    Some(url)
                }
                Ok(None) => {
                    debug!("No cache endpoint found via DNS, will use direct discovery");
                    None
                }
                Err(e) => {
                    warn!(
                        "DNS cache discovery failed: {}, falling back to direct discovery",
                        e
                    );
                    None
                }
            }
        };

        // Try cache-based discovery if cache is available
        if let Some(ref cache_url) = cache_url {
            debug!("Attempting cache-based discovery from: {}", cache_url);
            match self
                .http_client
                .fetch_from_cache(cache_url, target_domain)
                .await
            {
                Ok(document) => {
                    info!(
                        "Successfully discovered {} agents from cache",
                        document.records.len()
                    );
                    return Ok(document.records.into_iter().map(|r| r.agent).collect());
                }
                Err(e) => {
                    warn!(
                        "Cache discovery failed: {}, falling back to direct discovery",
                        e
                    );
                }
            }
        }

        // Fall back to direct HTTPS discovery
        debug!("Attempting direct HTTPS discovery for: {}", target_domain);
        let document = self.http_client.fetch_ax_record(target_domain).await?;
        info!(
            "Successfully discovered {} agents via direct discovery",
            document.records.len()
        );

        Ok(document.records.into_iter().map(|r| r.agent).collect())
    }

    /// Discover agents using a specific cache URL
    ///
    /// This method bypasses automatic cache discovery and uses the provided
    /// cache URL directly. If the cache request fails, it falls back to
    /// direct HTTPS discovery.
    ///
    /// # Arguments
    ///
    /// * `target_domain` - The domain to discover agents from
    /// * `cache_url` - The cache server URL to use (e.g., "https://cache.acme.com")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use open_tethyr::OpenTethyr;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OpenTethyr::new("acme.com")?;
    /// let agents = client.discover_with_cache(
    ///     "api.partner.com",
    ///     "https://cache.acme.com"
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_with_cache(
        &self,
        target_domain: &str,
        cache_url: &str,
    ) -> Result<Vec<Agent>, ClientError> {
        info!(
            "Discovering agents for domain: {} using cache: {}",
            target_domain, cache_url
        );

        // Try cache-based discovery
        match self
            .http_client
            .fetch_from_cache(cache_url, target_domain)
            .await
        {
            Ok(document) => {
                info!(
                    "Successfully discovered {} agents from cache",
                    document.records.len()
                );
                return Ok(document.records.into_iter().map(|r| r.agent).collect());
            }
            Err(e) => {
                warn!(
                    "Cache discovery failed: {}, falling back to direct discovery",
                    e
                );
            }
        }

        // Fall back to direct HTTPS discovery
        debug!("Attempting direct HTTPS discovery for: {}", target_domain);
        let document = self.http_client.fetch_ax_record(target_domain).await?;
        info!(
            "Successfully discovered {} agents via direct discovery",
            document.records.len()
        );

        Ok(document.records.into_iter().map(|r| r.agent).collect())
    }
}
