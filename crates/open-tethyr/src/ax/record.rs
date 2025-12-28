//! AX Record Data Structures
//!
//! Core data structures for AX protocol records following AX 1.0 specification.

use serde::{Deserialize, Serialize};

/// AX record data structure following AX 1.0 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExchangeRecord {
    #[serde(default = "default_record_type")]
    pub record_type: String,

    #[serde(default = "default_version")]
    pub version: String,

    pub agent: Agent,
    pub endpoints: Vec<super::Endpoint>,

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
