//! Malformed Input Tests - ensure the parser doesn't panic on garbage
//! Critical for infrastructure receiving untrusted data from the network.

use open_tethyr::ax::{parse_ax_json, AxValidator};

// === Malformed JSON ===

#[test]
fn empty_string_fails_gracefully() {
    assert!(parse_ax_json("").is_err());
}

#[test]
fn null_json_fails_gracefully() {
    assert!(parse_ax_json("null").is_err());
}

#[test]
fn number_json_fails_gracefully() {
    assert!(parse_ax_json("42").is_err());
}

#[test]
fn string_json_fails_gracefully() {
    assert!(parse_ax_json("\"hello\"").is_err());
}

#[test]
fn array_json_fails_gracefully() {
    assert!(parse_ax_json("[1,2,3]").is_err());
}

#[test]
fn truncated_json_fails_gracefully() {
    assert!(parse_ax_json(r#"{"record_type":"AX","version"#).is_err());
}

#[test]
fn deeply_nested_json_fails_gracefully() {
    let deep = "{".repeat(100) + &"}".repeat(100);
    assert!(parse_ax_json(&deep).is_err());
}

#[test]
fn huge_string_value_fails_gracefully() {
    let huge = format!(
        r#"{{"record_type":"AX","version":"1.0","agent":{{"name":"{}","description":"d"}},"endpoints":[{{"protocol":"rest","url":"https://x.com"}}]}}"#,
        "A".repeat(1_000_000)
    );
    // Should parse (large but valid) or fail, but NOT panic
    let _ = parse_ax_json(&huge);
}

#[test]
fn unicode_fields_handled() {
    let json = r#"{"record_type":"AX","version":"1.0","agent":{"name":"テスト","description":"Üñíçödé"},"endpoints":[{"protocol":"rest","url":"https://日本語.com"}]}"#;
    let records = parse_ax_json(json).unwrap();
    assert_eq!(records[0].agent.name, "テスト");
}

#[test]
fn extra_unknown_fields_preserved() {
    let json = r#"{"record_type":"AX","version":"1.0","agent":{"name":"t","description":"d","custom_field":"value"},"endpoints":[{"protocol":"rest","url":"https://x.com","custom_header":"X-Custom"}],"unknown_top_level":"preserved"}"#;
    let records = parse_ax_json(json).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].agent.name, "t");
}

// === Backward Compatibility ===

#[test]
fn flat_document_parses() {
    let json = r#"{"record_type":"AX","version":"1.0","agent":{"name":"flat","description":"d"},"endpoints":[{"protocol":"rest","url":"https://x.com"}]}"#;
    let records = parse_ax_json(json).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].agent.name, "flat");
}

#[test]
fn legacy_wrapper_parses() {
    let json = r#"{"records":[{"record_type":"AX","version":"1.0","agent":{"name":"legacy","description":"d"},"endpoints":[{"protocol":"rest","url":"https://x.com"}]}]}"#;
    let records = parse_ax_json(json).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].agent.name, "legacy");
}

#[test]
fn legacy_wrapper_multiple_records() {
    let json = r#"{"records":[
        {"record_type":"AX","version":"1.0","agent":{"name":"a1","description":"d1"},"endpoints":[{"protocol":"rest","url":"https://a.com"}]},
        {"record_type":"AX","version":"1.0","agent":{"name":"a2","description":"d2"},"endpoints":[{"protocol":"mcp","url":"https://b.com"}]}
    ]}"#;
    let records = parse_ax_json(json).unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].agent.name, "a1");
    assert_eq!(records[1].agent.name, "a2");
}

#[test]
fn minimal_valid_record_per_spec() {
    let json = r#"{"record_type":"AX","version":"1.0","agent":{"name":"min","description":"d"},"endpoints":[{"protocol":"rest","url":"https://x.com"}]}"#;
    let records = parse_ax_json(json).unwrap();
    assert!(AxValidator::validate_record(&records[0]).is_ok());
    // provider is absent (optional per spec)
    assert!(records[0].agent.provider.is_none());
    // auth is absent (optional per spec)
    assert!(records[0].endpoints[0].auth.is_empty());
}

