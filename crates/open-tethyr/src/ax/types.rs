//! AX Protocol Types

use serde::{Deserialize, Serialize};

/// Endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Endpoint {
    pub protocol: Protocol,
    pub url: String,
    pub auth: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
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

/// Security configuration including OAuth endpoints
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Security {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth: Option<OAuthEndpoints>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// OAuth endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OAuthEndpoints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwks_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userinfo_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint: Option<String>,
}

/// Capabilities configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Capabilities {
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Schema configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Schema {
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Limits configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Limits {
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
