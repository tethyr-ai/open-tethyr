//! Test Fixtures for AX Records
//!
//! This module provides pre-built AX record fixtures for use in tests.

use crate::ax::{Agent, AgentExchangeDocument, AgentExchangeRecord, Endpoint, Protocol};
use serde_json::json;

/// Create a minimal valid AX record for testing
pub fn minimal_ax_record() -> AgentExchangeRecord {
    AgentExchangeRecord {
        record_type: "AX".to_string(),
        version: "1.0".to_string(),
        agent: Agent {
            name: "Test Agent".to_string(),
            description: "A test agent for unit testing".to_string(),
            provider: "Test Provider".to_string(),
        },
        endpoints: vec![Endpoint {
            protocol: Protocol::Rest,
            url: "https://api.example.com/agents/test".to_string(),
            auth: vec!["OAuth2".to_string()],
            content_type: Some("application/json".to_string()),
        }],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    }
}

/// Create a comprehensive AX record with all optional fields for testing
pub fn comprehensive_ax_record() -> AgentExchangeRecord {
    AgentExchangeRecord {
        record_type: "AX".to_string(),
        version: "1.0".to_string(),
        agent: Agent {
            name: "Comprehensive Test Agent".to_string(),
            description: "A comprehensive test agent with all fields populated".to_string(),
            provider: "Comprehensive Test Provider".to_string(),
        },
        endpoints: vec![
            Endpoint {
                protocol: Protocol::Rest,
                url: "https://api.example.com/agents/comprehensive".to_string(),
                auth: vec!["OAuth2".to_string(), "JWT".to_string()],
                content_type: Some("application/json".to_string()),
            },
            Endpoint {
                protocol: Protocol::GraphQL,
                url: "https://graphql.example.com/agents".to_string(),
                auth: vec!["OIDC".to_string()],
                content_type: Some("application/graphql".to_string()),
            },
        ],
        capabilities: Some(crate::ax::Capabilities {}),
        schema: Some(crate::ax::Schema {}),
        limits: Some(crate::ax::Limits {}),
        security: Some(crate::ax::Security {}),
        extensions: Some(json!({
            "custom_field": "custom_value",
            "test_metadata": {
                "version": "1.0",
                "environment": "test"
            }
        })),
    }
}

/// Create an AX record with multiple endpoints for testing
pub fn multi_endpoint_ax_record() -> AgentExchangeRecord {
    AgentExchangeRecord {
        record_type: "AX".to_string(),
        version: "1.0".to_string(),
        agent: Agent {
            name: "Multi-Endpoint Agent".to_string(),
            description: "An agent with multiple endpoints".to_string(),
            provider: "Multi-Endpoint Provider".to_string(),
        },
        endpoints: vec![
            Endpoint {
                protocol: Protocol::Rest,
                url: "https://api.example.com/agents/multi/rest".to_string(),
                auth: vec!["OAuth2".to_string()],
                content_type: Some("application/json".to_string()),
            },
            Endpoint {
                protocol: Protocol::GraphQL,
                url: "https://api.example.com/agents/multi/graphql".to_string(),
                auth: vec!["OIDC".to_string()],
                content_type: Some("application/graphql".to_string()),
            },
            Endpoint {
                protocol: Protocol::MCP,
                url: "https://api.example.com/agents/multi/mcp".to_string(),
                auth: vec!["mTLS".to_string()],
                content_type: Some("application/json".to_string()),
            },
            Endpoint {
                protocol: Protocol::Custom("WebSocket".to_string()),
                url: "wss://api.example.com/agents/multi/ws".to_string(),
                auth: vec!["API_KEY".to_string()],
                content_type: Some("application/json".to_string()),
            },
        ],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    }
}

/// Create an AX document with a single record
pub fn single_record_document() -> AgentExchangeDocument {
    AgentExchangeDocument {
        records: vec![minimal_ax_record()],
    }
}

/// Create an AX document with multiple records
pub fn multi_record_document() -> AgentExchangeDocument {
    AgentExchangeDocument {
        records: vec![
            minimal_ax_record(),
            comprehensive_ax_record(),
            multi_endpoint_ax_record(),
        ],
    }
}

/// Create an empty AX document
pub fn empty_document() -> AgentExchangeDocument {
    AgentExchangeDocument { records: vec![] }
}

/// Create an AX record with invalid version for testing validation
pub fn invalid_version_ax_record() -> AgentExchangeRecord {
    let mut record = minimal_ax_record();
    record.version = "2.0".to_string(); // Invalid version
    record
}

/// Create an AX record with invalid record type for testing validation
pub fn invalid_record_type_ax_record() -> AgentExchangeRecord {
    let mut record = minimal_ax_record();
    record.record_type = "INVALID".to_string(); // Invalid record type
    record
}

/// Create an AX record with no endpoints for testing validation
pub fn no_endpoints_ax_record() -> AgentExchangeRecord {
    let mut record = minimal_ax_record();
    record.endpoints = vec![]; // No endpoints
    record
}

/// Create JSON string representations of fixtures for HTTP mocking
///
/// Get minimal AX record as JSON string
pub fn minimal_ax_record_json() -> String {
    let document = single_record_document();
    serde_json::to_string_pretty(&document).expect("Failed to serialize minimal AX record")
}

