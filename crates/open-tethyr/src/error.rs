//! Shared error types for the open-tethyr library.
//!
//! All error types use `thiserror` derive macros per constitution mandate.

use thiserror::Error;

/// AX protocol errors
#[derive(Debug, Error)]
pub enum AxError {
    #[error("Invalid record type: expected 'AX', got '{0}'")]
    InvalidRecordType(String),

    #[error("Unsupported AX version: {0}")]
    UnsupportedVersion(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Invalid auth method '{0}': must be one of OIDC, OAuth2, mTLS, JWT, API_KEY")]
    InvalidAuthMethod(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("File write failed: {0}")]
    FileWriteFailed(String),
}

/// Cache errors
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Cache entry not found for domain: {0}")]
    NotFound(String),

    #[error("Cache is full (max entries: {0})")]
    Full(usize),

    #[error("TTL expired for domain: {0}")]
    Expired(String),

    #[error("Cache operation failed: {0}")]
    OperationFailed(String),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
}

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid domain format: {0}")]
    InvalidDomain(String),

    #[error("Invalid port: {0} (must be 1-65535)")]
    InvalidPort(u32),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid TTL: {0} (must be positive)")]
    InvalidTtl(String),

    #[error("Configuration parse error: {0}")]
    ParseError(String),

    #[error("Missing required configuration: {0}")]
    MissingConfig(String),

    #[error("Configuration merge error: {0}")]
    MergeError(String),
}

/// DNS discovery errors
#[derive(Debug, Error)]
pub enum DnsError {
    #[error("DNS lookup failed for {0}: {1}")]
    LookupFailed(String, String),

    #[error("Malformed TXT record: {0}")]
    MalformedRecord(String),

    #[error("No cache endpoint found for domain: {0}")]
    NoCacheEndpoint(String),

    #[error("DNS resolver initialization failed: {0}")]
    ResolverFailed(String),
}

/// HTTP client errors
#[derive(Debug, Error)]
pub enum HttpError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Request timeout after {0}s")]
    Timeout(u64),

    #[error("Invalid response from {0}: {1}")]
    InvalidResponse(String, String),

    #[error("HTTPS certificate validation failed: {0}")]
    CertificateError(String),

    #[error("Invalid well-known path: {0}")]
    InvalidWellKnownPath(String),
}

/// Server errors
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Server startup failed: {0}")]
    StartupFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Rate limit exceeded for client: {0}")]
    RateLimitExceeded(String),

    #[error("Upstream fetch failed: {0}")]
    UpstreamFailed(String),
}

/// Client SDK errors
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Discovery failed for {0}: {1}")]
    DiscoveryFailed(String, String),

    #[error("Invalid domain: {0}")]
    InvalidDomain(String),

    #[error("Cache endpoint unreachable: {0}")]
    CacheUnreachable(String),
}

/// OAuth provider errors
#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("Unknown provider: {0}")]
    UnknownProvider(String),

    #[error("Invalid provider configuration: {0}")]
    InvalidConfig(String),

    #[error("Endpoint generation failed for {0}: {1}")]
    EndpointGenerationFailed(String, String),
}

/// Approved AX authentication methods
pub const APPROVED_AUTH_METHODS: &[&str] = &["OIDC", "OAuth2", "mTLS", "JWT", "API_KEY"];

/// Check if an auth method is in the approved set
pub fn is_valid_auth_method(method: &str) -> bool {
    APPROVED_AUTH_METHODS.contains(&method)
}
