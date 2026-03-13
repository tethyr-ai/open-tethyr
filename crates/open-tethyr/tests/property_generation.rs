//! Property 1: AX Record Generation Correctness

use open_tethyr::ax::AxGenerator;
use open_tethyr::config::*;

#[test]
fn generates_valid_ax_document() {
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
            capabilities: None, limits: None, security: None, extensions: None,
            oauth_provider: None, oauth_domain: None,
        }],
        server: None,
    };

    let doc = AxGenerator::generate_record(&config).unwrap();
    assert_eq!(doc.records.len(), 1);
    assert_eq!(doc.records[0].record_type, "AX");
    assert_eq!(doc.records[0].version, "1.0");
    assert_eq!(doc.records[0].agent.name, "agent-1");
    assert_eq!(doc.records[0].agent.provider, "test-corp");
    assert!(!doc.records[0].endpoints.is_empty());
}

#[test]
fn generates_with_oauth_provider() {
    let config = AgentConfig {
        defaults: Default::default(),
        agents: vec![AgentDefinition {
            name: "oauth-agent".into(),
            description: "Agent with OAuth".into(),
            provider: Some("acme".into()),
            endpoints: vec![EndpointDefinition {
                protocol: "rest".into(),
                url: "https://api.example.com".into(),
                auth: vec!["OAuth2".into()],
                content_type: None,
            }],
            auth: vec!["OAuth2".into()],
            capabilities: None, limits: None, security: None, extensions: None,
            oauth_provider: Some("okta".into()),
            oauth_domain: Some("dev-123.okta.com".into()),
        }],
        server: None,
    };

    let doc = AxGenerator::generate_record(&config).unwrap();
    let security = doc.records[0].security.as_ref().unwrap();
    let oauth = security.oauth.as_ref().unwrap();
    assert!(oauth.issuer.as_ref().unwrap().starts_with("https://"));
    assert!(oauth.token_endpoint.as_ref().unwrap().contains("/oauth2/token"));
}
