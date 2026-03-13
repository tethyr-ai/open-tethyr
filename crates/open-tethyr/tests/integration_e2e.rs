use open_tethyr::ax::*;
use open_tethyr::config::*;

#[test]
fn full_generate_validate_flow() {
    let config = AgentConfig {
        defaults: AgentDefaults {
            provider: Some("acme-corp".into()),
            auth: vec!["OAuth2".into()],
            ..Default::default()
        },
        agents: vec![AgentDefinition {
            name: "billing".into(),
            description: "Billing agent".into(),
            provider: None,
            endpoints: vec![EndpointDefinition {
                protocol: "rest".into(),
                url: "https://billing.acme.com/api".into(),
                auth: vec!["OAuth2".into(), "API_KEY".into()],
                content_type: Some("application/json".into()),
            }],
            auth: vec![],
            capabilities: None,
            limits: None,
            security: None,
            extensions: None,
            oauth_provider: Some("okta".into()),
            oauth_domain: Some("acme.okta.com".into()),
        }],
        server: None,
    };
    ConfigValidator::validate_config(&config).unwrap();
    let record = AxGenerator::generate_record(&config).unwrap();
    AxValidator::validate_record(&record).unwrap();
    assert_eq!(record.record_type, "AX");
    assert!(record.security.is_some());
    let sec = record.security.as_ref().unwrap();
    assert!(sec.issuer.is_some());
    assert!(sec.jwks_url.is_some());
    let tmp = tempfile::tempdir().unwrap();
    let result = AxGenerator::generate_well_known_structure(&record, tmp.path()).unwrap();
    assert!(result
        .ax_record_path
        .ends_with(".well-known/agent-exchange"));
    let content = std::fs::read_to_string(&result.ax_record_path).unwrap();
    let parsed: AgentExchangeRecord = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.agent.name, "billing");
}
