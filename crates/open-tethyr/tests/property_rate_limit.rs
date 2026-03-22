//! Property 21: Rate Limiting Enforcement
use open_tethyr::cache::RateLimiter;
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn rate_limit_allows_within_capacity() {
    let limiter = RateLimiter::new(10);
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    for _ in 0..10 {
        assert!(limiter.check_rate_limit(ip));
    }
}

#[test]
fn rate_limit_rejects_over_capacity() {
    let limiter = RateLimiter::new(5);
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    for _ in 0..5 {
        limiter.check_rate_limit(ip);
    }
    assert!(!limiter.check_rate_limit(ip));
}

#[test]
fn different_ips_have_separate_limits() {
    let limiter = RateLimiter::new(2);
    let ip1 = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip2 = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    assert!(limiter.check_rate_limit(ip1));
    assert!(limiter.check_rate_limit(ip1));
    assert!(!limiter.check_rate_limit(ip1));
    assert!(limiter.check_rate_limit(ip2)); // separate bucket
}
