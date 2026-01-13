//! # Agent trait and core types
//!
//! This module defines the core Agent trait and associated types for the
//! multi-agent orchestration framework.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error type for agent operations
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Timeout")]
    Timeout,

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Result type for agent operations
pub type Result<T> = std::result::Result<T, AgentError>;

/// Input to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    /// Main content/prompt for the agent
    pub content: String,

    /// Additional context data (JSON-serializable)
    #[serde(default)]
    pub context: serde_json::Value,

    /// Metadata key-value pairs
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl AgentInput {
    /// Create a new agent input with content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            context: serde_json::json!({}),
            metadata: HashMap::new(),
        }
    }

    /// Add context data
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = context;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Output from an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    /// Main response content
    pub content: String,

    /// Additional data (JSON-serializable)
    #[serde(default)]
    pub data: serde_json::Value,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,

    /// Metadata key-value pairs
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl AgentOutput {
    /// Create a new agent output with content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            data: serde_json::json!({}),
            confidence: 1.0,
            metadata: HashMap::new(),
        }
    }

    /// Set confidence score
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Add data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if output is successful (confidence > 0.5)
    pub fn is_successful(&self) -> bool {
        self.confidence > 0.5
    }
}

/// Core Agent trait
///
/// Agents implement this trait to participate in orchestration.
/// Each agent has a name, description, and execution logic.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Agent name (must be unique)
    fn name(&self) -> &str;

    /// Agent description (what it does)
    fn description(&self) -> &str;

    /// Execute the agent's logic
    ///
    /// Takes input and produces output asynchronously.
    /// Returns an error if execution fails.
    async fn execute(&self, input: AgentInput) -> Result<AgentOutput>;
}

/// Simple wrapper agent for easy creation
pub struct SimpleAgent<F>
where
    F: Fn(AgentInput) -> Result<AgentOutput> + Send + Sync,
{
    name: String,
    description: String,
    handler: F,
}

impl<F> SimpleAgent<F>
where
    F: Fn(AgentInput) -> Result<AgentOutput> + Send + Sync,
{
    /// Create a new simple agent
    pub fn new(name: impl Into<String>, description: impl Into<String>, handler: F) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            handler,
        }
    }
}

#[async_trait]
impl<F> Agent for SimpleAgent<F>
where
    F: Fn(AgentInput) -> Result<AgentOutput> + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // Call the synchronous handler
        (self.handler)(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_input_creation() {
        let input = AgentInput::new("test content")
            .with_context(serde_json::json!({"key": "value"}))
            .with_metadata("meta1", "value1");

        assert_eq!(input.content, "test content");
        assert_eq!(input.context["key"], "value");
        assert_eq!(input.metadata["meta1"], "value1");
    }

    #[test]
    fn test_agent_output_creation() {
        let output = AgentOutput::new("test response")
            .with_confidence(0.8)
            .with_data(serde_json::json!({"result": 42}))
            .with_metadata("meta1", "value1");

        assert_eq!(output.content, "test response");
        assert_eq!(output.confidence, 0.8);
        assert_eq!(output.data["result"], 42);
        assert_eq!(output.metadata["meta1"], "value1");
        assert!(output.is_successful());
    }

    #[test]
    fn test_simple_agent() {
        let agent = SimpleAgent::new("TestAgent", "A test agent", |input| {
            Ok(AgentOutput::new(format!("Processed: {}", input.content)))
        });

        assert_eq!(agent.name(), "TestAgent");
        assert_eq!(agent.description(), "A test agent");
    }

    #[tokio::test]
    async fn test_simple_agent_execute() {
        let agent = SimpleAgent::new("TestAgent", "A test agent", |input| {
            Ok(AgentOutput::new(format!("Echo: {}", input.content)).with_confidence(0.9))
        });

        let input = AgentInput::new("Hello");
        let output = agent.execute(input).await.unwrap();

        assert_eq!(output.content, "Echo: Hello");
        assert_eq!(output.confidence, 0.9);
        assert!(output.is_successful());
    }
}
