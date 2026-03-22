//! Server Route Tests - exercise every HTTP endpoint with real requests
//! Catches routing bugs, status code mismatches, JSON shape issues, and header problems.

#[cfg(feature = "server")]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use open_tethyr::cache::coordinator::CacheCoordinator;
    use open_tethyr::cache::stats::CacheStats;
    use open_tethyr::server::{AppState, CacheServer, PolicyEngine};
    use std::sync::Arc;
    use tower::ServiceExt; // for oneshot

    fn test_state() -> Arc<AppState> {
        Arc::new(AppState {
            coordinator: CacheCoordinator::new(100, 3600, None).unwrap(),
            policy: PolicyEngine::default(),
            stats: CacheStats::new(),
        })
    }

    fn test_state_locked(home: &str) -> Arc<AppState> {
        Arc::new(AppState {
            coordinator: CacheCoordinator::new(100, 3600, None).unwrap(),
            policy: PolicyEngine::new(true, Some(home.into()), vec![]),
            stats: CacheStats::new(),
        })
    }

    #[tokio::test]
    async fn health_returns_200_with_json() {
        let app = CacheServer::build_routes(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(
            ct.contains("json"),
            "Health should return JSON, got: {}",
            ct
        );
    }

    #[tokio::test]
    async fn metrics_returns_200() {
        let app = CacheServer::build_routes(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(
            text.contains("cache_hits"),
            "Metrics should include cache_hits: {}",
            text
        );
    }

    #[tokio::test]
    async fn discover_returns_502_on_upstream_failure() {
        let app = CacheServer::build_routes(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/discover/nonexistent-domain-12345.invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Should be 502 (upstream failure) or 404, not 500
        let status = resp.status();
        assert!(
            status == StatusCode::BAD_GATEWAY || status == StatusCode::NOT_FOUND,
            "Expected 502 or 404, got {}",
            status
        );
    }

    #[tokio::test]
    async fn discover_returns_403_when_policy_blocks() {
        let app = CacheServer::build_routes(test_state_locked("home.com"));
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/discover/evil.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "Locked domain should return 403 for external domain"
        );
    }

    #[tokio::test]
    async fn discover_allows_home_domain_when_locked() {
        let app = CacheServer::build_routes(test_state_locked("home.com"));
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/discover/home.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Should NOT be 403 (domain is allowed)
        assert_ne!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "Home domain should not be blocked"
        );
    }

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let app = CacheServer::build_routes(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/nonexistent/path")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn health_response_has_correct_shape() {
        let app = CacheServer::build_routes(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(
            json.get("status").is_some(),
            "Health response should have 'status' field: {:?}",
            json
        );
    }
}
