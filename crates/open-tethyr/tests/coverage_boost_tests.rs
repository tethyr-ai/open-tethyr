//! Comprehensive coverage tests for infrastructure-critical modules.
//! Covers: validator, dns, cache/memory, client, coordinator, generator.

use open_tethyr::ax::*;
use open_tethyr::cache::memory::{CacheEntry, MemoryCache};
use open_tethyr::config::*;
use open_tethyr::dns::DnsDiscovery;
use open_tethyr::error::*;
use std::time::{Duration, Instant};

// === AX Validator: validate_record_detailed ===

#[test]
fn validate_detailed_valid_record() {
    let record = make_valid_record();
    let report = AxValidator::validate_record_detailed(&record);
    assert!(!report.has_errors());
    assert!(report.errors().is_empty());
    assert!(report.warnings().is_empty());
}

#[test]
fn validate_detailed_wrong_record_type() {
    let mut record = make_valid_record();
    record.record_type = "WRONG".into();
    let report = AxValidator::validate_record_detailed(&record);
    assert!(report.has_errors());
    let errors = report.errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field, "record_type");
    assert!(errors[0].message.contains("WRONG"));
}

#[test]
fn validate_detailed_wrong_version() {
    let mut record = make_valid_record();
    record.version = "2.0".into();
    let report = AxValidator::validate_record_detailed(&record);
    assert!(!report.has_errors()); // version mismatch is a warning
    let warnings = report.warnings();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].field, "version");
}

#[test]
fn validate_detailed_empty_agent_name() {
    let mut record = make_valid_record();
    record.agent.name = "".into();
    let report = AxValidator::validate_record_detailed(&record);
    assert!(report.has_errors());
    assert!(report.errors().iter().any(|e| e.field == "agent.name"));
}

#[test]
fn validate_detailed_empty_agent_description() {
    let mut record = make_valid_record();
    record.agent.description = "".into();
    let report = AxValidator::validate_record_detailed(&record);
    assert!(report.has_errors());
    assert!(report
        .errors()
        .iter()
        .any(|e| e.field == "agent.description"));
}

#[test]
fn validate_detailed_empty_endpoints() {
    let mut record = make_valid_record();
    record.endpoints = vec![];
    let report = AxValidator::validate_record_detailed(&record);
    assert!(report.has_errors());
    assert!(report.errors().iter().any(|e| e.field == "endpoints"));
}

#[test]
fn validate_detailed_empty_endpoint_url() {
    let mut record = make_valid_record();
    record.endpoints[0].url = "".into();
    let report = AxValidator::validate_record_detailed(&record);
    assert!(report.has_errors());
    assert!(report
        .errors()
        .iter()
        .any(|e| e.field == "endpoints[0].url"));
}

#[test]
fn validate_detailed_unknown_auth_method() {
    let mut record = make_valid_record();
    record.endpoints[0].auth = vec!["CustomAuth".into()];
    let report = AxValidator::validate_record_detailed(&record);
    assert!(!report.has_errors()); // unknown auth is a warning
    let warnings = report.warnings();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].message.contains("CustomAuth"));
}

#[test]
fn validate_detailed_multiple_errors() {
    let record = AgentExchangeRecord {
        record_type: "WRONG".into(),
        version: "2.0".into(),
        agent: Agent {
            name: "".into(),
            description: "".into(),
            provider: None,
        },
        endpoints: vec![],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    };
    let report = AxValidator::validate_record_detailed(&record);
    assert!(report.has_errors());
    // record_type error + agent.name + agent.description + endpoints
    assert!(report.errors().len() >= 4);
    // version warning
    assert!(!report.warnings().is_empty());
}

#[test]
fn validate_record_empty_url() {
    let mut record = make_valid_record();
    record.endpoints[0].url = "".into();
    assert!(AxValidator::validate_record(&record).is_err());
}

#[test]
fn validate_record_unknown_auth_does_not_fail() {
    let mut record = make_valid_record();
    record.endpoints[0].auth = vec!["SomeCustom".into()];
    // Unknown auth is a warning, not an error
    assert!(AxValidator::validate_record(&record).is_ok());
}

#[test]
fn validate_version_1_0() {
    assert!(AxValidator::validate_version("1.0").is_ok());
}

