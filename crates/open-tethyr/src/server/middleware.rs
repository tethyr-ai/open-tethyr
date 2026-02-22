//! HTTP middleware
//!
//! Middleware for rate limiting, timeouts, and request handling.

use crate::cache::{RateLimitError, RateLimiter};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::warn;
use uuid::Uuid;

/// Extract client IP from request headers
fn extract_client_ip(headers: &HeaderMap) -> IpAddr {
    // Try X-Forwarded-For header first
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP in the list
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse() {
                    return ip;
                }
            }
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse() {
                return ip;
            }
        }
    }

    // Default to localhost
    IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    axum::extract::State(rate_limiter): axum::extract::State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip(request.headers());

    match rate_limiter.check_rate_limit(client_ip) {
        Ok(()) => next.run(request).await,
        Err(RateLimitError::RateLimitExceeded { client_ip }) => {
            warn!(
                client_ip = %client_ip,
                "Rate limit exceeded"
            );

            let correlation_id = Uuid::new_v4();
            let body = Json(json!({
                "error": format!("Rate limit exceeded for client {}", client_ip),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "correlation_id": correlation_id.to_string(),
            }));

            let mut response = (StatusCode::TOO_MANY_REQUESTS, body).into_response();
            response.headers_mut().insert(
                "x-correlation-id",
                correlation_id.to_string().parse().unwrap(),
            );
            response.headers_mut().insert(
                "retry-after",
                "60".parse().unwrap(), // Suggest retry after 60 seconds
            );

            response
        }
    }
}

/// Request timeout middleware
pub async fn timeout_middleware(
    request: Request,
    next: Next,
    timeout_duration: Duration,
) -> Response {
    match timeout(timeout_duration, next.run(request)).await {
        Ok(response) => response,
        Err(_) => {
            warn!("Request timeout after {:?}", timeout_duration);

            let correlation_id = Uuid::new_v4();
            let body = Json(json!({
                "error": "Request timeout",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "correlation_id": correlation_id.to_string(),
            }));

            let mut response = (StatusCode::GATEWAY_TIMEOUT, body).into_response();
            response.headers_mut().insert(
                "x-correlation-id",
                correlation_id.to_string().parse().unwrap(),
            );

            response
        }
    }
}

/// Middleware error response
#[derive(Debug)]
pub struct MiddlewareError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for MiddlewareError {
    fn into_response(self) -> Response {
        let correlation_id = Uuid::new_v4();
        let body = Json(json!({
            "error": self.message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "correlation_id": correlation_id.to_string(),
        }));

        let mut response = (self.status, body).into_response();
        response.headers_mut().insert(
            "x-correlation-id",
            correlation_id.to_string().parse().unwrap(),
        );

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::RateLimitConfig;
    use std::net::Ipv4Addr;

    #[test]
    fn test_extract_client_ip_from_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "192.168.1.100, 10.0.0.1".parse().unwrap(),
        );

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));
    }

    #[test]
    fn test_extract_client_ip_from_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "192.168.1.200".parse().unwrap());

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 200)));
    }

    #[test]
    fn test_extract_client_ip_default() {
        let headers = HeaderMap::new();

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }

    #[tokio::test]
    async fn test_rate_limiter_allows_requests() {
        let config = RateLimitConfig {
            requests_per_minute: 10,
            requests_per_hour: 600,
        };
        let limiter = Arc::new(RateLimiter::new(config));
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Should allow first request
        assert!(limiter.check_rate_limit(client_ip).is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_excess_requests() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            requests_per_hour: 120,
        };
        let limiter = Arc::new(RateLimiter::new(config));
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Exhaust limit
        assert!(limiter.check_rate_limit(client_ip).is_ok());
        assert!(limiter.check_rate_limit(client_ip).is_ok());

        // Should block third request
        assert!(limiter.check_rate_limit(client_ip).is_err());
    }
}
