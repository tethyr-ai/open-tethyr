use open_tethyr::auth::*;

#[test]
fn okta_returns_issuer_and_jwks() {
    let (issuer, jwks) = OktaProvider.generate_endpoints("dev.okta.com").unwrap();
    assert_eq!(issuer, "https://dev.okta.com");
    assert!(jwks.contains("keys"));
}

#[test]
fn auth0_returns_issuer_and_jwks() {
    let (issuer, jwks) = Auth0Provider.generate_endpoints("t.auth0.com").unwrap();
    assert_eq!(issuer, "https://t.auth0.com/");
    assert!(jwks.contains("jwks.json"));
}

#[test]
fn generic_returns_issuer_and_jwks() {
    let (issuer, jwks) = GenericOAuth2Provider
        .generate_endpoints("auth.example.com")
        .unwrap();
    assert!(issuer.starts_with("https://"));
    assert!(jwks.contains("jwks.json"));
}

#[test]
fn registry_finds_providers() {
    let r = ProviderRegistry::new();
    assert!(r.generate_oauth_config("okta", "t.com").is_ok());
    assert!(r.generate_oauth_config("auth0", "t.com").is_ok());
    assert!(r.generate_oauth_config("generic", "t.com").is_ok());
    assert!(r.generate_oauth_config("unknown", "t.com").is_err());
}