#[test]
fn fully_populated_record() {
    let json = serde_json::json!({
        "record_type": "AX", "version": "1.0",
        "agent": { "name": "full", "description": "Full agent", "provider": "acme" },
        "endpoints": [
            { "protocol": "rest", "url": "https://api.example.com", "auth": ["OAuth2", "JWT"], "content_type": "application/json" },
            { "protocol": "mcp", "url": "https://mcp.example.com", "auth": ["mTLS"] },
            { "protocol": "a2a", "url": "https://a2a.example.com" }
        ],
        "capabilities": { "intents": ["billing", "support"], "async": true, "supports_callbacks": true, "callback_modes": ["webhook", "poll"] },
        "schema": { "rest_openapi_url": "https://api.example.com/openapi.json", "mcp_manifest_url": "https://mcp.example.com/manifest", "introspection": false },
        "limits": { "max_concurrent_tasks": 10, "max_task_ttl_seconds": 300, "rate_limit_per_minute": 60 },
        "security": { "issuer": "https://auth.example.com", "jwks_url": "https://auth.example.com/.well-known/jwks.json" },
        "extensions": { "ax": { "capability_hash": "sha256:abc123" }, "vendor": { "tier": "premium" } }
    }).to_string();

    let records = parse_ax_json(&json).unwrap();
    let r = &records[0];
    assert!(AxValidator::validate_record(r).is_ok());
    assert_eq!(r.endpoints.len(), 3);
    let caps = r.capabilities.as_ref().unwrap();
    assert_eq!(caps.intents, vec!["billing", "support"]);
    assert_eq!(caps.async_exec, Some(true));
    let schema = r.schema.as_ref().unwrap();
    assert!(schema.rest_openapi_url.is_some());
    let limits = r.limits.as_ref().unwrap();
    assert_eq!(limits.max_concurrent_tasks, Some(10.0));
    let sec = r.security.as_ref().unwrap();
    assert_eq!(sec.issuer.as_deref(), Some("https://auth.example.com"));
}

// === Adversarial Inputs ===

#[test]
fn wrong_type_for_record_type_field() {
    let json = r#"{"record_type":42,"version":"1.0","agent":{"name":"t","description":"d"},"endpoints":[{"protocol":"rest","url":"https://x.com"}]}"#;
    assert!(parse_ax_json(json).is_err());
}

#[test]
fn wrong_type_for_endpoints_field() {
    let json = r#"{"record_type":"AX","version":"1.0","agent":{"name":"t","description":"d"},"endpoints":"not-an-array"}"#;
    assert!(parse_ax_json(json).is_err());
}

#[test]
fn wrong_type_for_agent_field() {
    let json = r#"{"record_type":"AX","version":"1.0","agent":"not-an-object","endpoints":[{"protocol":"rest","url":"https://x.com"}]}"#;
    assert!(parse_ax_json(json).is_err());
}

#[test]
fn empty_endpoints_array_parseable_but_invalid() {
    let json = r#"{"record_type":"AX","version":"1.0","agent":{"name":"t","description":"d"},"endpoints":[]}"#;
    let records = parse_ax_json(json).unwrap();
    assert!(AxValidator::validate_record(&records[0]).is_err());
}

#[test]
fn null_optional_fields_handled() {
    let json = r#"{"record_type":"AX","version":"1.0","agent":{"name":"t","description":"d","provider":null},"endpoints":[{"protocol":"rest","url":"https://x.com"}],"capabilities":null,"schema":null,"limits":null,"security":null,"extensions":null}"#;
    let records = parse_ax_json(json).unwrap();
    assert!(AxValidator::validate_record(&records[0]).is_ok());
}
