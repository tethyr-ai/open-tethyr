//! DNS Discovery Implementation
//!
//! DNS-based discovery for AX cache endpoints using TXT record lookups.

use crate::dns::types::DnsError;
use hickory_resolver::config::*;
use hickory_resolver::TokioAsyncResolver;
use std::sync::Arc;
use tracing::{debug, warn};

/// DNS discovery for AX cache endpoints
pub struct DnsDiscovery {
    resolver: Arc<TokioAsyncResolver>,
}

impl DnsDiscovery {
    /// Create a new DNS discovery instance with default system configuration
    pub fn new() -> Result<Self, DnsError> {
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        Ok(Self {
            resolver: Arc::new(resolver),
        })
    }

    /// Create a new DNS discovery instance with custom resolver
    pub fn with_resolver(resolver: TokioAsyncResolver) -> Self {
        Self {
            resolver: Arc::new(resolver),
        }
    }

    /// Discover cache endpoint for a domain using _ax-cache.<domain> TXT record lookup
    pub async fn discover_cache(&self, domain: &str) -> Result<Option<String>, DnsError> {
        let cache_record = format!("_ax-cache.{}", domain);
        debug!("Looking up cache endpoint for domain: {}", domain);

        match self.lookup_txt_record(&cache_record).await {
            Ok(records) => {
                for record in records {
                    if let Ok(endpoint) = self.parse_cache_endpoint(&record) {
                        debug!("Found cache endpoint: {}", endpoint);
                        return Ok(Some(endpoint));
                    }
                }
                debug!("No valid cache endpoint found for domain: {}", domain);
                Ok(None)
            }
            Err(DnsError::LookupFailed(_)) => {
                debug!("No cache TXT record found for domain: {}", domain);
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Discover root cache endpoint for hierarchical caching
    pub async fn discover_root_cache(&self, domain: &str) -> Result<Option<String>, DnsError> {
        // For root cache discovery, we look for _ax-cache-root.<domain>
        let root_cache_record = format!("_ax-cache-root.{}", domain);
        debug!("Looking up root cache endpoint for domain: {}", domain);

        match self.lookup_txt_record(&root_cache_record).await {
            Ok(records) => {
                for record in records {
                    if let Ok(endpoint) = self.parse_cache_endpoint(&record) {
                        debug!("Found root cache endpoint: {}", endpoint);
                        return Ok(Some(endpoint));
                    }
                }
                debug!("No valid root cache endpoint found for domain: {}", domain);
                Ok(None)
            }
            Err(DnsError::LookupFailed(_)) => {
                debug!("No root cache TXT record found for domain: {}", domain);
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Lookup TXT records for a given record name
    async fn lookup_txt_record(&self, record_name: &str) -> Result<Vec<String>, DnsError> {
        debug!("Performing TXT lookup for: {}", record_name);

        let response = self.resolver.txt_lookup(record_name).await.map_err(|e| {
            debug!("TXT lookup failed for {}: {}", record_name, e);
            DnsError::LookupFailed(format!("TXT lookup failed for {}: {}", record_name, e))
        })?;

        let mut records = Vec::new();
        for txt_record in response.iter() {
            // TXT records can have multiple strings, concatenate them
            let record_data = txt_record
                .txt_data()
                .iter()
                .map(|bytes| String::from_utf8_lossy(bytes))
                .collect::<Vec<_>>()
                .join("");

            debug!("Found TXT record: {}", record_data);
            records.push(record_data);
        }

        Ok(records)
    }

    /// Parse cache endpoint from TXT record format: endpoint=<url>
    pub fn parse_cache_endpoint(&self, txt_record: &str) -> Result<String, DnsError> {
        debug!("Parsing TXT record: {}", txt_record);

        if let Some(url) = txt_record.strip_prefix("endpoint=") {
            // Validate URL format
            let parsed_url = url::Url::parse(url).map_err(|_| {
                DnsError::InvalidFormat(format!("Invalid URL in TXT record: {}", txt_record))
            })?;

            // Ensure it's HTTPS
            if parsed_url.scheme() != "https" {
                return Err(DnsError::InvalidFormat(format!(
                    "Cache endpoint must use HTTPS: {}",
                    url
                )));
            }

            debug!("Successfully parsed cache endpoint: {}", url);
            Ok(url.to_string())
        } else {
            warn!(
                "TXT record does not match expected format 'endpoint=<url>': {}",
                txt_record
            );
            Err(DnsError::InvalidFormat(format!(
                "TXT record must start with 'endpoint=': {}",
                txt_record
            )))
        }
    }
}

impl Default for DnsDiscovery {
    fn default() -> Self {
        Self::new().expect("Failed to create default DNS discovery")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cache_endpoint_valid() {
        let dns = DnsDiscovery::default();
        let result = dns.parse_cache_endpoint("endpoint=https://cache.example.com/ax");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://cache.example.com/ax");
    }

    #[test]
    fn test_parse_cache_endpoint_invalid_format() {
        let dns = DnsDiscovery::default();
        let result = dns.parse_cache_endpoint("invalid-format");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DnsError::InvalidFormat(_)));
    }

    #[test]
    fn test_parse_cache_endpoint_non_https() {
        let dns = DnsDiscovery::default();
        let result = dns.parse_cache_endpoint("endpoint=http://cache.example.com/ax");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DnsError::InvalidFormat(_)));
    }

    #[test]
    fn test_parse_cache_endpoint_invalid_url() {
        let dns = DnsDiscovery::default();
        let result = dns.parse_cache_endpoint("endpoint=not-a-valid-url");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DnsError::InvalidFormat(_)));
    }
}
