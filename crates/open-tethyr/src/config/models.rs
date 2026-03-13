//! Configuration Models

use serde::{Deserialize, Serialize};

/// Top-level agent configuration (YAML input)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    #[serde(default)]
    pub defaults: AgentDefaults,
    #[serde(default)]
    pub agents: Vec<AgentDefinition>,
    #[serde(default)]
    pub server: Option<ServerConfig>,
}

/// Global defaults inherited by all agents
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default)]
    pub auth: Vec<String>,
    #[serde(default)]
    pub endpoints: Vec<EndpointDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

/// Per-agent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default)]
    pub endpoints: Vec<EndpointDefinition>,
    #[serde(default)]
    pub auth: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_domain: Option<String>,
}

/// Endpoint definition in configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointDefinition {
    pub protocol: String,
    pub url: String,
    #[serde(default)]
    pub auth: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_domain")]
    pub domain: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub policy: PolicyConfig,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_domain() -> String {
    "localhost".to_string()
}
fn default_port() -> u16 {
    8080
}
fn default_log_level() -> String {
    "info".to_string()
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
    #[serde(default = "default_ttl")]
    pub default_ttl: u64,
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval: u64,
    #[serde(default = "default_enable_lru")]
    pub enable_lru: bool,
}

fn default_max_entries() -> usize {
    10_000
}
fn default_ttl() -> u64 {
    3600
}
fn default_cleanup_interval() -> u64 {
    300
}
fn default_enable_lru() -> bool {
    true
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: default_max_entries(),
            default_ttl: default_ttl(),
            cleanup_interval: default_cleanup_interval(),
            enable_lru: default_enable_lru(),
        }
    }
}

/// Policy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyConfig {
    #[serde(default)]
    pub domain_locking: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_domain: Option<String>,
    #[serde(default)]
    pub allowlist: Vec<String>,
}

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_rpm")]
    pub requests_per_minute: u32,
    #[serde(default = "default_rph")]
    pub requests_per_hour: u32,
}

fn default_rpm() -> u32 {
    60
}
fn default_rph() -> u32 {
    1000
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: default_rpm(),
            requests_per_hour: default_rph(),
        }
    }
}
