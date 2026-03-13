//! Test Utilities and Fixtures

use open_tethyr::ax::*;
use proptest::prelude::*;

/// Generate arbitrary auth method from approved set
pub fn arb_auth_method() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("OIDC".to_string()),
        Just("OAuth2".to_string()),
        Just("mTLS".to_string()),
        Just("JWT".to_string()),
        Just("API_KEY".to_string()),
    ]
}

/// Generate arbitrary valid domain
pub fn arb_domain() -> impl Strategy<Value = String> {
    "[a-z]{3,8}\\.[a-z]{2,4}".prop_map(|s| s)
}

/// Generate arbitrary protocol
pub fn arb_protocol() -> impl Strategy<Value = Protocol> {
    prop_oneof![
        Just(Protocol::Rest),
        Just(Protocol::GraphQL),
        Just(Protocol::MCP),
        Just(Protocol::A2A),
    ]
}

/// Generate arbitrary endpoint
pub fn arb_endpoint() -> impl Strategy<Value = Endpoint> {
    (arb_protocol(), arb_domain(), prop::collection::vec(arb_auth_method(), 1..3))
        .prop_map(|(protocol, domain, auth)| Endpoint {
            protocol,
            url: format!("https://{}/api", domain),
            auth,
            content_type: Some("application/json".to_string()),
        })
}

/// Generate arbitrary agent
pub fn arb_agent() -> impl Strategy<Value = Agent> {
    ("[a-z]{3,12}", "[a-z ]{5,20}", "[a-z]{3,10}")
        .prop_map(|(name, desc, provider)| Agent { name, description: desc, provider })
}

/// Generate arbitrary valid AX record
pub fn arb_agent_record() -> impl Strategy<Value = AgentExchangeRecord> {
    (arb_agent(), prop::collection::vec(arb_endpoint(), 1..4))
        .prop_map(|(agent, endpoints)| AgentExchangeRecord {
            record_type: "AX".to_string(),
            version: "1.0".to_string(),
            agent, endpoints,
            capabilities: None, schema: None, limits: None, security: None, extensions: None,
        })
}

/// Create a valid test record (non-proptest)
pub fn valid_test_record() -> AgentExchangeRecord {
    AgentExchangeRecord {
        record_type: "AX".to_string(),
        version: "1.0".to_string(),
        agent: Agent {
            name: "test-agent".into(),
            description: "A test agent".into(),
            provider: "test-corp".into(),
        },
        endpoints: vec![Endpoint {
            protocol: Protocol::Rest,
            url: "https://api.example.com/v1".into(),
            auth: vec!["OAuth2".into(), "API_KEY".into()],
            content_type: Some("application/json".into()),
        }],
        capabilities: None, schema: None, limits: None, security: None, extensions: None,
    }
}

/// Create a valid YAML config string
pub fn valid_yaml_config() -> &'static str {
    r#"
defaults:
  provider: "test-corp"
  auth:
    - "OAuth2"

agents:
  - name: "agent-1"
    description: "First test agent"
    endpoints:
      - protocol: "rest"
        url: "https://api.example.com/v1"
        auth:
          - "OAuth2"
"#
}
