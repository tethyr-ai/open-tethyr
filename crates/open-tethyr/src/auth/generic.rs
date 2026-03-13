use super::provider::OAuthProvider;
use crate::error::OAuthError;
pub struct GenericOAuth2Provider;
impl OAuthProvider for GenericOAuth2Provider {
    fn provider_name(&self) -> &str {
        "generic"
    }
    fn generate_endpoints(&self, domain: &str) -> Result<(String, String), OAuthError> {
        Ok((
            format!("https://{}", domain),
            format!("https://{}/.well-known/jwks.json", domain),
        ))
    }
}
