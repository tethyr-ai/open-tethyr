//! CLI integration test: generate and validate commands

#[test]
fn generate_produces_valid_ax_json() {
    let tmp = tempfile::tempdir().unwrap();
    let config_path = tmp.path().join("agents.yaml");
    let output_dir = tmp.path().join("output");

    std::fs::write(
        &config_path,
        r#"
defaults:
  provider: "test-corp"
  auth:
    - "OAuth2"

agents:
  - name: "agent-1"
    description: "First test agent"
    endpoints:
      - protocol: "rest"
        url: "https://api.example.com/v1"
        auth:
          - "OAuth2"
"#,
    )
    .unwrap();

    // Generate
    let config = open_tethyr::config::load_config(&config_path).unwrap();
    open_tethyr::config::ConfigValidator::validate_config(&config).unwrap();
    let doc = open_tethyr::ax::AxGenerator::generate_record(&config).unwrap();

    // Write
    std::fs::create_dir_all(&output_dir).unwrap();
    let result =
        open_tethyr::ax::AxGenerator::generate_well_known_structure(&doc, &output_dir).unwrap();
    assert!(result.ax_record_path.exists());

    // Validate the generated output
    let content = std::fs::read_to_string(&result.ax_record_path).unwrap();
    let parsed: open_tethyr::ax::AgentExchangeDocument = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.records.len(), 1);
    assert_eq!(parsed.records[0].record_type, "AX");
    assert_eq!(parsed.records[0].version, "1.0");
    assert_eq!(parsed.records[0].agent.name, "agent-1");

    // Validate each record
    for record in &parsed.records {
        open_tethyr::ax::AxValidator::validate_record(record).unwrap();
    }
}

#[test]
fn validate_fails_on_invalid_input() {
    let record = open_tethyr::ax::AgentExchangeRecord {
        record_type: "INVALID".into(),
        version: "1.0".into(),
        agent: open_tethyr::ax::Agent {
            name: "test".into(),
            description: "test".into(),
            provider: "corp".into(),
        },
        endpoints: vec![open_tethyr::ax::Endpoint {
            protocol: open_tethyr::ax::Protocol::Rest,
            url: "https://api.example.com".into(),
            auth: vec!["OAuth2".into()],
            content_type: None,
        }],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    };
    assert!(open_tethyr::ax::AxValidator::validate_record(&record).is_err());
}
