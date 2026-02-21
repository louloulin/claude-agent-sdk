//! # Tracing Setup for Agent SDK
//!
//! This module provides structured tracing setup with spans, events, and
//! request tracing IDs for comprehensive observability.
//!
//! ## Features
//!
//! - **Subscriber Setup**: Easy initialization of tracing subscribers
//! - **Structured Spans**: Pre-configured spans for SDK operations
//! - **Request Tracing IDs**: Automatic generation and propagation of trace IDs
//! - **Error Category Integration**: Structured logging with error categories
//! - **Multiple Output Formats**: JSON, text, and compact formats
//!
//! ## Example
//!
//! ```no_run
//! use claude_agent_sdk::observability::tracing_setup::{
//!     init_tracing, TracingConfig, OutputFormat,
//! };
//!
//! // Initialize tracing at application startup
//! let config = TracingConfig {
//!     level: "info".to_string(),
//!     format: OutputFormat::Json,
//!     ..Default::default()
//! };
//! init_tracing(config);
//!
//! // Now all SDK operations will produce structured logs
//! ```

use std::sync::OnceLock;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Output format for tracing
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    /// Human-readable text format
    #[default]
    Text,
    /// JSON format (for structured logging)
    Json,
    /// Compact single-line format
    Compact,
}

/// Configuration for tracing setup
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Minimum log level (trace, debug, info, warn, error)
    pub level: String,
    /// Output format
    pub format: OutputFormat,
    /// Include thread IDs in logs
    pub with_thread_ids: bool,
    /// Include thread names in logs
    pub with_thread_names: bool,
    /// Include target in logs
    pub with_target: bool,
    /// Include file and line information
    pub with_file: bool,
    /// Include line number
    pub with_line_number: bool,
    /// Span events to include (creation, enter, exit, close)
    pub span_events: FmtSpan,
    /// Use ANSI colors (for text format)
    pub ansi: bool,
    /// Custom environment filter override
    pub env_filter_override: Option<String>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: OutputFormat::default(),
            with_thread_ids: false,
            with_thread_names: false,
            with_target: true,
            with_file: false,
            with_line_number: false,
            span_events: FmtSpan::NONE,
            ansi: true,
            env_filter_override: None,
        }
    }
}

impl TracingConfig {
    /// Create a config for production use (JSON, info level)
    pub fn production() -> Self {
        Self {
            level: "info".to_string(),
            format: OutputFormat::Json,
            with_thread_ids: true,
            with_thread_names: true,
            with_target: true,
            with_file: false,
            with_line_number: false,
            span_events: FmtSpan::CLOSE,
            ansi: false,
            env_filter_override: None,
        }
    }

    /// Create a config for development (text, debug level, colors)
    pub fn development() -> Self {
        Self {
            level: "debug".to_string(),
            format: OutputFormat::Text,
            with_thread_ids: false,
            with_thread_names: false,
            with_target: true,
            with_file: true,
            with_line_number: true,
            span_events: FmtSpan::NEW | FmtSpan::CLOSE,
            ansi: true,
            env_filter_override: None,
        }
    }

    /// Create a config for testing (compact, trace level)
    pub fn testing() -> Self {
        Self {
            level: "trace".to_string(),
            format: OutputFormat::Compact,
            with_thread_ids: false,
            with_thread_names: false,
            with_target: false,
            with_file: false,
            with_line_number: false,
            span_events: FmtSpan::NONE,
            ansi: false,
            env_filter_override: None,
        }
    }
}

static TRACING_INITIALIZED: OnceLock<bool> = OnceLock::new();

/// Initialize the tracing subscriber with the given configuration.
///
/// This should be called once at application startup. Subsequent calls
/// will be ignored (no-op).
///
/// # Arguments
///
/// * `config` - Configuration for the tracing setup
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk::observability::tracing_setup::{
///     init_tracing, TracingConfig, OutputFormat,
/// };
///
/// let config = TracingConfig {
///     level: "info".into(),
///     format: OutputFormat::Json,
///     ..Default::default()
/// };
/// init_tracing(config);
/// ```
pub fn init_tracing(config: TracingConfig) {
    // Only initialize once
    if TRACING_INITIALIZED.get().is_some() {
        return;
    }

    // Build the env filter
    let env_filter = if let Some(ref filter) = config.env_filter_override {
        EnvFilter::new(filter)
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&config.level))
    };

    // Create the formatting layer based on config
    let result = TRACING_INITIALIZED.set(true);

    if result.is_err() {
        // Already initialized
        return;
    }

    match config.format {
        OutputFormat::Json => {
            let layer = fmt::layer()
                .json()
                .with_thread_ids(config.with_thread_ids)
                .with_thread_names(config.with_thread_names)
                .with_target(config.with_target)
                .with_file(config.with_file)
                .with_line_number(config.with_line_number)
                .with_span_events(config.span_events)
                .with_filter(env_filter);

            if let Err(e) = tracing_subscriber::registry().with(layer).try_init() {
                eprintln!("Failed to initialize tracing: {:?}", e);
            }
        }
        OutputFormat::Text => {
            let layer = fmt::layer()
                .with_thread_ids(config.with_thread_ids)
                .with_thread_names(config.with_thread_names)
                .with_target(config.with_target)
                .with_file(config.with_file)
                .with_line_number(config.with_line_number)
                .with_span_events(config.span_events)
                .with_ansi(config.ansi)
                .with_filter(env_filter);

            if let Err(e) = tracing_subscriber::registry().with(layer).try_init() {
                eprintln!("Failed to initialize tracing: {:?}", e);
            }
        }
        OutputFormat::Compact => {
            let layer = fmt::layer()
                .compact()
                .with_thread_ids(config.with_thread_ids)
                .with_thread_names(config.with_thread_names)
                .with_target(config.with_target)
                .with_file(config.with_file)
                .with_line_number(config.with_line_number)
                .with_ansi(config.ansi)
                .with_filter(env_filter);

            if let Err(e) = tracing_subscriber::registry().with(layer).try_init() {
                eprintln!("Failed to initialize tracing: {:?}", e);
            }
        }
    }
}

