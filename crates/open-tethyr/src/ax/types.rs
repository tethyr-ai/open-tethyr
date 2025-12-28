//! AX Protocol Types
//!
//! Common types and enums used throughout the AX protocol implementation.

use serde::{Deserialize, Serialize};

/// Endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub protocol: Protocol,
    pub url: String,
    pub auth: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

/// Protocol enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Rest,
    #[serde(rename = "graphql")]
    GraphQL,
    #[serde(rename = "mcp")]
    MCP,
    #[serde(rename = "a2a")]
    A2A,
    Custom(String),
}

/// Security configuration (placeholder for future expansion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Security {
    // Will be expanded based on AX specification requirements
}

/// Capabilities configuration (placeholder for future expansion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    // Will be expanded based on AX specification requirements
}

/// Schema configuration (placeholder for future expansion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    // Will be expanded based on AX specification requirements
}

/// Limits configuration (placeholder for future expansion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    // Will be expanded based on AX specification requirements
}
