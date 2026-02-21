//! Error types for the Claude Agent SDK
//!
//! This module provides a comprehensive error handling system with:
//! - Error categories for classification
//! - Error codes for machine-readable identification
//! - Retryable detection for transient errors
//! - HTTP status code mapping for API responses

use std::path::PathBuf;
use thiserror::Error;

/// Error category for classifying errors by their source and nature.
///
/// Categories enable:
/// - Structured logging with category filters
/// - Metrics aggregation by error type
/// - Retry logic based on error source
/// - User-facing error messages by category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Network-related errors (connection, timeout, DNS)
    Network,
    /// CLI process errors (not found, crash, exit)
    Process,
    /// Data parsing/serialization errors (JSON, message format)
    Parsing,
    /// Configuration errors (invalid options, missing settings)
    Configuration,
    /// Input validation errors (invalid parameters, constraints)
    Validation,
    /// Permission/security errors (auth, access denied)
    Permission,
    /// Resource errors (not found, quota exceeded)
    Resource,
    /// Internal SDK errors (bugs, invariants violated)
    Internal,
    /// External service errors (API rate limits, service unavailable)
    External,
}

impl ErrorCategory {
    /// Returns true if errors in this category may be retried.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ErrorCategory::Network | ErrorCategory::External | ErrorCategory::Process
        )
    }

    /// Returns a human-readable description of the category.
    pub fn description(&self) -> &'static str {
        match self {
            ErrorCategory::Network => "Network connectivity or communication error",
            ErrorCategory::Process => "CLI process execution error",
            ErrorCategory::Parsing => "Data parsing or serialization error",
            ErrorCategory::Configuration => "Configuration or setup error",
            ErrorCategory::Validation => "Input validation error",
            ErrorCategory::Permission => "Permission or authentication error",
            ErrorCategory::Resource => "Resource not found or unavailable",
            ErrorCategory::Internal => "Internal SDK error",
            ErrorCategory::External => "External service error",
        }
    }
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::Network => write!(f, "network"),
            ErrorCategory::Process => write!(f, "process"),
            ErrorCategory::Parsing => write!(f, "parsing"),
            ErrorCategory::Configuration => write!(f, "configuration"),
            ErrorCategory::Validation => write!(f, "validation"),
            ErrorCategory::Permission => write!(f, "permission"),
            ErrorCategory::Resource => write!(f, "resource"),
            ErrorCategory::Internal => write!(f, "internal"),
            ErrorCategory::External => write!(f, "external"),
        }
    }
}

/// HTTP status code mapping for errors.
///
/// Provides a standard way to convert SDK errors to HTTP responses
/// for API integrations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpStatus {
    /// 400 Bad Request - invalid input
    BadRequest,
    /// 401 Unauthorized - authentication required
    Unauthorized,
    /// 403 Forbidden - permission denied
    Forbidden,
    /// 404 Not Found - resource not found
    NotFound,
    /// 408 Request Timeout - operation timed out
    RequestTimeout,
    /// 409 Conflict - state conflict
    Conflict,
    /// 422 Unprocessable Entity - validation error
    UnprocessableEntity,
    /// 429 Too Many Requests - rate limited
    TooManyRequests,
    /// 500 Internal Server Error - SDK bug
    InternalServerError,
    /// 502 Bad Gateway - upstream error (CLI/API)
    BadGateway,
    /// 503 Service Unavailable - temporary error
    ServiceUnavailable,
    /// 504 Gateway Timeout - upstream timeout
    GatewayTimeout,
}

impl HttpStatus {
    /// Returns the numeric HTTP status code.
    pub fn code(&self) -> u16 {
        match self {
            HttpStatus::BadRequest => 400,
            HttpStatus::Unauthorized => 401,
            HttpStatus::Forbidden => 403,
            HttpStatus::NotFound => 404,
            HttpStatus::RequestTimeout => 408,
            HttpStatus::Conflict => 409,
            HttpStatus::UnprocessableEntity => 422,
            HttpStatus::TooManyRequests => 429,
            HttpStatus::InternalServerError => 500,
            HttpStatus::BadGateway => 502,
            HttpStatus::ServiceUnavailable => 503,
            HttpStatus::GatewayTimeout => 504,
        }
    }
}

