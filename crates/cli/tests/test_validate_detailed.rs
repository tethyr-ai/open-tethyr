//! Integration test: detailed validation with severity levels

use open_tethyr::ax::*;

#[test]
fn valid_record_no_errors() {
    let record = AgentExchangeRecord {
        record_type: "AX".into(),
        version: "1.0".into(),
        agent: Agent {
            name: "test".into(),
            description: "desc".into(),
            provider: "corp".into(),
        },
        endpoints: vec![Endpoint {
            protocol: Protocol::Rest,
            url: "https://api.example.com".into(),
            auth: vec!["OAuth2".into()],
            content_type: None,
        }],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    };
    let report = AxValidator::validate_record_detailed(&record);
    assert!(!report.has_errors());
    assert!(report.warnings().is_empty());
}

#[test]
fn missing_agent_name_is_error() {
    let record = AgentExchangeRecord {
        record_type: "AX".into(),
        version: "1.0".into(),
        agent: Agent {
            name: "".into(),
            description: "desc".into(),
            provider: "corp".into(),
        },
        endpoints: vec![Endpoint {
            protocol: Protocol::Rest,
            url: "https://api.example.com".into(),
            auth: vec!["OAuth2".into()],
            content_type: None,
        }],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    };
    let report = AxValidator::validate_record_detailed(&record);
    assert!(report.has_errors());
    assert!(report.errors().iter().any(|e| e.field == "agent.name"));
}

#[test]
fn version_2_0_is_warning() {
    let record = AgentExchangeRecord {
        record_type: "AX".into(),
        version: "2.0".into(),
        agent: Agent {
            name: "test".into(),
            description: "desc".into(),
            provider: "corp".into(),
        },
        endpoints: vec![Endpoint {
            protocol: Protocol::Rest,
            url: "https://api.example.com".into(),
            auth: vec!["OAuth2".into()],
            content_type: None,
        }],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    };
    let report = AxValidator::validate_record_detailed(&record);
    assert!(!report.warnings().is_empty());
    assert!(report.warnings().iter().any(|w| w.field == "version"));
}

#[test]
fn invalid_auth_method_is_error() {
    let record = AgentExchangeRecord {
        record_type: "AX".into(),
        version: "1.0".into(),
        agent: Agent {
            name: "test".into(),
            description: "desc".into(),
            provider: "corp".into(),
        },
        endpoints: vec![Endpoint {
            protocol: Protocol::Rest,
            url: "https://api.example.com".into(),
            auth: vec!["INVALID".into()],
            content_type: None,
        }],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    };
    let report = AxValidator::validate_record_detailed(&record);
    assert!(report.has_errors());
    assert!(report
        .errors()
        .iter()
        .any(|e| e.message.contains("INVALID")));
}
