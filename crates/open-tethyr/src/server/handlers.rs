//! HTTP request handlers

use crate::cache::CacheCoordinator;
use crate::policy::enforcement::PolicyEngine;
use crate::server::cache_server::CacheMetrics;
use crate::server::error::ServerError;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, warn};
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

    // Default to localhost for MVP
    IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))
}

/// Discovery endpoint handler
pub async fn handle_discover(
    State(coordinator): State<Arc<CacheCoordinator>>,
    State(policy_engine): State<Arc<PolicyEngine>>,
    State(metrics): State<Arc<CacheMetrics>>,
    Path(domain): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ServerError> {
    let correlation_id = Uuid::new_v4();
    let start_time = Instant::now();

    info!(
        domain = %domain,
        correlation_id = %correlation_id,
        "Discovery request received"
    );

    // Extract client IP
    let client_ip = extract_client_ip(&headers);

    // Increment active connections
    metrics.increment_connections();

    // Policy enforcement
    if let Err(e) =
        policy_engine.validate_discovery_request(&domain, client_ip, &correlation_id.to_string())
    {
        warn!(
            domain = %domain,
            correlation_id = %correlation_id,
            error = %e,
            "Policy violation"
        );
        metrics.decrement_connections();
        return Err(ServerError::PolicyViolation(e));
    }

    // Attempt discovery through cache coordinator
    let result = coordinator.discover(&domain).await;

    // Record metrics
    let duration = start_time.elapsed();
    metrics.record_request_duration(duration);
    metrics.decrement_connections();

    match result {
        Ok(document) => {
            info!(
                domain = %domain,
                correlation_id = %correlation_id,
                records_count = document.records.len(),
                duration_ms = duration.as_millis(),
                "Discovery successful"
            );

            // Build response with correlation ID header
            let mut response_headers = HeaderMap::new();
            response_headers.insert(
                "x-correlation-id",
                correlation_id.to_string().parse().unwrap(),
            );

            Ok((StatusCode::OK, response_headers, Json(document)).into_response())
        }
        Err(e) => {
            error!(
                domain = %domain,
                correlation_id = %correlation_id,
                error = %e,
                duration_ms = duration.as_millis(),
                "Discovery failed"
            );
            Err(ServerError::DiscoveryFailed(e.to_string()))
        }
    }
}

/// Health check endpoint handler
pub async fn handle_health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Metrics endpoint handler (Prometheus text format)
pub async fn handle_metrics(State(metrics): State<Arc<CacheMetrics>>) -> String {
    let cache_stats = metrics.cache_stats();
    let snapshot = cache_stats.snapshot();

    // Basic Prometheus text format
    format!(
        "# HELP cache_hits_total Total number of cache hits\n\
         # TYPE cache_hits_total counter\n\
         cache_hits_total {}\n\
         # HELP cache_misses_total Total number of cache misses\n\
         # TYPE cache_misses_total counter\n\
         cache_misses_total {}\n\
         # HELP active_connections Current number of active connections\n\
         # TYPE active_connections gauge\n\
         active_connections {}\n\
         # HELP cache_entries Current number of cached entries\n\
         # TYPE cache_entries gauge\n\
         cache_entries {}\n\
         # HELP cache_evictions_total Total number of cache evictions\n\
         # TYPE cache_evictions_total counter\n\
         cache_evictions_total {}\n",
        snapshot.hit_count,
        snapshot.miss_count,
        metrics.active_connections(),
        snapshot.total_entries,
        snapshot.eviction_count,
    )
}
