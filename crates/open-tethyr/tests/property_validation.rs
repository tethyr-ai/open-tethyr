//! Property 2: AX Record Validation Correctness

use open_tethyr::ax::*;
use open_tethyr::error::AxError;
use proptest::prelude::*;

fn valid_record() -> AgentExchangeRecord {
    AgentExchangeRecord {
        record_type: "AX".to_string(),
        version: "1.0".to_string(),
        agent: Agent {
            name: "test".into(),
            description: "test agent".into(),
            provider: "acme".into(),
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
    }
}

#[test]
fn valid_record_passes() {
    assert!(AxValidator::validate_record(&valid_record()).is_ok());
}

#[test]
fn invalid_record_type_fails() {
    let mut r = valid_record();
    r.record_type = "INVALID".into();
    assert!(matches!(
        AxValidator::validate_record(&r),
        Err(AxError::InvalidRecordType(_))
    ));
}

#[test]
fn invalid_version_fails() {
    let mut r = valid_record();
    r.version = "2.0".into();
    assert!(matches!(
        AxValidator::validate_record(&r),
        Err(AxError::UnsupportedVersion(_))
    ));
}

#[test]
fn empty_agent_name_fails() {
    let mut r = valid_record();
    r.agent.name = "".into();
    assert!(matches!(
        AxValidator::validate_record(&r),
        Err(AxError::MissingField(_))
    ));
}

#[test]
fn empty_endpoints_fails() {
    let mut r = valid_record();
    r.endpoints.clear();
    assert!(matches!(
        AxValidator::validate_record(&r),
        Err(AxError::MissingField(_))
    ));
}

#[test]
fn invalid_auth_method_fails() {
    let mut r = valid_record();
    r.endpoints[0].auth = vec!["INVALID_METHOD".into()];
    assert!(matches!(
        AxValidator::validate_record(&r),
        Err(AxError::InvalidAuthMethod(_))
    ));
}

proptest! {
    #[test]
    fn random_record_type_rejected(rt in "[A-Z]{1,5}".prop_filter("not AX", |s| s != "AX")) {
        let mut r = valid_record();
        r.record_type = rt;
        prop_assert!(AxValidator::validate_record(&r).is_err());
    }
}
