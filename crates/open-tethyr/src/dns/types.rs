//! DNS Types
//!
//! Error types and other DNS-related types.

/// DNS discovery errors
#[derive(Debug, thiserror::Error)]
pub enum DnsError {
    #[error("DNS lookup failed: {0}")]
    LookupFailed(String),

    #[error("Invalid TXT record format: {0}")]
    InvalidFormat(String),
}
