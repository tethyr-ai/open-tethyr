//! Rate limiting implementation using token bucket algorithm

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::warn;

/// Rate limiting error types
#[derive(Debug, Error, Clone, PartialEq)]
pub enum RateLimitError {
    #[error(
        "Rate limit exceeded for client {client_ip}. Try again in {retry_after_seconds} seconds"
    )]
    RateLimitExceeded {
        client_ip: IpAddr,
        retry_after_seconds: u64,
    },
}

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
pub struct TokenBucket {
    /// Maximum number of tokens in the bucket
    capacity: u32,
    /// Current number of tokens
    tokens: u32,
    /// Rate at which tokens are refilled (tokens per second)
    refill_rate: u32,
    /// Last time the bucket was refilled
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket with the given capacity and refill rate
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            capacity,
            tokens: capacity, // Start with full bucket
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Try to consume a token from the bucket
    /// Returns true if successful, false if no tokens available
    pub fn try_consume(&mut self) -> bool {
        self.refill();

        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    /// Get the time until the next token will be available
    pub fn time_until_next_token(&self) -> Duration {
        if self.tokens > 0 {
            Duration::from_secs(0)
        } else {
            // Calculate time until next refill based on refill rate
            let time_per_token = 1.0 / self.refill_rate as f64;
            Duration::from_secs_f64(time_per_token)
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);

        if elapsed >= Duration::from_millis(100) {
            // Check every 100ms for more granular refill
            let tokens_to_add = (elapsed.as_secs_f64() * self.refill_rate as f64) as u32;
            if tokens_to_add > 0 {
                self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
                self.last_refill = now;
            }
        }
    }

    /// Get current token count (after refill)
    pub fn current_tokens(&mut self) -> u32 {
        self.refill();
        self.tokens
    }
}

/// Rate limiter with per-client IP tracking
#[derive(Debug)]
pub struct RateLimiter {
    /// Token buckets per client IP
    buckets: Arc<Mutex<HashMap<IpAddr, TokenBucket>>>,
    /// Default bucket capacity
    capacity: u32,
    /// Default refill rate (tokens per second)
    refill_rate: u32,
    /// Cleanup interval for unused buckets
    cleanup_interval: Duration,
    /// Last cleanup time
    last_cleanup: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter with the given capacity and refill rate
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            capacity,
            refill_rate,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Check if a request from the given client IP is allowed
    pub fn check_rate_limit(&self, client_ip: IpAddr) -> Result<(), RateLimitError> {
        // Periodic cleanup of unused buckets
        self.cleanup_if_needed();

        let mut buckets = self.buckets.lock().unwrap();

        // Get or create bucket for this client
        let bucket = buckets
            .entry(client_ip)
            .or_insert_with(|| TokenBucket::new(self.capacity, self.refill_rate));

        if bucket.try_consume() {
            Ok(())
        } else {
            // Get retry time before any potential refill
            let retry_after = bucket.time_until_next_token();
            // Ensure we always return at least 1 second for rate limiting
            let retry_after_seconds = retry_after.as_secs().max(1);

            warn!(
                client_ip = %client_ip,
                retry_after_seconds = retry_after_seconds,
                "Rate limit exceeded"
            );

            Err(RateLimitError::RateLimitExceeded {
                client_ip,
                retry_after_seconds,
            })
        }
    }

    /// Get current token count for a client IP
    pub fn get_current_tokens(&self, client_ip: IpAddr) -> u32 {
        let mut buckets = self.buckets.lock().unwrap();
        buckets
            .get_mut(&client_ip)
            .map(|bucket| bucket.current_tokens())
            .unwrap_or(self.capacity)
    }

    /// Clean up unused buckets periodically
    fn cleanup_if_needed(&self) {
        let mut last_cleanup = self.last_cleanup.lock().unwrap();
        let now = Instant::now();

        if now.duration_since(*last_cleanup) >= self.cleanup_interval {
            let mut buckets = self.buckets.lock().unwrap();

            // Remove buckets that are full (haven't been used recently)
            buckets.retain(|_, bucket| {
                let mut bucket_clone = bucket.clone();
                bucket_clone.current_tokens() < self.capacity
            });

            *last_cleanup = now;
        }
    }

    /// Get the number of tracked client IPs
    pub fn tracked_clients(&self) -> usize {
        self.buckets.lock().unwrap().len()
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            buckets: Arc::clone(&self.buckets),
            capacity: self.capacity,
            refill_rate: self.refill_rate,
            cleanup_interval: self.cleanup_interval,
            last_cleanup: Arc::clone(&self.last_cleanup),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use std::thread;

    #[test]
    fn test_token_bucket_basic_consumption() {
        let mut bucket = TokenBucket::new(5, 1);

        // Should be able to consume all tokens
        for _ in 0..5 {
            assert!(bucket.try_consume());
        }

        // Should fail to consume when empty
        assert!(!bucket.try_consume());
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(2, 2); // 2 tokens, refill 2 per second

        // Consume all tokens
        assert!(bucket.try_consume());
        assert!(bucket.try_consume());
        assert!(!bucket.try_consume());

        // Wait for refill (simulate time passing)
        thread::sleep(Duration::from_millis(1100));

        // Should have refilled
        assert!(bucket.try_consume());
        assert!(bucket.try_consume());
    }

    #[test]
    fn test_rate_limiter_per_client() {
        let limiter = RateLimiter::new(2, 1);
        let client1 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let client2 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2));

        // Each client should have their own bucket
        assert!(limiter.check_rate_limit(client1).is_ok());
        assert!(limiter.check_rate_limit(client1).is_ok());
        assert!(limiter.check_rate_limit(client2).is_ok());
        assert!(limiter.check_rate_limit(client2).is_ok());

        // Both clients should be rate limited now
        assert!(limiter.check_rate_limit(client1).is_err());
        assert!(limiter.check_rate_limit(client2).is_err());
    }

    #[test]
    fn test_rate_limit_error_details() {
        let limiter = RateLimiter::new(1, 1);
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Consume the token
        assert!(limiter.check_rate_limit(client_ip).is_ok());

        // Next request should be rate limited
        let result = limiter.check_rate_limit(client_ip);
        assert!(result.is_err());

        if let Err(RateLimitError::RateLimitExceeded {
            client_ip: ip,
            retry_after_seconds,
        }) = result
        {
            assert_eq!(ip, client_ip);
            assert!(retry_after_seconds > 0);
        } else {
            panic!("Expected RateLimitExceeded error");
        }
    }

    #[test]
    fn test_current_tokens() {
        let limiter = RateLimiter::new(3, 1);
        let client_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Should start with full bucket
        assert_eq!(limiter.get_current_tokens(client_ip), 3);

        // Consume one token
        assert!(limiter.check_rate_limit(client_ip).is_ok());
        assert_eq!(limiter.get_current_tokens(client_ip), 2);
    }
}
