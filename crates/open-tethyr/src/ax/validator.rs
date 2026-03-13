//! AX Record Validation

use super::{Agent, AgentExchangeRecord, Endpoint};
use crate::error::{AxError, APPROVED_AUTH_METHODS};

/// Severity level for validation items
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

/// A single validation finding
#[derive(Debug, Clone)]
pub struct ValidationItem {
    pub severity: Severity,
    pub field: String,
    pub message: String,
}

/// Validation report containing all findings
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub items: Vec<ValidationItem>,
}

impl ValidationReport {
    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|i| i.severity == Severity::Error)
    }

    pub fn errors(&self) -> Vec<&ValidationItem> {
        self.items.iter().filter(|i| i.severity == Severity::Error).collect()
    }

    pub fn warnings(&self) -> Vec<&ValidationItem> {
        self.items.iter().filter(|i| i.severity == Severity::Warning).collect()
    }
}

/// AX record validator
pub struct AxValidator;

impl AxValidator {
    /// Validate an AX record, returning a detailed report
    pub fn validate_record_detailed(record: &AgentExchangeRecord) -> ValidationReport {
        let mut report = ValidationReport::default();

        // Validate record_type
        if record.record_type != "AX" {
            report.items.push(ValidationItem {
                severity: Severity::Error,
                field: "record_type".to_string(),
                message: format!("Expected 'AX', got '{}'", record.record_type),
            });
        }

        // Validate version
        if record.version != "1.0" {
            report.items.push(ValidationItem {
                severity: Severity::Warning,
                field: "version".to_string(),
                message: format!("Unsupported version '{}', expected '1.0'", record.version),
            });
        }

        // Validate agent
        if record.agent.name.is_empty() {
            report.items.push(ValidationItem {
                severity: Severity::Error,
                field: "agent.name".to_string(),
                message: "Agent name is required".to_string(),
            });
        }
        if record.agent.description.is_empty() {
            report.items.push(ValidationItem {
                severity: Severity::Error,
                field: "agent.description".to_string(),
                message: "Agent description is required".to_string(),
            });
        }
        if record.agent.provider.is_empty() {
            report.items.push(ValidationItem {
                severity: Severity::Error,
                field: "agent.provider".to_string(),
                message: "Agent provider is required".to_string(),
            });
        }

        // Validate endpoints
        if record.endpoints.is_empty() {
            report.items.push(ValidationItem {
                severity: Severity::Error,
                field: "endpoints".to_string(),
                message: "At least one endpoint is required".to_string(),
            });
        }

        for (i, ep) in record.endpoints.iter().enumerate() {
            if ep.url.is_empty() {
                report.items.push(ValidationItem {
                    severity: Severity::Error,
                    field: format!("endpoints[{}].url", i),
                    message: "Endpoint URL is required".to_string(),
                });
            }
            if ep.auth.is_empty() {
                report.items.push(ValidationItem {
                    severity: Severity::Error,
                    field: format!("endpoints[{}].auth", i),
                    message: "Endpoint auth methods are required".to_string(),
                });
            }
            // Validate auth methods against approved set
            for method in &ep.auth {
                if !APPROVED_AUTH_METHODS.contains(&method.as_str()) {
                    report.items.push(ValidationItem {
                        severity: Severity::Error,
                        field: format!("endpoints[{}].auth", i),
                        message: format!(
                            "Invalid auth method '{}': must be one of {:?}",
                            method, APPROVED_AUTH_METHODS
                        ),
                    });
                }
            }
        }

        report
    }

    /// Simple validation returning Result (for backward compat)
    pub fn validate_record(record: &AgentExchangeRecord) -> Result<(), AxError> {
        if record.record_type != "AX" {
            return Err(AxError::InvalidRecordType(record.record_type.clone()));
        }
        Self::validate_version(&record.version)?;
        Self::validate_agent(&record.agent)?;
        Self::validate_endpoints(&record.endpoints)?;
        Ok(())
    }

    pub fn validate_agent(agent: &Agent) -> Result<(), AxError> {
        if agent.name.is_empty() {
            return Err(AxError::MissingField("agent.name".into()));
        }
        if agent.description.is_empty() {
            return Err(AxError::MissingField("agent.description".into()));
        }
        if agent.provider.is_empty() {
            return Err(AxError::MissingField("agent.provider".into()));
        }
        Ok(())
    }

    pub fn validate_version(version: &str) -> Result<(), AxError> {
        match version {
            "1.0" => Ok(()),
            _ => Err(AxError::UnsupportedVersion(version.to_string())),
        }
    }

    pub fn validate_endpoints(endpoints: &[Endpoint]) -> Result<(), AxError> {
        if endpoints.is_empty() {
            return Err(AxError::MissingField("endpoints".into()));
        }
        for ep in endpoints {
            if ep.url.is_empty() {
                return Err(AxError::MissingField("endpoint.url".into()));
            }
            if ep.auth.is_empty() {
                return Err(AxError::MissingField("endpoint.auth".into()));
            }
            Self::validate_auth_methods(&ep.auth)?;
        }
        Ok(())
    }

    pub fn validate_auth_methods(methods: &[String]) -> Result<(), AxError> {
        for method in methods {
            if !APPROVED_AUTH_METHODS.contains(&method.as_str()) {
                return Err(AxError::InvalidAuthMethod(method.clone()));
            }
        }
        Ok(())
    }
}
