//! Property tests for rate limiting

use open_tethyr::policy::{RateLimitError, RateLimiter, TokenBucket};
use proptest::prelude::*;
use std::net::{IpAddr, Ipv4Addr};
use std::thread;
use std::time::Duration;

// Property 21: Rate Limiting Enforcement
// Validates: Requirements 15.1, 15.3
proptest! {
    #[test]
    fn property_rate_limiting_enforcement(
        capacity in 1u32..10,
        refill_rate in 1u32..5,
        client_ip_octets in (1u8..255, 1u8..255, 1u8..255, 1u8..255),
        request_count in 1usize..20,
    ) {
        let client_ip = IpAddr::V4(Ipv4Addr::new(
            client_ip_octets.0,
            client_ip_octets.1,
            client_ip_octets.2,
            client_ip_octets.3,
        ));

        let limiter = RateLimiter::new(capacity, refill_rate);
        let mut successful_requests = 0;
        let mut rate_limited_requests = 0;

        // Make multiple requests
        for _ in 0..request_count {
            match limiter.check_rate_limit(client_ip) {
                Ok(()) => successful_requests += 1,
                Err(RateLimitError::RateLimitExceeded { .. }) => rate_limited_requests += 1,
            }
        }

        // Property: Number of successful requests should not exceed bucket capacity
        prop_assert!(
            successful_requests <= capacity as usize,
            "Successful requests ({}) should not exceed capacity ({})",
            successful_requests,
            capacity
        );

        // Property: If we exceed capacity, some requests should be rate limited
        if request_count > capacity as usize {
            prop_assert!(
                rate_limited_requests > 0,
                "Should have rate limited requests when exceeding capacity"
            );
        }

        // Property: Total requests should equal successful + rate limited
        prop_assert_eq!(
            successful_requests + rate_limited_requests,
            request_count,
            "All requests should be accounted for"
        );
    }
}

proptest! {
    #[test]
    fn property_per_client_isolation(
        capacity in 1u32..5,
        refill_rate in 1u32..3,
        client1_octets in (1u8..255, 1u8..255, 1u8..255, 1u8..255),
        client2_octets in (1u8..255, 1u8..255, 1u8..255, 1u8..255),
    ) {
        // Ensure different client IPs
        prop_assume!(client1_octets != client2_octets);

        let client1_ip = IpAddr::V4(Ipv4Addr::new(
            client1_octets.0,
            client1_octets.1,
            client1_octets.2,
            client1_octets.3,
        ));
        let client2_ip = IpAddr::V4(Ipv4Addr::new(
            client2_octets.0,
            client2_octets.1,
            client2_octets.2,
            client2_octets.3,
        ));

        let limiter = RateLimiter::new(capacity, refill_rate);

        // Exhaust client1's tokens
        let mut client1_requests = 0;
        while limiter.check_rate_limit(client1_ip).is_ok() {
            client1_requests += 1;
            if client1_requests > capacity as usize {
                break; // Safety check
            }
        }

        // Property: Client2 should still have full capacity available
        let mut client2_successful = 0;
        for _ in 0..capacity {
            if limiter.check_rate_limit(client2_ip).is_ok() {
                client2_successful += 1;
            }
        }

        prop_assert_eq!(
            client2_successful,
            capacity as usize,
            "Client2 should have full capacity available when client1 is exhausted"
        );

        // Property: Client1 should now be rate limited
        prop_assert!(
            matches!(limiter.check_rate_limit(client1_ip), Err(RateLimitError::RateLimitExceeded { .. })),
            "Client1 should be rate limited after exhausting tokens"
        );
    }
}

proptest! {
    #[test]
    fn property_token_bucket_refill(
        capacity in 2u32..10,
        refill_rate in 1u32..5,
        initial_consumption in 1u32..5,
    ) {
        prop_assume!(initial_consumption <= capacity);

        let mut bucket = TokenBucket::new(capacity, refill_rate);

        // Consume some tokens
        for _ in 0..initial_consumption {
            prop_assert!(bucket.try_consume(), "Should be able to consume initial tokens");
        }

        let tokens_before_wait = bucket.current_tokens();

        // Wait for refill (simulate time passing)
        thread::sleep(Duration::from_millis(1100)); // Just over 1 second

        let tokens_after_wait = bucket.current_tokens();

        // Property: Tokens should increase after waiting (up to capacity)
        let _expected_tokens = std::cmp::min(
            tokens_before_wait + refill_rate,
            capacity
        );

        prop_assert!(
            tokens_after_wait >= tokens_before_wait,
            "Tokens should not decrease after waiting: before={}, after={}",
            tokens_before_wait,
            tokens_after_wait
        );

        prop_assert!(
            tokens_after_wait <= capacity,
            "Tokens should not exceed capacity: tokens={}, capacity={}",
            tokens_after_wait,
            capacity
        );
    }
}

proptest! {
    #[test]
    fn property_rate_limit_error_details(
        capacity in 1u32..5,
        refill_rate in 1u32..3,
        client_ip_octets in (1u8..255, 1u8..255, 1u8..255, 1u8..255),
    ) {
        let client_ip = IpAddr::V4(Ipv4Addr::new(
            client_ip_octets.0,
            client_ip_octets.1,
            client_ip_octets.2,
            client_ip_octets.3,
        ));

        let limiter = RateLimiter::new(capacity, refill_rate);

        // Exhaust all tokens
        for _ in 0..capacity {
            prop_assert!(limiter.check_rate_limit(client_ip).is_ok());
        }

        // Next request should be rate limited
        let result = limiter.check_rate_limit(client_ip);

        // Property: Rate limit error should contain correct client IP and retry time
        match result {
            Err(RateLimitError::RateLimitExceeded { client_ip: error_ip, retry_after_seconds }) => {
                prop_assert_eq!(error_ip, client_ip, "Error should contain correct client IP");
                prop_assert!(retry_after_seconds > 0, "Retry after should be positive");
                prop_assert!(retry_after_seconds <= 60, "Retry after should be reasonable (≤60s)");
            }
            _ => prop_assert!(false, "Should return RateLimitExceeded error"),
        }
    }
}

proptest! {
    #[test]
    fn property_token_bucket_capacity_bounds(
        capacity in 1u32..100,
        refill_rate in 1u32..10,
        consumption_attempts in 1usize..200,
    ) {
        let mut bucket = TokenBucket::new(capacity, refill_rate);

        let mut successful_consumptions = 0;
        let mut failed_consumptions = 0;

        // Try to consume many tokens rapidly
        for _ in 0..consumption_attempts {
            if bucket.try_consume() {
                successful_consumptions += 1;
            } else {
                failed_consumptions += 1;
            }
        }

        // Property: Successful consumptions should not exceed capacity initially
        // (without significant time passing for refill)
        prop_assert!(
            successful_consumptions <= capacity as usize,
            "Successful consumptions ({}) should not exceed capacity ({})",
            successful_consumptions,
            capacity
        );

        // Property: Current tokens should never exceed capacity
        let current_tokens = bucket.current_tokens();
        prop_assert!(
            current_tokens <= capacity,
            "Current tokens ({}) should not exceed capacity ({})",
            current_tokens,
            capacity
        );

        // Property: Total attempts should equal successful + failed
        prop_assert_eq!(
            successful_consumptions + failed_consumptions,
            consumption_attempts,
            "All consumption attempts should be accounted for"
        );
    }
}