impl From<HttpStatus> for u16 {
    fn from(status: HttpStatus) -> u16 {
        status.code()
    }
}

/// Main error type for the Claude Agent SDK
#[derive(Debug, Error)]
pub enum ClaudeError {
    /// CLI connection error
    #[error("CLI connection error: {0}")]
    Connection(#[from] ConnectionError),

    /// Process error
    #[error("Process error: {0}")]
    Process(#[from] ProcessError),

    /// JSON decode error
    #[error("JSON decode error: {0}")]
    JsonDecode(#[from] JsonDecodeError),

    /// Message parse error
    #[error("Message parse error: {0}")]
    MessageParse(#[from] MessageParseError),

    /// Transport error
    #[error("Transport error: {0}")]
    Transport(String),

    /// Control protocol error
    #[error("Control protocol error: {0}")]
    ControlProtocol(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// CLI not found error
    #[error("CLI not found: {0}")]
    CliNotFound(#[from] CliNotFoundError),

    /// Image validation error
    #[error("Image validation error: {0}")]
    ImageValidation(#[from] ImageValidationError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other errors
    #[error(transparent)]
    Other(#[from] anyhow::Error),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid input error
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Error when Claude Code CLI cannot be found
#[derive(Debug, Error)]
#[error("CLI not found: {message}")]
pub struct CliNotFoundError {
    /// Error message
    pub message: String,
    /// Path that was checked
    pub cli_path: Option<PathBuf>,
}

impl CliNotFoundError {
    /// Create a new CLI not found error
    pub fn new(message: impl Into<String>, cli_path: Option<PathBuf>) -> Self {
        Self {
            message: message.into(),
            cli_path,
        }
    }
}

/// Error when connecting to Claude Code CLI
#[derive(Debug, Error)]
#[error("Connection error: {message}")]
pub struct ConnectionError {
    /// Error message
    pub message: String,
}

impl ConnectionError {
    /// Create a new connection error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Error when the CLI process fails
#[derive(Debug, Error)]
#[error("Process error (exit code {exit_code:?}): {message}")]
pub struct ProcessError {
    /// Error message
    pub message: String,
    /// Process exit code
    pub exit_code: Option<i32>,
    /// stderr output
    pub stderr: Option<String>,
}

impl ProcessError {
    /// Create a new process error
    pub fn new(message: impl Into<String>, exit_code: Option<i32>, stderr: Option<String>) -> Self {
        Self {
            message: message.into(),
            exit_code,
            stderr,
        }
    }
}

/// Error when JSON decoding fails
#[derive(Debug, Error)]
#[error("JSON decode error: {message}")]
pub struct JsonDecodeError {
    /// Error message
    pub message: String,
    /// The line that failed to decode
    pub line: String,
}

impl JsonDecodeError {
    /// Create a new JSON decode error
    pub fn new(message: impl Into<String>, line: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: line.into(),
        }
    }
}

/// Error when message parsing fails
#[derive(Debug, Error)]
#[error("Message parse error: {message}")]
pub struct MessageParseError {
    /// Error message
    pub message: String,
    /// The data that failed to parse
    pub data: Option<serde_json::Value>,
}

impl MessageParseError {
    /// Create a new message parse error
    pub fn new(message: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self {
            message: message.into(),
            data,
        }
    }
}

/// Image validation error
#[derive(Debug, Error)]
#[error("Image validation error: {message}")]
pub struct ImageValidationError {
    /// Error message
    pub message: String,
}

impl ImageValidationError {
    /// Create a new image validation error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Result type for the Claude Agent SDK
pub type Result<T> = std::result::Result<T, ClaudeError>;

impl ClaudeError {
    /// Returns the error category for this error.
    ///
    /// Categories group related errors together for:
    /// - Structured logging and filtering
    /// - Metrics aggregation
    /// - Retry logic decisions
    pub fn category(&self) -> ErrorCategory {
        match self {
            ClaudeError::Connection(_) => ErrorCategory::Network,
            ClaudeError::Process(_) => ErrorCategory::Process,
            ClaudeError::JsonDecode(_) => ErrorCategory::Parsing,
            ClaudeError::MessageParse(_) => ErrorCategory::Parsing,
            ClaudeError::Transport(_) => ErrorCategory::Network,
            ClaudeError::ControlProtocol(_) => ErrorCategory::Internal,
            ClaudeError::InvalidConfig(_) => ErrorCategory::Configuration,
            ClaudeError::CliNotFound(_) => ErrorCategory::Configuration,
            ClaudeError::ImageValidation(_) => ErrorCategory::Validation,
            ClaudeError::Io(_) => ErrorCategory::Internal,
            ClaudeError::Other(_) => ErrorCategory::Internal,
            ClaudeError::NotFound(_) => ErrorCategory::Resource,
            ClaudeError::InvalidInput(_) => ErrorCategory::Validation,
            ClaudeError::InternalError(_) => ErrorCategory::Internal,
        }
    }

    /// Returns a machine-readable error code for this error.
    ///
    /// Error codes are stable identifiers that can be used for:
    /// - Programmatic error handling
    /// - API response codes
    /// - Documentation references
    ///
    /// Format: `E{category_prefix}{number}` (e.g., `ENET001`, `EPROC001`)
    pub fn error_code(&self) -> &'static str {
        match self {
            ClaudeError::Connection(_) => "ENET001",
            ClaudeError::Process(_) => "EPROC001",
            ClaudeError::JsonDecode(_) => "EPARSE001",
            ClaudeError::MessageParse(_) => "EPARSE002",
            ClaudeError::Transport(_) => "ENET002",
            ClaudeError::ControlProtocol(_) => "EINT001",
            ClaudeError::InvalidConfig(_) => "ECFG001",
            ClaudeError::CliNotFound(_) => "ECFG002",
            ClaudeError::ImageValidation(_) => "EVAL001",
            ClaudeError::Io(_) => "EINT002",
            ClaudeError::Other(_) => "EINT003",
            ClaudeError::NotFound(_) => "ERES001",
            ClaudeError::InvalidInput(_) => "EVAL002",
            ClaudeError::InternalError(_) => "EINT004",
        }
    }

    /// Returns true if this error may be retried.
    ///
    /// Retryable errors are typically transient failures that may succeed
    /// on a subsequent attempt:
    /// - Network errors (connection refused, timeout)
    /// - Process errors (CLI crashed, can restart)
    /// - External service errors (rate limits, temporary unavailable)
    pub fn is_retryable(&self) -> bool {
        self.category().is_retryable()
    }

    /// Returns the recommended HTTP status code for this error.
    ///
    /// Useful for converting SDK errors to HTTP API responses.
    pub fn http_status(&self) -> HttpStatus {
        match self {
            ClaudeError::Connection(_) => HttpStatus::ServiceUnavailable,
            ClaudeError::Process(_) => HttpStatus::BadGateway,
            ClaudeError::JsonDecode(_) => HttpStatus::UnprocessableEntity,
            ClaudeError::MessageParse(_) => HttpStatus::UnprocessableEntity,
            ClaudeError::Transport(_) => HttpStatus::ServiceUnavailable,
            ClaudeError::ControlProtocol(_) => HttpStatus::InternalServerError,
            ClaudeError::InvalidConfig(_) => HttpStatus::InternalServerError,
            ClaudeError::CliNotFound(_) => HttpStatus::InternalServerError,
            ClaudeError::ImageValidation(_) => HttpStatus::BadRequest,
            ClaudeError::Io(_) => HttpStatus::InternalServerError,
            ClaudeError::Other(_) => HttpStatus::InternalServerError,
            ClaudeError::NotFound(_) => HttpStatus::NotFound,
            ClaudeError::InvalidInput(_) => HttpStatus::BadRequest,
            ClaudeError::InternalError(_) => HttpStatus::InternalServerError,
        }
    }

    /// Returns a detailed error context for logging and debugging.
    ///
    /// Includes:
    /// - Error code
    /// - Category
    /// - Whether it's retryable
    /// - Human-readable message
    pub fn to_error_context(&self) -> ErrorContext {
        ErrorContext {
            code: self.error_code().to_string(),
            category: self.category(),
            message: self.to_string(),
            retryable: self.is_retryable(),
            http_status: self.http_status().code(),
        }
    }
}

/// Detailed error context for logging and debugging.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Machine-readable error code (e.g., "ENET001")
    pub code: String,
    /// Error category
    pub category: ErrorCategory,
    /// Human-readable error message
    pub message: String,
    /// Whether the error may be retried
    pub retryable: bool,
    /// Recommended HTTP status code
    pub http_status: u16,
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] [{}] {} (retryable: {}, http: {})",
            self.code, self.category, self.message, self.retryable, self.http_status
        )
    }
}

impl std::fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpStatus::BadRequest => write!(f, "400 Bad Request"),
            HttpStatus::Unauthorized => write!(f, "401 Unauthorized"),
            HttpStatus::Forbidden => write!(f, "403 Forbidden"),
            HttpStatus::NotFound => write!(f, "404 Not Found"),
            HttpStatus::RequestTimeout => write!(f, "408 Request Timeout"),
            HttpStatus::Conflict => write!(f, "409 Conflict"),
            HttpStatus::UnprocessableEntity => write!(f, "422 Unprocessable Entity"),
            HttpStatus::TooManyRequests => write!(f, "429 Too Many Requests"),
            HttpStatus::InternalServerError => write!(f, "500 Internal Server Error"),
            HttpStatus::BadGateway => write!(f, "502 Bad Gateway"),
            HttpStatus::ServiceUnavailable => write!(f, "503 Service Unavailable"),
            HttpStatus::GatewayTimeout => write!(f, "504 Gateway Timeout"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        let error = ClaudeError::Connection(ConnectionError::new("test"));
        assert_eq!(error.category(), ErrorCategory::Network);
        assert!(error.is_retryable());
        assert_eq!(error.error_code(), "ENET001");

        let error = ClaudeError::InvalidConfig("test".to_string());
        assert_eq!(error.category(), ErrorCategory::Configuration);
        assert!(!error.is_retryable());
        assert_eq!(error.error_code(), "ECFG001");
    }

    #[test]
    fn test_http_status_mapping() {
        let error = ClaudeError::NotFound("test".to_string());
        assert_eq!(error.http_status(), HttpStatus::NotFound);
        assert_eq!(error.http_status().code(), 404);

        let error = ClaudeError::InvalidInput("test".to_string());
        assert_eq!(error.http_status(), HttpStatus::BadRequest);
        assert_eq!(error.http_status().code(), 400);
    }

    #[test]
    fn test_error_context() {
        let error = ClaudeError::Connection(ConnectionError::new("connection failed"));
        let ctx = error.to_error_context();
        assert_eq!(ctx.code, "ENET001");
        assert_eq!(ctx.category, ErrorCategory::Network);
        assert!(ctx.retryable);
        assert_eq!(ctx.http_status, 503);
    }

    #[test]
    fn test_category_display() {
        assert_eq!(ErrorCategory::Network.to_string(), "network");
        assert_eq!(ErrorCategory::Process.to_string(), "process");
        assert_eq!(ErrorCategory::Parsing.to_string(), "parsing");
    }

    #[test]
    fn test_category_description() {
        assert!(!ErrorCategory::Network.description().is_empty());
        assert!(ErrorCategory::Network.is_retryable());
        assert!(!ErrorCategory::Configuration.is_retryable());
    }
}
