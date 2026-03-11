//! Property tests for auth method validation
//!
//! **Feature: rust-toolkit-architecture, Property 25: Auth Method Validation**
//! **Validates: Requirements 9.3, 15.4**

use open_tethyr::ax::{
    Agent, AgentExchangeRecord, AxValidator, Endpoint, Protocol, ValidationError,
};
use proptest::prelude::*;

/// Generate valid auth methods from the supported set
fn arb_valid_auth_method() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("OIDC".to_string()),
        Just("OAuth2".to_string()),
        Just("mTLS".to_string()),
        Just("JWT".to_string()),
        Just("API_KEY".to_string()),
    ]
}

/// Generate invalid auth methods (not in the supported set)
fn arb_invalid_auth_method() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("BasicAuth".to_string()),
        Just("Custom".to_string()),
        Just("Unknown".to_string()),
        Just("SAML".to_string()),
        Just("Kerberos".to_string()),
        Just("Digest".to_string()),
        Just("Bearer".to_string()),
        Just("".to_string()),
        Just("   ".to_string()),
        "[a-zA-Z]{3,15}".prop_filter("Not supported", |s| {
            !["OIDC", "OAuth2", "mTLS", "JWT", "API_KEY"].contains(&s.as_str())
        }),
    ]
}

/// Generate a list of valid auth methods
fn arb_valid_auth_methods() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_valid_auth_method(), 1..=5)
}

/// Generate a list containing at least one invalid auth method
fn arb_invalid_auth_methods() -> impl Strategy<Value = Vec<String>> {
    prop_oneof![
        // Only invalid methods
        prop::collection::vec(arb_invalid_auth_method(), 1..3),
        // Mix of valid and invalid methods
        (
            prop::collection::vec(arb_valid_auth_method(), 0..2),
            prop::collection::vec(arb_invalid_auth_method(), 1..2)
        )
            .prop_map(|(mut valid, invalid)| {
                valid.extend(invalid);
                valid
            }),
    ]
}

/// Generate a valid agent for testing
fn arb_valid_agent() -> impl Strategy<Value = Agent> {
    (
        "[a-zA-Z][a-zA-Z0-9 _-]{2,49}",
        "[a-zA-Z][a-zA-Z0-9 .,!?_-]{5,199}",
        "[a-zA-Z][a-zA-Z0-9 ._-]{2,49}",
    )
        .prop_map(|(name, description, provider)| Agent {
            name: name.trim().to_string(),
            description: description.trim().to_string(),
            provider: provider.trim().to_string(),
        })
}

/// Generate a valid HTTPS URL
fn arb_valid_https_url() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-z]{3,10}", 1..4)
        .prop_map(|parts| format!("https://api.{}.com/agents/{}", parts.join("."), parts[0]))
}

