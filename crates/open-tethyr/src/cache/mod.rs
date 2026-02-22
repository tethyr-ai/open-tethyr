//! Cache Implementation
//!
//! This module provides in-memory caching, hierarchical cache coordination,
//! rate limiting, and cache statistics.

mod coordinator;
mod memory;
mod rate_limiter;
mod stats;
mod types;

pub use coordinator::{CacheCoordinator, CacheCoordinatorConfig, CacheError};
pub use memory::MemoryCache;
pub use rate_limiter::{RateLimitError, RateLimiter, TokenBucket};
pub use stats::{CacheStats, CacheStatsSnapshot, SimpleHistogram, ThreadSafeHistogram};
pub use types::{CacheConfig, CacheControl, CacheEntry, RateLimitConfig};
