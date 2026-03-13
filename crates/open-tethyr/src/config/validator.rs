//! Configuration Validation

use crate::error::ConfigError;

/// Configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate a full configuration
    pub fn validate_config(config: &super::AgentConfig) -> Result<(), ConfigError> {
        if let Some(ref server) = config.server {
            Self::validate_domain(&server.domain)?;
            Self::validate_port(server.port as u32)?;
        }
        for agent in &config.agents {
            if agent.name.is_empty() {
                return Err(ConfigError::MissingConfig("agent.name".into()));
            }
        }
        Ok(())
    }

    /// Validate domain name format (basic DNS check)
    pub fn validate_domain(domain: &str) -> Result<(), ConfigError> {
        if domain.is_empty() {
            return Err(ConfigError::InvalidDomain("empty domain".into()));
        }
        let re = regex_lite::Regex::new(
            r"^([a-zA-Z0-9]([a-zA-Z0-9\-]*[a-zA-Z0-9])?\.)*[a-zA-Z0-9]([a-zA-Z0-9\-]*[a-zA-Z0-9])?$"
        ).unwrap();
        if !re.is_match(domain) {
            return Err(ConfigError::InvalidDomain(domain.into()));
        }
        Ok(())
    }

    /// Validate URL (must be HTTPS)
    pub fn validate_url(url: &str) -> Result<(), ConfigError> {
        if url.is_empty() || !url.starts_with("https://") {
            return Err(ConfigError::InvalidUrl(format!("{} (must use HTTPS)", url)));
        }
        Ok(())
    }

    /// Validate port number
    pub fn validate_port(port: u32) -> Result<(), ConfigError> {
        if !(1..=65535).contains(&port) {
            return Err(ConfigError::InvalidPort(port));
        }
        Ok(())
    }

    /// Validate TTL value
    pub fn validate_ttl(ttl: &str) -> Result<u64, ConfigError> {
        ttl.parse::<u64>()
            .map_err(|_| ConfigError::InvalidTtl(ttl.into()))
            .and_then(|v| {
                if v == 0 { Err(ConfigError::InvalidTtl("must be positive".into())) }
                else { Ok(v) }
            })
    }
}
