//! Property 3: Well-Known File Structure Generation

use open_tethyr::ax::{AxGenerator, Agent, AgentExchangeDocument, AgentExchangeRecord, Endpoint, Protocol};

#[test]
fn generates_well_known_structure() {
    let doc = AgentExchangeDocument {
        records: vec![AgentExchangeRecord {
            record_type: "AX".into(),
            version: "1.0".into(),
            agent: Agent { name: "test".into(), description: "test".into(), provider: "corp".into() },
            endpoints: vec![Endpoint {
                protocol: Protocol::Rest,
                url: "https://api.example.com".into(),
                auth: vec!["OAuth2".into()],
                content_type: None,
            }],
            capabilities: None, schema: None, limits: None, security: None, extensions: None,
        }],
    };

    let tmp = tempfile::tempdir().unwrap();
    let result = AxGenerator::generate_well_known_structure(&doc, tmp.path()).unwrap();

    assert!(result.ax_record_path.exists());
    assert!(result.ax_record_path.ends_with(".well-known/agent-exchange.json"));

    // Verify the JSON is valid
    let content = std::fs::read_to_string(&result.ax_record_path).unwrap();
    let parsed: AgentExchangeDocument = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.records.len(), 1);
}
