//! # Observability Module
//!
//! This module provides comprehensive observability features including:
//!
//! - **Structured Logging**: Context-aware logging with multiple output formats
//! - **Metrics Collection**: Counters, gauges, histograms for performance monitoring
//! - **Tracing Support**: Integration with the tracing ecosystem
//!
//! ## Features
//!
//! - Thread-safe metrics and logging
//! - Prometheus-compatible metrics export
//! - JSON and text log formats
//! - Timer utilities for measuring code execution time
//!
//! ## Example
//!
//! ```no_run
//! use claude_agent_sdk_rs::observability::{Logger, MetricsCollector};
//!
//! let logger = Logger::new("MyAgent");
//! let metrics = MetricsCollector::new();
//!
//! logger.info("Starting agent execution", &[("task_id", "123")]);
//!
//! let _timer = metrics.start_timer("agent_execution", &[("agent", "researcher")]);
//! // ... do work ...
//! // Timer automatically recorded on drop
//! ```

pub mod logger;
pub mod metrics;

// Re-export commonly used types
pub use logger::{
    ConsoleLogObserver, GlobalLogger, LogEntry, LogFormat, LogLevel, LogObserver, Logger,
};
pub use metrics::{
    Histogram, HistogramBuckets, LabeledMetric, MetricKind, MetricStorage, MetricsCollector,
    TimerGuard,
};
