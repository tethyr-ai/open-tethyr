//! Okta OAuth Provider

use super::provider::OAuthProvider;
use crate::ax::OAuthEndpoints;
use crate::error::OAuthError;

pub struct OktaProvider;

impl OAuthProvider for OktaProvider {
    fn provider_name(&self) -> &str {
        "okta"
    }

    fn generate_endpoints(&self, domain: &str) -> Result<OAuthEndpoints, OAuthError> {
        Ok(OAuthEndpoints {
            issuer: Some(format!("https://{}", domain)),
            authorization_endpoint: Some(format!("https://{}/oauth2/authorize", domain)),
            token_endpoint: Some(format!("https://{}/oauth2/token", domain)),
            jwks_uri: Some(format!("https://{}/oauth2/v1/keys", domain)),
            userinfo_endpoint: Some(format!("https://{}/oauth2/v1/userinfo", domain)),
            revocation_endpoint: Some(format!("https://{}/oauth2/v1/revoke", domain)),
        })
    }
}
