//! AX Protocol Types - aligned with official AX draft spec

use serde::{Deserialize, Serialize};

/// Endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Endpoint {
    pub protocol: Protocol,
    pub url: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub auth: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Protocol enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Security configuration (flat, per AX spec)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Security {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwks_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_signature: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Capabilities configuration (typed per AX spec)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Capabilities {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intents: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "async")]
    pub async_exec: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_callbacks: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub callback_modes: Vec<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Schema configuration (typed per AX spec)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Schema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graphql_schema_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_manifest_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_openapi_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introspection: Option<bool>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Limits configuration (typed per AX spec)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Limits {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_tasks: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_task_ttl_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_minute: Option<f64>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
