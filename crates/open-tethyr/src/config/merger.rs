//! Configuration Merging
//!
//! Configuration inheritance and merging logic.

use super::{AgentDefaults, AgentDefinition};
use std::collections::HashMap;

/// Configuration merger for type-safe inheritance
pub struct ConfigMerger;

impl ConfigMerger {
    /// Merge agent definition with defaults
    pub fn merge_agent(agent: &AgentDefinition, defaults: &Option<AgentDefaults>) -> AgentDefinition {
        let Some(defaults) = defaults else {
            return agent.clone();
        };

        AgentDefinition {
            name: agent.name.clone(),
            description: agent.description.clone(),
            url: agent.url.clone(),
            provider: agent.provider.clone().or_else(|| defaults.provider.clone()),
            auth: agent.auth.clone().or_else(|| defaults.auth.clone()),
            protocol: agent.protocol.clone().or_else(|| defaults.protocol.clone()),
            content_type: agent.content_type.clone().or_else(|| defaults.content_type.clone()),
            capabilities: Self::merge_optional_maps(&agent.capabilities, &defaults.capabilities),
            limits: Self::merge_optional_maps(&agent.limits, &defaults.limits),
            security: Self::merge_optional_maps(&agent.security, &defaults.security),
            extensions: agent.extensions.clone().or_else(|| defaults.extensions.clone()),
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
                merged.extend(agent.clone());
                Some(merged)
            }
            (Some(agent), None) => Some(agent.clone()),
            (None, Some(defaults)) => Some(defaults.clone()),
            (None, None) => None,
        }
    }
}
