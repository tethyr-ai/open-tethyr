//! Config Models Tests - comprehensive coverage for all configuration data structures

use open_tethyr::config::models::*;
use serde_json::json;

#[test]
fn agent_config_default() {
    let config = AgentConfig::default();
    assert!(config.agents.is_empty());
    assert!(config.server.is_none());

    // Test defaults are properly initialized
    let defaults = config.defaults;
    assert!(defaults.provider.is_none());
    assert!(defaults.auth.is_empty());
    assert!(defaults.endpoints.is_empty());
    assert!(defaults.capabilities.is_none());
    assert!(defaults.limits.is_none());
    assert!(defaults.extensions.is_none());
}

#[test]
fn agent_defaults_serialization() {
    let defaults = AgentDefaults {
        provider: Some("test-corp".to_string()),
        auth: vec!["OAuth2".to_string(), "JWT".to_string()],
        endpoints: vec![],
        capabilities: Some(json!({"async": true})),
        limits: Some(json!({"max_requests": 100})),
        extensions: Some(json!({"custom": "value"})),
    };

    let json_str = serde_json::to_string(&defaults).unwrap();
    let deserialized: AgentDefaults = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.provider, Some("test-corp".to_string()));
    assert_eq!(deserialized.auth, vec!["OAuth2", "JWT"]);
    assert!(deserialized.capabilities.is_some());
    assert!(deserialized.limits.is_some());
    assert!(deserialized.extensions.is_some());
}

#[test]
fn agent_defaults_default() {
    let defaults = AgentDefaults::default();
    assert!(defaults.provider.is_none());
    assert!(defaults.auth.is_empty());
    assert!(defaults.endpoints.is_empty());
    assert!(defaults.capabilities.is_none());
    assert!(defaults.limits.is_none());
    assert!(defaults.extensions.is_none());
}

#[test]
fn agent_definition_serialization() {
    let agent = AgentDefinition {
        name: "test-agent".to_string(),
        description: "Test agent description".to_string(),
        provider: Some("acme-corp".to_string()),
        endpoints: vec![EndpointDefinition {
            protocol: "rest".to_string(),
            url: "https://api.example.com".to_string(),
            auth: vec!["OAuth2".to_string()],
            content_type: Some("application/json".to_string()),
        }],
        auth: vec!["JWT".to_string()],
        capabilities: Some(json!({"intents": ["billing"]})),
        limits: Some(json!({"rate_limit": 60})),
        security: Some(json!({"issuer": "https://auth.example.com"})),
        extensions: Some(json!({"vendor": {"tier": "premium"}})),
        oauth_provider: Some("google".to_string()),
        oauth_domain: Some("example.com".to_string()),
    };

    let json_str = serde_json::to_string(&agent).unwrap();
    let deserialized: AgentDefinition = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.name, "test-agent");
    assert_eq!(deserialized.description, "Test agent description");
    assert_eq!(deserialized.provider, Some("acme-corp".to_string()));
    assert_eq!(deserialized.endpoints.len(), 1);
    assert_eq!(deserialized.auth, vec!["JWT"]);
    assert!(deserialized.capabilities.is_some());
    assert!(deserialized.limits.is_some());
    assert!(deserialized.security.is_some());
    assert!(deserialized.extensions.is_some());
    assert_eq!(deserialized.oauth_provider, Some("google".to_string()));
    assert_eq!(deserialized.oauth_domain, Some("example.com".to_string()));
}

#[test]
fn endpoint_definition_serialization() {
    let endpoint = EndpointDefinition {
        protocol: "mcp".to_string(),
        url: "https://mcp.example.com".to_string(),
        auth: vec!["mTLS".to_string()],
        content_type: None,
    };

    let json_str = serde_json::to_string(&endpoint).unwrap();
    let deserialized: EndpointDefinition = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.protocol, "mcp");
    assert_eq!(deserialized.url, "https://mcp.example.com");
    assert_eq!(deserialized.auth, vec!["mTLS"]);
    assert!(deserialized.content_type.is_none());
}

#[test]
fn server_config_defaults() {
    let yaml_content = r#"
server:
  cache: {}
  policy: {}
  rate_limit: {}
"#;
    let config: AgentConfig = serde_yaml::from_str(yaml_content).unwrap();
    let server = config.server.unwrap();

    assert_eq!(server.domain, "localhost");
    assert_eq!(server.port, 8080);
    assert_eq!(server.log_level, "info");
}

#[test]
fn cache_config_default() {
    let cache = CacheConfig::default();
    assert_eq!(cache.max_entries, 10_000);
    assert_eq!(cache.default_ttl, 3600);
    assert_eq!(cache.cleanup_interval, 300);
    assert_eq!(cache.enable_lru, true);
}

#[test]
fn cache_config_serialization() {
    let cache = CacheConfig {
        max_entries: 5000,
        default_ttl: 1800,
        cleanup_interval: 600,
        enable_lru: false,
    };

    let json_str = serde_json::to_string(&cache).unwrap();
    let deserialized: CacheConfig = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.max_entries, 5000);
    assert_eq!(deserialized.default_ttl, 1800);
    assert_eq!(deserialized.cleanup_interval, 600);
    assert_eq!(deserialized.enable_lru, false);
}

