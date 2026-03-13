//! Test Utilities and Fixtures
use open_tethyr::ax::*;
use proptest::prelude::*;

pub fn arb_auth_method() -> impl Strategy<Value = String> {
    prop_oneof![Just("OIDC".into()), Just("OAuth2".into()), Just("mTLS".into()), Just("JWT".into()), Just("AWS_IAM".into())]
}

pub fn arb_domain() -> impl Strategy<Value = String> { "[a-z]{3,8}\\.[a-z]{2,4}".prop_map(|s| s) }

pub fn arb_protocol() -> impl Strategy<Value = Protocol> {
    prop_oneof![Just(Protocol::Rest), Just(Protocol::GraphQL), Just(Protocol::MCP), Just(Protocol::A2A)]
}

pub fn arb_endpoint() -> impl Strategy<Value = Endpoint> {
    (arb_protocol(), arb_domain(), prop::collection::vec(arb_auth_method(), 1..3))
        .prop_map(|(protocol, domain, auth)| Endpoint {
            protocol, url: format!("https://{}/api", domain), auth,
            content_type: Some("application/json".into()), extra: Default::default(),
        })
}

pub fn arb_agent() -> impl Strategy<Value = Agent> {
    ("[a-z]{3,12}", "[a-z ]{5,20}", "[a-z]{3,10}")
        .prop_map(|(name, desc, provider)| Agent { name, description: desc, provider: Some(provider) })
}

pub fn arb_agent_record() -> impl Strategy<Value = AgentExchangeRecord> {
    (arb_agent(), prop::collection::vec(arb_endpoint(), 1..4))
        .prop_map(|(agent, endpoints)| AgentExchangeRecord {
            record_type: "AX".into(), version: "1.0".into(), agent, endpoints,
            capabilities: None, schema: None, limits: None, security: None, extensions: None,
        })
}

pub fn valid_test_record() -> AgentExchangeRecord {
    AgentExchangeRecord {
        record_type: "AX".into(), version: "1.0".into(),
        agent: Agent { name: "test-agent".into(), description: "A test agent".into(), provider: Some("test-corp".into()) },
        endpoints: vec![Endpoint { protocol: Protocol::Rest, url: "https://api.example.com/v1".into(),
            auth: vec!["OAuth2".into(), "API_KEY".into()], content_type: Some("application/json".into()), extra: Default::default() }],
        capabilities: None, schema: None, limits: None, security: None, extensions: None,
    }
}
