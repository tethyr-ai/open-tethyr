//! Integration tests for the validate command

use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_validate_command_valid_record() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("valid-record.json");

    // Create a valid AX record
    let valid_record = r#"{
  "records": [
    {
      "record_type": "AX",
      "version": "1.0",
      "agent": {
        "name": "Test Agent",
        "description": "A test agent for validation",
        "provider": "Test Provider"
      },
      "endpoints": [
        {
          "protocol": "rest",
          "url": "https://api.example.com/agents/test",
          "auth": ["OAuth2"]
        }
      ]
    }
  ]
}"#;

    fs::write(&test_file, valid_record).expect("Failed to write test file");

    let cmd = open_tethyr_cli::commands::validate::ValidateCommand { file: test_file };

    let result = cmd.execute().await;
    assert!(
        result.is_ok(),
        "Validate command should succeed for valid record: {:?}",
        result
    );
}

#[tokio::test]
async fn test_validate_command_multiple_valid_records() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("multiple-records.json");

    // Create multiple valid AX records
    let valid_records = r#"{
  "records": [
    {
      "record_type": "AX",
      "version": "1.0",
      "agent": {
        "name": "Customer Lookup",
        "description": "Query customer database",
        "provider": "Acme Corp"
      },
      "endpoints": [
        {
          "protocol": "rest",
          "url": "https://api.acme.com/agents/customer-lookup",
          "auth": ["OAuth2", "JWT"]
        }
      ]
    },
    {
      "record_type": "AX",
      "version": "1.0",
      "agent": {
        "name": "Order Processing",
        "description": "Process customer orders",
        "provider": "Acme Corp"
      },
      "endpoints": [
        {
          "protocol": "rest",
          "url": "https://api.acme.com/agents/order-processing",
          "auth": ["OAuth2"]
        }
      ]
    }
  ]
}"#;

    fs::write(&test_file, valid_records).expect("Failed to write test file");

    let cmd = open_tethyr_cli::commands::validate::ValidateCommand { file: test_file };

    let result = cmd.execute().await;
    assert!(
        result.is_ok(),
        "Validate command should succeed for multiple valid records"
    );
}

#[tokio::test]
async fn test_validate_command_invalid_record_type() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("invalid-record-type.json");

    // Create record with invalid record_type
    let invalid_record = r#"{
  "records": [
    {
      "record_type": "INVALID",
      "version": "1.0",
      "agent": {
        "name": "Test Agent",
        "description": "A test agent",
        "provider": "Test Provider"
      },
      "endpoints": [
        {
          "protocol": "rest",
          "url": "https://api.example.com/agents/test",
          "auth": ["OAuth2"]
        }
      ]
    }
  ]
}"#;

    fs::write(&test_file, invalid_record).expect("Failed to write test file");

    let cmd = open_tethyr_cli::commands::validate::ValidateCommand { file: test_file };

    let result = cmd.execute().await;
    assert!(
        result.is_err(),
        "Validate command should fail for invalid record_type"
    );
}

#[tokio::test]
async fn test_validate_command_unsupported_version() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("unsupported-version.json");

    // Create record with unsupported version
    let invalid_record = r#"{
  "records": [
    {
      "record_type": "AX",
      "version": "2.0",
      "agent": {
        "name": "Test Agent",
        "description": "A test agent",
        "provider": "Test Provider"
      },
      "endpoints": [
        {
          "protocol": "rest",
          "url": "https://api.example.com/agents/test",
          "auth": ["OAuth2"]
        }
      ]
    }
  ]
}"#;

    fs::write(&test_file, invalid_record).expect("Failed to write test file");

    let cmd = open_tethyr_cli::commands::validate::ValidateCommand { file: test_file };

    let result = cmd.execute().await;
    assert!(
        result.is_err(),
        "Validate command should fail for unsupported version"
    );
}

#[tokio::test]
async fn test_validate_command_missing_required_fields() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("missing-fields.json");

    // Create record with missing required fields
    let invalid_record = r#"{
  "records": [
    {
      "record_type": "AX",
      "version": "1.0",
      "agent": {
        "name": "Test Agent",
        "description": "A test agent"
      },
      "endpoints": []
    }
  ]
}"#;

    fs::write(&test_file, invalid_record).expect("Failed to write test file");

    let cmd = open_tethyr_cli::commands::validate::ValidateCommand { file: test_file };

    let result = cmd.execute().await;
    assert!(
        result.is_err(),
        "Validate command should fail for missing required fields"
    );
}

#[tokio::test]
async fn test_validate_command_invalid_json() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("invalid-json.json");

    // Create invalid JSON
    let invalid_json = r#"{
  "records": [
    {
      "record_type": "AX",
      "version": "1.0",
      "agent": {
        "name": "Test Agent"
        "description": "Missing comma"
      }
    }
  ]
}"#;

    fs::write(&test_file, invalid_json).expect("Failed to write test file");

    let cmd = open_tethyr_cli::commands::validate::ValidateCommand { file: test_file };

    let result = cmd.execute().await;
    assert!(
        result.is_err(),
        "Validate command should fail for invalid JSON"
    );
}

#[tokio::test]
async fn test_validate_command_file_not_found() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("nonexistent.json");

    let cmd = open_tethyr_cli::commands::validate::ValidateCommand { file: test_file };

    let result = cmd.execute().await;
    assert!(
        result.is_err(),
        "Validate command should fail for nonexistent file"
    );
}

#[tokio::test]
async fn test_validate_command_with_optional_fields() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("with-optional-fields.json");

    // Create record with optional fields
    let record_with_optional = r#"{
  "records": [
    {
      "record_type": "AX",
      "version": "1.0",
      "agent": {
        "name": "Advanced Agent",
        "description": "An agent with all optional fields",
        "provider": "Test Provider"
      },
      "endpoints": [
        {
          "protocol": "rest",
          "url": "https://api.example.com/agents/advanced",
          "auth": ["OAuth2"],
          "content_type": "application/json"
        }
      ],
      "capabilities": {
        "async": true,
        "supports_callbacks": true
      },
      "limits": {
        "max_requests_per_minute": 100,
        "max_concurrent_requests": 10
      },
      "security": {
        "oauth": {
          "issuer": "https://auth.example.com",
          "authorization_endpoint": "https://auth.example.com/oauth2/authorize",
          "token_endpoint": "https://auth.example.com/oauth2/token"
        }
      }
    }
  ]
}"#;

    fs::write(&test_file, record_with_optional).expect("Failed to write test file");

    let cmd = open_tethyr_cli::commands::validate::ValidateCommand { file: test_file };

    let result = cmd.execute().await;
    assert!(
        result.is_ok(),
        "Validate command should succeed for record with optional fields: {:?}",
        result
    );
}
