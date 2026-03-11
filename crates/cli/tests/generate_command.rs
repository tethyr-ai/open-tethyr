//! Integration tests for the generate command

use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_generate_command_basic() {
    // Create temporary directories
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    let output_dir = temp_dir.path().join("output");

    // Create a simple test configuration
    let config_yaml = r#"
defaults:
  provider: "Test Provider"
  auth: ["OAuth2"]
  protocol: "rest"
  content_type: "application/json"

agents:
  - name: "Test Agent"
    description: "A test agent for integration testing"
    url: "https://api.example.com/agents/test"
"#;

    // Write configuration file
    fs::write(&config_path, config_yaml).expect("Failed to write config file");

    // Create the generate command
    let cmd = open_tethyr_cli::commands::generate::GenerateCommand {
        config: config_path,
        output: output_dir.clone(),
        validate: true,
    };

    // Execute the command
    let result = cmd.execute().await;
    assert!(
        result.is_ok(),
        "Generate command should succeed: {:?}",
        result
    );

    // Verify output file exists
    let output_file = output_dir.join(".well-known").join("agent-exchange.json");
    assert!(output_file.exists(), "Output file should exist");

    // Read and parse the generated file
    let content = fs::read_to_string(&output_file).expect("Failed to read output file");
    let document: open_tethyr::ax::AgentExchangeDocument =
        serde_json::from_str(&content).expect("Failed to parse generated JSON");

    // Verify the document structure
    assert_eq!(document.records.len(), 1, "Should have one record");

    let record = &document.records[0];
    assert_eq!(record.record_type, "AX");
    assert_eq!(record.version, "1.0");
    assert_eq!(record.agent.name, "Test Agent");
    assert_eq!(record.agent.provider, "Test Provider");
    assert!(!record.endpoints.is_empty());
}

#[tokio::test]
async fn test_generate_command_with_validation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    let output_dir = temp_dir.path().join("output");

    // Create configuration with multiple agents
    let config_yaml = r#"
defaults:
  provider: "Acme Corp"
  auth: ["OAuth2", "JWT"]
  protocol: "rest"

agents:
  - name: "Customer Lookup"
    description: "Query customer database"
    url: "https://api.acme.com/agents/customer-lookup"
  
  - name: "Order Processing"
    description: "Process customer orders"
    url: "https://api.acme.com/agents/order-processing"
    provider: "Acme Orders Division"
"#;

    fs::write(&config_path, config_yaml).expect("Failed to write config file");

    let cmd = open_tethyr_cli::commands::generate::GenerateCommand {
        config: config_path,
        output: output_dir.clone(),
        validate: true,
    };

    let result = cmd.execute().await;
    assert!(
        result.is_ok(),
        "Generate command with validation should succeed"
    );

    // Verify output
    let output_file = output_dir.join(".well-known").join("agent-exchange.json");
    let content = fs::read_to_string(&output_file).expect("Failed to read output file");
    let document: open_tethyr::ax::AgentExchangeDocument =
        serde_json::from_str(&content).expect("Failed to parse generated JSON");

    assert_eq!(document.records.len(), 2, "Should have two records");

    // Verify first agent uses default provider
    assert_eq!(document.records[0].agent.provider, "Acme Corp");

    // Verify second agent overrides provider
    assert_eq!(document.records[1].agent.provider, "Acme Orders Division");
}

#[tokio::test]
async fn test_generate_command_invalid_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    let output_dir = temp_dir.path().join("output");

    // Create invalid configuration (missing required fields)
    let config_yaml = r#"
agents:
  - name: "Invalid Agent"
    description: "Missing required fields"
"#;

    fs::write(&config_path, config_yaml).expect("Failed to write config file");

    let cmd = open_tethyr_cli::commands::generate::GenerateCommand {
        config: config_path,
        output: output_dir,
        validate: false,
    };

    let result = cmd.execute().await;
    assert!(
        result.is_err(),
        "Generate command should fail for invalid config"
    );
}