#[test]
fn validate_version_unsupported() {
    assert!(AxValidator::validate_version("2.0").is_err());
    assert!(AxValidator::validate_version("").is_err());
}

#[test]
fn validate_agent_valid() {
    let agent = Agent {
        name: "test".into(),
        description: "desc".into(),
        provider: None,
    };
    assert!(AxValidator::validate_agent(&agent).is_ok());
}

#[test]
fn validate_agent_empty_name() {
    let agent = Agent {
        name: "".into(),
        description: "desc".into(),
        provider: None,
    };
    assert!(AxValidator::validate_agent(&agent).is_err());
}

#[test]
fn validate_agent_empty_description() {
    let agent = Agent {
        name: "test".into(),
        description: "".into(),
        provider: None,
    };
    assert!(AxValidator::validate_agent(&agent).is_err());
}

#[test]
fn validate_endpoints_empty() {
    assert!(AxValidator::validate_endpoints(&[]).is_err());
}

#[test]
fn validate_endpoints_empty_url() {
    let endpoints = vec![Endpoint {
        protocol: Protocol::Rest,
        url: "".into(),
        auth: vec![],
        content_type: None,
        extra: serde_json::Map::new(),
    }];
    assert!(AxValidator::validate_endpoints(&endpoints).is_err());
}

// === DNS Discovery: parse_cache_endpoint ===

#[test]
fn parse_cache_endpoint_valid() {
    let result = DnsDiscovery::parse_cache_endpoint("endpoint=https://cache.example.com");
    assert_eq!(result, Some("https://cache.example.com".to_string()));
}

#[test]
fn parse_cache_endpoint_with_quotes() {
    let result = DnsDiscovery::parse_cache_endpoint("\"endpoint=https://cache.example.com\"");
    assert_eq!(result, Some("https://cache.example.com".to_string()));
}

#[test]
fn parse_cache_endpoint_with_whitespace() {
    let result = DnsDiscovery::parse_cache_endpoint("  endpoint=https://cache.example.com  ");
    assert_eq!(result, Some("https://cache.example.com".to_string()));
}

#[test]
fn parse_cache_endpoint_empty_url() {
    let result = DnsDiscovery::parse_cache_endpoint("endpoint=");
    assert!(result.is_none());
}

#[test]
fn parse_cache_endpoint_no_prefix() {
    let result = DnsDiscovery::parse_cache_endpoint("https://cache.example.com");
    assert!(result.is_none());
}

#[test]
fn parse_cache_endpoint_wrong_prefix() {
    let result = DnsDiscovery::parse_cache_endpoint("server=https://cache.example.com");
    assert!(result.is_none());
}

#[test]
fn parse_cache_endpoint_empty_string() {
    let result = DnsDiscovery::parse_cache_endpoint("");
    assert!(result.is_none());
}

// === Cache Memory: invalidate, clear, size, no_cache, expired ===

#[test]
fn cache_invalidate_existing() {
    let cache = MemoryCache::new(100);
    cache
        .put("example.com".into(), make_cache_entry("data", 3600))
        .unwrap();
    assert_eq!(cache.size(), 1);
    assert!(cache.invalidate("example.com"));
    assert_eq!(cache.size(), 0);
}

#[test]
fn cache_invalidate_nonexistent() {
    let cache = MemoryCache::new(100);
    assert!(!cache.invalidate("nonexistent.com"));
}

#[test]
fn cache_clear() {
    let cache = MemoryCache::new(100);
    for i in 0..10 {
        cache
            .put(format!("domain-{}.com", i), make_cache_entry("data", 3600))
            .unwrap();
    }
    assert_eq!(cache.size(), 10);
    cache.clear();
    assert_eq!(cache.size(), 0);
}

#[test]
fn cache_max_entries() {
    let cache = MemoryCache::new(42);
    assert_eq!(cache.max_entries(), 42);
}

#[test]
fn cache_no_cache_entry_not_stored() {
    let cache = MemoryCache::new(100);
    let entry = CacheEntry {
        data: "data".into(),
        created_at: Instant::now(),
        ttl: Duration::from_secs(3600),
        no_cache: true,
    };
    cache.put("example.com".into(), entry).unwrap();
    assert_eq!(cache.size(), 0);
    assert!(cache.get("example.com").is_none());
}

