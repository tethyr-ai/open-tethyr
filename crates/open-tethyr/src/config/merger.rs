//! Configuration Merging
//!
//! Configuration inheritance and merging logic.

use super::{AgentDefaults, AgentDefinition};
use std::collections::HashMap;

/// Configuration merger for type-safe inheritance
pub struct ConfigMerger;

impl ConfigMerger {
    /// Merge agent definition with defaults
    pub fn merge_agent(
        agent: &AgentDefinition,
        defaults: &Option<AgentDefaults>,
    ) -> AgentDefinition {
        let Some(defaults) = defaults else {
            return agent.clone();
        };

        AgentDefinition {
            name: agent.name.clone(),
            description: agent.description.clone(),
            url: agent.url.clone(),
            provider: agent.provider.clone().or_else(|| defaults.provider.clone()),
            auth: Self::merge_auth(&agent.auth, &defaults.auth),
            protocol: agent.protocol.clone().or_else(|| defaults.protocol.clone()),
            content_type: agent
                .content_type
                .clone()
                .or_else(|| defaults.content_type.clone()),
            domain: agent.domain.clone().or_else(|| defaults.domain.clone()),
            port: agent.port.or(defaults.port),
            ttl: agent.ttl.or(defaults.ttl),
            capabilities: Self::merge_optional_maps(&agent.capabilities, &defaults.capabilities),
            limits: Self::merge_optional_maps(&agent.limits, &defaults.limits),
            security: Self::merge_optional_maps(&agent.security, &defaults.security),
            extensions: Self::merge_extensions(&agent.extensions, &defaults.extensions),
        }
    }

    /// Merge auth configurations with agent-specific overrides taking precedence
    pub fn merge_auth(
        agent_auth: &Option<Vec<String>>,
        default_auth: &Option<Vec<String>>,
    ) -> Option<Vec<String>> {
        match (agent_auth, default_auth) {
            (Some(agent), _) => Some(agent.clone()), // Agent-specific auth overrides defaults
            (None, Some(defaults)) => Some(defaults.clone()),
            (None, None) => None,
        }
    }

    /// Merge optional hash maps, with agent-specific values taking precedence
    fn merge_optional_maps(
        agent_map: &Option<HashMap<String, serde_json::Value>>,
        default_map: &Option<HashMap<String, serde_json::Value>>,
    ) -> Option<HashMap<String, serde_json::Value>> {
        match (agent_map, default_map) {
            (Some(agent), Some(defaults)) => {
                let mut merged = defaults.clone();
                // Agent-specific values override defaults
                for (key, value) in agent {
                    merged.insert(key.clone(), value.clone());
                }
                Some(merged)
            }
            (Some(agent), None) => Some(agent.clone()),
            (None, Some(defaults)) => Some(defaults.clone()),
            (None, None) => None,
        }
    }

    /// Merge extensions with support for nested object merging
    fn merge_extensions(
        agent_extensions: &Option<serde_json::Value>,
        default_extensions: &Option<serde_json::Value>,
    ) -> Option<serde_json::Value> {
        match (agent_extensions, default_extensions) {
            (Some(agent), Some(defaults)) => {
                // If both are objects, merge them recursively
                if let (
                    serde_json::Value::Object(agent_obj),
                    serde_json::Value::Object(default_obj),
                ) = (agent, defaults)
                {
                    let mut merged = default_obj.clone();
                    for (key, value) in agent_obj {
                        merged.insert(key.clone(), value.clone());
                    }
                    Some(serde_json::Value::Object(merged))
                } else {
                    // If not both objects, agent value takes precedence
                    Some(agent.clone())
                }
            }
            (Some(agent), None) => Some(agent.clone()),
            (None, Some(defaults)) => Some(defaults.clone()),
            (None, None) => None,
        }
    }

    /// Apply defaults to all agents in a configuration
    pub fn apply_defaults_to_agents(
        agents: &[AgentDefinition],
        defaults: &Option<AgentDefaults>,
    ) -> Vec<AgentDefinition> {
        agents
            .iter()
            .map(|agent| Self::merge_agent(agent, defaults))
            .collect()
    }
}
