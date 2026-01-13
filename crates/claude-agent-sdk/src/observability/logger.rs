//! # Structured Logging for Agent SDK
//!
//! This module provides structured logging capabilities with level-based filtering,
//! context support, and integration with the tracing ecosystem.
//!
//! ## Features
//!
//! - **Structured Logging**: JSON-formatted logs with consistent field names
//! - **Context Support**: Attach context to log messages automatically
//! - **Level Filtering**: Support for trace, debug, info, warn, error levels
//! - **Tracing Integration**: Compatible with the `tracing` ecosystem
//! - **Performance**: Low-overhead logging with lazy evaluation
//!
//! ## Example
//!
//! ```no_run
//! use claude_agent_sdk_rs::observability::Logger;
//!
//! let logger = Logger::new("MyAgent");
//! logger.info("Starting agent execution", &[("task_id", "123")]);
//! logger.error("Failed to execute", Some(&anyhow::anyhow!("Connection error")));
//! ```

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TRACE" => Ok(LogLevel::Trace),
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO" => Ok(LogLevel::Info),
            "WARN" => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }
}

/// Structured log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp (milliseconds since epoch)
    pub timestamp: u64,

    /// Log level
    pub level: LogLevel,

    /// Logger context/name
    pub context: String,

    /// Log message
    pub message: String,

    /// Additional key-value pairs
    pub metadata: Vec<(String, String)>,

    /// Optional error details
    pub error: Option<String>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, context: impl Into<String>, message: impl Into<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            timestamp,
            level,
            context: context.into(),
            message: message.into(),
            metadata: Vec::new(),
            error: None,
        }
    }

    /// Add metadata field
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    /// Add multiple metadata fields
    pub fn with_fields(mut self, fields: &[(impl AsRef<str>, impl AsRef<str>)]) -> Self {
        for (key, value) in fields {
            self.metadata.push((key.as_ref().to_string(), value.as_ref().to_string()));
        }
        self
    }

    /// Add error information
    pub fn with_error(mut self, error: impl fmt::Display) -> Self {
        self.error = Some(error.to_string());
        self
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> String {
        let mut s = String::from('{');

        s.push_str(&format!(r#""timestamp":{}"#, self.timestamp));
        s.push_str(&format!(r#","level":"{}""#, self.level));
        s.push_str(&format!(r#","context":"{}""#, escape_json(&self.context)));
        s.push_str(&format!(r#","message":"{}""#, escape_json(&self.message)));

        for (key, value) in &self.metadata {
            s.push_str(&format!(r#","{}":"{}""#, escape_json(key), escape_json(value)));
        }

        if let Some(ref error) = self.error {
            s.push_str(&format!(r#","error":"{}""#, escape_json(error)));
        }

        s.push('}');
        s
    }

    /// Convert to human-readable text
    pub fn to_text(&self) -> String {
        let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(self.timestamp as i64)
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S%.3f");

        let mut s = format!("[{}] {} {}: {}", timestamp, self.level, self.context, self.message);

        for (key, value) in &self.metadata {
            s.push_str(&format!(" {}={}", key, value));
        }

        if let Some(ref error) = self.error {
            s.push_str(&format!(" error={}", error));
        }

        s
    }
}

/// Escape JSON string
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Observer for log entries
pub trait LogObserver: Send + Sync {
    /// Called when a log entry is created
    fn on_log(&self, entry: &LogEntry);
}

/// Default console observer that prints to stdout/stderr
pub struct ConsoleLogObserver {
    /// Output format
    format: LogFormat,

    /// Minimum level to output
    min_level: LogLevel,
}

/// Log output format
#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    /// Human-readable text format
    Text,

    /// JSON format
    Json,
}

impl ConsoleLogObserver {
    /// Create a new console observer
    pub fn new(min_level: LogLevel, format: LogFormat) -> Self {
        Self { format, min_level }
    }

    /// Create a text format observer
    pub fn text(min_level: LogLevel) -> Self {
        Self::new(min_level, LogFormat::Text)
    }

    /// Create a JSON format observer
    pub fn json(min_level: LogLevel) -> Self {
        Self::new(min_level, LogFormat::Json)
    }
}

impl LogObserver for ConsoleLogObserver {
    fn on_log(&self, entry: &LogEntry) {
        if entry.level < self.min_level {
            return;
        }

        let output = match self.format {
            LogFormat::Text => entry.to_text(),
            LogFormat::Json => entry.to_json(),
        };

        match entry.level {
            LogLevel::Error => eprintln!("{}", output),
            LogLevel::Warn => eprintln!("{}", output),
            _ => println!("{}", output),
        };
    }
}

/// Structured logger
pub struct Logger {
    /// Logger context/name
    context: String,

    /// Minimum log level
    min_level: LogLevel,

    /// Observers for log entries
    observers: Vec<std::sync::Arc<dyn LogObserver>>,
}

impl fmt::Debug for Logger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Logger")
            .field("context", &self.context)
            .field("min_level", &self.min_level)
            .field("observers_count", &self.observers.len())
            .finish()
    }
}

impl Logger {
    /// Create a new logger with the given context
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            min_level: LogLevel::Info,
            observers: Vec::new(),
        }
    }

    /// Set the minimum log level
    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    /// Add an observer
    pub fn with_observer(mut self, observer: std::sync::Arc<dyn LogObserver>) -> Self {
        self.observers.push(observer);
        self
    }

    /// Log a trace message
    pub fn trace(&self, message: impl fmt::Display, fields: &[(impl AsRef<str>, impl AsRef<str>)]) {
        self.log(LogLevel::Trace, message, fields, None as Option<&str>);
    }

    /// Log a debug message
    pub fn debug(&self, message: impl fmt::Display, fields: &[(impl AsRef<str>, impl AsRef<str>)]) {
        self.log(LogLevel::Debug, message, fields, None as Option<&str>);
    }

    /// Log an info message
    pub fn info(&self, message: impl fmt::Display, fields: &[(impl AsRef<str>, impl AsRef<str>)]) {
        self.log(LogLevel::Info, message, fields, None as Option<&str>);
    }

    /// Log a warning message
    pub fn warn(&self, message: impl fmt::Display, fields: &[(impl AsRef<str>, impl AsRef<str>)]) {
        self.log(LogLevel::Warn, message, fields, None as Option<&str>);
    }

    /// Log an error message
    pub fn error(&self, message: impl fmt::Display, error: Option<impl fmt::Display>) {
        const EMPTY: &[(&str, &str)] = &[];
        self.log(LogLevel::Error, message, EMPTY, error);
    }

    /// Internal log method
    fn log(
        &self,
        level: LogLevel,
        message: impl fmt::Display,
        fields: &[(impl AsRef<str>, impl AsRef<str>)],
        error: Option<impl fmt::Display>,
    ) {
        if level < self.min_level {
            return;
        }

        let mut entry = LogEntry::new(level, &self.context, message.to_string());

        for (key, value) in fields {
            entry = entry.with_field(key.as_ref(), value.as_ref());
        }

        if let Some(e) = error {
            entry = entry.with_error(e);
        }

        // Notify observers
        for observer in &self.observers {
            observer.on_log(&entry);
        }

        // Default: use tracing if available
        if self.observers.is_empty() {
            match level {
                LogLevel::Trace => {
                    tracing::trace!(context = %self.context, message = %entry.message, "TRACE")
                },
                LogLevel::Debug => {
                    tracing::debug!(context = %self.context, message = %entry.message, "DEBUG")
                },
                LogLevel::Info => {
                    tracing::info!(context = %self.context, message = %entry.message, "INFO")
                },
                LogLevel::Warn => {
                    tracing::warn!(context = %self.context, message = %entry.message, "WARN")
                },
                LogLevel::Error => {
                    tracing::error!(context = %self.context, message = %entry.message, error = ?entry.error, "ERROR")
                },
            }
        }
    }
}

impl Clone for Logger {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            min_level: self.min_level,
            observers: self.observers.clone(),
        }
    }
}

/// Global logger registry (for convenience)
pub struct GlobalLogger {
    loggers: std::sync::RwLock<std::collections::HashMap<String, Logger>>,
}

impl GlobalLogger {
    /// Get the global logger instance
    pub fn instance() -> std::sync::Arc<Self> {
        static INSTANCE: std::sync::OnceLock<std::sync::Arc<GlobalLogger>> = std::sync::OnceLock::new();
        INSTANCE
            .get_or_init(|| {
                std::sync::Arc::new(Self {
                    loggers: std::sync::RwLock::new(std::collections::HashMap::new()),
                })
            })
            .clone()
    }

    /// Get or create a logger for a context
    pub fn get(&self, context: &str) -> Logger {
        let loggers = self.loggers.read().unwrap();
        loggers
            .get(context)
            .cloned()
            .unwrap_or_else(|| Logger::new(context))
    }

    /// Register a logger
    pub fn register(&self, logger: Logger) {
        let mut loggers = self.loggers.write().unwrap();
        loggers.insert(logger.context.clone(), logger);
    }

    /// Set the default minimum log level for all loggers
    pub fn set_min_level(&self, level: LogLevel) {
        let mut loggers = self.loggers.write().unwrap();
        for logger in loggers.values_mut() {
            logger.min_level = level;
        }
    }
}

/// Get a logger for the given context
pub fn logger(context: &str) -> Logger {
    GlobalLogger::instance().get(context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("INFO").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("error").unwrap(), LogLevel::Error);
        assert!(LogLevel::from_str("INVALID").is_err());
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogLevel::Info, "TestContext", "Test message")
            .with_field("key1", "value1")
            .with_field("key2", "value2");

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.context, "TestContext");
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.metadata.len(), 2);
    }

    #[test]
    fn test_log_entry_json() {
        let entry = LogEntry::new(LogLevel::Error, "Test", "Error message")
            .with_field("code", "500")
            .with_error("Connection failed");

        let json = entry.to_json();
        assert!(json.contains(r#""level":"ERROR""#));
        assert!(json.contains(r#""context":"Test""#));
        assert!(json.contains(r#""message":"Error message""#));
        assert!(json.contains(r#""code":"500""#));
        assert!(json.contains(r#""error":"Connection failed""#));
    }

    #[test]
    fn test_log_entry_text() {
        let entry = LogEntry::new(LogLevel::Info, "MyAgent", "Processing complete")
            .with_field("duration_ms", "150");

        let text = entry.to_text();
        assert!(text.contains("INFO"));
        assert!(text.contains("MyAgent"));
        assert!(text.contains("Processing complete"));
        assert!(text.contains("duration_ms=150"));
    }

    #[test]
    fn test_logger() {
        let logger = Logger::new("TestLogger").with_min_level(LogLevel::Debug);

        const EMPTY_FIELDS: &[(&str, &str)] = &[];

        // These should not panic
        logger.trace("Trace msg", EMPTY_FIELDS);
        logger.debug("Debug msg", &[("key", "value")]);
        logger.info("Info msg", EMPTY_FIELDS);
        logger.warn("Warn msg", EMPTY_FIELDS);
        logger.error("Error msg", Some("error details"));
    }

    #[test]
    fn test_logger_with_observer() {
        struct TestObserver {
            entries: std::sync::Arc<std::sync::Mutex<Vec<LogEntry>>>,
        }

        impl LogObserver for TestObserver {
            fn on_log(&self, entry: &LogEntry) {
                self.entries.lock().unwrap().push(entry.clone());
            }
        }

        let entries = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let observer = std::sync::Arc::new(TestObserver {
            entries: entries.clone(),
        });

        let logger = Logger::new("Test")
            .with_min_level(LogLevel::Info)
            .with_observer(observer);

        logger.info("Test message", &[("key", "value")]);

        let logged = entries.lock().unwrap();
        assert_eq!(logged.len(), 1);
        assert_eq!(logged[0].message, "Test message");
        assert_eq!(logged[0].context, "Test");
    }
}
