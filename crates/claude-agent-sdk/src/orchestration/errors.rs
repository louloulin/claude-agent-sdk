//! # Orchestration error types


/// Main error type for orchestration operations
#[derive(Debug, thiserror::Error)]
pub enum OrchestrationError {
    #[error("Agent {0} failed: {1}")]
    AgentFailed(String, String),

    #[error("Agent error: {0}")]
    AgentError(anyhow::Error),

    #[error("Orchestrator {0} failed: {1}")]
    OrchestratorFailed(String, String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Execution cancelled")]
    Cancelled,

    #[error("Partial success: {0} agents failed")]
    PartialSuccess(usize),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Result type for orchestration operations
pub type Result<T> = std::result::Result<T, OrchestrationError>;

impl OrchestrationError {
    /// Create an agent failure error
    pub fn agent_failure(agent_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::AgentFailed(agent_name.into(), reason.into())
    }

    /// Create an orchestrator failure error
    pub fn orchestrator_failure(
        orchestrator_name: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::OrchestratorFailed(orchestrator_name.into(), reason.into())
    }

    /// Create a timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    /// Create an invalid configuration error
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Timeout(_) | Self::AgentFailed(_, _))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = OrchestrationError::agent_failure("TestAgent", "Network error");
        assert!(matches!(err, OrchestrationError::AgentFailed(_, _)));

        let err = OrchestrationError::timeout("Operation timed out");
        assert!(matches!(err, OrchestrationError::Timeout(_)));

        let err = OrchestrationError::invalid_config("Missing field");
        assert!(matches!(err, OrchestrationError::InvalidConfig(_)));
    }

    #[test]
    fn test_error_retryable() {
        assert!(OrchestrationError::timeout("test").is_retryable());
        assert!(OrchestrationError::agent_failure("test", "error").is_retryable());
        assert!(!OrchestrationError::invalid_config("test").is_retryable());
        assert!(!OrchestrationError::Cancelled.is_retryable());
    }
}
