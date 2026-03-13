//! Configuration Merging

use super::models::{AgentConfig, AgentDefaults, AgentDefinition};

/// Configuration merger for type-safe inheritance
pub struct ConfigMerger;

impl ConfigMerger {
    /// Merge agent definitions with global defaults
    pub fn merge_agent(defaults: &AgentDefaults, agent: &AgentDefinition) -> AgentDefinition {
        AgentDefinition {
            name: agent.name.clone(),
            description: agent.description.clone(),
            provider: agent.provider.clone().or_else(|| defaults.provider.clone()),
            endpoints: if agent.endpoints.is_empty() {
                defaults.endpoints.clone()
            } else {
                agent.endpoints.clone()
            },
            auth: if agent.auth.is_empty() {
                defaults.auth.clone()
            } else {
                agent.auth.clone()
            },
            capabilities: agent.capabilities.clone().or_else(|| defaults.capabilities.clone()),
            limits: agent.limits.clone().or_else(|| defaults.limits.clone()),
            security: agent.security.clone(),
            extensions: agent.extensions.clone().or_else(|| defaults.extensions.clone()),
            oauth_provider: agent.oauth_provider.clone(),
            oauth_domain: agent.oauth_domain.clone(),
        }
    }

    /// Merge all agents in config with defaults
    pub fn merge_all(config: &AgentConfig) -> Vec<AgentDefinition> {
        config.agents.iter()
            .map(|agent| Self::merge_agent(&config.defaults, agent))
            .collect()
    }
}
