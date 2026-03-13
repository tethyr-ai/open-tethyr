use open_tethyr::error::{is_known_auth_method, KNOWN_AUTH_METHODS};

#[test]
fn known_methods_accepted() {
    for method in KNOWN_AUTH_METHODS {
        assert!(is_known_auth_method(method));
    }
}

#[test]
fn aws_iam_accepted() {
    assert!(is_known_auth_method("AWS_IAM"));
}

#[test]
fn api_key_accepted() {
    assert!(is_known_auth_method("API_KEY"));
}

#[test]
fn unknown_methods_not_known() {
    assert!(!is_known_auth_method("INVALID"));
    assert!(!is_known_auth_method("basic"));
}

#[test]
fn unknown_auth_produces_warning_not_error() {
    use open_tethyr::ax::*;
    let record = AgentExchangeRecord {
        record_type: "AX".into(),
        version: "1.0".into(),
        agent: Agent {
            name: "t".into(),
            description: "t".into(),
            provider: None,
        },
        endpoints: vec![Endpoint {
            protocol: Protocol::Rest,
            url: "https://x.com".into(),
            auth: vec!["CustomAuth".into()],
            content_type: None,
            extra: Default::default(),
        }],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    };
    // Simple validation should NOT error on unknown auth (per spec)
    assert!(AxValidator::validate_record(&record).is_ok());
    // Detailed validation should produce WARNING, not error
    let report = AxValidator::validate_record_detailed(&record);
    assert!(!report.has_errors());
    assert!(!report.warnings().is_empty());
}
