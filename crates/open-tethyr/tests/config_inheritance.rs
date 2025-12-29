//! Property tests for configuration inheritance
//!
//! **Property 10: Configuration Inheritance Correctness**
//! **Validates: Requirements 7.2, 7.3, 7.4, 7.5**

use open_tethyr::config::{AgentDefaults, AgentDefinition, ConfigMerger};
use proptest::prelude::*;

// Property test generators
fn arb_agent_defaults() -> impl Strategy<Value = AgentDefaults> {
    (
        prop::option::of("[a-zA-Z0-9-]{1,20}"),
        prop::option::of(prop::collection::vec("[A-Z]{3,10}", 0..3)),
        prop::option::of("(rest|graphql|mcp|a2a|custom)"),
        prop::option::of("application/(json|xml|yaml)"),
        prop::option::of("[a-zA-Z0-9.-]{3,50}"),
        prop::option::of(1u16..65535u16),
        prop::option::of(1u32..86400u32),
    )
        .prop_map(
            |(provider, auth, protocol, content_type, domain, port, ttl)| AgentDefaults {
                provider,
                auth,
                protocol,
                content_type,
                domain,
                port,
                ttl,
                capabilities: None,
                limits: None,
                security: None,
                extensions: None,
            },
        )
}

fn arb_agent_definition() -> impl Strategy<Value = AgentDefinition> {
    (
        "[a-zA-Z0-9 ]{1,50}",
        "[a-zA-Z0-9 ]{1,100}",
        "https://[a-zA-Z0-9.-]{3,50}/[a-zA-Z0-9/-]*",
        prop::option::of("[a-zA-Z0-9-]{1,20}"),
        prop::option::of(prop::collection::vec("[A-Z]{3,10}", 0..3)),
        prop::option::of("(rest|graphql|mcp|a2a|custom)"),
        prop::option::of("application/(json|xml|yaml)"),
        prop::option::of("[a-zA-Z0-9.-]{3,50}"),
        prop::option::of(1u16..65535u16),
        prop::option::of(1u32..86400u32),
    )
        .prop_map(
            |(
                name,
                description,
                url,
                provider,
                auth,
                protocol,
                content_type,
                domain,
                port,
                ttl,
            )| {
                AgentDefinition {
                    name,
                    description,
                    url,
                    provider,
                    auth,
                    protocol,
                    content_type,
                    domain,
                    port,
                    ttl,
                    capabilities: None,
                    limits: None,
                    security: None,
                    extensions: None,
                }
            },
        )
}

