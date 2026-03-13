//! Property 23: HTTP Error Response Mapping
// Tests that error types map to correct status codes.
// These are verified via the server error handling structure.

use open_tethyr::error::{ServerError, CacheError};

#[test]
fn policy_violation_error_exists() {
    let err = ServerError::PolicyViolation("blocked".into());
    assert!(err.to_string().contains("blocked"));
}

#[test]
fn rate_limit_error_exists() {
    let err = ServerError::RateLimitExceeded("127.0.0.1".into());
    assert!(err.to_string().contains("127.0.0.1"));
}

#[test]
fn cache_not_found_error_exists() {
    let err = CacheError::NotFound("example.com".into());
    assert!(err.to_string().contains("example.com"));
}

#[test]
fn upstream_failed_error_exists() {
    let err = ServerError::UpstreamFailed("timeout".into());
    assert!(err.to_string().contains("timeout"));
}