#[test]
fn cache_expired_entry_evicted_on_get() {
    let cache = MemoryCache::new(100);
    let entry = CacheEntry {
        data: "old".into(),
        created_at: Instant::now() - Duration::from_secs(7200),
        ttl: Duration::from_secs(3600),
        no_cache: false,
    };
    cache.put("example.com".into(), entry).unwrap();
    assert!(cache.get("example.com").is_none());
}

#[test]
fn cache_entry_is_expired() {
    let entry = CacheEntry {
        data: "data".into(),
        created_at: Instant::now() - Duration::from_secs(100),
        ttl: Duration::from_secs(50),
        no_cache: false,
    };
    assert!(entry.is_expired());
}

#[test]
fn cache_entry_is_not_expired() {
    let entry = CacheEntry {
        data: "data".into(),
        created_at: Instant::now(),
        ttl: Duration::from_secs(3600),
        no_cache: false,
    };
    assert!(!entry.is_expired());
}

#[test]
fn cache_get_returns_valid_entry() {
    let cache = MemoryCache::new(100);
    cache
        .put("example.com".into(), make_cache_entry("hello", 3600))
        .unwrap();
    let entry = cache.get("example.com").unwrap();
    assert_eq!(entry.data, "hello");
}

#[test]
fn cache_get_miss() {
    let cache = MemoryCache::new(100);
    assert!(cache.get("nonexistent.com").is_none());
}

#[test]
fn cache_lru_eviction() {
    let cache = MemoryCache::new(3);
    for i in 0..5 {
        cache
            .put(format!("d{}.com", i), make_cache_entry("data", 3600))
            .unwrap();
    }
    // Only the 3 most recent should remain
    assert_eq!(cache.size(), 3);
    assert!(cache.get("d0.com").is_none());
    assert!(cache.get("d1.com").is_none());
    assert!(cache.get("d2.com").is_some());
}

// === Cache Coordinator: construction ===

#[test]
fn coordinator_new_no_root() {
    let coord = open_tethyr::cache::coordinator::CacheCoordinator::new(100, 3600, None);
    assert!(coord.is_ok());
    let coord = coord.unwrap();
    assert_eq!(coord.local_cache().size(), 0);
    assert_eq!(coord.local_cache().max_entries(), 100);
}

#[test]
fn coordinator_new_with_root() {
    let coord = open_tethyr::cache::coordinator::CacheCoordinator::new(
        100,
        3600,
        Some("https://cache.example.com".into()),
    );
    assert!(coord.is_ok());
}

#[test]
fn coordinator_rejects_empty_root_url() {
    let coord = open_tethyr::cache::coordinator::CacheCoordinator::new(100, 3600, Some("".into()));
    assert!(coord.is_err());
}

// === Client: construction ===

#[test]
fn client_new_valid_domain() {
    let client = open_tethyr::OpenTethyr::new("example.com");
    assert!(client.is_ok());
}

#[test]
fn client_new_empty_domain() {
    let client = open_tethyr::OpenTethyr::new("");
    assert!(client.is_err());
}

// === AX Generator: protocol mapping, OAuth, multi-agent ===

#[test]
fn generator_single_record() {
    let config = AgentConfig {
        defaults: AgentDefaults::default(),
        agents: vec![AgentDefinition {
            name: "test-agent".into(),
            description: "Test".into(),
            provider: Some("corp".into()),
            endpoints: vec![models::EndpointDefinition {
                protocol: "rest".into(),
                url: "https://api.example.com".into(),
                auth: vec!["OAuth2".into()],
                content_type: None,
            }],
            auth: vec![],
            capabilities: None,
            limits: None,
            security: None,
            extensions: None,
            oauth_provider: None,
            oauth_domain: None,
        }],
        server: None,
    };

    let record = open_tethyr::ax::AxGenerator::generate_record(&config).unwrap();
    assert_eq!(record.agent.name, "test-agent");
    assert_eq!(record.agent.provider, Some("corp".into()));
    assert_eq!(record.record_type, "AX");
    assert_eq!(record.version, "1.0");
    assert_eq!(record.endpoints.len(), 1);
    assert_eq!(record.endpoints[0].protocol, Protocol::Rest);
    assert_eq!(record.endpoints[0].auth, vec!["OAuth2"]);
}

