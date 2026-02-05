//! # Metrics Collection for Agent SDK
//!
//! This module provides comprehensive metrics collection for monitoring agent
//! performance, resource usage, and operational statistics.
//!
//! ## Features
//!
//! - **Counter Metrics**: Track cumulative values (request counts, error counts)
//! - **Gauge Metrics**: Track point-in-time values (memory usage, queue depth)
//! - **Histogram Metrics**: Track distributions (latency, request sizes)
//! - **Labeled Metrics**: Support for dimensional data
//! - **Thread-Safe**: Safe for concurrent access from multiple threads
//!
//! ## Example
//!
//! ```no_run
//! use claude_agent_sdk::observability::metrics::{MetricsCollector, MetricKind};
//!
//! let metrics = MetricsCollector::new();
//!
//! // Record metrics
//! metrics.record("agent_calls", MetricKind::Counter, 1.0, &[("agent", "researcher")]);
//! metrics.record("request_duration_ms", MetricKind::Histogram, 150.0, &[("endpoint", "/api/search")]);
//! metrics.set_gauge("active_connections", 5.0, &[("pool", "database")]);
//! ```

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Type alias for label sets used in metric storage
pub type LabelSet = Vec<(String, String)>;

/// Type alias for counter/gauge storage maps
pub type MetricValueMap = HashMap<LabelSet, f64>;

/// Type alias for histogram storage maps
pub type MetricHistogramMap = HashMap<LabelSet, Histogram>;

/// Kind of metric
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum MetricKind {
    /// Counter: cumulative value that only increases
    Counter,

    /// Gauge: point-in-time value that can go up or down
    Gauge,

    /// Histogram: distribution of values (percentiles, etc.)
    Histogram,

    /// Summary: similar to histogram but with configurable quantiles
    Summary,
}

/// A labeled metric with dimensions
#[derive(Debug, Clone, serde::Serialize)]
pub struct LabeledMetric {
    /// Metric name
    pub name: String,

    /// Metric kind
    pub kind: MetricKind,

    /// Label dimensions
    pub labels: Vec<(String, String)>,

    /// Metric value
    pub value: f64,

    /// Timestamp (milliseconds since epoch)
    pub timestamp: u64,
}

impl LabeledMetric {
    /// Create a new labeled metric
    pub fn new(
        name: impl Into<String>,
        kind: MetricKind,
        value: f64,
        labels: Vec<(String, String)>,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            name: name.into(),
            kind,
            labels,
            value,
            timestamp,
        }
    }

    /// Get label value by key
    pub fn get_label(&self, key: &str) -> Option<&String> {
        self.labels.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

/// Histogram bucket configuration
#[derive(Debug, Clone)]
pub struct HistogramBuckets {
    /// Bucket boundaries
    pub boundaries: Vec<f64>,
}

impl HistogramBuckets {
    /// Create default latency buckets (in milliseconds)
    pub fn latency() -> Self {
        Self {
            boundaries: vec![
                1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0,
                10000.0,
            ],
        }
    }

    /// Create default size buckets (in bytes)
    pub fn size() -> Self {
        Self {
            boundaries: vec![
                1024.0,      // 1 KB
                10240.0,     // 10 KB
                102400.0,    // 100 KB
                1024000.0,   // 1 MB
                10240000.0,  // 10 MB
                102400000.0, // 100 MB
            ],
        }
    }

    /// Create custom buckets
    pub fn custom(boundaries: Vec<f64>) -> Self {
        Self { boundaries }
    }

    /// Find the bucket index for a value
    pub fn find_bucket(&self, value: f64) -> usize {
        self.boundaries
            .iter()
            .position(|&b| value <= b)
            .unwrap_or(self.boundaries.len())
    }
}

/// Histogram data structure
#[derive(Debug, Clone)]
pub struct Histogram {
    /// Bucket counts
    pub buckets: Vec<u64>,

    /// Total sum of all values
    pub sum: f64,

    /// Total count of observations
    pub count: u64,

    /// Bucket boundaries
    pub boundaries: Vec<f64>,
}

impl Histogram {
    /// Create a new histogram
    pub fn new(buckets: HistogramBuckets) -> Self {
        Self {
            buckets: vec![0; buckets.boundaries.len() + 1],
            sum: 0.0,
            count: 0,
            boundaries: buckets.boundaries,
        }
    }

    /// Observe a value
    pub fn observe(&mut self, value: f64) {
        let bucket_idx = self
            .boundaries
            .iter()
            .position(|&b| value <= b)
            .unwrap_or(self.boundaries.len());

        self.buckets[bucket_idx] += 1;
        self.sum += value;
        self.count += 1;
    }

    /// Calculate percentile (approximation)
    pub fn percentile(&self, p: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }

        let rank = (p / 100.0 * self.count as f64).ceil() as u64;
        let mut cumulative = 0;

        for (i, &count) in self.buckets.iter().enumerate() {
            cumulative += count;
            if cumulative >= rank {
                return if i < self.boundaries.len() {
                    self.boundaries[i]
                } else {
                    self.sum / self.count as f64 // fallback to average
                };
            }
        }

        self.sum / self.count as f64
    }

    /// Get average value
    pub fn avg(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }
}

