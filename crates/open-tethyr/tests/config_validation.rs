//! Property tests for configuration validation
//!
//! **Property 20: Configuration Validation Correctness**
//! **Validates: Requirements 13.4**

use open_tethyr::config::{AgentConfig, AgentDefaults, AgentDefinition, ConfigValidator};
use proptest::prelude::*;

// Property test generators for valid values
fn arb_valid_domain() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("example.com".to_string()),
        Just("api.example.com".to_string()),
        Just("test-domain.org".to_string()),
        Just("sub.domain.co.uk".to_string()),
        Just("localhost".to_string()),
        Just("a.b".to_string()),
    ]
}

fn arb_valid_port() -> impl Strategy<Value = u16> {
    1u16..=65535u16
}

fn arb_valid_https_url() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("https://example.com".to_string()),
        Just("https://api.example.com/path".to_string()),
        Just("https://example.com:8080/api/v1".to_string()),
        Just("https://sub.domain.com/agents/test".to_string()),
    ]
}

fn arb_valid_ttl() -> impl Strategy<Value = u32> {
    1u32..=86400u32
}

// Property test generators for invalid values
fn arb_invalid_domain() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),             // Empty
        Just(".".to_string()),            // Just dot
        Just(".example.com".to_string()), // Leading dot
        Just("example.com.".to_string()), // Trailing dot (actually valid in DNS but we reject it)
        Just("ex ample.com".to_string()), // Space
        Just("example..com".to_string()), // Double dot
        Just("-example.com".to_string()), // Leading hyphen
        Just("example-.com".to_string()), // Trailing hyphen in label
        Just("a".repeat(254)),            // Too long
        Just("a".repeat(64) + ".com"),    // Label too long
    ]
}

fn arb_invalid_url() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),                   // Empty
        Just("http://example.com".to_string()), // HTTP instead of HTTPS
        Just("ftp://example.com".to_string()),  // Wrong protocol
        Just("example.com".to_string()),        // No protocol
        Just("https://".to_string()),           // No host
        Just("not-a-url".to_string()),          // Invalid format
    ]
}

fn arb_valid_agent_config() -> impl Strategy<Value = AgentConfig> {
    (
        prop::option::of(
            (
                prop::option::of(arb_valid_domain()),
                prop::option::of(arb_valid_port()),
                prop::option::of(arb_valid_ttl()),
            )
                .prop_map(|(domain, port, ttl)| AgentDefaults {
                    provider: Some("Test Provider".to_string()),
                    auth: Some(vec!["OAuth2".to_string()]),
                    protocol: Some("rest".to_string()),
                    content_type: Some("application/json".to_string()),
                    domain,
                    port,
                    ttl,
                    capabilities: None,
                    limits: None,
                    security: None,
                    extensions: None,
                }),
        ),
        prop::collection::vec(
            (
                "[a-zA-Z][a-zA-Z0-9 ]{1,49}",
                "[a-zA-Z][a-zA-Z0-9 .,!?]{1,99}",
                arb_valid_https_url(),
                prop::option::of(arb_valid_domain()),
                prop::option::of(arb_valid_port()),
                prop::option::of(arb_valid_ttl()),
            )
                .prop_map(|(name, description, url, domain, port, ttl)| {
                    AgentDefinition {
                        name,
                        description,
                        url,
                        provider: Some("Agent Provider".to_string()),
                        auth: Some(vec!["JWT".to_string()]),
                        protocol: Some("graphql".to_string()),
                        content_type: Some("application/json".to_string()),
                        domain,
                        port,
                        ttl,
                        capabilities: None,
                        limits: None,
                        security: None,
                        extensions: None,
                    }
                }),
            1..3,
        ),
    )
        .prop_map(|(defaults, agents)| AgentConfig { defaults, agents })
}