#[test]
fn generator_all_protocols() {
    let protocols = vec![
        ("rest", Protocol::Rest),
        ("graphql", Protocol::GraphQL),
        ("mcp", Protocol::MCP),
        ("a2a", Protocol::A2A),
        ("custom-proto", Protocol::Custom("custom-proto".into())),
    ];

    for (proto_str, expected) in protocols {
        let config = make_config_with_protocol(proto_str);
        let record = open_tethyr::ax::AxGenerator::generate_record(&config).unwrap();
        assert_eq!(record.endpoints[0].protocol, expected);
    }
}

#[test]
fn generator_multi_agent() {
    let config = AgentConfig {
        defaults: AgentDefaults::default(),
        agents: vec![
            AgentDefinition {
                name: "agent-1".into(),
                description: "First".into(),
                provider: None,
                endpoints: vec![models::EndpointDefinition {
                    protocol: "rest".into(),
                    url: "https://a1.example.com".into(),
                    auth: vec![],
                    content_type: None,
                }],
                auth: vec![],
                capabilities: None,
                limits: None,
                security: None,
                extensions: None,
                oauth_provider: None,
                oauth_domain: None,
            },
            AgentDefinition {
                name: "agent-2".into(),
                description: "Second".into(),
                provider: None,
                endpoints: vec![models::EndpointDefinition {
                    protocol: "mcp".into(),
                    url: "https://a2.example.com".into(),
                    auth: vec![],
                    content_type: None,
                }],
                auth: vec![],
                capabilities: None,
                limits: None,
                security: None,
                extensions: None,
                oauth_provider: None,
                oauth_domain: None,
            },
        ],
        server: None,
    };

    let records = open_tethyr::ax::AxGenerator::generate_all_records(&config).unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].agent.name, "agent-1");
    assert_eq!(records[1].agent.name, "agent-2");
}

#[test]
fn generator_empty_config_fails() {
    let config = AgentConfig::default();
    assert!(open_tethyr::ax::AxGenerator::generate_record(&config).is_err());
}

#[test]
fn generator_with_oauth() {
    let config = AgentConfig {
        defaults: AgentDefaults::default(),
        agents: vec![AgentDefinition {
            name: "oauth-agent".into(),
            description: "OAuth test".into(),
            provider: None,
            endpoints: vec![models::EndpointDefinition {
                protocol: "rest".into(),
                url: "https://api.example.com".into(),
                auth: vec!["OAuth2".into()],
                content_type: None,
            }],
            auth: vec![],
            capabilities: None,
            limits: None,
            security: None,
            extensions: None,
            oauth_provider: Some("okta".into()),
            oauth_domain: Some("dev-12345.okta.com".into()),
        }],
        server: None,
    };

    let record = open_tethyr::ax::AxGenerator::generate_record(&config).unwrap();
    assert!(record.security.is_some());
    let sec = record.security.unwrap();
    assert!(sec.issuer.is_some());
    assert!(sec.jwks_url.is_some());
}

#[test]
fn generator_with_unknown_oauth_provider_fails() {
    let config = AgentConfig {
        defaults: AgentDefaults::default(),
        agents: vec![AgentDefinition {
            name: "bad-oauth".into(),
            description: "Bad OAuth".into(),
            provider: None,
            endpoints: vec![models::EndpointDefinition {
                protocol: "rest".into(),
                url: "https://api.example.com".into(),
                auth: vec![],
                content_type: None,
            }],
            auth: vec![],
            capabilities: None,
            limits: None,
            security: None,
            extensions: None,
            oauth_provider: Some("nonexistent-provider".into()),
            oauth_domain: Some("example.com".into()),
        }],
        server: None,
    };

    assert!(open_tethyr::ax::AxGenerator::generate_record(&config).is_err());
}

#[test]
fn generator_endpoint_auth_fallback_to_agent_auth() {
    let config = AgentConfig {
        defaults: AgentDefaults::default(),
        agents: vec![AgentDefinition {
            name: "fallback-auth".into(),
            description: "Auth fallback test".into(),
            provider: None,
            endpoints: vec![models::EndpointDefinition {
                protocol: "rest".into(),
                url: "https://api.example.com".into(),
                auth: vec![], // empty - should fall back to agent-level auth
                content_type: None,
            }],
            auth: vec!["JWT".into()], // agent-level auth
            capabilities: None,
            limits: None,
            security: None,
            extensions: None,
            oauth_provider: None,
            oauth_domain: None,
        }],
        server: None,
    };

    let record = open_tethyr::ax::AxGenerator::generate_record(&config).unwrap();
    assert_eq!(record.endpoints[0].auth, vec!["JWT"]);
}

