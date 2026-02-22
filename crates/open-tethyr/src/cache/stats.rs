//! Cache Statistics
//!
//! Statistics tracking for cache operations with atomic counters.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Cache statistics with atomic counters
#[derive(Debug)]
pub struct CacheStats {
    /// Total number of entries currently in cache
    pub total_entries: AtomicUsize,
    /// Total cache hits
    pub hit_count: AtomicU64,
    /// Total cache misses
    pub miss_count: AtomicU64,
    /// Total evictions performed
    pub eviction_count: AtomicU64,
    /// Estimated memory usage in bytes
    pub memory_usage_bytes: AtomicUsize,
}

impl CacheStats {
    /// Create new cache statistics
    pub fn new() -> Self {
        Self {
            total_entries: AtomicUsize::new(0),
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
            eviction_count: AtomicU64::new(0),
            memory_usage_bytes: AtomicUsize::new(0),
        }
    }

    /// Record a cache hit
    pub fn record_hit(&self) {
        self.hit_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_miss(&self) {
        self.miss_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an eviction
    pub fn record_eviction(&self) {
        self.eviction_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Update memory usage
    pub fn update_memory_usage(&self, bytes: usize) {
        self.memory_usage_bytes.store(bytes, Ordering::Relaxed);
    }

    /// Update entry count
    pub fn update_entry_count(&self, count: usize) {
        self.total_entries.store(count, Ordering::Relaxed);
    }

    /// Get hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let misses = self.miss_count.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            (hits as f64 / total as f64) * 100.0
        }
    }

    /// Get current statistics snapshot
    pub fn snapshot(&self) -> CacheStatsSnapshot {
        CacheStatsSnapshot {
            total_entries: self.total_entries.load(Ordering::Relaxed),
            hit_count: self.hit_count.load(Ordering::Relaxed),
            miss_count: self.miss_count.load(Ordering::Relaxed),
            eviction_count: self.eviction_count.load(Ordering::Relaxed),
            memory_usage_bytes: self.memory_usage_bytes.load(Ordering::Relaxed),
        }
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of cache statistics at a point in time
#[derive(Debug, Clone)]
pub struct CacheStatsSnapshot {
    pub total_entries: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub memory_usage_bytes: usize,
}

/// Simple histogram for request duration tracking
#[derive(Debug)]
pub struct SimpleHistogram {
    pub buckets: Vec<(Duration, u64)>, // (upper_bound, count)
    pub total_count: u64,
    pub sum: Duration,
}

impl SimpleHistogram {
    /// Create a new histogram with predefined buckets
    pub fn new() -> Self {
        Self {
            buckets: vec![
                (Duration::from_millis(1), 0),
                (Duration::from_millis(5), 0),
                (Duration::from_millis(10), 0),
                (Duration::from_millis(50), 0),
                (Duration::from_millis(100), 0),
                (Duration::from_millis(500), 0),
                (Duration::from_secs(1), 0),
                (Duration::from_secs(5), 0),
            ],
            total_count: 0,
            sum: Duration::ZERO,
        }
    }

    /// Record a duration measurement
    pub fn record(&mut self, duration: Duration) {
        self.total_count += 1;
        self.sum += duration;

        for (upper_bound, count) in &mut self.buckets {
            if duration <= *upper_bound {
                *count += 1;
                break;
            }
        }
    }

    /// Get the total count of recorded measurements
    pub fn total_count(&self) -> u64 {
        self.total_count
    }

    /// Get the sum of all recorded durations
    pub fn sum(&self) -> Duration {
        self.sum
    }

    /// Get the average duration
    pub fn average(&self) -> Duration {
        if self.total_count == 0 {
            Duration::ZERO
        } else {
            self.sum / self.total_count as u32
        }
    }

    /// Get bucket counts
    pub fn buckets(&self) -> &[(Duration, u64)] {
        &self.buckets
    }
}

impl Default for SimpleHistogram {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe histogram wrapper
#[derive(Debug)]
pub struct ThreadSafeHistogram {
    inner: Arc<Mutex<SimpleHistogram>>,
}

impl ThreadSafeHistogram {
    /// Create a new thread-safe histogram
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SimpleHistogram::new())),
        }
    }

    /// Record a duration measurement
    pub fn record(&self, duration: Duration) {
        if let Ok(mut histogram) = self.inner.lock() {
            histogram.record(duration);
        }
    }

    /// Get a snapshot of the histogram
    pub fn snapshot(&self) -> Option<SimpleHistogram> {
        self.inner.lock().ok().map(|h| SimpleHistogram {
            buckets: h.buckets.clone(),
            total_count: h.total_count,
            sum: h.sum,
        })
    }
}

impl Default for ThreadSafeHistogram {
    fn default() -> Self {
        Self::new()
    }
}
