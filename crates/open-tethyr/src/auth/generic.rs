//! Generic OAuth2 Provider
//!
//! Implementation for generic OAuth2 provider supporting RFC 8414 compliant providers.
//! Supports both discovery URL and manual endpoint specification.

use crate::auth::provider::OAuthProvider;
use crate::auth::types::{OAuthConfig, OAuthEndpoints, OAuthError};
use std::collections::HashMap;

/// Generic OAuth2 provider for RFC 8414 compliant providers
pub struct GenericOAuth2Provider;

impl GenericOAuth2Provider {
    /// Create a new Generic OAuth2 provider instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for GenericOAuth2Provider {
    fn default() -> Self {
        Self::new()
    }
}

impl OAuthProvider for GenericOAuth2Provider {
    fn generate_endpoints(&self, config: &OAuthConfig) -> Result<OAuthEndpoints, OAuthError> {
        // Validate configuration first
        self.validate_config(config)?;

        let domain = config.domain.as_ref().ok_or(OAuthError::MissingDomain)?;

        // For generic provider, we support manual endpoint configuration
        // Check if endpoints are manually specified in additional_config
        if let Some(issuer) = config.additional_config.get("issuer") {
            return self.generate_from_manual_config(config, issuer);
        }

        // Default: assume standard OAuth2 discovery pattern
        self.generate_from_domain(domain)
    }

    fn provider_name(&self) -> &'static str {
        "generic_oauth2"
    }

    fn validate_config(&self, config: &OAuthConfig) -> Result<(), OAuthError> {
        // Call parent validation first
        if config.domain.is_none() {
            return Err(OAuthError::MissingDomain);
        }

        let domain = config.domain.as_ref().unwrap();

        // Validate domain doesn't have protocol
        if domain.starts_with("http://") || domain.starts_with("https://") {
            return Err(OAuthError::InvalidDomain(
                "Domain should not include protocol (http/https)".to_string(),
            ));
        }

        // If manual endpoints are provided, validate required ones
        if config.additional_config.contains_key("issuer") {
            let required_endpoints = ["authorization_endpoint", "token_endpoint", "jwks_uri"];
            for endpoint in &required_endpoints {
                if !config.additional_config.contains_key(*endpoint) {
                    return Err(OAuthError::MissingConfig(format!(
                        "Manual configuration requires '{}' endpoint",
                        endpoint
                    )));
                }
            }
        }

        Ok(())
    }
}

impl GenericOAuth2Provider {
    /// Generate endpoints from manual configuration
    fn generate_from_manual_config(
        &self,
        config: &OAuthConfig,
        issuer: &str,
    ) -> Result<OAuthEndpoints, OAuthError> {
        let get_endpoint = |key: &str| -> Result<String, OAuthError> {
            config
                .additional_config
                .get(key)
                .cloned()
                .ok_or_else(|| OAuthError::MissingConfig(key.to_string()))
        };

        Ok(OAuthEndpoints {
            issuer: issuer.to_string(),
            authorization_endpoint: get_endpoint("authorization_endpoint")?,
            token_endpoint: get_endpoint("token_endpoint")?,
            jwks_uri: get_endpoint("jwks_uri")?,
            userinfo_endpoint: config.additional_config.get("userinfo_endpoint").cloned(),
            revocation_endpoint: config.additional_config.get("revocation_endpoint").cloned(),
            additional_endpoints: config
                .additional_config
                .iter()
                .filter(|(k, _)| {
                    !matches!(
                        k.as_str(),
                        "issuer"
                            | "authorization_endpoint"
                            | "token_endpoint"
                            | "jwks_uri"
                            | "userinfo_endpoint"
                            | "revocation_endpoint"
                    )
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        })
    }

    /// Generate endpoints from domain using standard patterns
    fn generate_from_domain(&self, domain: &str) -> Result<OAuthEndpoints, OAuthError> {
        let base_url = format!("https://{}", domain);

        // Standard OAuth2/OIDC endpoint patterns
        Ok(OAuthEndpoints {
            issuer: base_url.clone(),
            authorization_endpoint: format!("{}/oauth2/authorize", base_url),
            token_endpoint: format!("{}/oauth2/token", base_url),
            jwks_uri: format!("{}/.well-known/jwks.json", base_url),
            userinfo_endpoint: Some(format!("{}/oauth2/userinfo", base_url)),
            revocation_endpoint: Some(format!("{}/oauth2/revoke", base_url)),
            additional_endpoints: HashMap::new(),
        })
    }
}
