//! OAuth Types
//!
//! Common types for OAuth provider implementations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// OAuth endpoints configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OAuthEndpoints {
    /// OAuth2/OIDC issuer URL
    pub issuer: String,
    /// Authorization endpoint URL
    pub authorization_endpoint: String,
    /// Token endpoint URL
    pub token_endpoint: String,
    /// JWKS URI for token verification
    pub jwks_uri: String,
    /// UserInfo endpoint URL (optional)
    pub userinfo_endpoint: Option<String>,
    /// Token revocation endpoint URL (optional)
    pub revocation_endpoint: Option<String>,
    /// Additional provider-specific endpoints
    pub additional_endpoints: HashMap<String, String>,
}

/// OAuth configuration for provider template generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// Provider domain (e.g., "dev-123.okta.com", "acme.auth0.com")
    pub domain: Option<String>,
    /// Client ID for the OAuth application
    pub client_id: Option<String>,
    /// Additional provider-specific configuration
    pub additional_config: HashMap<String, String>,
}

/// OAuth provider errors
#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("Missing required domain configuration")]
    MissingDomain,
    #[error("Invalid domain format: {0}")]
    InvalidDomain(String),
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