/// Initialize tracing with default configuration.
///
/// Uses the `RUST_LOG` environment variable if set, otherwise defaults to `info`.
pub fn init_default() {
    init_tracing(TracingConfig::default());
}

/// Check if tracing has been initialized.
pub fn is_initialized() -> bool {
    TRACING_INITIALIZED.get().is_some()
}

// ============================================================================
// Request Tracing ID Support
// ============================================================================

use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique request tracing ID.
///
/// The ID is a combination of:
/// - A UUID v4 prefix (8 characters)
/// - A monotonically increasing counter
///
/// Format: `{uuid_prefix}-{counter}` (e.g., `a1b2c3d4-000001`)
pub fn generate_request_id() -> String {
    let uuid = Uuid::new_v4();
    let uuid_prefix = &uuid.to_string().replace('-', "")[..8];
    let counter = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{:06}", uuid_prefix, counter)
}

/// Generate a unique span ID for distributed tracing.
pub fn generate_span_id() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string().replace('-', "")[..16].to_string()
}

// ============================================================================
// Span Helpers for SDK Operations
// ============================================================================

/// Create a span for a query operation.
#[macro_export]
macro_rules! query_span {
    ($request_id:expr) => {
        tracing::info_span!(
            "query",
            request_id = %$request_id,
            sdk.component = "query",
            sdk.version = env!("CARGO_PKG_VERSION")
        )
    };
}

/// Create a span for a transport operation.
#[macro_export]
macro_rules! transport_span {
    ($operation:expr, $transport_type:expr) => {
        tracing::debug_span!(
            "transport",
            operation = %$operation,
            transport_type = %$transport_type,
            sdk.component = "transport"
        )
    };
}

/// Create a span for a skill operation.
#[macro_export]
macro_rules! skill_span {
    ($skill_name:expr, $operation:expr) => {
        tracing::info_span!(
            "skill",
            skill_name = %$skill_name,
            operation = %$operation,
            sdk.component = "skills"
        )
    };
}

/// Create a span for a connection pool operation.
#[macro_export]
macro_rules! pool_span {
    ($operation:expr) => {
        tracing::debug_span!(
            "connection_pool",
            operation = %$operation,
            sdk.component = "pool"
        )
    };
}

/// Create a span for an MCP operation.
#[macro_export]
macro_rules! mcp_span {
    ($tool_name:expr, $operation:expr) => {
        tracing::info_span!(
            "mcp",
            tool_name = %$tool_name,
            operation = %$operation,
            sdk.component = "mcp"
        )
    };
}

// ============================================================================
// Error Category Logging Helpers
// ============================================================================

