//! OAuth Provider Registry
//!
//! Registry for managing multiple OAuth providers.

use crate::auth::provider::OAuthProvider;
use crate::auth::types::{OAuthConfig, OAuthEndpoints, OAuthError};
use std::collections::HashMap;
use std::sync::Arc;

/// Provider registry for multiple OAuth providers
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn OAuthProvider>>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a new OAuth provider
    pub fn register_provider(&mut self, name: String, provider: Arc<dyn OAuthProvider>) {
        self.providers.insert(name, provider);
    }

    /// Generate OAuth configuration for a specific provider
    pub fn generate_oauth_config(
        &self,
        provider_name: &str,
        config: &OAuthConfig,
    ) -> Result<OAuthEndpoints, OAuthError> {
        let provider = self
            .providers
            .get(provider_name)
            .ok_or_else(|| OAuthError::ProviderNotFound(provider_name.to_string()))?;

        // Validate configuration first
        provider.validate_config(config)?;

        // Generate endpoints
        provider.generate_endpoints(config)
    }

    /// Get list of registered provider names
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Check if a provider is registered
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
