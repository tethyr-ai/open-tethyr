//! Property tests for AX record generation
//!
//! **Feature: rust-toolkit-architecture, Property 1: AX Record Generation Correctness**
//! **Validates: Requirements 2.1, 9.1, 9.3**

use open_tethyr::ax::{AxGenerator, AxValidator};
use open_tethyr::config::{AgentConfig, AgentDefaults, AgentDefinition};
use proptest::prelude::*;

/// Generate valid agent names (non-empty, reasonable length)
fn arb_agent_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 _-]{0,49}".prop_map(|s| s.trim().to_string())
}

/// Generate valid descriptions (non-empty, reasonable length)
fn arb_description() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 .,!?_-]{0,199}".prop_map(|s| s.trim().to_string())
}

/// Generate valid provider names
fn arb_provider() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9 ._-]{0,49}".prop_map(|s| s.trim().to_string())
}

/// Generate valid HTTPS URLs
fn arb_https_url() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-z]{3,10}", 1..4)
        .prop_map(|parts| format!("https://api.{}.com/agents/{}", parts.join("."), parts[0]))
}

/// Generate valid auth methods from supported set
fn arb_auth_methods() -> impl Strategy<Value = Vec<String>> {
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

/// Generate valid protocols
fn arb_protocol() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("rest".to_string()),
        Just("graphql".to_string()),
        Just("mcp".to_string()),
        Just("a2a".to_string()),
    ]
}

/// Generate agent definition
fn arb_agent_definition() -> impl Strategy<Value = AgentDefinition> {
    (
        arb_agent_name(),
        arb_description(),
        arb_https_url(),
        arb_provider(),
        arb_auth_methods(),
        arb_protocol(),
    )
        .prop_map(
            |(name, description, url, provider, auth, protocol)| AgentDefinition {
                name,
                description,
                url,
                provider: Some(provider),
                auth: Some(auth),
                protocol: Some(protocol),
                content_type: Some("application/json".to_string()),
                domain: None,
                port: None,
                ttl: None,
                capabilities: None,
                limits: None,
                security: None,
                extensions: None,
            },
        )
}

/// Generate agent defaults
fn arb_agent_defaults() -> impl Strategy<Value = Option<AgentDefaults>> {
    prop_oneof![
        Just(None),
        (arb_provider(), arb_auth_methods(), arb_protocol()).prop_map(
            |(provider, auth, protocol)| Some(AgentDefaults {
                provider: Some(provider),
                auth: Some(auth),
                protocol: Some(protocol),
                content_type: Some("application/json".to_string()),
                domain: None,
                port: None,
                ttl: None,
                capabilities: None,
                limits: None,
                security: None,
                extensions: None,
            })
        )
    ]
}

/// Generate agent configuration
fn arb_agent_config() -> impl Strategy<Value = AgentConfig> {
    (
        arb_agent_defaults(),
        prop::collection::vec(arb_agent_definition(), 1..5),
    )
        .prop_map(|(defaults, agents)| AgentConfig { defaults, agents })
}

proptest! {
    /// Property 1: AX Record Generation Correctness
    /// For any valid agent configuration, generating AX records should produce
    /// valid AX-compliant records that pass validation
    /// **Validates: Requirements 2.1, 9.1, 9.3**
    #[test]
    fn prop_ax_record_generation_correctness(config in arb_agent_config()) {
        // Generate AX records from configuration
        let document = AxGenerator::generate_record(&config)
            .expect("Generation should succeed for valid configuration");

        // Verify document structure
        prop_assert!(!document.records.is_empty(), "Document should contain at least one record");
        prop_assert_eq!(document.records.len(), config.agents.len(), "Should generate one record per agent");

        // Validate each generated record
        for record in &document.records {
            // Validate record passes AX validation
            AxValidator::validate_record(record)
                .expect("Generated record should pass AX validation");

            // Verify AX protocol compliance
            prop_assert_eq!(&record.record_type, "AX", "Record type must be 'AX'");
            prop_assert_eq!(&record.version, "1.0", "Version must be '1.0'");

            // Verify agent fields are populated
            prop_assert!(!record.agent.name.trim().is_empty(), "Agent name must not be empty");
            prop_assert!(!record.agent.description.trim().is_empty(), "Agent description must not be empty");
            prop_assert!(!record.agent.provider.trim().is_empty(), "Agent provider must not be empty");

            // Verify endpoints
            prop_assert!(!record.endpoints.is_empty(), "Must have at least one endpoint");

            for endpoint in &record.endpoints {
                // Verify URL is HTTPS
                prop_assert!(endpoint.url.starts_with("https://"), "Endpoint URL must use HTTPS");

                // Verify auth methods are from supported set
                prop_assert!(!endpoint.auth.is_empty(), "Endpoint must have auth methods");
                for auth_method in &endpoint.auth {
                    prop_assert!(
                        ["OIDC", "OAuth2", "mTLS", "JWT", "API_KEY"].contains(&auth_method.as_str()),
                        "Auth method '{}' must be from supported set", auth_method
                    );
                }
            }
        }
    }

    /// Property: Well-known file structure generation
    /// For any valid AX document, generating well-known structure should produce
    /// valid JSON that can be parsed back to equivalent document
    /// **Validates: Requirements 2.1, 9.2**
    #[test]
    fn prop_well_known_structure_generation(config in arb_agent_config()) {
        // Generate AX document
        let document = AxGenerator::generate_record(&config)
            .expect("Generation should succeed");

        // Generate well-known structure
        let well_known = AxGenerator::generate_well_known_structure(&document)
            .expect("Well-known structure generation should succeed");

        // Verify path is correct
        prop_assert_eq!(well_known.path, "/.well-known/agent-exchange.json");

        // Verify JSON is valid and can be parsed back
        let parsed_document: open_tethyr::ax::AgentExchangeDocument =
            serde_json::from_str(&well_known.agent_exchange_json)
                .expect("Generated JSON should be valid");

        // Verify parsed document has same structure
        prop_assert_eq!(parsed_document.records.len(), document.records.len());

        // Verify each record in parsed document is valid
        for record in &parsed_document.records {
            AxValidator::validate_record(record)
                .expect("Parsed record should be valid");
        }
    }
}
