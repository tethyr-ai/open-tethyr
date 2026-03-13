#[test]
fn generate_produces_valid_flat_ax() {
    let tmp = tempfile::tempdir().unwrap();
    let config_path = tmp.path().join("agents.yaml");
    std::fs::write(&config_path, "defaults:\n  provider: test-corp\n  auth: [OAuth2]\nagents:\n  - name: agent-1\n    description: First agent\n    endpoints:\n      - protocol: rest\n        url: https://api.example.com/v1\n        auth: [OAuth2]\n").unwrap();
    let config = open_tethyr::config::load_config(&config_path).unwrap();
    let record = open_tethyr::ax::AxGenerator::generate_record(&config).unwrap();
    let output_dir = tmp.path().join("output");
    std::fs::create_dir_all(&output_dir).unwrap();
    let result =
        open_tethyr::ax::AxGenerator::generate_well_known_structure(&record, &output_dir).unwrap();
    assert!(result
        .ax_record_path
        .ends_with(".well-known/agent-exchange"));
    let content = std::fs::read_to_string(&result.ax_record_path).unwrap();
    let parsed: open_tethyr::ax::AgentExchangeRecord = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.record_type, "AX");
    open_tethyr::ax::AxValidator::validate_record(&parsed).unwrap();
}
