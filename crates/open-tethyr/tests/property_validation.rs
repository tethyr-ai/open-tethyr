use open_tethyr::ax::*;
use open_tethyr::error::AxError;
use proptest::prelude::*;

fn valid_record() -> AgentExchangeRecord {
    AgentExchangeRecord {
        record_type: "AX".into(),
        version: "1.0".into(),
        agent: Agent {
            name: "test".into(),
            description: "test agent".into(),
            provider: Some("acme".into()),
        },
        endpoints: vec![Endpoint {
            protocol: Protocol::Rest,
            url: "https://api.example.com".into(),
            auth: vec!["OAuth2".into()],
            content_type: None,
            extra: Default::default(),
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
fn missing_provider_still_valid() {
    let mut r = valid_record();
    r.agent.provider = None;
    assert!(AxValidator::validate_record(&r).is_ok());
}

#[test]
fn empty_auth_still_valid() {
    let mut r = valid_record();
    r.endpoints[0].auth.clear();
    assert!(AxValidator::validate_record(&r).is_ok());
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

proptest! {
    #[test]
    fn random_record_type_rejected(rt in "[A-Z]{1,5}".prop_filter("not AX", |s| s != "AX")) {
        let mut r = valid_record();
        r.record_type = rt;
        prop_assert!(AxValidator::validate_record(&r).is_err());
    }
}
