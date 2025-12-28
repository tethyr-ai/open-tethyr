//! AX protocol implementation
//!
//! This module provides data structures and validation for the Agent Discovery Exchange (AX) protocol.

use serde::{Deserialize, Serialize};

/// AX record data structure following AX 1.0 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExchangeRecord {
    #[serde(default = "default_record_type")]
    pub record_type: String,
    
    #[serde(default = "default_version")]
    pub version: String,
    
    pub agent: Agent,
    pub endpoints: Vec<Endpoint>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<serde_json::Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<serde_json::Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

fn default_record_type() -> String {
    "AX".to_string()
}

fn default_version() -> String {
    "1.0".to_string()
}

/// Agent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub provider: String,
}

/// Container for multiple AX records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExchangeDocument {
    pub records: Vec<AgentExchangeRecord>,
}

/// Endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub protocol: Protocol,
    pub url: String,
    pub auth: Vec<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

/// Protocol enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Rest,
    #[serde(rename = "graphql")]
    GraphQL,
    #[serde(rename = "mcp")]
    MCP,
    #[serde(rename = "a2a")]
    A2A,
    #[serde(untagged)]
    Custom(String),
}

/// AX protocol validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid record type: {0}, expected 'AX'")]
    InvalidRecordType(String),
    
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

/// AX record validator
pub struct AxValidator;

impl AxValidator {
    /// Validate an AX record for compliance
    pub fn validate_record(record: &AgentExchangeRecord) -> Result<(), ValidationError> {
        // Validate record_type is "AX"
        if record.record_type != "AX" {
            return Err(ValidationError::InvalidRecordType(record.record_type.clone()));
        }
        
        // Validate version is supported
        Self::validate_version(&record.version)?;
        
        // Validate agent and endpoints
        Self::validate_agent(&record.agent)?;
        Self::validate_endpoints(&record.endpoints)?;
        
        Ok(())
    }
    
    /// Validate agent structure
    pub fn validate_agent(agent: &Agent) -> Result<(), ValidationError> {
        if agent.name.is_empty() {
            return Err(ValidationError::MissingField("agent.name".to_string()));
        }
        if agent.description.is_empty() {
            return Err(ValidationError::MissingField("agent.description".to_string()));
        }
        if agent.provider.is_empty() {
            return Err(ValidationError::MissingField("agent.provider".to_string()));
        }
        Ok(())
    }
    
    /// Validate version compatibility
    pub fn validate_version(version: &str) -> Result<(), ValidationError> {
        match version {
            "1.0" => Ok(()),
            _ => Err(ValidationError::UnsupportedVersion(version.to_string())),
        }
    }
    
    /// Validate endpoints
    pub fn validate_endpoints(endpoints: &[Endpoint]) -> Result<(), ValidationError> {
        if endpoints.is_empty() {
            return Err(ValidationError::MissingField("endpoints".to_string()));
        }
        
        for endpoint in endpoints {
            if endpoint.url.is_empty() {
                return Err(ValidationError::MissingField("endpoint.url".to_string()));
            }
            if endpoint.auth.is_empty() {
                return Err(ValidationError::MissingField("endpoint.auth".to_string()));
            }
        }
        
        Ok(())
    }
}

/// AX record generator
pub struct AxGenerator;

impl AxGenerator {
    /// Generate AX record from configuration
    pub fn generate_record(_config: &crate::config::AgentConfig) -> Result<AgentExchangeDocument, Box<dyn std::error::Error>> {
        // Placeholder implementation
        todo!("Implementation will be added in task 2")
    }
}