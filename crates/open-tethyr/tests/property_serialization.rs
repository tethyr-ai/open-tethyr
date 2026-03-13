//! Property 8: Serialization Round-Trip Consistency

use open_tethyr::ax::*;
use proptest::prelude::*;

fn arb_protocol() -> impl Strategy<Value = Protocol> {
    prop_oneof![
        Just(Protocol::Rest),
        Just(Protocol::GraphQL),
        Just(Protocol::MCP),
        Just(Protocol::A2A),
    ]
}

fn arb_endpoint() -> impl Strategy<Value = Endpoint> {
    (arb_protocol(), "[a-z]{3,10}", prop::collection::vec("[A-Z_]{2,6}", 1..3))
        .prop_map(|(protocol, url, auth)| Endpoint {
            protocol,
            url: format!("https://{}.example.com/api", url),
            auth,
            content_type: Some("application/json".to_string()),
        })
}

fn arb_agent() -> impl Strategy<Value = Agent> {
    ("[a-z]{3,12}", "[a-z ]{5,20}", "[a-z]{3,10}")
        .prop_map(|(name, desc, provider)| Agent { name, description: desc, provider })
}

fn arb_record() -> impl Strategy<Value = AgentExchangeRecord> {
    (arb_agent(), prop::collection::vec(arb_endpoint(), 1..4))
        .prop_map(|(agent, endpoints)| AgentExchangeRecord {
            record_type: "AX".to_string(),
            version: "1.0".to_string(),
            agent,
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
        let deserialized: AgentExchangeRecord = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(record.record_type, deserialized.record_type);
        prop_assert_eq!(record.version, deserialized.version);
        prop_assert_eq!(record.agent.name, deserialized.agent.name);
        prop_assert_eq!(record.endpoints.len(), deserialized.endpoints.len());
    }

    #[test]
    fn document_roundtrip(record in arb_record()) {
        let doc = AgentExchangeDocument { records: vec![record] };
        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: AgentExchangeDocument = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(doc.records.len(), deserialized.records.len());
    }
}