proptest! {
    /// Property 25: Auth Method Validation
    /// For any endpoint auth configuration with valid auth methods from the supported set,
    /// validation should succeed
    /// **Validates: Requirements 9.3, 15.4**
    #[test]
    fn prop_valid_auth_methods_accepted(
        agent in arb_valid_agent(),
        url in arb_valid_https_url(),
        auth_methods in arb_valid_auth_methods()
    ) {
        let endpoint = Endpoint {
            protocol: Protocol::Rest,
            url,
            auth: auth_methods.clone(),
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

        // Valid auth methods should pass validation
        let result = AxValidator::validate_record(&record);
        prop_assert!(
            result.is_ok(),
            "Valid auth methods {:?} should be accepted, but got error: {:?}",
            auth_methods,
            result
        );
    }

    /// Property 25: Auth Method Validation - Invalid Methods Rejected
    /// For any endpoint auth configuration with invalid auth methods,
    /// validation should fail with InvalidAuthMethod error
    /// **Validates: Requirements 9.3, 15.4**
    #[test]
    fn prop_invalid_auth_methods_rejected(
        agent in arb_valid_agent(),
        url in arb_valid_https_url(),
        auth_methods in arb_invalid_auth_methods()
    ) {
        let endpoint = Endpoint {
            protocol: Protocol::Rest,
            url,
            auth: auth_methods.clone(),
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

        // Invalid auth methods should be rejected
        let result = AxValidator::validate_record(&record);
        prop_assert!(
            result.is_err(),
            "Invalid auth methods {:?} should be rejected",
            auth_methods
        );

        // Verify it's the correct error type
        if let Err(ValidationError::InvalidAuthMethod(method)) = result {
            prop_assert!(
                !["OIDC", "OAuth2", "mTLS", "JWT", "API_KEY"].contains(&method.trim()),
                "Error should report an unsupported auth method, got: {}",
                method
            );
        } else {
            prop_assert!(
                false,
                "Should return InvalidAuthMethod error, got: {:?}",
                result
            );
        }
    }

    /// Property 25: Auth Method Validation - Direct Validation Function
    /// Test the validate_auth_methods function directly with valid methods
    /// **Validates: Requirements 9.3, 15.4**
    #[test]
    fn prop_validate_auth_methods_accepts_valid(auth_methods in arb_valid_auth_methods()) {
        let result = AxValidator::validate_auth_methods(&auth_methods);
        prop_assert!(
            result.is_ok(),
            "Valid auth methods {:?} should pass validation, but got: {:?}",
            auth_methods,
            result
        );
    }

    /// Property 25: Auth Method Validation - Direct Validation Function Rejects Invalid
    /// Test the validate_auth_methods function directly with invalid methods
    /// **Validates: Requirements 9.3, 15.4**
    #[test]
    fn prop_validate_auth_methods_rejects_invalid(auth_methods in arb_invalid_auth_methods()) {
        let result = AxValidator::validate_auth_methods(&auth_methods);
        prop_assert!(
            result.is_err(),
            "Invalid auth methods {:?} should be rejected",
            auth_methods
        );

        if let Err(ValidationError::InvalidAuthMethod(_)) = result {
            // Expected error type
        } else {
            prop_assert!(
                false,
                "Should return InvalidAuthMethod error, got: {:?}",
                result
            );
        }
    }

    /// Property 25: Auth Method Validation - Empty Auth Methods Rejected
    /// For any endpoint with empty auth methods list, validation should fail
    /// **Validates: Requirements 9.3, 15.4**
    #[test]
    fn prop_empty_auth_methods_rejected(
        agent in arb_valid_agent(),
        url in arb_valid_https_url()
    ) {
        let endpoint = Endpoint {
            protocol: Protocol::Rest,
            url,
            auth: vec![], // Empty auth methods
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

        // Empty auth methods should be rejected
        let result = AxValidator::validate_record(&record);
        prop_assert!(
            result.is_err(),
            "Empty auth methods should be rejected"
        );

        if let Err(ValidationError::MissingField(field)) = result {
            prop_assert!(
                field.contains("auth"),
                "Should indicate missing auth field, got: {}",
                field
            );
        } else {
            prop_assert!(
                false,
                "Should return MissingField error for auth, got: {:?}",
                result
            );
        }
    }

    /// Property 25: Auth Method Validation - Multiple Endpoints
    /// For any record with multiple endpoints, all auth methods must be valid
    /// **Validates: Requirements 9.3, 15.4**
    #[test]
    fn prop_multiple_endpoints_all_auth_valid(
        agent in arb_valid_agent(),
        endpoints_data in prop::collection::vec(
            (arb_valid_https_url(), arb_valid_auth_methods()),
            1..=5
        )
    ) {
        let endpoints: Vec<Endpoint> = endpoints_data
            .into_iter()
            .map(|(url, auth)| Endpoint {
                protocol: Protocol::Rest,
                url,
                auth,
                content_type: Some("application/json".to_string()),
            })
            .collect();

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

        // All valid auth methods across multiple endpoints should pass
        let result = AxValidator::validate_record(&record);
        prop_assert!(
            result.is_ok(),
            "Multiple endpoints with valid auth methods should pass validation, got: {:?}",
            result
        );
    }

    /// Property 25: Auth Method Validation - Case Sensitivity
    /// Auth method validation should be case-sensitive (only exact matches accepted)
    /// **Validates: Requirements 9.3, 15.4**
    #[test]
    fn prop_auth_methods_case_sensitive(
        agent in arb_valid_agent(),
        url in arb_valid_https_url(),
        case_variant in prop_oneof![
            Just("oidc".to_string()),      // lowercase
            Just("oauth2".to_string()),    // lowercase
            Just("mtls".to_string()),      // lowercase
            Just("jwt".to_string()),       // lowercase
            Just("api_key".to_string()),   // lowercase
            Just("Oidc".to_string()),      // mixed case
            Just("OAUTH2".to_string()),    // already uppercase, but different from OAuth2
        ]
    ) {
        // Filter out exact matches with supported methods
        prop_assume!(!["OIDC", "OAuth2", "mTLS", "JWT", "API_KEY"].contains(&case_variant.as_str()));

        let endpoint = Endpoint {
            protocol: Protocol::Rest,
            url,
            auth: vec![case_variant.clone()],
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

        // Case variants should be rejected (validation is case-sensitive)
        let result = AxValidator::validate_record(&record);
        prop_assert!(
            result.is_err(),
            "Case variant '{}' should be rejected (case-sensitive validation)",
            case_variant
        );
    }
}