/// Get comprehensive AX record as JSON string
pub fn comprehensive_ax_record_json() -> String {
    let document = AgentExchangeDocument {
        records: vec![comprehensive_ax_record()],
    };
    serde_json::to_string_pretty(&document).expect("Failed to serialize comprehensive AX record")
}

/// Get multi-record document as JSON string
pub fn multi_record_document_json() -> String {
    let document = multi_record_document();
    serde_json::to_string_pretty(&document).expect("Failed to serialize multi-record document")
}

/// Get empty document as JSON string
pub fn empty_document_json() -> String {
    let document = empty_document();
    serde_json::to_string_pretty(&document).expect("Failed to serialize empty document")
}

/// Get invalid JSON for testing error handling
pub fn invalid_json() -> String {
    r#"{"invalid": json, "missing": quotes}"#.to_string()
}

/// Get malformed AX record JSON for testing validation
pub fn malformed_ax_record_json() -> String {
    json!({
        "record_type": "AX",
        "version": "1.0",
        // Missing required agent field
        "endpoints": []
    })
    .to_string()
}

/// Create test domains for consistent testing
pub struct TestDomains;

impl TestDomains {
    pub const EXAMPLE_COM: &'static str = "example.com";
    pub const API_EXAMPLE_COM: &'static str = "api.example.com";
    pub const TEST_DOMAIN_COM: &'static str = "test-domain.com";
    pub const SUBDOMAIN_EXAMPLE_COM: &'static str = "subdomain.example.com";
    pub const CACHE_EXAMPLE_COM: &'static str = "cache.example.com";
    pub const INVALID_DOMAIN: &'static str = "invalid..domain";
    pub const EMPTY_DOMAIN: &'static str = "";
}

/// Create test URLs for consistent testing
pub struct TestUrls;

impl TestUrls {
    pub const CACHE_ENDPOINT: &'static str = "https://cache.example.com/ax";
    pub const ROOT_CACHE_ENDPOINT: &'static str = "https://root-cache.example.com/ax";
    pub const AGENT_ENDPOINT: &'static str = "https://api.example.com/agents/test";
    pub const INVALID_HTTP_URL: &'static str = "http://insecure.example.com/ax";
    pub const MALFORMED_URL: &'static str = "not-a-valid-url";
}

/// Create test auth methods for consistent testing
pub struct TestAuthMethods;

impl TestAuthMethods {
    pub const OAUTH2: &'static str = "OAuth2";
    pub const OIDC: &'static str = "OIDC";
    pub const JWT: &'static str = "JWT";
    pub const MTLS: &'static str = "mTLS";
    pub const API_KEY: &'static str = "API_KEY";
    pub const INVALID_AUTH: &'static str = "InvalidAuth";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_ax_record_creation() {
        let record = minimal_ax_record();
        assert_eq!(record.record_type, "AX");
        assert_eq!(record.version, "1.0");
        assert_eq!(record.agent.name, "Test Agent");
        assert_eq!(record.endpoints.len(), 1);
    }

    #[test]
    fn test_comprehensive_ax_record_creation() {
        let record = comprehensive_ax_record();
        assert_eq!(record.record_type, "AX");
        assert_eq!(record.version, "1.0");
        assert_eq!(record.endpoints.len(), 2);
        assert!(record.capabilities.is_some());
        assert!(record.extensions.is_some());
    }

    #[test]
    fn test_multi_endpoint_ax_record_creation() {
        let record = multi_endpoint_ax_record();
        assert_eq!(record.endpoints.len(), 4);

        // Verify different protocols
        assert!(matches!(record.endpoints[0].protocol, Protocol::Rest));
        assert!(matches!(record.endpoints[1].protocol, Protocol::GraphQL));
        assert!(matches!(record.endpoints[2].protocol, Protocol::MCP));
        assert!(matches!(record.endpoints[3].protocol, Protocol::Custom(_)));
    }

    #[test]
    fn test_document_creation() {
        let single_doc = single_record_document();
        assert_eq!(single_doc.records.len(), 1);

        let multi_doc = multi_record_document();
        assert_eq!(multi_doc.records.len(), 3);

        let empty_doc = empty_document();
        assert_eq!(empty_doc.records.len(), 0);
    }

    #[test]
    fn test_json_serialization() {
        let json = minimal_ax_record_json();
        assert!(json.contains("AX"));
        assert!(json.contains("1.0"));
        assert!(json.contains("Test Agent"));

        // Verify it can be deserialized back
        let document: AgentExchangeDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(document.records.len(), 1);
    }

    #[test]
    fn test_invalid_fixtures() {
        let invalid_version = invalid_version_ax_record();
        assert_eq!(invalid_version.version, "2.0");

        let invalid_type = invalid_record_type_ax_record();
        assert_eq!(invalid_type.record_type, "INVALID");

        let no_endpoints = no_endpoints_ax_record();
        assert_eq!(no_endpoints.endpoints.len(), 0);
    }

    #[test]
    fn test_constants() {
        assert_eq!(TestDomains::EXAMPLE_COM, "example.com");
        assert_eq!(TestUrls::CACHE_ENDPOINT, "https://cache.example.com/ax");
        assert_eq!(TestAuthMethods::OAUTH2, "OAuth2");
    }
}
