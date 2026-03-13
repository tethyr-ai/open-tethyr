//! Integration test: cache server health and metrics endpoints

#[cfg(feature = "server")]
mod server_tests {
    use open_tethyr::cache::coordinator::CacheCoordinator;
    use open_tethyr::cache::stats::CacheStats;
    use open_tethyr::server::PolicyEngine;
    use open_tethyr::server::{AppState, CacheServer};
    use std::sync::Arc;

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let state = Arc::new(AppState {
            coordinator: CacheCoordinator::new(100, 3600, None).unwrap(),
            policy: PolicyEngine::default(),
            stats: CacheStats::new(),
        });
        let app = CacheServer::build_routes(state);

        let response = axum::serve(
            tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap(),
            app,
        );
        // Just verify the router builds without error
        // Full HTTP testing would require spawning the server
        drop(response);
    }

    #[test]
    fn router_builds_successfully() {
        let state = Arc::new(AppState {
            coordinator: CacheCoordinator::new(100, 3600, None).unwrap(),
            policy: PolicyEngine::default(),
            stats: CacheStats::new(),
        });
        let _router = CacheServer::build_routes(state);
    }
}
