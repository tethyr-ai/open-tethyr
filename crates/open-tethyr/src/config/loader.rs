//! Configuration Loading
//!
//! YAML configuration loading functionality.

use super::{AgentConfig, ConfigValidator};
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;

/// Configuration loader for YAML files
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from a YAML file
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<AgentConfig> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read configuration file: {}", path.display()))?;

        Self::load_from_string(&content)
    }

    /// Load configuration from a YAML string
    pub fn load_from_string(content: &str) -> Result<AgentConfig> {
        let config: AgentConfig =
            serde_yaml::from_str(content).context("Failed to parse YAML configuration")?;

        // Validate the loaded configuration
        ConfigValidator::validate_config(&config).context("Configuration validation failed")?;

        Ok(config)
    }
}
