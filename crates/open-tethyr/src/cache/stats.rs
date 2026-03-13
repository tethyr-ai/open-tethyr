//! Cache Statistics

use std::sync::atomic::{AtomicU64, Ordering};

/// Cache statistics with atomic counters
pub struct CacheStats {
    pub hit_count: AtomicU64,
    pub miss_count: AtomicU64,
    pub eviction_count: AtomicU64,
    pub total_entries: AtomicU64,
    pub memory_usage_bytes: AtomicU64,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
            eviction_count: AtomicU64::new(0),
            total_entries: AtomicU64::new(0),
            memory_usage_bytes: AtomicU64::new(0),
        }
    }

    pub fn record_hit(&self) {
        self.hit_count.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_miss(&self) {
        self.miss_count.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_eviction(&self) {
        self.eviction_count.fetch_add(1, Ordering::Relaxed);
    }
    pub fn update_entries(&self, count: u64) {
        self.total_entries.store(count, Ordering::Relaxed);
    }
    pub fn update_memory_usage(&self, bytes: u64) {
        self.memory_usage_bytes.store(bytes, Ordering::Relaxed);
    }

    pub fn hits(&self) -> u64 {
        self.hit_count.load(Ordering::Relaxed)
    }
    pub fn misses(&self) -> u64 {
        self.miss_count.load(Ordering::Relaxed)
    }
    pub fn evictions(&self) -> u64 {
        self.eviction_count.load(Ordering::Relaxed)
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple histogram for request duration tracking
pub struct SimpleHistogram {
    buckets: Vec<(f64, AtomicU64)>,
    count: AtomicU64,
    sum: AtomicU64,
}

impl SimpleHistogram {
    pub fn new(boundaries: &[f64]) -> Self {
        let buckets = boundaries.iter().map(|&b| (b, AtomicU64::new(0))).collect();
        Self {
            buckets,
            count: AtomicU64::new(0),
            sum: AtomicU64::new(0),
        }
    }

    pub fn observe(&self, value_ms: f64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(value_ms as u64, Ordering::Relaxed);
        for (boundary, counter) in &self.buckets {
            if value_ms <= *boundary {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn to_prometheus(&self, name: &str) -> String {
        let mut out = String::new();
        for (boundary, counter) in &self.buckets {
            out.push_str(&format!(
                "{}_bucket{{le=\"{}\"}} {}\n",
                name,
                boundary,
                counter.load(Ordering::Relaxed)
            ));
        }
        out.push_str(&format!(
            "{}_bucket{{le=\"+Inf\"}} {}\n",
            name,
            self.count.load(Ordering::Relaxed)
        ));
        out.push_str(&format!(
            "{}_count {}\n",
            name,
            self.count.load(Ordering::Relaxed)
        ));
        out.push_str(&format!(
            "{}_sum {}\n",
            name,
            self.sum.load(Ordering::Relaxed)
        ));
        out
    }
}
