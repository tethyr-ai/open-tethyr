//! Property tests for AX record serialization
//!
//! **Feature: rust-toolkit-architecture, Property 8: Serialization Round-Trip Consistency**
//! **Validates: Requirements 5.2, 5.3**

use open_tethyr::ax::{Agent, AgentExchangeDocument, AgentExchangeRecord, Endpoint, Protocol};
use proptest::prelude::*;
use serde_json;

/// Generate arbitrary Agent instances for property testing
fn arb_agent() -> impl Strategy<Value = Agent> {
    (
        "[a-zA-Z0-9 ]{1,50}",
        "[a-zA-Z0-9 .,]{1,200}",
        "[a-zA-Z0-9 ]{1,50}",
    )
        .prop_map(|(name, description, provider)| Agent {
            name,
            description,
            provider,
        })
}

/// Generate arbitrary Protocol instances for property testing
fn arb_protocol() -> impl Strategy<Value = Protocol> {
    prop_oneof![
        Just(Protocol::Rest),
        Just(Protocol::GraphQL),
        Just(Protocol::MCP),
        Just(Protocol::A2A),
        "[a-zA-Z0-9-_]{1,20}".prop_map(Protocol::Custom),
    ]
}

/// Generate arbitrary Endpoint instances for property testing
fn arb_endpoint() -> impl Strategy<Value = Endpoint> {
    (
        arb_protocol(),
        "https://[a-zA-Z0-9.-]{1,50}/[a-zA-Z0-9/-]{0,100}",
        prop::collection::vec("[A-Z0-9_]{1,20}", 1..5),
        prop::option::of("[a-zA-Z0-9/+]{1,50}"),
    )
        .prop_map(|(protocol, url, auth, content_type)| Endpoint {
            protocol,
            url,
            auth,
            content_type,
        })
}

/// Generate arbitrary AgentExchangeRecord instances for property testing
fn arb_agent_exchange_record() -> impl Strategy<Value = AgentExchangeRecord> {
    (
        arb_agent(),
        prop::collection::vec(arb_endpoint(), 1..5),
    )
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

/// Generate arbitrary AgentExchangeDocument instances for property testing
fn arb_agent_exchange_document() -> impl Strategy<Value = AgentExchangeDocument> {
    prop::collection::vec(arb_agent_exchange_record(), 1..10)
        .prop_map(|records| AgentExchangeDocument { records })
}

proptest! {
    /// Property 8: Serialization Round-Trip Consistency
    /// For any valid AgentExchangeDocument, serializing to JSON and then deserializing
    /// should produce an equivalent document.
    #[test]
    fn test_ax_record_serialization_round_trip(
        document in arb_agent_exchange_document()
    ) {
        // Serialize to JSON
        let json_str = serde_json::to_string(&document)
            .expect("Serialization should succeed for valid AX document");

        // Deserialize back from JSON
        let deserialized: AgentExchangeDocument = serde_json::from_str(&json_str)
            .expect("Deserialization should succeed for valid JSON");

        // Verify round-trip consistency
        prop_assert_eq!(document.records.len(), deserialized.records.len());

        for (original, deserialized) in document.records.iter().zip(deserialized.records.iter()) {
            prop_assert_eq!(&original.record_type, &deserialized.record_type);
            prop_assert_eq!(&original.version, &deserialized.version);
            prop_assert_eq!(&original.agent.name, &deserialized.agent.name);
            prop_assert_eq!(&original.agent.description, &deserialized.agent.description);
            prop_assert_eq!(&original.agent.provider, &deserialized.agent.provider);
            prop_assert_eq!(original.endpoints.len(), deserialized.endpoints.len());

            for (orig_endpoint, deser_endpoint) in original.endpoints.iter().zip(deserialized.endpoints.iter()) {
                prop_assert_eq!(&orig_endpoint.url, &deser_endpoint.url);
                prop_assert_eq!(&orig_endpoint.auth, &deser_endpoint.auth);
                prop_assert_eq!(&orig_endpoint.content_type, &deser_endpoint.content_type);
            }
        }
    }

    /// Property test for individual AgentExchangeRecord serialization
    #[test]
    fn test_individual_ax_record_serialization(
        record in arb_agent_exchange_record()
    ) {
        // Serialize to JSON
        let json_str = serde_json::to_string(&record)
            .expect("Serialization should succeed for valid AX record");

        // Deserialize back from JSON
        let deserialized: AgentExchangeRecord = serde_json::from_str(&json_str)
            .expect("Deserialization should succeed for valid JSON");

        // Verify round-trip consistency
        prop_assert_eq!(&record.record_type, &deserialized.record_type);
        prop_assert_eq!(&record.version, &deserialized.version);
        prop_assert_eq!(&record.agent.name, &deserialized.agent.name);
        prop_assert_eq!(&record.agent.description, &deserialized.agent.description);
        prop_assert_eq!(&record.agent.provider, &deserialized.agent.provider);
        prop_assert_eq!(record.endpoints.len(), deserialized.endpoints.len());
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_default_record_type_and_version() {
        let agent = Agent {
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            provider: "Test Provider".to_string(),
        };

        let endpoint = Endpoint {
            protocol: Protocol::Rest,
            url: "https://example.com/api".to_string(),
            auth: vec!["OAuth2".to_string()],
            content_type: Some("application/json".to_string()),
        };

        let record = AgentExchangeRecord {
            record_type: "AX".to_string(),
            version: "1.0".to_string(),
            agent,
            endpoints: vec![endpoint],
            capabilities: None,
            schema: None,
            limits: None,
            security: None,
            extensions: None,
        };

        let json_str = serde_json::to_string(&record).unwrap();
        let deserialized: AgentExchangeRecord = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.record_type, "AX");
        assert_eq!(deserialized.version, "1.0");
    }

    #[test]
    fn test_protocol_serialization() {
        use open_tethyr::ax::Protocol;
        
        // Test all protocol variants serialize correctly
        let protocols = vec![
            (Protocol::Rest, "\"rest\""),
            (Protocol::GraphQL, "\"graphql\""),
            (Protocol::MCP, "\"mcp\""),
            (Protocol::A2A, "\"a2a\""),
            // Custom variant serializes as an object with the field name
            (Protocol::Custom("websocket".to_string()), "{\"custom\":\"websocket\"}"),
        ];

        for (protocol, expected_json) in protocols {
            let json = serde_json::to_string(&protocol).unwrap();
            assert_eq!(json, expected_json, "Protocol {:?} should serialize to {}", protocol, expected_json);
            
            // Test round-trip
            let deserialized: Protocol = serde_json::from_str(&json).unwrap();
            match (&protocol, &deserialized) {
                (Protocol::Rest, Protocol::Rest) => {},
                (Protocol::GraphQL, Protocol::GraphQL) => {},
                (Protocol::MCP, Protocol::MCP) => {},
                (Protocol::A2A, Protocol::A2A) => {},
                (Protocol::Custom(a), Protocol::Custom(b)) => assert_eq!(a, b),
                _ => panic!("Round-trip failed for {:?}", protocol),
            }
        }
    }
}