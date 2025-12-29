//! Okta OAuth Provider
//!
//! Implementation for Okta OAuth provider with standard endpoint patterns.

use crate::auth::provider::OAuthProvider;
use crate::auth::types::{OAuthConfig, OAuthEndpoints, OAuthError};
use std::collections::HashMap;

/// Okta OAuth provider
pub struct OktaProvider;

impl OktaProvider {
    /// Create a new Okta provider instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for OktaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OAuthProvider for OktaProvider {
    fn generate_endpoints(&self, config: &OAuthConfig) -> Result<OAuthEndpoints, OAuthError> {
        // Validate configuration first
        self.validate_config(config)?;

        let domain = config.domain.as_ref().ok_or(OAuthError::MissingDomain)?;

        // Generate standard Okta OAuth endpoints
        let base_url = format!("https://{}", domain);

        Ok(OAuthEndpoints {
            issuer: base_url.clone(),
            authorization_endpoint: format!("{}/oauth2/authorize", base_url),
            token_endpoint: format!("{}/oauth2/token", base_url),
            jwks_uri: format!("{}/oauth2/v1/keys", base_url),
            userinfo_endpoint: Some(format!("{}/oauth2/v1/userinfo", base_url)),
            revocation_endpoint: Some(format!("{}/oauth2/v1/revoke", base_url)),
            additional_endpoints: HashMap::new(),
        })
    }

    fn provider_name(&self) -> &'static str {
        "okta"
    }

    fn validate_config(&self, config: &OAuthConfig) -> Result<(), OAuthError> {
        // Call parent validation first
        if config.domain.is_none() {
            return Err(OAuthError::MissingDomain);
        }

        let domain = config.domain.as_ref().unwrap();

        // Okta-specific validation - accept okta.com, oktapreview.com, okta-emea.com, etc.
        // But reject just "okta.com" as it's not a valid tenant domain
        if !domain.contains("okta") || domain == "okta.com" {
            return Err(OAuthError::InvalidDomain(format!(
                "Okta domain must be a valid tenant domain (e.g., 'tenant.okta.com'), got: {}",
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
