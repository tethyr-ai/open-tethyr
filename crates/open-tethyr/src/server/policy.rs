//! Policy Enforcement

use crate::error::ServerError;

/// Policy engine for domain locking and allowlists
pub struct PolicyEngine {
    domain_locking: bool,
    home_domain: Option<String>,
    allowlist: Vec<String>,
}

impl PolicyEngine {
    pub fn new(domain_locking: bool, home_domain: Option<String>, allowlist: Vec<String>) -> Self {
        Self {
            domain_locking,
            home_domain,
            allowlist,
        }
    }

    /// Check if discovery is allowed for a domain
    pub fn check_discovery_allowed(&self, domain: &str) -> Result<(), ServerError> {
        if !self.domain_locking {
            return Ok(());
        }

        // Home domain always allowed
        if let Some(ref home) = self.home_domain {
            if domain == home || domain.ends_with(&format!(".{}", home)) {
                return Ok(());
            }
        }

        // Check allowlist
        if self.allowlist.iter().any(|d| d == domain) {
            return Ok(());
        }

        Err(ServerError::PolicyViolation(format!(
            "Domain '{}' is not allowed by policy",
            domain
        )))
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new(false, None, vec![])
    }
}
