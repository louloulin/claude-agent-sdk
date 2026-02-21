//! # Observability Module
//!
//! This module provides comprehensive observability features including:
//!
//! - **Structured Logging**: Context-aware logging with multiple output formats
//! - **Metrics Collection**: Counters, gauges, histograms for performance monitoring
//! - **Tracing Support**: Integration with the tracing ecosystem
//! - **Request Tracing IDs**: Automatic generation and propagation of trace IDs
//!
//! ## Features
//!
//! - Thread-safe metrics and logging
//! - Prometheus-compatible metrics export
//! - JSON and text log formats
//! - Timer utilities for measuring code execution time
//! - Structured spans for SDK operations
//!
//! ## Example
//!
//! ```no_run
//! use claude_agent_sdk::observability::{Logger, MetricsCollector, init_tracing, TracingConfig};
//!
//! // Initialize tracing at startup
//! init_tracing(TracingConfig::production());
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
pub mod tracing_setup;

// Re-export commonly used types
pub use logger::{
    ConsoleLogObserver, GlobalLogger, LogEntry, LogFormat, LogLevel, LogObserver, Logger,
};
pub use metrics::{
    Histogram, HistogramBuckets, LabeledMetric, MetricKind, MetricStorage, MetricsCollector,
    TimerGuard,
};
pub use tracing_setup::{
    generate_request_id, generate_span_id, init_default, init_tracing, is_initialized, log_counter,
    log_gauge, log_timing, OutputFormat, TracingConfig,
};
