//! DNS Discovery Implementation

use crate::error::DnsError;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;

/// DNS discovery for AX cache endpoints
pub struct DnsDiscovery {
    resolver: TokioAsyncResolver,
}

impl DnsDiscovery {
    /// Create a new DNS discovery instance
    pub fn new() -> Result<Self, DnsError> {
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
        Ok(Self { resolver })
    }

    /// Discover cache endpoint for a domain via _ax-cache TXT record
    pub async fn discover_cache(&self, domain: &str) -> Option<String> {
        let query = format!("_ax-cache.{}", domain);
        match self.resolver.txt_lookup(&query).await {
            Ok(records) => {
                for record in records.iter() {
                    let txt = record.to_string();
                    if let Some(endpoint) = Self::parse_cache_endpoint(&txt) {
                        return Some(endpoint);
                    }
                }
                None
            }
            Err(e) => {
                tracing::debug!("DNS cache discovery failed for {}: {}", domain, e);
                None
            }
        }
    }

    /// Discover root cache endpoint
    pub async fn discover_root_cache(&self, domain: &str) -> Option<String> {
        self.discover_cache(domain).await
    }

    /// Parse "endpoint=<url>" from TXT record
    pub fn parse_cache_endpoint(txt: &str) -> Option<String> {
        let txt = txt.trim().trim_matches('"');
        if let Some(url) = txt.strip_prefix("endpoint=") {
            let url = url.trim();
            if !url.is_empty() {
                return Some(url.to_string());
            }
        }
        None
    }
}
