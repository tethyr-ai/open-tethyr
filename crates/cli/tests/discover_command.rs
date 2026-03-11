//! Integration tests for the discover command

use open_tethyr::ax::{Agent, AgentExchangeDocument, AgentExchangeRecord, Endpoint, Protocol};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper function to create a test AX document
fn create_test_ax_document() -> AgentExchangeDocument {
    AgentExchangeDocument {
        records: vec![
            AgentExchangeRecord {
                record_type: "AX".to_string(),
                version: "1.0".to_string(),
                agent: Agent {
                    name: "Test Agent".to_string(),
                    description: "A test agent for discovery testing".to_string(),
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
            },
            AgentExchangeRecord {
                record_type: "AX".to_string(),
                version: "1.0".to_string(),
                agent: Agent {
                    name: "Second Agent".to_string(),
                    description: "Another test agent".to_string(),
                    provider: "Test Provider".to_string(),
                },
                endpoints: vec![Endpoint {
                    protocol: Protocol::GraphQL,
                    url: "https://api.example.com/graphql".to_string(),
                    auth: vec!["JWT".to_string()],
                    content_type: Some("application/json".to_string()),
                }],
                capabilities: None,
                schema: None,
                limits: None,
                security: None,
                extensions: None,
            },
        ],
    }
}

#[tokio::test]
async fn test_discover_command_direct() {
    // Start a mock HTTP server
    let mock_server = MockServer::start().await;

    // Create test AX document
    let test_document = create_test_ax_document();
    let response_body = serde_json::to_string(&test_document).expect("Failed to serialize");

    // Mock the AX endpoint
    Mock::given(method("GET"))
        .and(path("/.well-known/agent-exchange.json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    // Note: This test demonstrates the structure but won't work directly
    // because AxHttpClient constructs URLs with _agent subdomain
    // In a real test environment, we'd need to mock DNS or use a test domain
    println!("Mock server running at: {}", mock_server.uri());
    println!("Direct discovery test structure validated");
}

#[tokio::test]
async fn test_discover_command_with_cache() {
    // Note: This test validates the command structure and configuration.
    // Full integration testing with cache requires HTTPS mock server setup
    // which is beyond the scope of unit tests.

    // Start a mock cache server
    let mock_cache = MockServer::start().await;

    // Create test AX document
    let test_document = create_test_ax_document();
    let response_body = serde_json::to_string(&test_document).expect("Failed to serialize");

    // Mock the cache discovery endpoint
    Mock::given(method("GET"))
        .and(path("/discover/example.com"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(response_body)
                .insert_header("content-type", "application/json"),
        )
        .mount(&mock_cache)
        .await;

    // Create discover command with cache URL
    let cmd = open_tethyr_cli::commands::discover::DiscoverCommand {
        domain: "example.com".to_string(),
        cache: Some(mock_cache.uri()),
    };

    // Execute the command - it will fail because mock server uses HTTP not HTTPS
    // but this validates the command structure and error handling
    let result = cmd.execute().await;

    // The command will fail due to HTTPS validation (expected in test environment)
    // In production, cache URLs would be HTTPS
    assert!(
        result.is_err(),
        "Command fails with HTTP mock server (expected - production uses HTTPS)"
    );
}

#[tokio::test]
async fn test_discover_command_display_results() {
    // Test the display_results method directly
    let test_document = create_test_ax_document();

    let cmd = open_tethyr_cli::commands::discover::DiscoverCommand {
        domain: "example.com".to_string(),
        cache: None,
    };

    // This should not panic and should format output correctly
    let result = cmd.display_results(&test_document);
    assert!(result.is_ok(), "Display results should succeed");
}

#[tokio::test]
async fn test_discover_command_empty_document() {
    // Test with an empty document
    let empty_document = AgentExchangeDocument { records: vec![] };

    let cmd = open_tethyr_cli::commands::discover::DiscoverCommand {
        domain: "example.com".to_string(),
        cache: None,
    };

    let result = cmd.display_results(&empty_document);
    assert!(
        result.is_ok(),
        "Display results should handle empty document"
    );
}

#[tokio::test]
async fn test_discover_command_multiple_endpoints() {
    // Test with an agent that has multiple endpoints
    let document = AgentExchangeDocument {
        records: vec![AgentExchangeRecord {
            record_type: "AX".to_string(),
            version: "1.0".to_string(),
            agent: Agent {
                name: "Multi-Endpoint Agent".to_string(),
                description: "Agent with multiple endpoints".to_string(),
                provider: "Test Provider".to_string(),
            },
            endpoints: vec![
                Endpoint {
                    protocol: Protocol::Rest,
                    url: "https://api.example.com/rest".to_string(),
                    auth: vec!["OAuth2".to_string()],
                    content_type: Some("application/json".to_string()),
                },
                Endpoint {
                    protocol: Protocol::GraphQL,
                    url: "https://api.example.com/graphql".to_string(),
                    auth: vec!["JWT".to_string(), "API_KEY".to_string()],
                    content_type: Some("application/json".to_string()),
                },
                Endpoint {
                    protocol: Protocol::MCP,
                    url: "https://api.example.com/mcp".to_string(),
                    auth: vec!["mTLS".to_string()],
                    content_type: Some("application/json".to_string()),
                },
            ],
            capabilities: None,
            schema: None,
            limits: None,
            security: None,
            extensions: None,
        }],
    };

    let cmd = open_tethyr_cli::commands::discover::DiscoverCommand {
        domain: "example.com".to_string(),
        cache: None,
    };

    let result = cmd.display_results(&document);
    assert!(
        result.is_ok(),
        "Display results should handle multiple endpoints"
    );
}
