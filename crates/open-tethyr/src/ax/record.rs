//! AX Record Data Structures

use serde::{Deserialize, Serialize};
use super::types::{Capabilities, Endpoint, Limits, Schema, Security};

/// AX record data structure following AX 1.0 specification
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

fn default_record_type() -> String { "AX".to_string() }
fn default_version() -> String { "1.0".to_string() }

/// Agent definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub provider: String,
}

/// Container for multiple AX records
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentExchangeDocument {
    pub records: Vec<AgentExchangeRecord>,
}
