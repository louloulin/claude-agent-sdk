//! Error types for the Skills system

use std::fmt;
use thiserror::Error;

/// Errors that can occur in the Skills system
#[derive(Error, Debug, Clone)]
pub enum SkillError {
    /// Validation error
    #[error("Skill validation failed: {0}")]
    Validation(String),

    /// Execution error
    #[error("Skill execution failed: {0}")]
    Execution(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Skill not found
    #[error("Skill not found: {0}")]
    NotFound(String),

    /// Skill already exists
    #[error("Skill already exists: {0}")]
    AlreadyExists(String),

    /// Invalid skill metadata
    #[error("Invalid skill metadata: {0}")]
    InvalidMetadata(String),

    /// Skill version conflict
    #[error("Skill version conflict: {0}")]
    VersionConflict(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Result type for Skill operations
pub type Result<T> = std::result::Result<T, SkillError>;

/// Result of a Skill execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillOutput {
    /// Whether execution was successful
    pub success: bool,

    /// Output data
    pub data: serde_json::Value,

    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl SkillOutput {
    /// Create a successful output
    pub fn ok(data: impl Into<serde_json::Value>) -> Self {
        SkillOutput {
            success: true,
            data: data.into(),
            error: None,
            metadata: None,
        }
    }

    /// Create a failed output
    pub fn err(error: impl Into<String>) -> Self {
        SkillOutput {
            success: false,
            data: serde_json::Value::Null,
            error: Some(error.into()),
            metadata: None,
        }
    }

    /// Add metadata to the output
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl fmt::Display for SkillOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.success {
            write!(f, "Success: {}", self.data)
        } else {
            write!(
                f,
                "Error: {}",
                self.error.as_deref().unwrap_or("Unknown error")
            )
        }
    }
}

/// Convenience type for Skill execution results
pub type SkillResult = Result<SkillOutput>;
