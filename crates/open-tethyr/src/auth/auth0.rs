//! Auth0 OAuth Provider

use crate::ax::OAuthEndpoints;
use crate::error::OAuthError;
use super::provider::OAuthProvider;

pub struct Auth0Provider;

impl OAuthProvider for Auth0Provider {
    fn provider_name(&self) -> &str { "auth0" }

    fn generate_endpoints(&self, domain: &str) -> Result<OAuthEndpoints, OAuthError> {
        Ok(OAuthEndpoints {
            issuer: Some(format!("https://{}/", domain)),
            authorization_endpoint: Some(format!("https://{}/authorize", domain)),
            token_endpoint: Some(format!("https://{}/oauth/token", domain)),
            jwks_uri: Some(format!("https://{}/.well-known/jwks.json", domain)),
            userinfo_endpoint: Some(format!("https://{}/userinfo", domain)),
            revocation_endpoint: Some(format!("https://{}/oauth/revoke", domain)),
        })
    }
}