#[test]
fn policy_config_default() {
    let policy = PolicyConfig::default();
    assert_eq!(policy.domain_locking, false);
    assert!(policy.home_domain.is_none());
    assert!(policy.allowlist.is_empty());
}

#[test]
fn policy_config_serialization() {
    let policy = PolicyConfig {
        domain_locking: true,
        home_domain: Some("example.com".to_string()),
        allowlist: vec!["trusted1.com".to_string(), "trusted2.com".to_string()],
    };

    let json_str = serde_json::to_string(&policy).unwrap();
    let deserialized: PolicyConfig = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.domain_locking, true);
    assert_eq!(deserialized.home_domain, Some("example.com".to_string()));
    assert_eq!(deserialized.allowlist, vec!["trusted1.com", "trusted2.com"]);
}

#[test]
fn rate_limit_config_default() {
    let rate_limit = RateLimitConfig::default();
    assert_eq!(rate_limit.requests_per_minute, 60);
    assert_eq!(rate_limit.requests_per_hour, 1000);
}

#[test]
fn rate_limit_config_serialization() {
    let rate_limit = RateLimitConfig {
        requests_per_minute: 120,
        requests_per_hour: 2000,
    };

    let json_str = serde_json::to_string(&rate_limit).unwrap();
    let deserialized: RateLimitConfig = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.requests_per_minute, 120);
    assert_eq!(deserialized.requests_per_hour, 2000);
}

#[test]
fn complete_agent_config_yaml_deserialization() {
    let yaml_content = r#"
defaults:
  provider: "default-corp"
  auth: ["OAuth2"]
  capabilities:
    async: true
  limits:
    max_concurrent_tasks: 10
  extensions:
    vendor:
      tier: "standard"

agents:
  - name: "agent1"
    description: "First agent"
    provider: "specific-corp"
    endpoints:
      - protocol: "rest"
        url: "https://api1.example.com"
        auth: ["JWT"]
        content_type: "application/json"
    auth: ["mTLS"]
    oauth_provider: "github"
    oauth_domain: "github.com"

server:
  domain: "api.example.com"
  port: 8443
  log_level: "warn"
  cache:
    max_entries: 5000
    default_ttl: 7200
    cleanup_interval: 600
    enable_lru: false
  policy:
    domain_locking: true
    home_domain: "example.com"
    allowlist: ["trusted.com"]
  rate_limit:
    requests_per_minute: 120
    requests_per_hour: 2400
"#;

    let config: AgentConfig = serde_yaml::from_str(yaml_content).unwrap();

    // Test defaults
    assert_eq!(config.defaults.provider, Some("default-corp".to_string()));
    assert_eq!(config.defaults.auth, vec!["OAuth2"]);

    // Test agents
    assert_eq!(config.agents.len(), 1);
    let agent = &config.agents[0];
    assert_eq!(agent.name, "agent1");
    assert_eq!(agent.description, "First agent");
    assert_eq!(agent.provider, Some("specific-corp".to_string()));
    assert_eq!(agent.endpoints.len(), 1);
    assert_eq!(agent.endpoints[0].protocol, "rest");
    assert_eq!(agent.auth, vec!["mTLS"]);
    assert_eq!(agent.oauth_provider, Some("github".to_string()));

    // Test server config
    let server = config.server.unwrap();
    assert_eq!(server.domain, "api.example.com");
    assert_eq!(server.port, 8443);
    assert_eq!(server.log_level, "warn");

    // Test cache config
    assert_eq!(server.cache.max_entries, 5000);
    assert_eq!(server.cache.default_ttl, 7200);
    assert_eq!(server.cache.cleanup_interval, 600);
    assert_eq!(server.cache.enable_lru, false);

    // Test policy config
    assert_eq!(server.policy.domain_locking, true);
    assert_eq!(server.policy.home_domain, Some("example.com".to_string()));
    assert_eq!(server.policy.allowlist, vec!["trusted.com"]);

    // Test rate limit config
    assert_eq!(server.rate_limit.requests_per_minute, 120);
    assert_eq!(server.rate_limit.requests_per_hour, 2400);
}

#[test]
fn minimal_agent_config() {
    let yaml_content = r#"
agents:
  - name: "minimal"
    description: "Minimal agent"
    endpoints:
      - protocol: "rest"
        url: "https://minimal.example.com"
"#;

    let config: AgentConfig = serde_yaml::from_str(yaml_content).unwrap();

    // Test that defaults are properly applied
    assert!(config.defaults.provider.is_none());
    assert!(config.defaults.auth.is_empty());
    assert!(config.server.is_none());

    // Test minimal agent
    assert_eq!(config.agents.len(), 1);
    let agent = &config.agents[0];
    assert_eq!(agent.name, "minimal");
    assert_eq!(agent.description, "Minimal agent");
    assert!(agent.provider.is_none());
    assert_eq!(agent.endpoints.len(), 1);
    assert!(agent.auth.is_empty());
    assert!(agent.capabilities.is_none());
    assert!(agent.oauth_provider.is_none());
}

#[test]
fn empty_config_deserializes() {
    let yaml_content = "{}";
    let config: AgentConfig = serde_yaml::from_str(yaml_content).unwrap();

    assert!(config.agents.is_empty());
    assert!(config.server.is_none());
    assert!(config.defaults.provider.is_none());
}
