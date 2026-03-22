//! OAuth Provider Trait - returns (issuer, jwks_url) for flat security block

use crate::error::OAuthError;

pub trait OAuthProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn generate_endpoints(&self, domain: &str) -> Result<(String, String), OAuthError>;
}

pub struct ProviderRegistry {
    providers: Vec<Box<dyn OAuthProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut r = Self { providers: vec![] };
        r.register_provider(Box::new(super::okta::OktaProvider));
        r.register_provider(Box::new(super::auth0::Auth0Provider));
        r.register_provider(Box::new(super::generic::GenericOAuth2Provider));
        r
    }
    pub fn register_provider(&mut self, p: Box<dyn OAuthProvider>) {
        self.providers.push(p);
    }
    pub fn generate_oauth_config(
        &self,
        name: &str,
        domain: &str,
    ) -> Result<(String, String), OAuthError> {
        self.providers
            .iter()
            .find(|p| p.provider_name() == name)
            .ok_or_else(|| OAuthError::UnknownProvider(name.into()))
            .and_then(|p| p.generate_endpoints(domain))
    }
}
impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
