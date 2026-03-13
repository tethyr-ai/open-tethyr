//! Generic OAuth2 Provider (RFC 8414)

use super::provider::OAuthProvider;
use crate::ax::OAuthEndpoints;
use crate::error::OAuthError;

pub struct GenericOAuth2Provider;

impl OAuthProvider for GenericOAuth2Provider {
    fn provider_name(&self) -> &str {
        "generic"
    }

    fn generate_endpoints(&self, domain: &str) -> Result<OAuthEndpoints, OAuthError> {
        let base = format!("https://{}", domain);
        Ok(OAuthEndpoints {
            issuer: Some(base.clone()),
            authorization_endpoint: Some(format!("{}/authorize", base)),
            token_endpoint: Some(format!("{}/token", base)),
            jwks_uri: Some(format!("{}/.well-known/jwks.json", base)),
            userinfo_endpoint: Some(format!("{}/userinfo", base)),
            revocation_endpoint: Some(format!("{}/revoke", base)),
        })
    }
}
