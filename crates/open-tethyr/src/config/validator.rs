//! Configuration Validation
//!
//! Configuration validation logic with domain, port, URL, and TTL validation.

use super::{AgentConfig, AgentDefaults, AgentDefinition, ConfigError};
use regex::Regex;
use std::sync::OnceLock;
use url::Url;

/// Configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate complete agent configuration
    pub fn validate_config(config: &AgentConfig) -> Result<(), ConfigError> {
        // Validate defaults if present
        if let Some(defaults) = &config.defaults {
            Self::validate_defaults(defaults)?;
        }

        // Validate each agent definition
        for (index, agent) in config.agents.iter().enumerate() {
            Self::validate_agent_definition(agent).map_err(|e| {
                ConfigError::AgentValidationError {
                    index,
                    message: e.to_string(),
                }
            })?;
        }

        Ok(())
    }

    /// Validate agent defaults
    fn validate_defaults(defaults: &AgentDefaults) -> Result<(), ConfigError> {
        if let Some(domain) = &defaults.domain {
            Self::validate_domain(domain)?;
        }

        if let Some(port) = defaults.port {
            Self::validate_port(port)?;
        }

        if let Some(ttl) = defaults.ttl {
            Self::validate_ttl(ttl)?;
        }

        Ok(())
    }

    /// Validate individual agent definition
    fn validate_agent_definition(agent: &AgentDefinition) -> Result<(), ConfigError> {
        // Validate required URL field
        Self::validate_url(&agent.url)?;

        // Validate optional fields
        if let Some(domain) = &agent.domain {
            Self::validate_domain(domain)?;
        }

        if let Some(port) = agent.port {
            Self::validate_port(port)?;
        }

        if let Some(ttl) = agent.ttl {
            Self::validate_ttl(ttl)?;
        }

        Ok(())
    }

    /// Validate domain name matches DNS format
    pub fn validate_domain(domain: &str) -> Result<(), ConfigError> {
        static DOMAIN_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = DOMAIN_REGEX.get_or_init(|| {
            // DNS domain name regex - allows letters, numbers, hyphens, and dots
            // Must start and end with alphanumeric, labels can't start/end with hyphen
            Regex::new(r"^([a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?\.)*[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?$").unwrap()
        });

        if domain.is_empty() {
            return Err(ConfigError::InvalidDomain {
                domain: domain.to_string(),
            });
        }

        if domain.len() > 253 {
            return Err(ConfigError::InvalidDomain {
                domain: format!("{} (too long, max 253 characters)", domain),
            });
        }

        if !regex.is_match(domain) {
            return Err(ConfigError::InvalidDomain {
                domain: domain.to_string(),
            });
        }

        Ok(())
    }

    /// Validate port number is in valid range (1-65535)
    pub fn validate_port(port: u16) -> Result<(), ConfigError> {
        if port == 0 {
            return Err(ConfigError::InvalidPort { port });
        }
        // u16 max is 65535, so no need to check upper bound
        Ok(())
    }

    /// Validate URL is HTTPS format
    pub fn validate_url(url_str: &str) -> Result<(), ConfigError> {
        let url = Url::parse(url_str).map_err(|_| ConfigError::InvalidUrl {
            url: url_str.to_string(),
        })?;

        if url.scheme() != "https" {
            return Err(ConfigError::InvalidUrl {
                url: format!("{} (must use HTTPS)", url_str),
            });
        }

        if url.host().is_none() {
            return Err(ConfigError::InvalidUrl {
                url: format!("{} (missing host)", url_str),
            });
        }

        Ok(())
    }

    /// Validate TTL value is positive integer
    pub fn validate_ttl(ttl: u32) -> Result<(), ConfigError> {
        if ttl == 0 {
            return Err(ConfigError::InvalidTtl { ttl });
        }
        Ok(())
    }
}
