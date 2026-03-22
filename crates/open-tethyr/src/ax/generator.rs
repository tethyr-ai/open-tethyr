//! AX Record Generation - produces flat AX documents per spec

use super::{Agent, AgentExchangeRecord, Endpoint, Protocol, Security};
use crate::auth::ProviderRegistry;
use crate::config::{AgentConfig, AgentDefinition, ConfigMerger};
use crate::error::AxError;
use crate::http::file_writer::{FileWriter, WellKnownFiles};
use std::path::Path;

pub struct AxGenerator;

impl AxGenerator {
    /// Generate a single AX record from the first agent in config (flat per spec)
    pub fn generate_record(config: &AgentConfig) -> Result<AgentExchangeRecord, AxError> {
        let merged = ConfigMerger::merge_all(config);
        let registry = ProviderRegistry::new();
        let agent_def = merged
            .first()
            .ok_or_else(|| AxError::GenerationFailed("No agents in config".into()))?;
        Self::agent_to_record(agent_def, &registry)
    }

    /// Generate all records (for multi-agent configs)
    pub fn generate_all_records(config: &AgentConfig) -> Result<Vec<AgentExchangeRecord>, AxError> {
        let merged = ConfigMerger::merge_all(config);
        let registry = ProviderRegistry::new();
        merged
            .iter()
            .map(|a| Self::agent_to_record(a, &registry))
            .collect()
    }

    pub fn generate_well_known_structure(
        record: &AgentExchangeRecord,
        output_dir: &Path,
    ) -> Result<WellKnownFiles, AxError> {
        FileWriter::write_structure(record, output_dir)
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
                    extra: Default::default(),
                }
            })
            .collect();

        // OAuth providers now populate flat security.issuer + security.jwks_url
        let security = if let (Some(provider), Some(domain)) =
            (&agent_def.oauth_provider, &agent_def.oauth_domain)
        {
            match registry.generate_oauth_config(provider, domain) {
                Ok((issuer, jwks_url)) => Some(Security {
                    issuer: Some(issuer),
                    jwks_url: Some(jwks_url),
                    signature: None,
                    metadata_signature: None,
                    extra: Default::default(),
                }),
                Err(e) => return Err(AxError::GenerationFailed(e.to_string())),
            }
        } else {
            None
        };

        Ok(AgentExchangeRecord {
            record_type: "AX".into(),
            version: "1.0".into(),
            agent: Agent {
                name: agent_def.name.clone(),
                description: agent_def.description.clone(),
                provider: agent_def.provider.clone(),
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
