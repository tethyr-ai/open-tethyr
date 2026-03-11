//! Server Error Types
//!
//! Comprehensive error types for HTTP server with proper status code mapping.

use crate::cache::CacheError;
use crate::policy::enforcement::PolicyViolation;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

/// Server error types with HTTP status code mapping
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Policy violation: {0}")]
    PolicyViolation(#[from] PolicyViolation),

    #[error("Discovery failed: {0}")]
    DiscoveryFailed(String),

    #[error("Cache error: {0}")]
    CacheError(#[from] CacheError),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Domain not found: {0}")]
    NotFound(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Request timeout after {0:?}")]
    Timeout(std::time::Duration),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Server startup failed: {0}")]
    StartupFailed(String),

    #[error("Invalid bind address: {0}")]
    InvalidBindAddress(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let correlation_id = Uuid::new_v4();
        let timestamp = chrono::Utc::now().to_rfc3339();

        let (status, error_message, error_code) = match self {
            // 400 Bad Request - Invalid requests
            ServerError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg, "INVALID_REQUEST"),
            ServerError::PolicyViolation(PolicyViolation::InvalidDomain { domain }) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid domain format: '{}'", domain),
                "INVALID_DOMAIN",
            ),

            // 403 Forbidden - Policy violations
            ServerError::PolicyViolation(PolicyViolation::DomainNotAllowed { domain }) => (
                StatusCode::FORBIDDEN,
                format!(
                    "Domain '{}' is not allowed by domain locking policy",
                    domain
                ),
                "DOMAIN_NOT_ALLOWED",
            ),
            ServerError::PolicyViolation(PolicyViolation::ExternalDomainBlocked { domain }) => (
                StatusCode::FORBIDDEN,
                format!("External domain '{}' is not in allowlist", domain),
                "EXTERNAL_DOMAIN_BLOCKED",
            ),

            // 404 Not Found
            ServerError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, "NOT_FOUND"),

            // 429 Too Many Requests - Rate limiting
            ServerError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded".to_string(),
                "RATE_LIMIT_EXCEEDED",
            ),

            // 502 Bad Gateway - Upstream failures
            ServerError::DiscoveryFailed(msg) => (
                StatusCode::BAD_GATEWAY,
                format!("Discovery failed: {}", msg),
                "DISCOVERY_FAILED",
            ),
            ServerError::CacheError(ref e) => (
                StatusCode::BAD_GATEWAY,
                format!("Cache error: {}", e),
                "CACHE_ERROR",
            ),

            // 503 Service Unavailable
            ServerError::ServiceUnavailable(msg) => {
                (StatusCode::SERVICE_UNAVAILABLE, msg, "SERVICE_UNAVAILABLE")
            }
            ServerError::Timeout(duration) => (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Request timeout after {:?}", duration),
                "TIMEOUT",
            ),

            // 500 Internal Server Error - Catch-all
            ServerError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg, "INTERNAL_ERROR")
            }
            ServerError::ConfigError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Configuration error: {}", msg),
                "CONFIG_ERROR",
            ),
            ServerError::StartupFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Server startup failed: {}", msg),
                "STARTUP_FAILED",
            ),
            ServerError::InvalidBindAddress(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invalid bind address: {}", msg),
                "INVALID_BIND_ADDRESS",
            ),
            ServerError::IoError(ref e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("IO error: {}", e),
                "IO_ERROR",
            ),
        };

        // Build JSON error response with detailed context
        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": error_message,
            },
            "timestamp": timestamp,
            "correlation_id": correlation_id.to_string(),
        }));

        // Build response with correlation ID header
        let mut response = (status, body).into_response();
        response.headers_mut().insert(
            "x-correlation-id",
            correlation_id.to_string().parse().unwrap(),
        );

        response
    }
}
