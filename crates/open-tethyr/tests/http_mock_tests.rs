//! HTTP Mock Tests - verify the HTTP client against simulated AX endpoints
//! These catch parsing bugs, timeout handling, and error response handling.

use open_tethyr::http::AxHttpClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn valid_ax_json() -> String {
    serde_json::json!({
        "record_type": "AX",
        "version": "1.0",
        "agent": { "name": "test-agent", "description": "A test agent" },
        "endpoints": [{ "protocol": "rest", "url": "https://api.example.com" }]
    })
    .to_string()
}

#[tokio::test]
async fn fetch_valid_ax_record_from_mock() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/discover/example.com"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(valid_ax_json())
                .insert_header("content-type", "application/json"),
        )
        .mount(&mock_server)
        .await;

    let client = AxHttpClient::new(Some(5)).unwrap();
    let record = client
        .fetch_from_cache(&mock_server.uri(), "example.com")
        .await
        .unwrap();
    assert_eq!(record.record_type, "AX");
    assert_eq!(record.version, "1.0");
    assert_eq!(record.agent.name, "test-agent");
    assert_eq!(record.endpoints.len(), 1);
}

#[tokio::test]
async fn fetch_returns_error_on_404() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/discover/missing.com"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let client = AxHttpClient::new(Some(5)).unwrap();
    let result = client
        .fetch_from_cache(&mock_server.uri(), "missing.com")
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("404"), "Error should mention 404: {}", err);
}

#[tokio::test]
async fn fetch_returns_error_on_502() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/discover/broken.com"))
        .respond_with(ResponseTemplate::new(502))
        .mount(&mock_server)
        .await;

    let client = AxHttpClient::new(Some(5)).unwrap();
    let result = client
        .fetch_from_cache(&mock_server.uri(), "broken.com")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn fetch_returns_error_on_invalid_json() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/discover/garbled.com"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("this is not json")
                .insert_header("content-type", "application/json"),
        )
        .mount(&mock_server)
        .await;

    let client = AxHttpClient::new(Some(5)).unwrap();
    let result = client
        .fetch_from_cache(&mock_server.uri(), "garbled.com")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn fetch_returns_error_on_valid_json_but_not_ax() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/discover/notax.com"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"status":"ok","data":[]}"#)
                .insert_header("content-type", "application/json"),
        )
        .mount(&mock_server)
        .await;

    let client = AxHttpClient::new(Some(5)).unwrap();
    let result = client
        .fetch_from_cache(&mock_server.uri(), "notax.com")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn fetch_handles_timeout() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/discover/slow.com"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(valid_ax_json())
                .set_delay(std::time::Duration::from_secs(10)),
        )
        .mount(&mock_server)
        .await;

    let client = AxHttpClient::new(Some(1)).unwrap(); // 1 second timeout
    let result = client
        .fetch_from_cache(&mock_server.uri(), "slow.com")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn fetch_handles_empty_body() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/discover/empty.com"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("")
                .insert_header("content-type", "application/json"),
        )
        .mount(&mock_server)
        .await;

    let client = AxHttpClient::new(Some(5)).unwrap();
    let result = client
        .fetch_from_cache(&mock_server.uri(), "empty.com")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn fetch_ax_record_with_optional_fields() {
    let mock_server = MockServer::start().await;
    let json = serde_json::json!({
        "record_type": "AX",
        "version": "1.0",
        "agent": { "name": "minimal", "description": "Minimal agent" },
        "endpoints": [{ "protocol": "mcp", "url": "https://mcp.example.com" }],
        "capabilities": { "intents": ["billing", "support"], "async": true },
        "security": { "issuer": "https://auth.example.com", "jwks_url": "https://auth.example.com/.well-known/jwks.json" },
        "limits": { "rate_limit_per_minute": 60 }
    }).to_string();
    Mock::given(method("GET"))
        .and(path("/discover/rich.com"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(json)
                .insert_header("content-type", "application/json"),
        )
        .mount(&mock_server)
        .await;

    let client = AxHttpClient::new(Some(5)).unwrap();
    let record = client
        .fetch_from_cache(&mock_server.uri(), "rich.com")
        .await
        .unwrap();
    assert_eq!(record.agent.name, "minimal");
    let caps = record.capabilities.unwrap();
    assert_eq!(caps.intents, vec!["billing", "support"]);
    assert_eq!(caps.async_exec, Some(true));
    let sec = record.security.unwrap();
    assert_eq!(sec.issuer.as_deref(), Some("https://auth.example.com"));
    let limits = record.limits.unwrap();
    assert_eq!(limits.rate_limit_per_minute, Some(60.0));
}

#[tokio::test]
async fn fetch_ax_record_with_unknown_extensions() {
    let mock_server = MockServer::start().await;
    let json = serde_json::json!({
        "record_type": "AX",
        "version": "1.0",
        "agent": { "name": "ext-agent", "description": "Agent with extensions", "custom_field": "value" },
        "endpoints": [{ "protocol": "rest", "url": "https://api.example.com", "custom_header": "X-Custom" }],
        "extensions": { "ax": { "capability_hash": "sha256:abc123" }, "vendor": { "billing_tier": "premium" } }
    }).to_string();
    Mock::given(method("GET"))
        .and(path("/discover/extended.com"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(json)
                .insert_header("content-type", "application/json"),
        )
        .mount(&mock_server)
        .await;

    let client = AxHttpClient::new(Some(5)).unwrap();
    let record = client
        .fetch_from_cache(&mock_server.uri(), "extended.com")
        .await
        .unwrap();
    assert_eq!(record.agent.name, "ext-agent");
    assert!(record.extensions.is_some());
}
