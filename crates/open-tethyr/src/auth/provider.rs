//! OAuth Provider Trait
//!
//! Defines the interface for OAuth provider implementations.

use crate::auth::types::{OAuthConfig, OAuthEndpoints, OAuthError};

/// OAuth provider trait for endpoint generation
pub trait OAuthProvider: Send + Sync {
    /// Generate OAuth endpoints from configuration
    fn generate_endpoints(&self, config: &OAuthConfig) -> Result<OAuthEndpoints, OAuthError>;

    /// Get the provider name
    fn provider_name(&self) -> &'static str;

    /// Validate provider-specific configuration
    fn validate_config(&self, config: &OAuthConfig) -> Result<(), OAuthError> {
        // Default implementation - providers can override for specific validation
        if config.domain.is_none() {
            return Err(OAuthError::MissingDomain);
        }
        Ok(())
    }
}