proptest! {
    /// Property 10: Configuration Inheritance Correctness
    /// For any agent definition and defaults, merging should preserve agent-specific values
    /// and inherit defaults only when agent values are None
    /// **Validates: Requirements 7.2, 7.3, 7.4, 7.5**
    #[test]
    fn test_configuration_inheritance_correctness(
        agent in arb_agent_definition(),
        defaults in prop::option::of(arb_agent_defaults())
    ) {
        let merged = ConfigMerger::merge_agent(&agent, &defaults);

        // Agent-specific values should always be preserved
        prop_assert_eq!(merged.name, agent.name);
        prop_assert_eq!(merged.description, agent.description);
        prop_assert_eq!(merged.url, agent.url);

        // Test inheritance behavior for optional fields
        // If agent has a value, it should be preserved
        if agent.provider.is_some() {
            prop_assert_eq!(&merged.provider, &agent.provider);
        }
        if agent.auth.is_some() {
            prop_assert_eq!(&merged.auth, &agent.auth);
        }
        if agent.protocol.is_some() {
            prop_assert_eq!(&merged.protocol, &agent.protocol);
        }
        if agent.content_type.is_some() {
            prop_assert_eq!(&merged.content_type, &agent.content_type);
        }
        if agent.domain.is_some() {
            prop_assert_eq!(&merged.domain, &agent.domain);
        }
        if agent.port.is_some() {
            prop_assert_eq!(merged.port, agent.port);
        }
        if agent.ttl.is_some() {
            prop_assert_eq!(merged.ttl, agent.ttl);
        }

        // If agent has no value but defaults exist, defaults should be inherited
        if agent.provider.is_none() && defaults.is_some() {
            let defaults_ref = defaults.as_ref().unwrap();
            prop_assert_eq!(&merged.provider, &defaults_ref.provider);
        }
        if agent.auth.is_none() && defaults.is_some() {
            let defaults_ref = defaults.as_ref().unwrap();
            prop_assert_eq!(&merged.auth, &defaults_ref.auth);
        }
        if agent.protocol.is_none() && defaults.is_some() {
            let defaults_ref = defaults.as_ref().unwrap();
            prop_assert_eq!(&merged.protocol, &defaults_ref.protocol);
        }
        if agent.content_type.is_none() && defaults.is_some() {
            let defaults_ref = defaults.as_ref().unwrap();
            prop_assert_eq!(&merged.content_type, &defaults_ref.content_type);
        }
        if agent.domain.is_none() && defaults.is_some() {
            let defaults_ref = defaults.as_ref().unwrap();
            prop_assert_eq!(&merged.domain, &defaults_ref.domain);
        }
        if agent.port.is_none() && defaults.is_some() {
            let defaults_ref = defaults.as_ref().unwrap();
            prop_assert_eq!(merged.port, defaults_ref.port);
        }
        if agent.ttl.is_none() && defaults.is_some() {
            let defaults_ref = defaults.as_ref().unwrap();
            prop_assert_eq!(merged.ttl, defaults_ref.ttl);
        }
    }

    /// Property: Auth inheritance should preserve agent-specific auth completely
    /// **Validates: Requirements 7.3**
    #[test]
    fn test_auth_inheritance_override(
        agent_auth in prop::option::of(prop::collection::vec("[A-Z]{3,10}", 1..3)),
        default_auth in prop::option::of(prop::collection::vec("[A-Z]{3,10}", 1..3))
    ) {
        let merged_auth = ConfigMerger::merge_auth(&agent_auth, &default_auth);

        if agent_auth.is_some() {
            // Agent auth should completely override defaults
            prop_assert_eq!(merged_auth, agent_auth);
        } else {
            // Should inherit from defaults
            prop_assert_eq!(merged_auth, default_auth);
        }
    }

    /// Property: Nested object merging should preserve agent-specific overrides
    /// **Validates: Requirements 7.4, 7.5**
    #[test]
    fn test_nested_inheritance_correctness(
        agent_caps in prop::option::of(prop::collection::hash_map("[a-z]{3,10}", any::<bool>().prop_map(serde_json::Value::Bool), 0..3)),
        default_caps in prop::option::of(prop::collection::hash_map("[a-z]{3,10}", any::<bool>().prop_map(serde_json::Value::Bool), 0..3))
    ) {
        let agent = AgentDefinition {
            name: "test".to_string(),
            description: "test".to_string(),
            url: "https://example.com".to_string(),
            provider: None,
            auth: None,
            protocol: None,
            content_type: None,
            domain: None,
            port: None,
            ttl: None,
            capabilities: agent_caps.clone(),
            limits: None,
            security: None,
            extensions: None,
        };

        let defaults = AgentDefaults {
            provider: None,
            auth: None,
            protocol: None,
            content_type: None,
            domain: None,
            port: None,
            ttl: None,
            capabilities: default_caps.clone(),
            limits: None,
            security: None,
            extensions: None,
        };

        let merged = ConfigMerger::merge_agent(&agent, &Some(defaults));

        match (&agent_caps, &default_caps) {
            (Some(agent_map), Some(default_map)) => {
                let merged_caps = merged.capabilities.unwrap();
                // All agent keys should be present
                for (key, value) in agent_map {
                    prop_assert_eq!(merged_caps.get(key), Some(value));
                }
                // Default keys should be present unless overridden
                for (key, value) in default_map {
                    if !agent_map.contains_key(key) {
                        prop_assert_eq!(merged_caps.get(key), Some(value));
                    }
                }
            }
            (Some(agent_map), None) => {
                prop_assert_eq!(merged.capabilities.as_ref(), Some(agent_map));
            }
            (None, Some(default_map)) => {
                prop_assert_eq!(merged.capabilities.as_ref(), Some(default_map));
            }
            (None, None) => {
                prop_assert_eq!(merged.capabilities, None);
            }
        }
    }
}
