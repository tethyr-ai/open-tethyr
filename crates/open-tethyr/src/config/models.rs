//! Configuration Models
//!
//! Data structures for YAML configuration parsing and inheritance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent configuration with defaults and agents sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defaults: Option<AgentDefaults>,

    pub agents: Vec<AgentDefinition>,
}

/// Agent defaults configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

/// Individual agent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub description: String,
    pub url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}