/// Log an error with its category and structured fields.
#[macro_export]
macro_rules! log_error_with_category {
    ($error:expr, $category:expr, $message:expr) => {
        match $category {
            $crate::errors::ErrorCategory::Network => {
                tracing::error!(
                    error.category = "network",
                    error.code = %$error.error_code(),
                    error.retryable = $error.is_retryable(),
                    error.http_status = $error.http_status().code(),
                    message = %$message,
                    error = %$error
                )
            }
            $crate::errors::ErrorCategory::Process => {
                tracing::error!(
                    error.category = "process",
                    error.code = %$error.error_code(),
                    error.retryable = $error.is_retryable(),
                    message = %$message,
                    error = %$error
                )
            }
            $crate::errors::ErrorCategory::Parsing => {
                tracing::error!(
                    error.category = "parsing",
                    error.code = %$error.error_code(),
                    error.retryable = false,
                    message = %$message,
                    error = %$error
                )
            }
            $crate::errors::ErrorCategory::Configuration => {
                tracing::error!(
                    error.category = "configuration",
                    error.code = %$error.error_code(),
                    error.retryable = false,
                    message = %$message,
                    error = %$error
                )
            }
            $crate::errors::ErrorCategory::Validation => {
                tracing::error!(
                    error.category = "validation",
                    error.code = %$error.error_code(),
                    error.retryable = false,
                    message = %$message,
                    error = %$error
                )
            }
            $crate::errors::ErrorCategory::Permission => {
                tracing::error!(
                    error.category = "permission",
                    error.code = %$error.error_code(),
                    error.retryable = false,
                    error.http_status = $error.http_status().code(),
                    message = %$message,
                    error = %$error
                )
            }
            $crate::errors::ErrorCategory::Resource => {
                tracing::error!(
                    error.category = "resource",
                    error.code = %$error.error_code(),
                    error.retryable = $error.is_retryable(),
                    message = %$message,
                    error = %$error
                )
            }
            $crate::errors::ErrorCategory::Internal => {
                tracing::error!(
                    error.category = "internal",
                    error.code = %$error.error_code(),
                    error.retryable = false,
                    message = %$message,
                    error = %$error
                )
            }
            $crate::errors::ErrorCategory::External => {
                tracing::error!(
                    error.category = "external",
                    error.code = %$error.error_code(),
                    error.retryable = $error.is_retryable(),
                    message = %$message,
                    error = %$error
                )
            }
        }
    };
}

/// Log a warning for a retryable error.
#[macro_export]
macro_rules! log_retryable_error {
    ($error:expr, $attempt:expr, $max_attempts:expr, $message:expr) => {
        tracing::warn!(
            error.category = ?$error.category(),
            error.code = %$error.error_code(),
            retry.attempt = $attempt,
            retry.max_attempts = $max_attempts,
            message = %$message,
            error = %$error,
            "Retryable error, will retry"
        )
    };
}

// ============================================================================
// Structured Metrics Logging
// ============================================================================

/// Log a timing metric.
pub fn log_timing(operation: &str, duration_ms: u64, labels: &[(&str, &str)]) {
    let labels_str = labels
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",");

    tracing::info!(
        metric.name = operation,
        metric.kind = "timing",
        metric.value_ms = duration_ms,
        metric.labels = %labels_str,
        "Operation completed"
    );
}

/// Log a counter increment.
pub fn log_counter(name: &str, increment: u64, labels: &[(&str, &str)]) {
    let labels_str = labels
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",");

    tracing::debug!(
        metric.name = name,
        metric.kind = "counter",
        metric.increment = increment,
        metric.labels = %labels_str,
        "Counter incremented"
    );
}

/// Log a gauge value.
pub fn log_gauge(name: &str, value: f64, labels: &[(&str, &str)]) {
    let labels_str = labels
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",");

    tracing::debug!(
        metric.name = name,
        metric.kind = "gauge",
        metric.value = value,
        metric.labels = %labels_str,
        "Gauge recorded"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_request_id() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();

        // IDs should be different (counter incremented)
        assert_ne!(id1, id2);

        // Should have the correct format: {8-char-uuid}-{6-digit-counter}
        assert_eq!(id1.len(), 15); // 8 + 1 + 6
        assert!(id1.contains('-'));

        let parts: Vec<&str> = id1.split('-').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 6);
    }

    #[test]
    fn test_generate_span_id() {
        let span_id = generate_span_id();
        assert_eq!(span_id.len(), 16);
        assert!(span_id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_tracing_config_defaults() {
        let config = TracingConfig::default();
        assert_eq!(config.level, "info");
        assert!(matches!(config.format, OutputFormat::Text));
        assert!(config.with_target);
        assert!(!config.with_file);
    }

    #[test]
    fn test_tracing_config_production() {
        let config = TracingConfig::production();
        assert_eq!(config.level, "info");
        assert!(matches!(config.format, OutputFormat::Json));
        assert!(config.with_thread_ids);
        assert!(!config.ansi);
    }

    #[test]
    fn test_tracing_config_development() {
        let config = TracingConfig::development();
        assert_eq!(config.level, "debug");
        assert!(matches!(config.format, OutputFormat::Text));
        assert!(config.ansi);
        assert!(config.with_file);
    }

    #[test]
    fn test_is_initialized_before_init() {
        // Can't test this easily since it's global state
        // Just ensure the function exists and compiles
        let _ = is_initialized();
    }

    #[test]
    fn test_log_timing() {
        // This test just ensures the function compiles and runs
        // The actual output goes to the tracing subscriber
        log_timing("test_operation", 100, &[("key", "value")]);
    }

    #[test]
    fn test_log_counter() {
        log_counter("test_counter", 1, &[("endpoint", "/api/test")]);
    }

    #[test]
    fn test_log_gauge() {
        log_gauge("test_gauge", 42.5, &[("location", "room1")]);
    }
}
