//! Integration test: OAuth provider templates with generation

use open_tethyr::ax::AxGenerator;
use open_tethyr::config::*;

#[test]
fn okta_provider_generates_security_in_ax_record() {
    let config = AgentConfig {
        defaults: Default::default(),
        agents: vec![AgentDefinition {
            name: "oauth-agent".into(),
            description: "Agent with Okta".into(),
            provider: Some("acme".into()),
            endpoints: vec![EndpointDefinition {
                protocol: "rest".into(),
                url: "https://api.example.com".into(),
                auth: vec!["OIDC".into()],
                content_type: None,
            }],
            auth: vec!["OIDC".into()],
            capabilities: None, limits: None, security: None, extensions: None,
            oauth_provider: Some("okta".into()),
            oauth_domain: Some("dev-123.okta.com".into()),
        }],
        server: None,
    };

    let doc = AxGenerator::generate_record(&config).unwrap();
    assert_eq!(doc.records.len(), 1);

    let security = doc.records[0].security.as_ref().expect("Should have security");
    let oauth = security.oauth.as_ref().expect("Should have OAuth endpoints");
    assert_eq!(oauth.issuer.as_deref(), Some("https://dev-123.okta.com"));
    assert!(oauth.token_endpoint.as_ref().unwrap().contains("/oauth2/token"));
    assert!(oauth.jwks_uri.as_ref().unwrap().contains("/oauth2/v1/keys"));
}

#[test]
fn auth0_provider_generates_correct_endpoints() {
    let config = AgentConfig {
        defaults: Default::default(),
        agents: vec![AgentDefinition {
            name: "auth0-agent".into(),
            description: "Agent with Auth0".into(),
            provider: Some("acme".into()),
            endpoints: vec![EndpointDefinition {
                protocol: "rest".into(),
                url: "https://api.example.com".into(),
                auth: vec!["OAuth2".into()],
                content_type: None,
            }],
            auth: vec!["OAuth2".into()],
            capabilities: None, limits: None, security: None, extensions: None,
            oauth_provider: Some("auth0".into()),
            oauth_domain: Some("tenant.auth0.com".into()),
        }],
        server: None,
    };

    let doc = AxGenerator::generate_record(&config).unwrap();
    let oauth = doc.records[0].security.as_ref().unwrap().oauth.as_ref().unwrap();
    assert_eq!(oauth.issuer.as_deref(), Some("https://tenant.auth0.com/"));
    assert!(oauth.token_endpoint.as_ref().unwrap().contains("/oauth/token"));
}
