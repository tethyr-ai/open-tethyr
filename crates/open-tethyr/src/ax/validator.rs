//! AX Record Validation - aligned with official AX draft spec

use super::{Agent, AgentExchangeRecord, Endpoint};
use crate::error::{AxError, KNOWN_AUTH_METHODS};

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct ValidationItem {
    pub severity: Severity,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub items: Vec<ValidationItem>,
}

impl ValidationReport {
    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|i| i.severity == Severity::Error)
    }
    pub fn errors(&self) -> Vec<&ValidationItem> {
        self.items
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect()
    }
    pub fn warnings(&self) -> Vec<&ValidationItem> {
        self.items
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .collect()
    }
}

pub struct AxValidator;

impl AxValidator {
    pub fn validate_record_detailed(record: &AgentExchangeRecord) -> ValidationReport {
        let mut report = ValidationReport::default();
        if record.record_type != "AX" {
            report.items.push(ValidationItem {
                severity: Severity::Error,
                field: "record_type".into(),
                message: format!("Expected 'AX', got '{}'", record.record_type),
            });
        }
        if record.version != "1.0" {
            report.items.push(ValidationItem {
                severity: Severity::Warning,
                field: "version".into(),
                message: format!("Unsupported version '{}', expected '1.0'", record.version),
            });
        }
        if record.agent.name.is_empty() {
            report.items.push(ValidationItem {
                severity: Severity::Error,
                field: "agent.name".into(),
                message: "Agent name is required".into(),
            });
        }
        if record.agent.description.is_empty() {
            report.items.push(ValidationItem {
                severity: Severity::Error,
                field: "agent.description".into(),
                message: "Agent description is required".into(),
            });
        }
        // agent.provider is OPTIONAL per spec - no validation
        if record.endpoints.is_empty() {
            report.items.push(ValidationItem {
                severity: Severity::Error,
                field: "endpoints".into(),
                message: "At least one endpoint is required".into(),
            });
        }
        for (i, ep) in record.endpoints.iter().enumerate() {
            if ep.url.is_empty() {
                report.items.push(ValidationItem {
                    severity: Severity::Error,
                    field: format!("endpoints[{}].url", i),
                    message: "Endpoint URL is required".into(),
                });
            }
            // auth is OPTIONAL per spec - warn on unknown methods if present
            for method in &ep.auth {
                if !KNOWN_AUTH_METHODS.contains(&method.as_str()) {
                    report.items.push(ValidationItem {
                        severity: Severity::Warning,
                        field: format!("endpoints[{}].auth", i),
                        message: format!(
                            "Unknown auth method '{}' (known: {:?})",
                            method, KNOWN_AUTH_METHODS
                        ),
                    });
                }
            }
        }
        report
    }

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
        // provider is optional per spec
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
            // auth is optional per spec - just warn on unknown
            for method in &ep.auth {
                if !KNOWN_AUTH_METHODS.contains(&method.as_str()) {
                    tracing::warn!("Unknown auth method: {}", method);
                }
            }
        }
        Ok(())
    }
}
