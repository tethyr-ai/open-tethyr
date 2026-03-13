use super::provider::OAuthProvider;
use crate::error::OAuthError;
pub struct OktaProvider;
impl OAuthProvider for OktaProvider {
    fn provider_name(&self) -> &str {
        "okta"
    }
    fn generate_endpoints(&self, domain: &str) -> Result<(String, String), OAuthError> {
        Ok((
            format!("https://{}", domain),
            format!("https://{}/oauth2/v1/keys", domain),
        ))
    }
}
