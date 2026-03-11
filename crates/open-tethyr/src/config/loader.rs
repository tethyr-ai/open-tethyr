//! Configuration Loading
//!
//! YAML configuration loading functionality.

use super::{AgentConfig, ConfigError, ConfigValidator};
use std::path::Path;
use tokio::fs;
use tracing::{debug, info};

/// Configuration loader for YAML files
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from a YAML file
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<AgentConfig, ConfigError> {
        let path = path.as_ref();
        debug!(path = ?path, "Loading configuration from file");

        let content = fs::read_to_string(path)
            .await
            .map_err(|e| ConfigError::FileReadError {
                path: path.display().to_string(),
                source: e,
            })?;

        let config = Self::load_from_string(&content)?;
        info!(
            path = ?path,
            agent_count = config.agents.len(),
            "Successfully loaded configuration"
        );

        Ok(config)
    }

    /// Load configuration from a YAML string
    pub fn load_from_string(content: &str) -> Result<AgentConfig, ConfigError> {
        debug!("Parsing YAML configuration");
        let config: AgentConfig = serde_yaml::from_str(content)?;

        // Validate the loaded configuration
        ConfigValidator::validate_config(&config)?;

        debug!(
            agent_count = config.agents.len(),
            "Configuration parsed and validated"
        );
        Ok(config)
    }
}
