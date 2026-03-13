//! AX Record Generation

use super::{Agent, AgentExchangeDocument, AgentExchangeRecord, Endpoint, Protocol, Security};
use crate::auth::ProviderRegistry;
use crate::config::{AgentConfig, AgentDefinition, ConfigMerger};
use crate::error::AxError;
use crate::http::file_writer::{FileWriter, WellKnownFiles};
use std::path::Path;

/// AX record generator
pub struct AxGenerator;

impl AxGenerator {
    /// Generate AX document from configuration
    pub fn generate_record(config: &AgentConfig) -> Result<AgentExchangeDocument, AxError> {
        let merged_agents = ConfigMerger::merge_all(config);
        let registry = ProviderRegistry::new();
        let mut records = Vec::new();

        for agent_def in &merged_agents {
            let record = Self::agent_to_record(agent_def, &registry)?;
            records.push(record);
        }

        Ok(AgentExchangeDocument { records })
    }

    /// Generate well-known file structure from document
    pub fn generate_well_known_structure(
        doc: &AgentExchangeDocument,
        output_dir: &Path,
    ) -> Result<WellKnownFiles, AxError> {
        FileWriter::write_structure(doc, output_dir)
    }

    fn agent_to_record(
        agent_def: &AgentDefinition,
        registry: &ProviderRegistry,
    ) -> Result<AgentExchangeRecord, AxError> {
        let endpoints: Vec<Endpoint> = agent_def
            .endpoints
            .iter()
            .map(|ep| {
                let protocol = match ep.protocol.to_lowercase().as_str() {
                    "rest" => Protocol::Rest,
                    "graphql" => Protocol::GraphQL,
                    "mcp" => Protocol::MCP,
                    "a2a" => Protocol::A2A,
                    other => Protocol::Custom(other.to_string()),
                };
                let auth = if ep.auth.is_empty() {
                    agent_def.auth.clone()
                } else {
                    ep.auth.clone()
                };
                Endpoint {
                    protocol,
                    url: ep.url.clone(),
                    auth,
                    content_type: ep.content_type.clone(),
                }
            })
            .collect();

        let security = if let (Some(provider), Some(domain)) =
            (&agent_def.oauth_provider, &agent_def.oauth_domain)
        {
            match registry.generate_oauth_config(provider, domain) {
                Ok(oauth) => Some(Security {
                    oauth: Some(oauth),
                    extra: Default::default(),
                }),
                Err(e) => return Err(AxError::GenerationFailed(e.to_string())),
            }
        } else {
            None
        };

        Ok(AgentExchangeRecord {
            record_type: "AX".to_string(),
            version: "1.0".to_string(),
            agent: Agent {
                name: agent_def.name.clone(),
                description: agent_def.description.clone(),
                provider: agent_def.provider.clone().unwrap_or_default(),
            },
            endpoints,
            capabilities: None,
            schema: None,
            limits: None,
            security,
            extensions: agent_def.extensions.clone(),
        })
    }
}