// === Error module ===

#[test]
fn known_auth_methods_contains_expected() {
    assert!(is_known_auth_method("OAuth2"));
    assert!(is_known_auth_method("JWT"));
    assert!(is_known_auth_method("mTLS"));
    assert!(is_known_auth_method("OIDC"));
    assert!(is_known_auth_method("AWS_IAM"));
    assert!(is_known_auth_method("API_KEY"));
    assert!(!is_known_auth_method("CustomAuth"));
    assert!(!is_known_auth_method(""));
}

// === Auth Provider Registry ===

#[test]
fn provider_registry_known_providers() {
    let registry = open_tethyr::auth::ProviderRegistry::new();

    let okta = registry.generate_oauth_config("okta", "dev-123.okta.com");
    assert!(okta.is_ok());

    let auth0 = registry.generate_oauth_config("auth0", "myapp.auth0.com");
    assert!(auth0.is_ok());

    let generic = registry.generate_oauth_config("generic", "auth.example.com");
    assert!(generic.is_ok());
}

#[test]
fn provider_registry_unknown_provider() {
    let registry = open_tethyr::auth::ProviderRegistry::new();
    assert!(registry
        .generate_oauth_config("nonexistent", "example.com")
        .is_err());
}

#[test]
fn provider_registry_default() {
    let registry = open_tethyr::auth::ProviderRegistry::default();
    assert!(registry
        .generate_oauth_config("okta", "dev-123.okta.com")
        .is_ok());
}

// === DNS Discovery: construction ===

#[test]
fn dns_discovery_new() {
    let dns = DnsDiscovery::new();
    assert!(dns.is_ok());
}

// === Server: construction and routes ===

#[cfg(feature = "server")]
mod server_coverage {
    use open_tethyr::cache::coordinator::CacheCoordinator;
    use open_tethyr::cache::stats::CacheStats;
    use open_tethyr::config::*;
    use open_tethyr::server::{AppState, CacheServer, PolicyEngine};
    use std::sync::Arc;

