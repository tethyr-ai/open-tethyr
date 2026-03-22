//! Rate Limiting with Token Bucket

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Token bucket for rate limiting
pub struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub fn consume(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = Instant::now();
    }
}

/// Per-client IP rate limiter
pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<IpAddr, TokenBucket>>>,
    capacity: f64,
    refill_rate: f64,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        let capacity = requests_per_minute as f64;
        let refill_rate = capacity / 60.0;
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            capacity,
            refill_rate,
        }
    }

    pub fn check_rate_limit(&self, client_ip: IpAddr) -> bool {
        let mut buckets = match self.buckets.write() {
            Ok(b) => b,
            Err(_) => return false,
        };
        let bucket = buckets
            .entry(client_ip)
            .or_insert_with(|| TokenBucket::new(self.capacity, self.refill_rate));
        bucket.consume()
    }

    pub fn reset_limits(&self) {
        if let Ok(mut buckets) = self.buckets.write() {
            buckets.clear();
        }
    }
}
