use open_tethyr::ax::AxGenerator;
use open_tethyr::config::*;

#[test]
fn okta_provider_populates_flat_security() {
    let config = AgentConfig {
        defaults: Default::default(),
        agents: vec![AgentDefinition {
            name: "a".into(),
            description: "d".into(),
            provider: Some("x".into()),
            endpoints: vec![EndpointDefinition {
                protocol: "rest".into(),
                url: "https://x.com".into(),
                auth: vec!["OIDC".into()],
                content_type: None,
            }],
            auth: vec!["OIDC".into()],
            capabilities: None,
            limits: None,
            security: None,
            extensions: None,
            oauth_provider: Some("okta".into()),
            oauth_domain: Some("dev.okta.com".into()),
        }],
        server: None,
    };
    let record = AxGenerator::generate_record(&config).unwrap();
    let sec = record.security.unwrap();
    assert_eq!(sec.issuer.as_deref(), Some("https://dev.okta.com"));
    assert!(sec.jwks_url.unwrap().contains("keys"));
}
