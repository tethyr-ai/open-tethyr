use open_tethyr::ax::*;

#[test]
fn valid_record_no_errors() {
    let r = AgentExchangeRecord {
        record_type: "AX".into(),
        version: "1.0".into(),
        agent: Agent {
            name: "t".into(),
            description: "d".into(),
            provider: Some("c".into()),
        },
        endpoints: vec![Endpoint {
            protocol: Protocol::Rest,
            url: "https://x.com".into(),
            auth: vec!["OAuth2".into()],
            content_type: None,
            extra: Default::default(),
        }],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    };
    let report = AxValidator::validate_record_detailed(&r);
    assert!(!report.has_errors());
}

#[test]
fn missing_provider_is_ok() {
    let r = AgentExchangeRecord {
        record_type: "AX".into(),
        version: "1.0".into(),
        agent: Agent {
            name: "t".into(),
            description: "d".into(),
            provider: None,
        },
        endpoints: vec![Endpoint {
            protocol: Protocol::Rest,
            url: "https://x.com".into(),
            auth: vec![],
            content_type: None,
            extra: Default::default(),
        }],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    };
    assert!(!AxValidator::validate_record_detailed(&r).has_errors());
}

#[test]
fn version_2_0_is_warning() {
    let r = AgentExchangeRecord {
        record_type: "AX".into(),
        version: "2.0".into(),
        agent: Agent {
            name: "t".into(),
            description: "d".into(),
            provider: None,
        },
        endpoints: vec![Endpoint {
            protocol: Protocol::Rest,
            url: "https://x.com".into(),
            auth: vec![],
            content_type: None,
            extra: Default::default(),
        }],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    };
    let report = AxValidator::validate_record_detailed(&r);
    assert!(!report.warnings().is_empty());
}

#[test]
fn unknown_auth_is_warning_not_error() {
    let r = AgentExchangeRecord {
        record_type: "AX".into(),
        version: "1.0".into(),
        agent: Agent {
            name: "t".into(),
            description: "d".into(),
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
    let report = AxValidator::validate_record_detailed(&r);
    assert!(!report.has_errors());
    assert!(!report.warnings().is_empty());
}
