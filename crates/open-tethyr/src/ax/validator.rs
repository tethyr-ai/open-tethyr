//! AX Record Validation
//!
//! Validation logic for AX protocol compliance.

use super::{Agent, AgentExchangeRecord, Endpoint};

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
            return Err(ValidationError::InvalidRecordType(
                record.record_type.clone(),
            ));
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
            return Err(ValidationError::MissingField(
                "agent.description".to_string(),
            ));
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
