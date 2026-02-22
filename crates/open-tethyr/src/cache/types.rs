//! Cache Types
//!
//! Common types used in cache implementation.

use crate::ax::AgentExchangeDocument;
use std::time::{Duration, SystemTime};

/// Cache entry with TTL and metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub document: AgentExchangeDocument,
    pub expires_at: SystemTime,
    pub fetched_at: SystemTime,
    pub domain: String,
    pub cache_control: Option<CacheControl>,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(
        document: AgentExchangeDocument,
        domain: String,
        ttl: Duration,
        cache_control: Option<CacheControl>,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            document,
            expires_at: now + ttl,
            fetched_at: now,
            domain,
            cache_control,
        }
    }

    /// Check if the entry has expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    /// Get the remaining TTL
    pub fn remaining_ttl(&self) -> Option<Duration> {
        self.expires_at.duration_since(SystemTime::now()).ok()
    }

    /// Get the age of the entry
    pub fn age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.fetched_at)
            .unwrap_or(Duration::ZERO)
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    /// Default TTL for cached entries
    pub default_ttl: Duration,
    /// Whether to respect Cache-Control headers
    pub respect_cache_control: bool,
    /// Maximum TTL allowed (prevents excessively long caching)
    pub max_ttl: Duration,
    /// Minimum TTL allowed (prevents too frequent refreshes)
    pub min_ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: Duration::from_secs(300), // 5 minutes
            respect_cache_control: true,
            max_ttl: Duration::from_secs(3600), // 1 hour
            min_ttl: Duration::from_secs(60),   // 1 minute
        }
    }
}

/// Cache-Control header information
#[derive(Debug, Clone)]
pub struct CacheControl {
    /// max-age directive in seconds
    pub max_age: Option<u64>,
    /// no-cache directive
    pub no_cache: bool,
    /// no-store directive
    pub no_store: bool,
    /// must-revalidate directive
    pub must_revalidate: bool,
}

impl CacheControl {
    /// Parse Cache-Control header value
    pub fn parse(header_value: &str) -> Self {
        let mut cache_control = Self {
            max_age: None,
            no_cache: false,
            no_store: false,
            must_revalidate: false,
        };

        for directive in header_value.split(',') {
            let directive = directive.trim();
            if directive == "no-cache" {
                cache_control.no_cache = true;
            } else if directive == "no-store" {
                cache_control.no_store = true;
            } else if directive == "must-revalidate" {
                cache_control.must_revalidate = true;
            } else if let Some(max_age_str) = directive.strip_prefix("max-age=") {
                if let Ok(max_age) = max_age_str.parse::<u64>() {
                    cache_control.max_age = Some(max_age);
                }
            }
        }

        cache_control
    }

    /// Check if caching is allowed
    pub fn allows_caching(&self) -> bool {
        // Don't cache if no-cache or no-store directives are present
        if self.no_cache || self.no_store {
            return false;
        }

        // Don't cache if max-age is 0
        if let Some(max_age) = self.max_age {
            if max_age == 0 {
                return false;
            }
        }

        true
    }

    /// Get the effective TTL based on max-age
    pub fn effective_ttl(&self) -> Option<Duration> {
        self.max_age.map(Duration::from_secs)
    }
}

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per minute per client IP
    pub requests_per_minute: u32,
    /// Maximum requests per hour per client IP
    pub requests_per_hour: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 3600,
        }
    }
}