    #[tokio::test]
    async fn cache_server_new_default_config() {
        let config = ServerConfig {
            domain: "localhost".into(),
            port: 0,
            cache: CacheConfig::default(),
            policy: PolicyConfig::default(),
            rate_limit: RateLimitConfig::default(),
            log_level: "info".into(),
        };
        let server = CacheServer::new(config).await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn cache_server_new_with_policy() {
        let config = ServerConfig {
            domain: "test.com".into(),
            port: 0,
            cache: CacheConfig {
                max_entries: 500,
                default_ttl: 1800,
                cleanup_interval: 60,
                enable_lru: true,
            },
            policy: PolicyConfig {
                domain_locking: true,
                home_domain: Some("test.com".into()),
                allowlist: vec!["trusted.com".into()],
            },
            rate_limit: RateLimitConfig {
                requests_per_minute: 30,
                requests_per_hour: 500,
            },
            log_level: "debug".into(),
        };
        let server = CacheServer::new(config).await;
        assert!(server.is_ok());
    }

    #[test]
    fn build_routes_returns_router() {
        let state = Arc::new(AppState {
            coordinator: CacheCoordinator::new(100, 3600, None).unwrap(),
            policy: PolicyEngine::default(),
            stats: CacheStats::new(),
        });
        // Just verify it doesn't panic
        let _router = CacheServer::build_routes(state);
    }
}

// === Generator: well-known structure ===

#[test]
fn generator_well_known_structure() {
    let record = make_valid_record();
    let tmp = tempfile::tempdir().unwrap();
    let result = open_tethyr::ax::AxGenerator::generate_well_known_structure(&record, tmp.path());
    assert!(result.is_ok());
    let files = result.unwrap();
    assert!(files.ax_record_path.exists());
}

#[test]
fn generator_with_content_type() {
    let config = AgentConfig {
        defaults: AgentDefaults::default(),
        agents: vec![AgentDefinition {
            name: "ct-agent".into(),
            description: "Content-type test".into(),
            provider: None,
            endpoints: vec![models::EndpointDefinition {
                protocol: "rest".into(),
                url: "https://api.example.com".into(),
                auth: vec![],
                content_type: Some("application/json".into()),
            }],
            auth: vec![],
            capabilities: None,
            limits: None,
            security: None,
            extensions: None,
            oauth_provider: None,
            oauth_domain: None,
        }],
        server: None,
    };
    let record = open_tethyr::ax::AxGenerator::generate_record(&config).unwrap();
    assert_eq!(
        record.endpoints[0].content_type,
        Some("application/json".into())
    );
}

#[test]
fn generator_with_auth0() {
    let config = AgentConfig {
        defaults: AgentDefaults::default(),
        agents: vec![AgentDefinition {
            name: "auth0-agent".into(),
            description: "Auth0 test".into(),
            provider: None,
            endpoints: vec![models::EndpointDefinition {
                protocol: "rest".into(),
                url: "https://api.example.com".into(),
                auth: vec![],
                content_type: None,
            }],
            auth: vec![],
            capabilities: None,
            limits: None,
            security: None,
            extensions: None,
            oauth_provider: Some("auth0".into()),
            oauth_domain: Some("myapp.auth0.com".into()),
        }],
        server: None,
    };
    let record = open_tethyr::ax::AxGenerator::generate_record(&config).unwrap();
    assert!(record.security.is_some());
}

#[test]
fn generator_with_generic_oauth() {
    let config = AgentConfig {
        defaults: AgentDefaults::default(),
        agents: vec![AgentDefinition {
            name: "generic-agent".into(),
            description: "Generic OAuth test".into(),
            provider: None,
            endpoints: vec![models::EndpointDefinition {
                protocol: "rest".into(),
                url: "https://api.example.com".into(),
                auth: vec![],
                content_type: None,
            }],
            auth: vec![],
            capabilities: None,
            limits: None,
            security: None,
            extensions: None,
            oauth_provider: Some("generic".into()),
            oauth_domain: Some("auth.example.com".into()),
        }],
        server: None,
    };
    let record = open_tethyr::ax::AxGenerator::generate_record(&config).unwrap();
    assert!(record.security.is_some());
}

#[test]
fn generator_with_extensions() {
    let config = AgentConfig {
        defaults: AgentDefaults::default(),
        agents: vec![AgentDefinition {
            name: "ext-agent".into(),
            description: "Extensions test".into(),
            provider: None,
            endpoints: vec![models::EndpointDefinition {
                protocol: "rest".into(),
                url: "https://api.example.com".into(),
                auth: vec![],
                content_type: None,
            }],
            auth: vec![],
            capabilities: None,
            limits: None,
            security: None,
            extensions: Some(serde_json::json!({"vendor": {"tier": "premium"}})),
            oauth_provider: None,
            oauth_domain: None,
        }],
        server: None,
    };
    let record = open_tethyr::ax::AxGenerator::generate_record(&config).unwrap();
    assert!(record.extensions.is_some());
}

// === Helpers ===

fn make_valid_record() -> AgentExchangeRecord {
    AgentExchangeRecord {
        record_type: "AX".into(),
        version: "1.0".into(),
        agent: Agent {
            name: "test-agent".into(),
            description: "A test agent".into(),
            provider: Some("corp".into()),
        },
        endpoints: vec![Endpoint {
            protocol: Protocol::Rest,
            url: "https://api.example.com".into(),
            auth: vec!["OAuth2".into()],
            content_type: None,
            extra: serde_json::Map::new(),
        }],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    }
}

fn make_cache_entry(data: &str, ttl_secs: u64) -> CacheEntry {
    CacheEntry {
        data: data.into(),
        created_at: Instant::now(),
        ttl: Duration::from_secs(ttl_secs),
        no_cache: false,
    }
}

fn make_config_with_protocol(proto: &str) -> AgentConfig {
    AgentConfig {
        defaults: AgentDefaults::default(),
        agents: vec![AgentDefinition {
            name: "proto-test".into(),
            description: "Protocol test".into(),
            provider: None,
            endpoints: vec![models::EndpointDefinition {
                protocol: proto.into(),
                url: "https://api.example.com".into(),
                auth: vec![],
                content_type: None,
            }],
            auth: vec![],
            capabilities: None,
            limits: None,
            security: None,
            extensions: None,
            oauth_provider: None,
            oauth_domain: None,
        }],
        server: None,
    }
}
