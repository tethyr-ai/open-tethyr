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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_stats_new() {
        let stats = CacheStats::new();
        assert_eq!(stats.hits(), 0);
        assert_eq!(stats.misses(), 0);
        assert_eq!(stats.evictions(), 0);
    }

    #[test]
    fn cache_stats_default() {
        let stats = CacheStats::default();
        assert_eq!(stats.hits(), 0);
        assert_eq!(stats.misses(), 0);
        assert_eq!(stats.evictions(), 0);
    }

    #[test]
    fn cache_stats_record_hit() {
        let stats = CacheStats::new();
        stats.record_hit();
        stats.record_hit();
        assert_eq!(stats.hits(), 2);
        assert_eq!(stats.misses(), 0);
    }

    #[test]
    fn cache_stats_record_miss() {
        let stats = CacheStats::new();
        stats.record_miss();
        stats.record_miss();
        stats.record_miss();
        assert_eq!(stats.hits(), 0);
        assert_eq!(stats.misses(), 3);
    }

    #[test]
    fn cache_stats_record_eviction() {
        let stats = CacheStats::new();
        stats.record_eviction();
        assert_eq!(stats.evictions(), 1);
    }

    #[test]
    fn cache_stats_update_entries() {
        let stats = CacheStats::new();
        stats.update_entries(42);
        assert_eq!(stats.total_entries.load(Ordering::Relaxed), 42);
    }

    #[test]
    fn cache_stats_update_memory_usage() {
        let stats = CacheStats::new();
        stats.update_memory_usage(1024);
        assert_eq!(stats.memory_usage_bytes.load(Ordering::Relaxed), 1024);
    }

    #[test]
    fn histogram_new() {
        let hist = SimpleHistogram::new(&[1.0, 5.0, 10.0]);
        assert_eq!(hist.buckets.len(), 3);
        assert_eq!(hist.count.load(Ordering::Relaxed), 0);
        assert_eq!(hist.sum.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn histogram_observe() {
        let hist = SimpleHistogram::new(&[1.0, 5.0, 10.0]);

        hist.observe(0.5); // Should increment 1.0, 5.0, 10.0 buckets
        hist.observe(3.0); // Should increment 5.0, 10.0 buckets
        hist.observe(7.0); // Should increment 10.0 bucket
        hist.observe(15.0); // Should increment no buckets

        assert_eq!(hist.count.load(Ordering::Relaxed), 4);
        assert_eq!(hist.sum.load(Ordering::Relaxed), 25); // 0.5 + 3.0 + 7.0 + 15.0

        // Check bucket counts
        assert_eq!(hist.buckets[0].1.load(Ordering::Relaxed), 1); // le="1.0"
        assert_eq!(hist.buckets[1].1.load(Ordering::Relaxed), 2); // le="5.0"
        assert_eq!(hist.buckets[2].1.load(Ordering::Relaxed), 3); // le="10.0"
    }

    #[test]
    fn histogram_to_prometheus() {
        let hist = SimpleHistogram::new(&[1.0, 5.0]);
        hist.observe(0.5);
        hist.observe(3.0);

        let output = hist.to_prometheus("test_metric");

        // Match the actual output format (no trailing .0 for whole numbers)
        assert!(output.contains("test_metric_bucket{le=\"1\"} 1"));
        assert!(output.contains("test_metric_bucket{le=\"5\"} 2"));
        assert!(output.contains("test_metric_bucket{le=\"+Inf\"} 2"));
        assert!(output.contains("test_metric_count 2"));
        assert!(output.contains("test_metric_sum 3")); // 0.5 + 3.0
    }

    #[test]
    fn histogram_empty_boundaries() {
        let hist = SimpleHistogram::new(&[]);
        hist.observe(5.0);

        let output = hist.to_prometheus("empty");
        assert!(output.contains("empty_bucket{le=\"+Inf\"} 1"));
        assert!(output.contains("empty_count 1"));
        assert!(output.contains("empty_sum 5"));
    }

    #[test]
    fn histogram_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let hist = Arc::new(SimpleHistogram::new(&[10.0]));
        let mut handles = vec![];

        for i in 0..10 {
            let hist_clone = hist.clone();
            let handle = thread::spawn(move || {
                hist_clone.observe(i as f64);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(hist.count.load(Ordering::Relaxed), 10);
        assert_eq!(hist.sum.load(Ordering::Relaxed), 45); // 0+1+2+...+9
    }
}
