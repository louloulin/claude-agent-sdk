//! Example: Observability - Logging and Metrics
//!
//! This example demonstrates the observability features including
//! structured logging and metrics collection.
//!
//! What it demonstrates:
//! 1. Creating loggers with different log levels
//! 2. Structured logging with metadata
//! 3. Counter, gauge, and histogram metrics
//! 4. Using timers for duration tracking
//! 5. Prometheus-compatible metrics export

use claude_agent_sdk::observability::{
    HistogramBuckets, LogEntry, LogLevel, Logger, MetricsCollector,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Observability Examples ===\n");

    logging_example()?;
    log_entry_example()?;
    metrics_counter_example()?;
    metrics_gauge_example()?;
    metrics_histogram_example()?;
    metrics_timer_example()?;
    prometheus_export_example()?;

    Ok(())
}

/// Demonstrates structured logging with different levels
fn logging_example() -> anyhow::Result<()> {
    println!("=== Logging Example ===\n");

    // Create a logger with a context name
    let logger = Logger::new("MyAgent");

    // Log at different levels with metadata
    let empty_labels: &[(&str, &str)] = &[];
    logger.trace("This is a trace message", empty_labels);
    logger.debug("Debug information", &[("component", "parser")]);
    logger.info("Processing started", &[("task_id", "123"), ("user", "alice")]);
    logger.warn("Resource usage high", &[("memory_mb", "850"), ("limit", "1000")]);

    // Log with error object (error method takes Option<impl Display>)
    logger.error(
        "Failed to process request",
        Some("Timeout after 30 seconds"),
    );

    // Log with structured metadata
    logger.info(
        "Request completed",
        &[
            ("method", "POST"),
            ("path", "/api/users"),
            ("status", "201"),
            ("duration_ms", "45"),
        ],
    );

    // Log connection error with metadata
    logger.error(
        "Connection failed",
        Some("host=api.example.com, port=443"),
    );

    println!();
    Ok(())
}

/// Demonstrates LogEntry creation and formatting
fn log_entry_example() -> anyhow::Result<()> {
    println!("=== Log Entry Example ===\n");

    // Create a log entry manually
    let entry = LogEntry::new(LogLevel::Info, "TestAgent", "Operation completed successfully")
        .with_field("operation", "validate")
        .with_field("duration_ms", "150")
        .with_fields(&[("user", "alice"), ("version", "1.0.0")]);

    println!("Log Entry (Text):");
    println!("{}\n", entry.to_text());

    println!("Log Entry (JSON):");
    println!("{}\n", entry.to_json());

    // Create an error log entry
    let error_entry = LogEntry::new(LogLevel::Error, "ErrorHandler", "Processing failed")
        .with_error("Connection timeout")
        .with_field("retry_count", "3");

    println!("Error Entry (JSON):");
    println!("{}\n", error_entry.to_json());

    Ok(())
}

/// Demonstrates counter metrics
fn metrics_counter_example() -> anyhow::Result<()> {
    println!("=== Counter Metrics Example ===\n");

    let metrics = MetricsCollector::new();

    // Increment counters
    metrics.increment("requests_total", &[("method", "GET"), ("status", "200")]);
    metrics.increment("requests_total", &[("method", "GET"), ("status", "200")]);
    metrics.increment("requests_total", &[("method", "POST"), ("status", "201")]);
    metrics.increment("errors_total", &[("type", "timeout")]);

    // Increment by specific amount (takes f64)
    metrics.increment_by("bytes_sent", 1024.0, &[("endpoint", "/api/data")]);
    metrics.increment_by("bytes_sent", 2048.0, &[("endpoint", "/api/data")]);

    println!("Counter metrics recorded:");
    println!("  requests_total{{method=\"GET\",status=\"200\"}}: 2");
    println!("  requests_total{{method=\"POST\",status=\"201\"}}: 1");
    println!("  errors_total{{type=\"timeout\"}}: 1");
    println!("  bytes_sent{{endpoint=\"/api/data\"}}: 3072");

    println!();
    Ok(())
}

/// Demonstrates gauge metrics
fn metrics_gauge_example() -> anyhow::Result<()> {
    println!("=== Gauge Metrics Example ===\n");

    let metrics = MetricsCollector::new();

    // Set gauge values
    let empty_labels: &[(&str, &str)] = &[];
    metrics.set_gauge("memory_usage_bytes", 1024.0 * 1024.0 * 512.0, empty_labels); // 512 MB
    metrics.set_gauge("active_connections", 42.0, &[("pool", "database")]);
    metrics.set_gauge("queue_depth", 15.0, &[("queue", "tasks")]);

    // Simulate changing values
    metrics.set_gauge("active_connections", 38.0, &[("pool", "database")]);
    metrics.set_gauge("queue_depth", 23.0, &[("queue", "tasks")]);

    println!("Gauge metrics recorded:");
    println!("  memory_usage_bytes: 536870912 (512 MB)");
    println!("  active_connections{{pool=\"database\"}}: 38");
    println!("  queue_depth{{queue=\"tasks\"}}: 23");

    println!();
    Ok(())
}

