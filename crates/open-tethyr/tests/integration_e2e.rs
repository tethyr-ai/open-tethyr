//! End-to-end integration test

use open_tethyr::ax::*;
use open_tethyr::config::*;

#[test]
fn full_generate_validate_flow() {
    // 1. Create config
    let config = AgentConfig {
        defaults: AgentDefaults {
            provider: Some("acme-corp".into()),
            auth: vec!["OAuth2".into()],
            ..Default::default()
        },
        agents: vec![
            AgentDefinition {
                name: "billing-agent".into(),
                description: "Handles billing queries".into(),
                provider: None,
                endpoints: vec![EndpointDefinition {
                    protocol: "rest".into(),
                    url: "https://billing.acme.com/api/v1".into(),
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
            },
            AgentDefinition {
                name: "support-agent".into(),
                description: "Customer support assistant".into(),
                provider: Some("partner-inc".into()),
                endpoints: vec![EndpointDefinition {
                    protocol: "graphql".into(),
                    url: "https://support.partner.com/graphql".into(),
                    auth: vec!["JWT".into()],
                    content_type: None,
                }],
                auth: vec!["JWT".into()],
                capabilities: None,
                limits: None,
                security: None,
                extensions: None,
                oauth_provider: None,
                oauth_domain: None,
            },
        ],
        server: None,
    };

    // 2. Validate config
    ConfigValidator::validate_config(&config).unwrap();

    // 3. Generate AX document
    let doc = AxGenerator::generate_record(&config).unwrap();
    assert_eq!(doc.records.len(), 2);

    // 4. Validate all generated records
    for record in &doc.records {
        AxValidator::validate_record(record).unwrap();
        assert_eq!(record.record_type, "AX");
        assert_eq!(record.version, "1.0");
    }

    // 5. Verify first agent has OAuth security
    let billing = &doc.records[0];
    assert_eq!(billing.agent.name, "billing-agent");
    assert_eq!(billing.agent.provider, "acme-corp"); // inherited
    assert!(billing.security.is_some());
    let oauth = billing.security.as_ref().unwrap().oauth.as_ref().unwrap();
    assert!(oauth.issuer.as_ref().unwrap().contains("acme.okta.com"));

    // 6. Verify second agent has overridden provider
    let support = &doc.records[1];
    assert_eq!(support.agent.name, "support-agent");
    assert_eq!(support.agent.provider, "partner-inc"); // overridden

    // 7. Write to disk and read back
    let tmp = tempfile::tempdir().unwrap();
    let result = AxGenerator::generate_well_known_structure(&doc, tmp.path()).unwrap();
    assert!(result.ax_record_path.exists());

    let content = std::fs::read_to_string(&result.ax_record_path).unwrap();
    let parsed: AgentExchangeDocument = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.records.len(), 2);

    // 8. Detailed validation of round-tripped records
    for record in &parsed.records {
        let report = AxValidator::validate_record_detailed(record);
        assert!(
            !report.has_errors(),
            "Errors: {:?}",
            report
                .errors()
                .iter()
                .map(|e| &e.message)
                .collect::<Vec<_>>()
        );
    }
}
