//! Property tests for AX record validation
//!
//! **Feature: rust-toolkit-architecture, Property 2: AX Record Validation Correctness**
//! **Validates: Requirements 2.2, 5.5**

use open_tethyr::ax::{Agent, AgentExchangeRecord, AxValidator, Endpoint, Protocol, ValidationError};
use proptest::prelude::*;

/// Generate valid agent names (non-empty, reasonable length)
fn arb_valid_agent_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 _-]{0,49}".prop_map(|s| s.trim().to_string())
}

/// Generate invalid agent names (empty or whitespace-only)
fn arb_invalid_agent_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just("   ".to_string()),
        Just("\t\n".to_string()),
    ]
}

/// Generate valid descriptions
fn arb_valid_description() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 .,!?_-]{0,199}".prop_map(|s| s.trim().to_string())
}

/// Generate invalid descriptions (empty or whitespace-only)
fn arb_invalid_description() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just("   ".to_string()),
        Just("\t\n".to_string()),
    ]
}

/// Generate valid provider names
fn arb_valid_provider() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 ._-]{0,49}".prop_map(|s| s.trim().to_string())
}

/// Generate invalid provider names (empty or whitespace-only)
fn arb_invalid_provider() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just("   ".to_string()),
        Just("\t\n".to_string()),
    ]
}

/// Generate valid HTTPS URLs
fn arb_valid_https_url() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-z]{3,10}", 1..4)
        .prop_map(|parts| format!("https://api.{}.com/agents/{}", parts.join("."), parts[0]))
}

/// Generate invalid URLs (non-HTTPS or malformed)
fn arb_invalid_url() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just("http://insecure.com/api".to_string()), // HTTP instead of HTTPS
        Just("not-a-url".to_string()),
        Just("ftp://example.com".to_string()),
        Just("   ".to_string()),
    ]
}

/// Generate valid auth methods from supported set
fn arb_valid_auth_methods() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        prop_oneof![
            Just("OIDC".to_string()),
            Just("OAuth2".to_string()),
            Just("mTLS".to_string()),
            Just("JWT".to_string()),
            Just("API_KEY".to_string()),
        ],
        1..3,
    )
}

/// Generate invalid auth methods (unsupported or empty)
fn arb_invalid_auth_methods() -> impl Strategy<Value = Vec<String>> {
    prop_oneof![
        Just(vec![]), // Empty auth methods
        prop::collection::vec(
            prop_oneof![
                Just("BasicAuth".to_string()),
                Just("Custom".to_string()),
                Just("Unknown".to_string()),
                Just("".to_string()),
            ],
            1..3,
        ),
    ]
}

/// Generate valid agent
fn arb_valid_agent() -> impl Strategy<Value = Agent> {
    (arb_valid_agent_name(), arb_valid_description(), arb_valid_provider())
        .prop_map(|(name, description, provider)| Agent {
            name,
            description,
            provider,
        })
}

/// Generate valid endpoint
fn arb_valid_endpoint() -> impl Strategy<Value = Endpoint> {
    (arb_valid_https_url(), arb_valid_auth_methods())
        .prop_map(|(url, auth)| Endpoint {
            protocol: Protocol::Rest,
            url,
            auth,
            content_type: Some("application/json".to_string()),
        })
}

