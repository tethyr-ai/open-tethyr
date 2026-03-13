use open_tethyr::ax::AxGenerator;
use open_tethyr::config::*;

#[test]
fn generates_flat_ax_record() {
    let config = AgentConfig {
        defaults: AgentDefaults {
            provider: Some("test-corp".into()),
            auth: vec!["OAuth2".into()],
            ..Default::default()
        },
        agents: vec![AgentDefinition {
            name: "agent-1".into(),
            description: "Test agent".into(),
            provider: None,
            endpoints: vec![EndpointDefinition {
                protocol: "rest".into(),
                url: "https://api.example.com/v1".into(),
                auth: vec!["OAuth2".into()],
                content_type: None,
            }],
            auth: vec![],
            capabilities: None,
            limits: None,
            security: None,
            extensions: None,
            oauth_provider: None,
            oauth_domain: None,
        }],
        server: None,
    };
    let record = AxGenerator::generate_record(&config).unwrap();
    assert_eq!(record.record_type, "AX");
    assert_eq!(record.version, "1.0");
    assert_eq!(record.agent.name, "agent-1");
    assert_eq!(record.agent.provider.as_deref(), Some("test-corp"));
}

#[test]
fn generates_with_flat_security() {
    let config = AgentConfig {
        defaults: Default::default(),
        agents: vec![AgentDefinition {
            name: "oauth-agent".into(),
            description: "Agent".into(),
            provider: Some("acme".into()),
            endpoints: vec![EndpointDefinition {
                protocol: "rest".into(),
                url: "https://api.example.com".into(),
                auth: vec!["OAuth2".into()],
                content_type: None,
            }],
            auth: vec!["OAuth2".into()],
            capabilities: None,
            limits: None,
            security: None,
            extensions: None,
            oauth_provider: Some("okta".into()),
            oauth_domain: Some("dev-123.okta.com".into()),
        }],
        server: None,
    };
    let record = AxGenerator::generate_record(&config).unwrap();
    let sec = record.security.as_ref().unwrap();
    assert!(sec.issuer.as_ref().unwrap().starts_with("https://"));
    assert!(sec.jwks_url.as_ref().unwrap().contains("keys"));
}
