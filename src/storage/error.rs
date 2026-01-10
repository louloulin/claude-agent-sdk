// Storage-specific error types following Claude Agent SDK error patterns
use thiserror::Error;

/// Storage-specific errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database connection error: {0}")]
    DatabaseConnection(String),

    #[error("Query execution error: {0}")]
    QueryExecution(String),

    #[error("Embedding generation error: {0}")]
    EmbeddingGeneration(String),

    #[error("Vector search error: {0}")]
    VectorSearch(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Paper not found: {0}")]
    PaperNotFound(String),

    #[error("Index not found: {0}")]
    IndexNotFound(String),

    #[error("Storage capacity exceeded")]
    CapacityExceeded,

    #[error("Invalid vector dimension: expected {expected}, got {actual}")]
    InvalidDimension { expected: usize, actual: usize },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Internal storage error: {0}")]
    Internal(String),
}

/// Storage result type
pub type StorageResult<T> = Result<T, StorageError>;
