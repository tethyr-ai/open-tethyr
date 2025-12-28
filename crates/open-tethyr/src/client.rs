//! Client SDK for agent discovery
//!
//! This module provides the main client interface for discovering agents.

use crate::ax::{Agent, AgentExchangeDocument};

/// Client SDK for agent discovery
pub struct OpenTethyr {
    domain: String,
}

/// Client errors
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Discovery failed: {0}")]
    DiscoveryFailed(String),

    #[error("Invalid domain: {0}")]
    InvalidDomain(String),
}

impl OpenTethyr {
    /// Create a new client for the given domain
    pub fn new(domain: &str) -> Result<Self, ClientError> {
        if domain.is_empty() {
            return Err(ClientError::InvalidDomain(
                "Domain cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            domain: domain.to_string(),
        })
    }

    /// Discover agents from target domain
    pub async fn discover(&self, _target_domain: &str) -> Result<Vec<Agent>, ClientError> {
        todo!("Implementation will be added in task 13")
    }

    /// Discover agents using specific cache URL
    pub async fn discover_with_cache(
        &self,
        _target_domain: &str,
        _cache_url: &str,
    ) -> Result<Vec<Agent>, ClientError> {
        todo!("Implementation will be added in task 13")
    }
}
