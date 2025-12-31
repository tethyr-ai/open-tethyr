//! Policy enforcement engine for domain locking and allowlist management

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::{info, warn};

/// Policy violation error types
#[derive(Debug, Error, Clone, PartialEq)]
pub enum PolicyViolation {
    #[error("Domain '{domain}' is not allowed by domain locking policy")]
    DomainNotAllowed { domain: String },

    #[error("External domain '{domain}' is not in allowlist")]
    ExternalDomainBlocked { domain: String },

    #[error("Invalid domain format: '{domain}'")]
    InvalidDomain { domain: String },
}

/// Domain policy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainPolicy {
    /// Enable domain locking (only allow configured domains)
    pub domain_locking_enabled: bool,

    /// List of allowed domains for discovery
    pub allowed_domains: HashSet<String>,

    /// List of allowed external domains (outside organization)
    pub external_allowlist: HashSet<String>,

    /// Organization domain suffix for internal domain detection
    pub organization_domain: Option<String>,
}

/// Policy enforcement engine
#[derive(Debug, Clone)]
pub struct PolicyEngine {
    domain_policy: DomainPolicy,
}

impl PolicyEngine {
    /// Create a new policy engine with the given domain policy
    pub fn new(domain_policy: DomainPolicy) -> Self {
        Self { domain_policy }
    }

    /// Create a policy engine with default (permissive) settings
    pub fn permissive() -> Self {
        Self::new(DomainPolicy::default())
    }

    /// Validate a discovery request for the given domain and client IP
    pub fn validate_discovery_request(
        &self,
        target_domain: &str,
        client_ip: IpAddr,
        correlation_id: &str,
    ) -> Result<(), PolicyViolation> {
        // Log the discovery request for audit purposes
        self.log_discovery_request(target_domain, client_ip, correlation_id);

        // Validate domain format
        if !self.is_valid_domain(target_domain) {
            return Err(PolicyViolation::InvalidDomain {
                domain: target_domain.to_string(),
            });
        }

        // If domain locking is disabled, allow all requests
        if !self.domain_policy.domain_locking_enabled {
            return Ok(());
        }

        // Check if domain is in allowed list
        if self.domain_policy.allowed_domains.contains(target_domain) {
            return Ok(());
        }

        // Check if it's an external domain in the allowlist
        if self.is_external_domain(target_domain) {
            if self
                .domain_policy
                .external_allowlist
                .contains(target_domain)
            {
                return Ok(());
            } else {
                return Err(PolicyViolation::ExternalDomainBlocked {
                    domain: target_domain.to_string(),
                });
            }
        }

        // Domain not allowed
        Err(PolicyViolation::DomainNotAllowed {
            domain: target_domain.to_string(),
        })
    }

    /// Check if a domain is considered external (outside organization)
    fn is_external_domain(&self, domain: &str) -> bool {
        if let Some(org_domain) = &self.domain_policy.organization_domain {
            !domain.ends_with(org_domain)
        } else {
            // If no organization domain is set, consider all domains as internal
            false
        }
    }

    /// Validate domain format (basic DNS name validation)
    fn is_valid_domain(&self, domain: &str) -> bool {
        // Basic domain validation
        if domain.is_empty() || domain.len() > 253 {
            return false;
        }

        // Check for valid characters and structure
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() < 2 {
            return false;
        }

        for part in parts {
            if part.is_empty() || part.len() > 63 {
                return false;
            }

            // Check for valid DNS label characters
            if !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return false;
            }

            // Labels cannot start or end with hyphen
            if part.starts_with('-') || part.ends_with('-') {
                return false;
            }
        }

        true
    }

    /// Log discovery request for audit purposes
    fn log_discovery_request(&self, target_domain: &str, client_ip: IpAddr, correlation_id: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        info!(
            target_domain = target_domain,
            client_ip = %client_ip,
            correlation_id = correlation_id,
            timestamp = timestamp,
            "Discovery request received"
        );
    }

    /// Get the current domain policy configuration
    pub fn domain_policy(&self) -> &DomainPolicy {
        &self.domain_policy
    }

    /// Update the domain policy configuration
    pub fn update_domain_policy(&mut self, policy: DomainPolicy) {
        warn!("Domain policy updated");
        self.domain_policy = policy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_permissive_policy_allows_all() {
        let engine = PolicyEngine::permissive();
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        assert!(engine
            .validate_discovery_request("example.com", client_ip, "test-123")
            .is_ok());
        assert!(engine
            .validate_discovery_request("external.org", client_ip, "test-456")
            .is_ok());
    }

    #[test]
    fn test_domain_locking_blocks_unlisted_domains() {
        let policy = DomainPolicy {
            domain_locking_enabled: true,
            allowed_domains: {
                let mut set = HashSet::new();
                set.insert("allowed.com".to_string());
                set
            },
            ..Default::default()
        };

        let engine = PolicyEngine::new(policy);
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Allowed domain should pass
        assert!(engine
            .validate_discovery_request("allowed.com", client_ip, "test-123")
            .is_ok());

        // Unlisted domain should be blocked
        let result = engine.validate_discovery_request("blocked.com", client_ip, "test-456");
        assert!(matches!(
            result,
            Err(PolicyViolation::DomainNotAllowed { .. })
        ));
    }

    #[test]
    fn test_external_allowlist() {
        let policy = DomainPolicy {
            domain_locking_enabled: true,
            organization_domain: Some("acme.com".to_string()),
            external_allowlist: {
                let mut set = HashSet::new();
                set.insert("partner.org".to_string());
                set
            },
            ..Default::default()
        };

        let engine = PolicyEngine::new(policy);
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // External domain in allowlist should pass
        assert!(engine
            .validate_discovery_request("partner.org", client_ip, "test-123")
            .is_ok());

        // External domain not in allowlist should be blocked
        let result = engine.validate_discovery_request("external.org", client_ip, "test-456");
        assert!(matches!(
            result,
            Err(PolicyViolation::ExternalDomainBlocked { .. })
        ));
    }

    #[test]
    fn test_invalid_domain_format() {
        let engine = PolicyEngine::permissive();
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Invalid domains should be rejected
        let toolong_domain = "toolong.".repeat(50);
        let invalid_domains = vec![
            "",
            "single",
            "invalid..domain.com",
            "-invalid.com",
            "invalid-.com",
            toolong_domain.as_str(),
        ];

        for domain in invalid_domains {
            let result = engine.validate_discovery_request(domain, client_ip, "test");
            assert!(matches!(result, Err(PolicyViolation::InvalidDomain { .. })));
        }
    }

    #[test]
    fn test_valid_domain_format() {
        let engine = PolicyEngine::permissive();
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        let valid_domains = vec![
            "example.com",
            "sub.example.com",
            "api-v2.example.com",
            "test123.example.org",
        ];

        for domain in valid_domains {
            assert!(engine
                .validate_discovery_request(domain, client_ip, "test")
                .is_ok());
        }
    }
}
