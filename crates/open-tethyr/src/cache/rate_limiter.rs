//! Rate Limiting
//!
//! Token bucket algorithm for rate limiting with per-client IP tracking.

use crate::cache::RateLimitConfig;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use thiserror::Error;

/// Rate limiting errors
#[derive(Debug, Error, Clone)]
pub enum RateLimitError {
    #[error("Rate limit exceeded for client {client_ip}")]
    RateLimitExceeded { client_ip: String },
}

/// Rate limiter with token bucket algorithm
pub struct RateLimiter {
    limits: Arc<RwLock<HashMap<IpAddr, TokenBucket>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limits: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if a request from the given client IP is allowed
    pub fn check_rate_limit(&self, client_ip: IpAddr) -> Result<(), RateLimitError> {
        let mut limits = self.limits.write().unwrap();

        let bucket = limits
            .entry(client_ip)
            .or_insert_with(|| TokenBucket::new(self.config.clone()));

        if bucket.consume(1.0) {
            Ok(())
        } else {
            Err(RateLimitError::RateLimitExceeded {
                client_ip: client_ip.to_string(),
            })
        }
    }

    /// Reset all rate limits
    pub fn reset_limits(&self) {
        if let Ok(mut limits) = self.limits.write() {
            limits.clear();
        }
    }

    /// Get the number of tracked clients
    pub fn tracked_clients(&self) -> usize {
        self.limits.read().map(|l| l.len()).unwrap_or(0)
    }
}

/// Token bucket for rate limiting
pub struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket from rate limit configuration
    pub fn new(config: RateLimitConfig) -> Self {
        // Calculate capacity and refill rate from requests per minute
        let capacity = config.requests_per_minute as f64;
        let refill_rate = capacity / 60.0; // tokens per second

        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Create a new token bucket with custom parameters
    pub fn with_params(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Try to consume tokens from the bucket
    pub fn consume(&mut self, tokens: f64) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = elapsed * self.refill_rate;

        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;
    }

    /// Get current token count
    pub fn tokens(&self) -> f64 {
        self.tokens
    }

    /// Get bucket capacity
    pub fn capacity(&self) -> f64 {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_token_bucket_consume() {
        let mut bucket = TokenBucket::with_params(10.0, 1.0);

        // Should be able to consume up to capacity
        assert!(bucket.consume(5.0));
        assert!(bucket.consume(5.0));

        // Should fail when empty
        assert!(!bucket.consume(1.0));
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::with_params(10.0, 10.0); // 10 tokens per second

        // Consume all tokens
        assert!(bucket.consume(10.0));
        assert!(!bucket.consume(1.0));

        // Wait for refill
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Should have refilled ~2 tokens
        assert!(bucket.consume(1.0));
    }

    #[test]
    fn test_rate_limiter_per_client() {
        let config = RateLimitConfig {
            requests_per_minute: 10,
            requests_per_hour: 600,
        };
        let limiter = RateLimiter::new(config);

        let client1 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let client2 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2));

        // Both clients should be able to make requests
        assert!(limiter.check_rate_limit(client1).is_ok());
        assert!(limiter.check_rate_limit(client2).is_ok());

        // Exhaust client1's limit
        for _ in 0..9 {
            assert!(limiter.check_rate_limit(client1).is_ok());
        }

        // Client1 should be rate limited
        assert!(limiter.check_rate_limit(client1).is_err());

        // Client2 should still be able to make requests
        assert!(limiter.check_rate_limit(client2).is_ok());
    }

    #[test]
    fn test_rate_limiter_reset() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            requests_per_hour: 120,
        };
        let limiter = RateLimiter::new(config);

        let client = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Exhaust limit
        assert!(limiter.check_rate_limit(client).is_ok());
        assert!(limiter.check_rate_limit(client).is_ok());
        assert!(limiter.check_rate_limit(client).is_err());

        // Reset limits
        limiter.reset_limits();

        // Should be able to make requests again
        assert!(limiter.check_rate_limit(client).is_ok());
    }
}
