//! Property 10: Configuration Inheritance Correctness

use open_tethyr::config::*;

#[test]
fn agent_inherits_defaults_provider() {
    let defaults = AgentDefaults {
        provider: Some("default-corp".into()),
        ..Default::default()
    };
    let agent = AgentDefinition {
        name: "test".into(), description: "desc".into(),
        provider: None, endpoints: vec![], auth: vec![],
        capabilities: None, limits: None, security: None, extensions: None,
        oauth_provider: None, oauth_domain: None,
    };
    let merged = ConfigMerger::merge_agent(&defaults, &agent);
    assert_eq!(merged.provider, Some("default-corp".into()));
}

#[test]
fn agent_override_takes_precedence() {
    let defaults = AgentDefaults {
        provider: Some("default-corp".into()),
        auth: vec!["OAuth2".into()],
        ..Default::default()
    };
    let agent = AgentDefinition {
        name: "test".into(), description: "desc".into(),
        provider: Some("custom-corp".into()),
        endpoints: vec![], auth: vec!["JWT".into()],
        capabilities: None, limits: None, security: None, extensions: None,
        oauth_provider: None, oauth_domain: None,
    };
    let merged = ConfigMerger::merge_agent(&defaults, &agent);
    assert_eq!(merged.provider, Some("custom-corp".into()));
    assert_eq!(merged.auth, vec!["JWT".to_string()]);
}

#[test]
fn empty_agent_auth_inherits_defaults() {
    let defaults = AgentDefaults {
        auth: vec!["mTLS".into()],
        ..Default::default()
    };
    let agent = AgentDefinition {
        name: "test".into(), description: "desc".into(),
        provider: None, endpoints: vec![], auth: vec![],
        capabilities: None, limits: None, security: None, extensions: None,
        oauth_provider: None, oauth_domain: None,
    };
    let merged = ConfigMerger::merge_agent(&defaults, &agent);
    assert_eq!(merged.auth, vec!["mTLS".to_string()]);
}
