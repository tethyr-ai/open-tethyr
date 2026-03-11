//! AX Record Validation
//!
//! Validation logic for AX protocol compliance.

use super::{Agent, AgentExchangeRecord, Endpoint};
use std::collections::HashSet;
use tracing::{debug, warn};

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

    #[error("Invalid auth method: {0}")]
    InvalidAuthMethod(String),

    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
}

/// AX record validator
pub struct AxValidator;

impl AxValidator {
    /// Supported authentication methods as per AX specification
    const SUPPORTED_AUTH_METHODS: &'static [&'static str] =
        &["OIDC", "OAuth2", "mTLS", "JWT", "API_KEY"];

    /// Validate an AX record for compliance
    pub fn validate_record(record: &AgentExchangeRecord) -> Result<(), ValidationError> {
        debug!(
            record_type = %record.record_type,
            version = %record.version,
            agent_name = %record.agent.name,
            "Validating AX record"
        );

        // Validate record_type is "AX"
        if record.record_type != "AX" {
            warn!(
                record_type = %record.record_type,
                "Invalid record type, expected 'AX'"
            );
            return Err(ValidationError::InvalidRecordType(
                record.record_type.clone(),
            ));
        }

        // Validate version is "1.0"
        Self::validate_version(&record.version)?;

        // Validate required fields and format constraints
        Self::validate_agent(&record.agent)?;
        Self::validate_endpoints(&record.endpoints)?;

        debug!(agent_name = %record.agent.name, "AX record validation successful");
        Ok(())
    }

    /// Validate agent structure
    pub fn validate_agent(agent: &Agent) -> Result<(), ValidationError> {
        if agent.name.trim().is_empty() {
            return Err(ValidationError::MissingField("agent.name".to_string()));
        }
        if agent.description.trim().is_empty() {
            return Err(ValidationError::MissingField(
                "agent.description".to_string(),
            ));
        }
        if agent.provider.trim().is_empty() {
            return Err(ValidationError::MissingField("agent.provider".to_string()));
        }
        Ok(())
    }

    /// Validate version is "1.0"
    pub fn validate_version(version: &str) -> Result<(), ValidationError> {
        match version {
            "1.0" => Ok(()),
            _ => {
                warn!(version = %version, "Unsupported AX version");
                Err(ValidationError::UnsupportedVersion(version.to_string()))
            }
        }
    }

    /// Validate endpoints with comprehensive checks
    pub fn validate_endpoints(endpoints: &[Endpoint]) -> Result<(), ValidationError> {
        if endpoints.is_empty() {
            return Err(ValidationError::MissingField("endpoints".to_string()));
        }

        for (index, endpoint) in endpoints.iter().enumerate() {
            // Validate URL format
            if endpoint.url.trim().is_empty() {
                return Err(ValidationError::MissingField(format!(
                    "endpoints[{}].url",
                    index
                )));
            }

            // Validate URL is properly formatted
            Self::validate_url(&endpoint.url)?;

            // Validate auth methods are present
            if endpoint.auth.is_empty() {
                return Err(ValidationError::MissingField(format!(
                    "endpoints[{}].auth",
                    index
                )));
            }

            // Validate auth methods are from supported set
            Self::validate_auth_methods(&endpoint.auth)?;
        }

        Ok(())
    }

    /// Validate auth methods are from supported set (OIDC, OAuth2, mTLS, JWT, API_KEY)
    pub fn validate_auth_methods(auth_methods: &[String]) -> Result<(), ValidationError> {
        let supported_methods: HashSet<&str> =
            Self::SUPPORTED_AUTH_METHODS.iter().copied().collect();

        for auth_method in auth_methods {
            let trimmed_method = auth_method.trim();
            if !supported_methods.contains(trimmed_method) {
                return Err(ValidationError::InvalidAuthMethod(auth_method.clone()));
            }
        }

        Ok(())
    }

    /// Validate URL format
    pub fn validate_url(url: &str) -> Result<(), ValidationError> {
        let trimmed_url = url.trim();

        // Basic URL validation - must be HTTPS
        if !trimmed_url.starts_with("https://") {
            return Err(ValidationError::InvalidUrl(format!(
                "URL must use HTTPS: {}",
                url
            )));
        }

        // Use url crate for more comprehensive validation
        match url::Url::parse(trimmed_url) {
            Ok(_) => Ok(()),
            Err(_) => Err(ValidationError::InvalidUrl(format!(
                "Invalid URL format: {}",
                url
            ))),
        }
    }
}
