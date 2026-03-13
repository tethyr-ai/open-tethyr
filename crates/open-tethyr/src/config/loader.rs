//! Configuration Loading

use super::models::AgentConfig;
use crate::error::ConfigError;
use std::path::Path;

/// Load configuration from a YAML file
pub fn load_config(path: &Path) -> Result<AgentConfig, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        ConfigError::ParseError(format!("Failed to read {}: {}", path.display(), e))
    })?;
    serde_yaml::from_str(&content)
        .map_err(|e| ConfigError::ParseError(format!("YAML parse error: {}", e)))
}
