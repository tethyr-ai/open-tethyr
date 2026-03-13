//! OAuth Provider Trait and Registry

use crate::ax::OAuthEndpoints;
use crate::error::OAuthError;

/// OAuth provider trait for endpoint generation
pub trait OAuthProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn generate_endpoints(&self, domain: &str) -> Result<OAuthEndpoints, OAuthError>;
}

/// Provider registry
pub struct ProviderRegistry {
    providers: Vec<Box<dyn OAuthProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut registry = Self { providers: vec![] };
        registry.register_provider(Box::new(super::okta::OktaProvider));
        registry.register_provider(Box::new(super::auth0::Auth0Provider));
        registry.register_provider(Box::new(super::generic::GenericOAuth2Provider));
        registry
    }

    pub fn register_provider(&mut self, provider: Box<dyn OAuthProvider>) {
        self.providers.push(provider);
    }

    pub fn generate_oauth_config(&self, provider_name: &str, domain: &str) -> Result<OAuthEndpoints, OAuthError> {
        self.providers.iter()
            .find(|p| p.provider_name() == provider_name)
            .ok_or_else(|| OAuthError::UnknownProvider(provider_name.into()))
            .and_then(|p| p.generate_endpoints(domain))
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self { Self::new() }
}