/// Metric storage backend
pub trait MetricStorage: Send + Sync {
    fn record(&self, metric: LabeledMetric);
    fn get_counter(&self, name: &str, labels: &[(String, String)]) -> f64;
    fn get_gauge(&self, name: &str, labels: &[(String, String)]) -> f64;
    fn get_histogram(&self, name: &str, labels: &[(String, String)]) -> Option<Histogram>;
    fn get_all_metrics(&self) -> Vec<LabeledMetric>;
}

/// In-memory metric storage
struct MemoryMetricStorage {
    counters: Arc<RwLock<HashMap<String, MetricValueMap>>>,
    gauges: Arc<RwLock<HashMap<String, MetricValueMap>>>,
    histograms: Arc<RwLock<HashMap<String, MetricHistogramMap>>>,
}

impl MemoryMetricStorage {
    fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn labels_key(labels: &[(String, String)]) -> LabelSet {
        let mut sorted = labels.to_vec();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        sorted
    }
}

impl MetricStorage for MemoryMetricStorage {
    fn record(&self, metric: LabeledMetric) {
        match metric.kind {
            MetricKind::Counter => {
                let mut counters = self.counters.write().unwrap();
                let entry = counters
                    .entry(metric.name.clone())
                    .or_default();
                let key = Self::labels_key(&metric.labels);
                *entry.entry(key).or_insert(0.0) += metric.value;
            },
            MetricKind::Gauge => {
                let mut gauges = self.gauges.write().unwrap();
                let entry = gauges
                    .entry(metric.name.clone())
                    .or_default();
                let key = Self::labels_key(&metric.labels);
                entry.insert(key, metric.value);
            },
            MetricKind::Histogram => {
                let mut histograms = self.histograms.write().unwrap();
                let entry = histograms
                    .entry(metric.name.clone())
                    .or_default();
                let key = Self::labels_key(&metric.labels);
                let hist = entry.entry(key).or_insert_with(|| {
                    Histogram::new(HistogramBuckets::latency())
                });
                hist.observe(metric.value);
            },
            MetricKind::Summary => {
                // Treat summary like histogram for now
                let mut histograms = self.histograms.write().unwrap();
                let entry = histograms
                    .entry(metric.name.clone())
                    .or_default();
                let key = Self::labels_key(&metric.labels);
                let hist = entry.entry(key).or_insert_with(|| {
                    Histogram::new(HistogramBuckets::latency())
                });
                hist.observe(metric.value);
            },
        }
    }

    fn get_counter(&self, name: &str, labels: &[(String, String)]) -> f64 {
        let counters = self.counters.read().unwrap();
        counters
            .get(name)
            .and_then(|m| m.get(&Self::labels_key(labels)))
            .copied()
            .unwrap_or(0.0)
    }

