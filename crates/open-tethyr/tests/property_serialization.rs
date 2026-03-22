use open_tethyr::ax::*;
use proptest::prelude::*;

fn arb_endpoint() -> impl Strategy<Value = Endpoint> {
    ("[a-z]{3,10}",).prop_map(|(url,)| Endpoint {
        protocol: Protocol::Rest,
        url: format!("https://{}.com/api", url),
        auth: vec!["OAuth2".into()],
        content_type: Some("application/json".into()),
        extra: Default::default(),
    })
}

fn arb_record() -> impl Strategy<Value = AgentExchangeRecord> {
    (
        "[a-z]{3,12}",
        "[a-z ]{5,20}",
        prop::collection::vec(arb_endpoint(), 1..3),
    )
        .prop_map(|(name, desc, endpoints)| AgentExchangeRecord {
            record_type: "AX".into(),
            version: "1.0".into(),
            agent: Agent {
                name,
                description: desc,
                provider: Some("corp".into()),
            },
            endpoints,
            capabilities: None,
            schema: None,
            limits: None,
            security: None,
            extensions: None,
        })
}

proptest! {
    #[test]
    fn serialization_roundtrip(record in arb_record()) {
        let json = serde_json::to_string(&record).unwrap();
        let deser: AgentExchangeRecord = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(record.record_type, deser.record_type);
        prop_assert_eq!(record.agent.name, deser.agent.name);
        prop_assert_eq!(record.endpoints.len(), deser.endpoints.len());
    }
}

#[test]
fn capabilities_roundtrip() {
    let caps = Capabilities {
        intents: vec!["billing".into()],
        async_exec: Some(true),
        supports_callbacks: Some(true),
        callback_modes: vec!["webhook".into()],
        extra: Default::default(),
    };
    let json = serde_json::to_string(&caps).unwrap();
    let parsed: Capabilities = serde_json::from_str(&json).unwrap();
    assert_eq!(caps, parsed);
    assert!(json.contains("\"async\":true"));
}

#[test]
fn schema_roundtrip() {
    let s = Schema {
        graphql_schema_url: Some("https://x.com/schema".into()),
        mcp_manifest_url: Some("https://x.com/mcp".into()),
        rest_openapi_url: None,
        introspection: Some(true),
        extra: Default::default(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let parsed: Schema = serde_json::from_str(&json).unwrap();
    assert_eq!(s, parsed);
}

#[test]
fn limits_roundtrip() {
    let l = Limits {
        max_concurrent_tasks: Some(10.0),
        max_task_ttl_seconds: Some(300.0),
        rate_limit_per_minute: Some(60.0),
        extra: Default::default(),
    };
    let json = serde_json::to_string(&l).unwrap();
    let parsed: Limits = serde_json::from_str(&json).unwrap();
    assert_eq!(l, parsed);
}

#[test]
fn flat_security_roundtrip() {
    let s = Security {
        issuer: Some("https://auth.example.com".into()),
        jwks_url: Some("https://auth.example.com/.well-known/jwks.json".into()),
        signature: None,
        metadata_signature: None,
        extra: Default::default(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let parsed: Security = serde_json::from_str(&json).unwrap();
    assert_eq!(s, parsed);
    assert!(json.contains("jwks_url"));
    assert!(!json.contains("oauth"));
}

#[test]
fn flat_document_parse() {
    let json = r#"{"record_type":"AX","version":"1.0","agent":{"name":"t","description":"d"},"endpoints":[{"protocol":"rest","url":"https://x.com"}]}"#;
    let records = parse_ax_json(json).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].agent.name, "t");
}

#[test]
fn legacy_wrapper_parse() {
    let json = r#"{"records":[{"record_type":"AX","version":"1.0","agent":{"name":"t","description":"d"},"endpoints":[{"protocol":"rest","url":"https://x.com"}]}]}"#;
    let records = parse_ax_json(json).unwrap();
    assert_eq!(records.len(), 1);
}