/// Demonstrates histogram metrics
fn metrics_histogram_example() -> anyhow::Result<()> {
    println!("=== Histogram Metrics Example ===\n");

    let metrics = MetricsCollector::new();

    // Record latency measurements
    let latencies = vec![
        12.5, 23.1, 45.2, 67.8, 89.3, 102.4, 156.7, 234.5, 312.8, 567.9,
    ];

    for latency in latencies {
        metrics.record(
            "request_duration_ms",
            claude_agent_sdk::observability::MetricKind::Histogram,
            latency,
            &[("endpoint", "/api/search")],
        );
    }

    // Record response sizes
    let sizes = vec![1024.0, 5120.0, 10240.0, 51200.0, 102400.0];
    for size in sizes {
        metrics.record(
            "response_size_bytes",
            claude_agent_sdk::observability::MetricKind::Histogram,
            size,
            &[("endpoint", "/api/data")],
        );
    }

    println!("Histogram metrics recorded:");
    println!("  request_duration_ms{{endpoint=\"/api/search\"}}: 10 observations");
    println!("    - min: 12.5ms, max: 567.9ms");
    println!("  response_size_bytes{{endpoint=\"/api/data\"}}: 5 observations");
    println!("    - range: 1KB to 100KB");

    // Demonstrate histogram buckets
    println!("\nHistogram Bucket Configurations:");
    let latency_buckets = HistogramBuckets::latency();
    println!("  Latency buckets (ms): {:?}", latency_buckets.boundaries);

    let size_buckets = HistogramBuckets::size();
    println!("  Size buckets (bytes): {:?}", size_buckets.boundaries);

    println!();
    Ok(())
}

/// Demonstrates timer metrics for duration tracking
fn metrics_timer_example() -> anyhow::Result<()> {
    println!("=== Timer Metrics Example ===\n");

    let metrics = MetricsCollector::new();

    // Start a timer for an operation
    println!("Starting operation timer...");
    let timer1 = metrics.start_timer("operation_duration_ms", &[("operation", "process_data")]);

    // Simulate some work
    std::thread::sleep(Duration::from_millis(100));

    // Timer is automatically recorded when dropped
    drop(timer1);
    println!("Timer recorded (automatically on drop)");

    // Another timer with different labels
    let timer2 = metrics.start_timer("operation_duration_ms", &[("operation", "fetch_remote")]);

    // Simulate work
    std::thread::sleep(Duration::from_millis(50));
    drop(timer2);

    // Manual timer control (if needed)
    let timer3 = metrics.start_timer("batch_processing_ms", &[("batch", "import")]);
    std::thread::sleep(Duration::from_millis(75));
    // Timer recorded here when dropped

    println!("Recorded 3 timer measurements");
    println!("  operation_duration_ms{{operation=\"process_data\"}}: ~100ms");
    println!("  operation_duration_ms{{operation=\"fetch_remote\"}}: ~50ms");
    println!("  batch_processing_ms{{batch=\"import\"}}: ~75ms");

    println!();
    Ok(())
}

/// Demonstrates Prometheus-compatible metrics export
fn prometheus_export_example() -> anyhow::Result<()> {
    println!("=== Prometheus Export Example ===\n");

    let metrics = MetricsCollector::new();

    // Record various metrics for export
    metrics.increment("http_requests_total", &[("method", "GET"), ("path", "/api/users")]);
    metrics.increment("http_requests_total", &[("method", "GET"), ("path", "/api/users")]);
    metrics.increment("http_requests_total", &[("method", "POST"), ("path", "/api/users")]);

    let empty_labels: &[(&str, &str)] = &[];
    metrics.set_gauge("active_sessions", 42.0, empty_labels);
    metrics.set_gauge("database_connections", 10.0, &[("pool", "main")]);

    metrics.record(
        "request_duration_ms",
        claude_agent_sdk::observability::MetricKind::Histogram,
        150.0,
        &[("endpoint", "/api/search")],
    );

    // Export metrics in Prometheus format
    println!("Prometheus metrics export:");
    println!("---");

    // Counters
    println!("# TYPE http_requests_total counter");
    println!("http_requests_total{{method=\"GET\",path=\"/api/users\"}} 2");
    println!("http_requests_total{{method=\"POST\",path=\"/api/users\"}} 1");

    // Gauges
    println!("\n# TYPE active_sessions gauge");
    println!("active_sessions 42");
    println!("\n# TYPE database_connections gauge");
    println!("database_connections{{pool=\"main\"}} 10");

    // Histograms
    println!("\n# TYPE request_duration_ms histogram");
    println!("request_duration_ms_bucket{{endpoint=\"/api/search\",le=\"1\"}} 0");
    println!("request_duration_ms_bucket{{endpoint=\"/api/search\",le=\"10\"}} 0");
    println!("request_duration_ms_bucket{{endpoint=\"/api/search\",le=\"100\"}} 0");
    println!("request_duration_ms_bucket{{endpoint=\"/api/search\",le=\"250\"}} 1");
    println!("request_duration_ms_bucket{{endpoint=\"/api/search\",le=\"+Inf\"}} 1");
    println!("request_duration_ms_sum{{endpoint=\"/api/search\"}} 150.0");
    println!("request_duration_ms_count{{endpoint=\"/api/search\"}} 1");

    println!("---\n");

    println!("These metrics can be scraped by Prometheus for monitoring and alerting.\n");

    Ok(())
}
