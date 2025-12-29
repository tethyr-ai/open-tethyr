//! Property tests for AX version validation
//!
//! **Feature: rust-toolkit-architecture, Property 24: AX Version Validation and Handling**
//! **Validates: Requirements 18.1, 18.3**

use open_tethyr::ax::{
    Agent, AgentExchangeRecord, AxValidator, Endpoint, Protocol, ValidationError,
};
use proptest::prelude::*;

/// Generate arbitrary version strings for property testing
fn arb_version() -> impl Strategy<Value = String> {
    prop_oneof![
        // Valid version
        Just("1.0".to_string()),
        // Invalid versions
        "[0-9]+\\.[0-9]+".prop_filter("Not 1.0", |v| v != "1.0"),
        "[a-zA-Z0-9.-_]{1,20}".prop_filter("Not 1.0", |v| v != "1.0"),
        Just("".to_string()), // Empty version
    ]
}

/// Generate a valid agent for testing
fn valid_agent() -> Agent {
    Agent {
        name: "Test Agent".to_string(),
        description: "A test agent for validation".to_string(),
        provider: "Test Provider".to_string(),
    }
}

/// Generate a valid endpoint for testing
fn valid_endpoint() -> Endpoint {
    Endpoint {
        protocol: Protocol::Rest,
        url: "https://example.com/api".to_string(),
        auth: vec!["OAuth2".to_string()],
        content_type: Some("application/json".to_string()),
    }
}

/// Generate arbitrary AgentExchangeRecord with varying versions for property testing
fn arb_agent_exchange_record_with_version() -> impl Strategy<Value = AgentExchangeRecord> {
    arb_version().prop_map(|version| AgentExchangeRecord {
        record_type: "AX".to_string(),
        version,
        agent: valid_agent(),
        endpoints: vec![valid_endpoint()],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    })
}

proptest! {
    /// Property 24: AX Version Validation and Handling
    /// For any AX record, validation should succeed only for version "1.0"
    /// and fail with UnsupportedVersion error for any other version.
    #[test]
    fn test_ax_version_validation_property(
        record in arb_agent_exchange_record_with_version()
    ) {
        let validation_result = AxValidator::validate_record(&record);

        if record.version == "1.0" {
            // Version 1.0 should always be valid (assuming other fields are valid)
            prop_assert!(validation_result.is_ok(),
                "Version 1.0 should be valid, but got error: {:?}", validation_result);
        } else {
            // Any other version should be invalid
            prop_assert!(validation_result.is_err(),
                "Version {} should be invalid, but validation passed", record.version);

            // Check that the error is specifically about unsupported version
            if let Err(ValidationError::UnsupportedVersion(version)) = validation_result {
                prop_assert_eq!(&version, &record.version);
            } else {
                prop_assert!(false,
                    "Expected UnsupportedVersion error for version {}, but got: {:?}",
                    record.version, validation_result);
            }
        }
    }

    /// Property test for direct version validation
    #[test]
    fn test_version_validation_direct(version in arb_version()) {
        let validation_result = AxValidator::validate_version(&version);

        if version == "1.0" {
            prop_assert!(validation_result.is_ok(),
                "Version 1.0 should be valid, but got error: {:?}", validation_result);
        } else {
            prop_assert!(validation_result.is_err(),
                "Version {} should be invalid, but validation passed", version);

            if let Err(ValidationError::UnsupportedVersion(v)) = validation_result {
                prop_assert_eq!(&v, &version);
            } else {
                prop_assert!(false,
                    "Expected UnsupportedVersion error for version {}, but got: {:?}",
                    version, validation_result);
            }
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_valid_version_1_0() {
        let result = AxValidator::validate_version("1.0");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_version_2_0() {
        let result = AxValidator::validate_version("2.0");
        assert!(result.is_err());

        if let Err(ValidationError::UnsupportedVersion(version)) = result {
            assert_eq!(version, "2.0");
        } else {
            panic!("Expected UnsupportedVersion error");
        }
    }

    #[test]
    fn test_invalid_empty_version() {
        let result = AxValidator::validate_version("");
        assert!(result.is_err());

        if let Err(ValidationError::UnsupportedVersion(version)) = result {
            assert_eq!(version, "");
        } else {
            panic!("Expected UnsupportedVersion error");
        }
    }

    #[test]
    fn test_invalid_version_with_record() {
        let record = AgentExchangeRecord {
            record_type: "AX".to_string(),
            version: "2.0".to_string(),
            agent: valid_agent(),
            endpoints: vec![valid_endpoint()],
            capabilities: None,
            schema: None,
            limits: None,
            security: None,
            extensions: None,
        };

        let result = AxValidator::validate_record(&record);
        assert!(result.is_err());

        if let Err(ValidationError::UnsupportedVersion(version)) = result {
            assert_eq!(version, "2.0");
        } else {
            panic!("Expected UnsupportedVersion error, got: {:?}", result);
        }
    }
}
