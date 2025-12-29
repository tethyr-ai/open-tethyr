//! Auth0 OAuth Provider
//!
//! Implementation for Auth0 OAuth provider with tenant-specific URLs.

use crate::auth::provider::OAuthProvider;
use crate::auth::types::{OAuthConfig, OAuthEndpoints, OAuthError};
use std::collections::HashMap;

/// Auth0 OAuth provider
pub struct Auth0Provider;

impl Auth0Provider {
    /// Create a new Auth0 provider instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for Auth0Provider {
    fn default() -> Self {
        Self::new()
    }
}

impl OAuthProvider for Auth0Provider {
    fn generate_endpoints(&self, config: &OAuthConfig) -> Result<OAuthEndpoints, OAuthError> {
        // Validate configuration first
        self.validate_config(config)?;

        let domain = config.domain.as_ref().ok_or(OAuthError::MissingDomain)?;

        // Generate Auth0-specific OAuth endpoints
        let base_url = format!("https://{}", domain);

        Ok(OAuthEndpoints {
            issuer: format!("{}/", base_url), // Auth0 issuer includes trailing slash
            authorization_endpoint: format!("{}/authorize", base_url),
            token_endpoint: format!("{}/oauth/token", base_url),
            jwks_uri: format!("{}/.well-known/jwks.json", base_url),
            userinfo_endpoint: Some(format!("{}/userinfo", base_url)),
            revocation_endpoint: Some(format!("{}/oauth/revoke", base_url)),
            additional_endpoints: HashMap::new(),
        })
    }

    fn provider_name(&self) -> &'static str {
        "auth0"
    }

    fn validate_config(&self, config: &OAuthConfig) -> Result<(), OAuthError> {
        // Call parent validation first
        if config.domain.is_none() {
            return Err(OAuthError::MissingDomain);
        }

        let domain = config.domain.as_ref().unwrap();

        // Auth0-specific validation - accept auth0.com domains but reject just "auth0.com"
        if !domain.contains("auth0.com") || domain == "auth0.com" {
            return Err(OAuthError::InvalidDomain(format!(
                "Auth0 domain must be a valid tenant domain (e.g., 'tenant.auth0.com'), got: {}",
                domain
            )));
        }

        // Validate domain doesn't have protocol
        if domain.starts_with("http://") || domain.starts_with("https://") {
            return Err(OAuthError::InvalidDomain(
                "Domain should not include protocol (http/https)".to_string(),
            ));
        }

        Ok(())
    }
}
