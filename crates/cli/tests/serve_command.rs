//! Tests for the serve command

use open_tethyr::cache::{CacheCoordinatorConfig, RateLimitConfig};
use open_tethyr::policy::enforcement::DomainPolicy;
use open_tethyr::server::{CacheServer, ServerConfig};
use std::time::Duration;

#[tokio::test]
async fn test_serve_command_config_building() {
    // Test that we can build a valid server configuration
    let config = ServerConfig {
        bind_address: "0.0.0.0:8080".to_string(),
        domain: "example.com".to_string(),
        cache_config: CacheCoordinatorConfig::default(),
        domain_policy: DomainPolicy::default(),
        rate_limit_config: RateLimitConfig::default(),
        request_timeout: Duration::from_secs(30),
    };

    // Verify we can create a cache server with this config
    let server = CacheServer::new(config).await;
    assert!(server.is_ok());
}

#[tokio::test]
async fn test_serve_command_with_custom_port() {
    // Test that we can configure a custom port
    let config = ServerConfig {
        bind_address: "0.0.0.0:9090".to_string(),
        domain: "test.com".to_string(),
        cache_config: CacheCoordinatorConfig::default(),
        domain_policy: DomainPolicy::default(),
        rate_limit_config: RateLimitConfig::default(),
        request_timeout: Duration::from_secs(30),
    };

    let server = CacheServer::new(config).await;
    assert!(server.is_ok());

    let server = server.unwrap();
    assert_eq!(server.config().bind_address, "0.0.0.0:9090");
    assert_eq!(server.config().domain, "test.com");
}

#[tokio::test]
async fn test_serve_command_with_custom_domain() {
    // Test that we can configure a custom domain
    let config = ServerConfig {
        bind_address: "127.0.0.1:8080".to_string(),
        domain: "acme.com".to_string(),
        cache_config: CacheCoordinatorConfig::default(),
        domain_policy: DomainPolicy::default(),
        rate_limit_config: RateLimitConfig::default(),
        request_timeout: Duration::from_secs(30),
    };

    let server = CacheServer::new(config).await;
    assert!(server.is_ok());

    let server = server.unwrap();
    assert_eq!(server.config().domain, "acme.com");
}

#[tokio::test]
async fn test_serve_command_default_config() {
    // Test that default configuration works
    let config = ServerConfig::default();

    let server = CacheServer::new(config).await;
    assert!(server.is_ok());

    let server = server.unwrap();
    assert_eq!(server.config().bind_address, "127.0.0.1:8080");
    assert_eq!(server.config().domain, "localhost");
}