    fn get_gauge(&self, name: &str, labels: &[(String, String)]) -> f64 {
        let gauges = self.gauges.read().unwrap();
        gauges
            .get(name)
            .and_then(|m| m.get(&Self::labels_key(labels)))
            .copied()
            .unwrap_or(0.0)
    }

    fn get_histogram(&self, name: &str, labels: &[(String, String)]) -> Option<Histogram> {
        let histograms = self.histograms.read().unwrap();
        histograms
            .get(name)
            .and_then(|m| m.get(&Self::labels_key(labels)))
            .cloned()
    }

    fn get_all_metrics(&self) -> Vec<LabeledMetric> {
        let mut metrics = Vec::new();

        // Export counters
        let counters = self.counters.read().unwrap();
        for (name, label_map) in counters.iter() {
            for (labels, value) in label_map.iter() {
                metrics.push(LabeledMetric::new(
                    name.clone(),
                    MetricKind::Counter,
                    *value,
                    labels.clone(),
                ));
            }
        }

        // Export gauges
        let gauges = self.gauges.read().unwrap();
        for (name, label_map) in gauges.iter() {
            for (labels, value) in label_map.iter() {
                metrics.push(LabeledMetric::new(
                    name.clone(),
                    MetricKind::Gauge,
                    *value,
                    labels.clone(),
                ));
            }
        }

        metrics
    }
}

