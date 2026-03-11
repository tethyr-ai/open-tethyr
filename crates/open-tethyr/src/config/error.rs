//! Configuration Error Types
//!
//! Structured error types for configuration loading, validation, and merging.

use thiserror::Error;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {path}")]
    FileReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse YAML configuration: {0}")]
    YamlParseError(#[from] serde_yaml::Error),

    #[error("Configuration validation failed: {0}")]
    ValidationError(String),

    #[error("Invalid domain format: {domain}")]
    InvalidDomain { domain: String },

    #[error("Invalid port number: {port} (must be 1-65535)")]
    InvalidPort { port: u16 },

    #[error("Invalid URL format: {url}")]
    InvalidUrl { url: String },

    #[error("Invalid TTL value: {ttl} (must be positive)")]
    InvalidTtl { ttl: u32 },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Agent {index} validation failed: {message}")]
    AgentValidationError { index: usize, message: String },

    #[error("Configuration merge error: {0}")]
    MergeError(String),
}
