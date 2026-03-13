//! Property 9: OAuth Provider Template Correctness

use open_tethyr::auth::*;

#[test]
fn okta_generates_valid_endpoints() {
    let provider = OktaProvider;
    let eps = provider.generate_endpoints("dev-123.okta.com").unwrap();
    assert_eq!(eps.issuer.unwrap(), "https://dev-123.okta.com");
    assert!(eps.authorization_endpoint.unwrap().starts_with("https://"));
    assert!(eps.token_endpoint.unwrap().contains("/oauth2/token"));
    assert!(eps.jwks_uri.unwrap().contains("/oauth2/v1/keys"));
    assert!(eps.userinfo_endpoint.unwrap().contains("/oauth2/v1/userinfo"));
    assert!(eps.revocation_endpoint.unwrap().contains("/oauth2/v1/revoke"));
}

#[test]
fn auth0_generates_valid_endpoints() {
    let provider = Auth0Provider;
    let eps = provider.generate_endpoints("tenant.auth0.com").unwrap();
    assert_eq!(eps.issuer.unwrap(), "https://tenant.auth0.com/");
    assert!(eps.authorization_endpoint.unwrap().contains("/authorize"));
    assert!(eps.token_endpoint.unwrap().contains("/oauth/token"));
    assert!(eps.jwks_uri.unwrap().contains("/.well-known/jwks.json"));
}

#[test]
fn generic_generates_valid_endpoints() {
    let provider = GenericOAuth2Provider;
    let eps = provider.generate_endpoints("auth.example.com").unwrap();
    assert!(eps.issuer.unwrap().starts_with("https://"));
    assert!(eps.token_endpoint.unwrap().contains("/token"));
}

#[test]
fn registry_finds_providers() {
    let registry = ProviderRegistry::new();
    assert!(registry.generate_oauth_config("okta", "test.okta.com").is_ok());
    assert!(registry.generate_oauth_config("auth0", "test.auth0.com").is_ok());
    assert!(registry.generate_oauth_config("generic", "auth.example.com").is_ok());
    assert!(registry.generate_oauth_config("unknown", "x.com").is_err());
}