proptest! {
    /// Property 20: Configuration Validation Correctness
    /// For any valid configuration, validation should succeed
    /// **Validates: Requirements 13.4**
    #[test]
    fn test_valid_configuration_passes_validation(
        config in arb_valid_agent_config()
    ) {
        let result = ConfigValidator::validate_config(&config);
        prop_assert!(result.is_ok(), "Valid configuration should pass validation: {:?}", result);
    }

    /// Property: Domain validation correctness
    /// **Validates: Requirements 13.4**
    #[test]
    fn test_domain_validation_correctness(
        valid_domain in arb_valid_domain(),
        invalid_domain in arb_invalid_domain()
    ) {
        // Valid domains should pass
        let valid_result = ConfigValidator::validate_domain(&valid_domain);
        prop_assert!(valid_result.is_ok(), "Valid domain '{}' should pass validation: {:?}", valid_domain, valid_result);

        // Invalid domains should fail
        let invalid_result = ConfigValidator::validate_domain(&invalid_domain);
        prop_assert!(invalid_result.is_err(), "Invalid domain '{}' should fail validation", invalid_domain);
    }

    /// Property: Port validation correctness
    /// **Validates: Requirements 13.4**
    #[test]
    fn test_port_validation_correctness(
        valid_port in arb_valid_port()
    ) {
        // Valid ports should pass
        let result = ConfigValidator::validate_port(valid_port);
        prop_assert!(result.is_ok(), "Valid port {} should pass validation: {:?}", valid_port, result);

        // Port 0 should fail
        let zero_result = ConfigValidator::validate_port(0);
        prop_assert!(zero_result.is_err(), "Port 0 should fail validation");
    }

    /// Property: URL validation correctness
    /// **Validates: Requirements 13.4**
    #[test]
    fn test_url_validation_correctness(
        valid_url in arb_valid_https_url(),
        invalid_url in arb_invalid_url()
    ) {
        // Valid HTTPS URLs should pass
        let valid_result = ConfigValidator::validate_url(&valid_url);
        prop_assert!(valid_result.is_ok(), "Valid URL '{}' should pass validation: {:?}", valid_url, valid_result);

        // Invalid URLs should fail
        let invalid_result = ConfigValidator::validate_url(&invalid_url);
        prop_assert!(invalid_result.is_err(), "Invalid URL '{}' should fail validation", invalid_url);
    }

    /// Property: TTL validation correctness
    /// **Validates: Requirements 13.4**
    #[test]
    fn test_ttl_validation_correctness(
        valid_ttl in arb_valid_ttl()
    ) {
        // Valid TTL values should pass
        let result = ConfigValidator::validate_ttl(valid_ttl);
        prop_assert!(result.is_ok(), "Valid TTL {} should pass validation: {:?}", valid_ttl, result);

        // TTL 0 should fail
        let zero_result = ConfigValidator::validate_ttl(0);
        prop_assert!(zero_result.is_err(), "TTL 0 should fail validation");
    }

    /// Property: Configuration with invalid defaults should fail
    /// **Validates: Requirements 13.4**
    #[test]
    fn test_invalid_defaults_fail_validation(
        invalid_domain in arb_invalid_domain()
    ) {
        let config = AgentConfig {
            defaults: Some(AgentDefaults {
                provider: Some("Test".to_string()),
                auth: None,
                protocol: None,
                content_type: None,
                domain: Some(invalid_domain.clone()),
                port: None,
                ttl: None,
                capabilities: None,
                limits: None,
                security: None,
                extensions: None,
            }),
            agents: vec![AgentDefinition {
                name: "Test Agent".to_string(),
                description: "Test description".to_string(),
                url: "https://example.com".to_string(),
                provider: None,
                auth: None,
                protocol: None,
                content_type: None,
                domain: None,
                port: None,
                ttl: None,
                capabilities: None,
                limits: None,
                security: None,
                extensions: None,
            }],
        };

        let result = ConfigValidator::validate_config(&config);
        prop_assert!(result.is_err(), "Configuration with invalid domain '{}' in defaults should fail validation", invalid_domain);
    }

    /// Property: Configuration with invalid agent URL should fail
    /// **Validates: Requirements 13.4**
    #[test]
    fn test_invalid_agent_url_fails_validation(
        invalid_url in arb_invalid_url()
    ) {
        let config = AgentConfig {
            defaults: None,
            agents: vec![AgentDefinition {
                name: "Test Agent".to_string(),
                description: "Test description".to_string(),
                url: invalid_url.clone(),
                provider: None,
                auth: None,
                protocol: None,
                content_type: None,
                domain: None,
                port: None,
                ttl: None,
                capabilities: None,
                limits: None,
                security: None,
                extensions: None,
            }],
        };

        let result = ConfigValidator::validate_config(&config);
        prop_assert!(result.is_err(), "Configuration with invalid URL '{}' should fail validation", invalid_url);
    }
}
