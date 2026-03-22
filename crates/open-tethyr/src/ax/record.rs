//! AX Record Data Structures - aligned with official AX draft spec

use super::types::{Capabilities, Endpoint, Limits, Schema, Security};
use serde::{Deserialize, Serialize};

/// AX document (single flat record per spec)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentExchangeRecord {
    #[serde(default = "default_record_type")]
    pub record_type: String,
    #[serde(default = "default_version")]
    pub version: String,
    pub agent: Agent,
    pub endpoints: Vec<Endpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Capabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<Limits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Security>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

fn default_record_type() -> String {
    "AX".to_string()
}
fn default_version() -> String {
    "1.0".to_string()
}

/// Agent definition - provider is optional per spec
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// Legacy wrapper format for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentExchangeDocument {
    pub records: Vec<AgentExchangeRecord>,
}

/// Parse AX JSON that could be either flat (spec) or legacy wrapper
pub fn parse_ax_json(json: &str) -> Result<Vec<AgentExchangeRecord>, serde_json::Error> {
    // Try flat document first (spec-compliant)
    if let Ok(record) = serde_json::from_str::<AgentExchangeRecord>(json) {
        return Ok(vec![record]);
    }
    // Fall back to legacy wrapper
    let doc: AgentExchangeDocument = serde_json::from_str(json)?;
    Ok(doc.records)
}
