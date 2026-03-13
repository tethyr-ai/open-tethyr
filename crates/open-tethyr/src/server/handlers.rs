//! Request Handlers

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;

use super::AppState;

/// Discovery handler
pub async fn handle_discover(
    Path(domain): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Check rate limit
    // (simplified - in production would extract client IP from request)

    // Check policy
    if let Err(e) = state.policy.check_discovery_allowed(&domain) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": e.to_string(),
                "timestamp": chrono_now(),
            })),
        )
            .into_response();
    }

    // Discover via coordinator
    match state.coordinator.discover(&domain).await {
        Ok(data) => {
            state.stats.record_hit();
            (StatusCode::OK, data).into_response()
        }
        Err(e) => {
            state.stats.record_miss();
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": e.to_string(),
                    "timestamp": chrono_now(),
                })),
            )
                .into_response()
        }
    }
}

/// Health check handler
pub async fn handle_health() -> impl IntoResponse {
    Json(json!({ "status": "ok", "version": crate::VERSION }))
}

/// Metrics handler (Prometheus text format)
pub async fn handle_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let stats = &state.stats;
    let body = format!(
        "# HELP cache_hits_total Total cache hits\n# TYPE cache_hits_total counter\ncache_hits_total {}\n\
         # HELP cache_misses_total Total cache misses\n# TYPE cache_misses_total counter\ncache_misses_total {}\n\
         # HELP cache_evictions_total Total cache evictions\n# TYPE cache_evictions_total counter\ncache_evictions_total {}\n",
        stats.hits(), stats.misses(), stats.evictions(),
    );
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        body,
    )
}

fn chrono_now() -> String {
    // Simple ISO timestamp without chrono dependency
    format!(
        "{:?}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    )
}