/// Generate valid AX record
fn arb_valid_ax_record() -> impl Strategy<Value = AgentExchangeRecord> {
    (arb_valid_agent(), prop::collection::vec(arb_valid_endpoint(), 1..3))
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
    /// Property 2: AX Record Validation Correctness
    /// For any valid AX record, validation should succeed
    /// **Validates: Requirements 2.2, 5.5**
    #[test]
    fn prop_valid_ax_records_pass_validation(record in arb_valid_ax_record()) {
        // Valid records should pass validation
        let result = AxValidator::validate_record(&record);
        prop_assert!(result.is_ok(), "Valid AX record should pass validation: {:?}", result);
    }

    /// Property: Invalid record types are rejected
    /// For any record with invalid record_type, validation should fail
    /// **Validates: Requirements 2.2, 5.5**
    #[test]
    fn prop_invalid_record_type_rejected(
        record_type in "[a-zA-Z]{1,10}".prop_filter("Not AX", |s| s != "AX"),
        agent in arb_valid_agent(),
        endpoints in prop::collection::vec(arb_valid_endpoint(), 1..3)
    ) {
        let record = AgentExchangeRecord {
            record_type,
            version: "1.0".to_string(),
            agent,
            endpoints,
            capabilities: None,
            schema: None,
            limits: None,
            security: None,
            extensions: None,
        };

        let result = AxValidator::validate_record(&record);
        prop_assert!(result.is_err(), "Invalid record type should be rejected");
        
        if let Err(ValidationError::InvalidRecordType(_)) = result {
            // Expected error type
        } else {
            prop_assert!(false, "Should return InvalidRecordType error, got: {:?}", result);
        }
    }

    /// Property: Invalid versions are rejected
    /// For any record with unsupported version, validation should fail
    /// **Validates: Requirements 2.2, 5.5**
    #[test]
    fn prop_invalid_version_rejected(
        version in "[0-9]\\.[0-9]".prop_filter("Not 1.0", |s| s != "1.0"),
        agent in arb_valid_agent(),
        endpoints in prop::collection::vec(arb_valid_endpoint(), 1..3)
    ) {
        let record = AgentExchangeRecord {
            record_type: "AX".to_string(),
            version,
            agent,
            endpoints,
            capabilities: None,
            schema: None,
            limits: None,
            security: None,
            extensions: None,
        };

        let result = AxValidator::validate_record(&record);
        prop_assert!(result.is_err(), "Invalid version should be rejected");
        
        if let Err(ValidationError::UnsupportedVersion(_)) = result {
            // Expected error type
        } else {
            prop_assert!(false, "Should return UnsupportedVersion error, got: {:?}", result);
        }
    }

    /// Property: Invalid agent names are rejected
    /// For any agent with invalid name, validation should fail
    /// **Validates: Requirements 2.2, 5.5**
    #[test]
    fn prop_invalid_agent_name_rejected(
        invalid_name in arb_invalid_agent_name(),
        description in arb_valid_description(),
        provider in arb_valid_provider(),
        endpoints in prop::collection::vec(arb_valid_endpoint(), 1..3)
    ) {
        let agent = Agent {
            name: invalid_name,
            description,
            provider,
        };
        
        let record = AgentExchangeRecord {
            record_type: "AX".to_string(),
            version: "1.0".to_string(),
            agent,
            endpoints,
            capabilities: None,
            schema: None,
            limits: None,
            security: None,
            extensions: None,
        };

        let result = AxValidator::validate_record(&record);
        prop_assert!(result.is_err(), "Invalid agent name should be rejected");
        
        if let Err(ValidationError::MissingField(field)) = result {
            prop_assert!(field.contains("agent.name"), "Should indicate missing agent.name field");
        } else {
            prop_assert!(false, "Should return MissingField error for agent.name, got: {:?}", result);
        }
    }

    /// Property: Invalid URLs are rejected
    /// For any endpoint with invalid URL, validation should fail
    /// **Validates: Requirements 2.2, 5.5**
    #[test]
    fn prop_invalid_url_rejected(
        agent in arb_valid_agent(),
        invalid_url in arb_invalid_url(),
        auth in arb_valid_auth_methods()
    ) {
        let endpoint = Endpoint {
            protocol: Protocol::Rest,
            url: invalid_url,
            auth,
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

        let result = AxValidator::validate_record(&record);
        prop_assert!(result.is_err(), "Invalid URL should be rejected");
    }

    /// Property: Invalid auth methods are rejected
    /// For any endpoint with invalid auth methods, validation should fail
    /// **Validates: Requirements 2.2, 5.5**
    #[test]
    fn prop_invalid_auth_methods_rejected(
        agent in arb_valid_agent(),
        url in arb_valid_https_url(),
        invalid_auth in arb_invalid_auth_methods()
    ) {
        let endpoint = Endpoint {
            protocol: Protocol::Rest,
            url,
            auth: invalid_auth,
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

        let result = AxValidator::validate_record(&record);
        prop_assert!(result.is_err(), "Invalid auth methods should be rejected");
    }

    /// Property: Empty endpoints are rejected
    /// For any record with no endpoints, validation should fail
    /// **Validates: Requirements 2.2, 5.5**
    #[test]
    fn prop_empty_endpoints_rejected(agent in arb_valid_agent()) {
        let record = AgentExchangeRecord {
            record_type: "AX".to_string(),
            version: "1.0".to_string(),
            agent,
            endpoints: vec![], // Empty endpoints
            capabilities: None,
            schema: None,
            limits: None,
            security: None,
            extensions: None,
        };

        let result = AxValidator::validate_record(&record);
        prop_assert!(result.is_err(), "Empty endpoints should be rejected");
        
        if let Err(ValidationError::MissingField(field)) = result {
            prop_assert!(field.contains("endpoints"), "Should indicate missing endpoints field");
        } else {
            prop_assert!(false, "Should return MissingField error for endpoints, got: {:?}", result);
        }
    }
}