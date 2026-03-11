//! Property tests for well-known file structure generation
//!
//! **Feature: rust-toolkit-architecture, Property 3: Well-Known File Structure Generation**
//! **Validates: Requirements 2.5, 9.2**

use open_tethyr::ax::{AxGenerator, AxValidator};
use open_tethyr::config::{AgentConfig, AgentDefaults, AgentDefinition};
use proptest::prelude::*;
use tempfile::TempDir;

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
    /// Property 3: Well-Known File Structure Generation
    /// For any generated AX output, the file structure should include the standard
    /// /.well-known/agent-exchange.json path and correct directory organization
    /// **Validates: Requirements 2.5, 9.2**
    #[test]
    fn prop_well_known_file_structure_generation(config in arb_agent_config()) {
        // Generate AX document
        let document = AxGenerator::generate_record(&config)
            .expect("Generation should succeed for valid configuration");

        // Generate well-known structure
        let well_known = AxGenerator::generate_well_known_structure(&document)
            .expect("Well-known structure generation should succeed");

        // Verify path is correct standard path
        prop_assert_eq!(
            &well_known.path,
            "/.well-known/agent-exchange.json",
            "Path must be the standard well-known URI"
        );

        // Verify JSON content is valid and can be parsed
        let parsed_document: open_tethyr::ax::AgentExchangeDocument =
            serde_json::from_str(&well_known.agent_exchange_json)
                .expect("Generated JSON should be valid and parseable");

        // Verify parsed document has same number of records
        prop_assert_eq!(
            parsed_document.records.len(),
            document.records.len(),
            "Parsed document should have same number of records"
        );

        // Verify each record in parsed document is valid AX record
        for record in &parsed_document.records {
            AxValidator::validate_record(record)
                .expect("Parsed record should pass AX validation");

            // Verify AX protocol compliance
            prop_assert_eq!(&record.record_type, "AX", "Record type must be 'AX'");
            prop_assert_eq!(&record.version, "1.0", "Version must be '1.0'");
        }

        // Verify JSON is properly formatted (not empty, valid structure)
        prop_assert!(
            !well_known.agent_exchange_json.is_empty(),
            "JSON content must not be empty"
        );
        prop_assert!(
            well_known.agent_exchange_json.contains("\"record_type\""),
            "JSON must contain record_type field"
        );
        prop_assert!(
            well_known.agent_exchange_json.contains("\"version\""),
            "JSON must contain version field"
        );
        prop_assert!(
            well_known.agent_exchange_json.contains("\"records\""),
            "JSON must contain records array"
        );
    }

    /// Property: Well-known file structure filesystem write
    /// For any generated AX output, writing to filesystem should create correct
    /// directory structure with .well-known/agent-exchange.json
    /// **Validates: Requirements 2.5, 9.2**
    #[test]
    fn prop_well_known_filesystem_write(config in arb_agent_config()) {
        // Create temporary directory for test
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output_path = temp_dir.path();

        // Generate AX document
        let document = AxGenerator::generate_record(&config)
            .expect("Generation should succeed");

        // Generate well-known structure
        let well_known = AxGenerator::generate_well_known_structure(&document)
            .expect("Well-known structure generation should succeed");

        // Write to filesystem
        AxGenerator::write_well_known_structure(output_path, &well_known)
            .expect("Writing well-known structure should succeed");

        // Verify directory structure exists
        let well_known_dir = output_path.join(".well-known");
        prop_assert!(
            well_known_dir.exists(),
            ".well-known directory should exist"
        );
        prop_assert!(
            well_known_dir.is_dir(),
            ".well-known should be a directory"
        );

        // Verify agent-exchange.json file exists
        let json_file = well_known_dir.join("agent-exchange.json");
        prop_assert!(
            json_file.exists(),
            "agent-exchange.json file should exist"
        );
        prop_assert!(
            json_file.is_file(),
            "agent-exchange.json should be a file"
        );

        // Read and verify file content
        let file_content = std::fs::read_to_string(&json_file)
            .expect("Should be able to read agent-exchange.json");

        // Verify content matches generated JSON
        prop_assert_eq!(
            &file_content,
            &well_known.agent_exchange_json,
            "File content should match generated JSON"
        );

        // Verify file content is valid JSON and can be parsed
        let parsed_document: open_tethyr::ax::AgentExchangeDocument =
            serde_json::from_str(&file_content)
                .expect("File content should be valid JSON");

        // Verify parsed document is valid
        for record in &parsed_document.records {
            AxValidator::validate_record(record)
                .expect("Record from file should be valid");
        }
    }
}
