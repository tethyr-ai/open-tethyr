//! AX Record Generation
//!
//! Generation logic for creating AX records from configuration.

use super::{Agent, AgentExchangeDocument, AgentExchangeRecord, Endpoint, Protocol};
use crate::config::{AgentConfig, ConfigMerger};
use std::path::Path;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Generation errors
#[derive(Debug, Error)]
pub enum GenerationError {
    #[error("Invalid protocol: {0}")]
    InvalidProtocol(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Well-known file structure for output
#[derive(Debug, Clone)]
pub struct WellKnownFiles {
    pub agent_exchange_json: String,
    pub path: String,
}

/// AX record generator
pub struct AxGenerator;

impl AxGenerator {
    /// Generate AX record document from configuration
    pub fn generate_record(config: &AgentConfig) -> Result<AgentExchangeDocument, GenerationError> {
        debug!(
            "Starting AX record generation for {} agents",
            config.agents.len()
        );
        let mut records = Vec::new();

        for agent_def in &config.agents {
            debug!(agent_name = %agent_def.name, "Generating record for agent");

            // Merge with defaults
            let merged_agent = ConfigMerger::merge_agent(agent_def, &config.defaults);

            // Create agent
            let agent = Agent {
                name: merged_agent.name.clone(),
                description: merged_agent.description.clone(),
                provider: merged_agent.provider.ok_or_else(|| {
                    warn!(agent_name = %agent_def.name, "Missing provider field");
                    GenerationError::MissingField("provider".to_string())
                })?,
            };

            // Create endpoint
            let protocol =
                Self::parse_protocol(&merged_agent.protocol.unwrap_or_else(|| "rest".to_string()))?;
            let auth = merged_agent.auth.ok_or_else(|| {
                warn!(agent_name = %agent_def.name, "Missing auth field");
                GenerationError::MissingField("auth".to_string())
            })?;

            let endpoint = Endpoint {
                protocol,
                url: merged_agent.url.clone(),
                auth,
                content_type: merged_agent.content_type,
            };

            // Create AX record
            let record = AgentExchangeRecord {
                record_type: "AX".to_string(),
                version: "1.0".to_string(),
                agent,
                endpoints: vec![endpoint],
                capabilities: merged_agent.capabilities.map(|_caps| {
                    // Convert HashMap to Capabilities struct
                    // For now, use a simple placeholder
                    super::Capabilities {}
                }),
                schema: None, // Will be expanded in future versions
                limits: merged_agent.limits.map(|_| super::Limits {}),
                security: merged_agent.security.map(|_| super::Security {}),
                extensions: merged_agent.extensions,
            };

            records.push(record);
        }

        info!(
            record_count = records.len(),
            "Successfully generated AX records"
        );
        Ok(AgentExchangeDocument { records })
    }

    /// Generate well-known file structure
    pub fn generate_well_known_structure(
        document: &AgentExchangeDocument,
    ) -> Result<WellKnownFiles, GenerationError> {
        debug!("Generating well-known file structure");
        let json_content = serde_json::to_string_pretty(document)?;

        info!(
            "Generated well-known structure with {} bytes",
            json_content.len()
        );
        Ok(WellKnownFiles {
            agent_exchange_json: json_content,
            path: "/.well-known/agent-exchange.json".to_string(),
        })
    }

    /// Write well-known structure to filesystem
    pub fn write_well_known_structure(
        output_dir: &Path,
        files: &WellKnownFiles,
    ) -> Result<(), GenerationError> {
        debug!(output_dir = ?output_dir, "Writing well-known structure to filesystem");
        let well_known_dir = output_dir.join(".well-known");
        std::fs::create_dir_all(&well_known_dir)?;

        let file_path = well_known_dir.join("agent-exchange.json");
        std::fs::write(&file_path, &files.agent_exchange_json)?;

        info!(file_path = ?file_path, "Successfully wrote well-known structure");
        Ok(())
    }

    /// Parse protocol string to Protocol enum
    fn parse_protocol(protocol_str: &str) -> Result<Protocol, GenerationError> {
        match protocol_str.to_lowercase().as_str() {
            "rest" => Ok(Protocol::Rest),
            "graphql" => Ok(Protocol::GraphQL),
            "mcp" => Ok(Protocol::MCP),
            "a2a" => Ok(Protocol::A2A),
            custom => Ok(Protocol::Custom(custom.to_string())),
        }
    }
}