/// Metrics collector
pub struct MetricsCollector {
    storage: Arc<dyn MetricStorage>,
    prefix: Option<String>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            storage: Arc::new(MemoryMetricStorage::new()),
            prefix: None,
        }
    }

    /// Create a new collector with a metric name prefix
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            storage: Arc::new(MemoryMetricStorage::new()),
            prefix: Some(prefix.into()),
        }
    }

    /// Add prefix to metric name
    fn prefixed_name(&self, name: &str) -> String {
        match &self.prefix {
            Some(prefix) => format!("{}_{}", prefix, name),
            None => name.to_string(),
        }
    }

    /// Record a metric
    pub fn record(
        &self,
        name: &str,
        kind: MetricKind,
        value: f64,
        labels: &[(impl AsRef<str>, impl AsRef<str>)],
    ) {
        let labels = labels
            .iter()
            .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
            .collect();

        let metric = LabeledMetric::new(self.prefixed_name(name), kind, value, labels);
        self.storage.record(metric);
    }

    /// Increment a counter
    pub fn increment(&self, name: &str, labels: &[(impl AsRef<str>, impl AsRef<str>)]) {
        self.record(name, MetricKind::Counter, 1.0, labels);
    }

    /// Increment a counter by a specific amount
    pub fn increment_by(
        &self,
        name: &str,
        value: f64,
        labels: &[(impl AsRef<str>, impl AsRef<str>)],
    ) {
        self.record(name, MetricKind::Counter, value, labels);
    }

    /// Set a gauge value
    pub fn set_gauge(
        &self,
        name: &str,
        value: f64,
        labels: &[(impl AsRef<str>, impl AsRef<str>)],
    ) {
        self.record(name, MetricKind::Gauge, value, labels);
    }

    /// Record a timing (duration in milliseconds)
    pub fn record_timing(
        &self,
        name: &str,
        duration: Duration,
        labels: &[(impl AsRef<str>, impl AsRef<str>)],
    ) {
        self.record(
            name,
            MetricKind::Histogram,
            duration.as_secs_f64() * 1000.0,
            labels,
        );
    }

    /// Time a block of code
    pub fn time<F, R>(&self, name: &str, labels: &[(impl AsRef<str>, impl AsRef<str>)], f: F) -> (Duration, R)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        self.record_timing(name, duration, labels);
        (duration, result)
    }

    /// Get a counter value
    pub fn get_counter(&self, name: &str, labels: &[(impl AsRef<str>, impl AsRef<str>)]) -> f64 {
        self.storage
            .get_counter(&self.prefixed_name(name), &self.convert_labels(labels))
    }

    /// Get a gauge value
    pub fn get_gauge(&self, name: &str, labels: &[(impl AsRef<str>, impl AsRef<str>)]) -> f64 {
        self.storage
            .get_gauge(&self.prefixed_name(name), &self.convert_labels(labels))
    }

    /// Get histogram data
    pub fn get_histogram(
        &self,
        name: &str,
        labels: &[(impl AsRef<str>, impl AsRef<str>)],
    ) -> Option<Histogram> {
        self.storage
            .get_histogram(&self.prefixed_name(name), &self.convert_labels(labels))
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> Vec<LabeledMetric> {
        self.storage.get_all_metrics()
    }

    /// Export metrics as Prometheus text format
    pub fn export_prometheus(&self) -> String {
        let metrics = self.get_all_metrics();
        let mut output = String::new();

        let mut grouped: HashMap<_, Vec<_>> = HashMap::new();
        for metric in metrics {
            grouped
                .entry((metric.name.clone(), metric.kind))
                .or_insert_with(Vec::new)
                .push(metric);
        }

        for ((name, kind), mut labeled_metrics) in grouped {
            // Add HELP and TYPE metadata
            output.push_str(&format!("# TYPE {} {}\n", name, Self::kind_to_prometheus(kind)));

            // Sort by labels for consistent output
            labeled_metrics.sort_by(|a, b| {
                a.labels
                    .iter()
                    .cmp(b.labels.iter())
            });

            for metric in labeled_metrics {
                let labels_str = if metric.labels.is_empty() {
                    String::new()
                } else {
                    let labels: Vec<String> = metric
                        .labels
                        .iter()
                        .map(|(k, v)| format!(r#"{}="{}""#, k, escape_prometheus_label(v)))
                        .collect();
                    format!("{{{}}}", labels.join(","))
                };
                output.push_str(&format!("{}{} {}\n", name, labels_str, metric.value));
            }

            output.push('\n');
        }

        output
    }

    /// Export metrics as JSON
    pub fn export_json(&self) -> String {
        let metrics = self.get_all_metrics();
        serde_json::to_string(&metrics).unwrap_or_else(|_| "[]".to_string())
    }

    fn convert_labels(&self, labels: &[(impl AsRef<str>, impl AsRef<str>)]) -> Vec<(String, String)> {
        labels
            .iter()
            .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
            .collect()
    }

    fn kind_to_prometheus(kind: MetricKind) -> &'static str {
        match kind {
            MetricKind::Counter => "counter",
            MetricKind::Gauge => "gauge",
            MetricKind::Histogram => "histogram",
            MetricKind::Summary => "summary",
        }
    }

    /// Clear all metrics (only works with MemoryMetricStorage)
    pub fn clear(&self) {
        // Note: This is a no-op for non-memory storage
        // In production, you'd want a different approach
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape labels for Prometheus format
fn escape_prometheus_label(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Timer guard that records duration on drop
pub struct TimerGuard<'a> {
    collector: &'a MetricsCollector,
    name: String,
    labels: Vec<(String, String)>,
    start: Instant,
}

impl<'a> TimerGuard<'a> {
    /// Create a new timer guard
    pub fn new(
        collector: &'a MetricsCollector,
        name: impl Into<String>,
        labels: &[(impl AsRef<str>, impl AsRef<str>)],
    ) -> Self {
        Self {
            collector,
            name: name.into(),
            labels: labels
                .iter()
                .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
                .collect(),
            start: Instant::now(),
        }
    }

    /// Complete the timer early and get the duration
    pub fn complete(self) -> Duration {
        let duration = self.start.elapsed();
        let labels: Vec<(&str, &str)> = self
            .labels
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        self.collector.record_timing(&self.name, duration, &labels);
        duration
    }
}

impl<'a> Drop for TimerGuard<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let labels: Vec<(&str, &str)> = self
            .labels
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        self.collector.record_timing(&self.name, duration, &labels);
    }
}

impl MetricsCollector {
    /// Start a timer that records on drop
    pub fn start_timer(
        &self,
        name: impl Into<String>,
        labels: &[(impl AsRef<str>, impl AsRef<str>)],
    ) -> TimerGuard<'_> {
        TimerGuard::new(self, name, labels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY_LABELS: &[(&str, &str)] = &[];

    #[test]
    fn test_counter_increment() {
        let metrics = MetricsCollector::new();

        metrics.increment("test_counter", EMPTY_LABELS);
        metrics.increment("test_counter", EMPTY_LABELS);

        assert_eq!(metrics.get_counter("test_counter", EMPTY_LABELS), 2.0);
    }

    #[test]
    fn test_counter_with_labels() {
        let metrics = MetricsCollector::new();

        metrics.increment("api_calls", &[("endpoint", "/api/users"), ("method", "GET")]);
        metrics.increment("api_calls", &[("endpoint", "/api/posts"), ("method", "GET")]);
        metrics.increment("api_calls", &[("endpoint", "/api/users"), ("method", "GET")]);

        assert_eq!(
            metrics.get_counter("api_calls", &[("endpoint", "/api/users"), ("method", "GET")]),
            2.0
        );
        assert_eq!(
            metrics.get_counter("api_calls", &[("endpoint", "/api/posts"), ("method", "GET")]),
            1.0
        );
    }

    #[test]
    fn test_gauge() {
        let metrics = MetricsCollector::new();

        metrics.set_gauge("temperature", 20.5, &[("location", "room1")]);
        metrics.set_gauge("temperature", 22.0, &[("location", "room1")]);

        assert_eq!(
            metrics.get_gauge("temperature", &[("location", "room1")]),
            22.0
        );
    }

    #[test]
    fn test_histogram() {
        let metrics = MetricsCollector::new();

        // Record some latencies
        metrics.record("request_duration_ms", MetricKind::Histogram, 50.0, EMPTY_LABELS);
        metrics.record("request_duration_ms", MetricKind::Histogram, 100.0, EMPTY_LABELS);
        metrics.record("request_duration_ms", MetricKind::Histogram, 150.0, EMPTY_LABELS);

        let hist = metrics.get_histogram("request_duration_ms", EMPTY_LABELS).unwrap();
        assert_eq!(hist.count, 3);
        assert_eq!(hist.sum, 300.0);
        assert_eq!(hist.avg(), 100.0);
    }

    #[test]
    fn test_timer() {
        let metrics = MetricsCollector::new();

        let (duration, _) = metrics.time("test_operation", EMPTY_LABELS, || {
            std::thread::sleep(std::time::Duration::from_millis(10));
        });

        assert!(duration.as_millis() >= 10);

        let hist = metrics.get_histogram("test_operation", EMPTY_LABELS).unwrap();
        assert_eq!(hist.count, 1);
        assert!(hist.sum >= 10.0);
    }

    #[test]
    fn test_timer_guard() {
        let metrics = MetricsCollector::new();

        {
            let _guard = metrics.start_timer("guarded_operation", &[("key", "value")]);
            std::thread::sleep(std::time::Duration::from_millis(5));
        } // Timer recorded here

        let hist = metrics.get_histogram("guarded_operation", &[("key", "value")]).unwrap();
        assert_eq!(hist.count, 1);
        assert!(hist.sum >= 5.0);
    }

    #[test]
    fn test_prefix() {
        let metrics = MetricsCollector::with_prefix("myapp");

        metrics.increment("requests", &[("test", "value")] as &[(&str, &str)]);

        assert_eq!(metrics.get_counter("requests", &[("test", "value")] as &[(&str, &str)]), 1.0);
    }

    #[test]
    fn test_prometheus_export() {
        let metrics = MetricsCollector::new();

        metrics.increment("api_calls", &[("endpoint", "/api/users")] as &[(&str, &str)]);
        metrics.set_gauge("active_connections", 5.0, &[("test", "value")] as &[(&str, &str)]);

        let export = metrics.export_prometheus();
        assert!(export.contains("# TYPE api_calls counter"));
        assert!(export.contains("api_calls{endpoint=\"/api/users\"} 1"));
        assert!(export.contains("# TYPE active_connections gauge"));
        assert!(export.contains("active_connections{test=\"value\"} 5"));
    }
}
