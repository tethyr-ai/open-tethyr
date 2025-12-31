//! Policy enforcement module
//!
//! This module provides policy enforcement capabilities including:
//! - Domain locking policies
//! - Allowlist configuration for external domains
//! - HTTP error responses for policy violations
//! - Audit logging for all discovery requests

pub mod enforcement;
pub mod rate_limit;

pub use enforcement::{DomainPolicy, PolicyEngine, PolicyViolation};
pub use rate_limit::{RateLimitError, RateLimiter, TokenBucket};
