use super::provider::OAuthProvider;
use crate::error::OAuthError;
pub struct Auth0Provider;
impl OAuthProvider for Auth0Provider {
    fn provider_name(&self) -> &str {
        "auth0"
    }
    fn generate_endpoints(&self, domain: &str) -> Result<(String, String), OAuthError> {
        Ok((
            format!("https://{}/", domain),
            format!("https://{}/.well-known/jwks.json", domain),
        ))
    }
}
