//! # Orchestrator trait and core types
//!
//! This module defines the Orchestrator trait which coordinates multiple agents
//! to accomplish complex tasks through various patterns.

use crate::orchestration::{
    agent::{Agent, AgentInput, AgentOutput},
    context::ExecutionTrace,
    errors::Result,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Input to an orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorInput {
    /// Main content/prompt for the orchestration
    pub content: String,

    /// Additional context data (JSON-serializable)
    #[serde(default)]
    pub context: serde_json::Value,

    /// Metadata key-value pairs
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl OrchestratorInput {
    /// Create a new orchestrator input
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

/// Output from an orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorOutput {
    /// Final result of orchestration
    pub result: String,

    /// Individual agent outputs (in execution order)
    pub agent_outputs: Vec<AgentOutput>,

    /// Execution trace
    pub execution_trace: ExecutionTrace,

    /// Whether orchestration succeeded
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,
}

impl OrchestratorOutput {
    /// Create a successful output
    pub fn success(
        result: impl Into<String>,
        agent_outputs: Vec<AgentOutput>,
        execution_trace: ExecutionTrace,
    ) -> Self {
        Self {
            result: result.into(),
            agent_outputs,
            execution_trace,
            success: true,
            error: None,
        }
    }

    /// Create a failed output
    pub fn failure(error: impl Into<String>, execution_trace: ExecutionTrace) -> Self {
        Self {
            result: String::new(),
            agent_outputs: Vec::new(),
            execution_trace,
            success: false,
            error: Some(error.into()),
        }
    }

    /// Check if orchestration succeeded
    pub fn is_successful(&self) -> bool {
        self.success
    }
}

/// Core Orchestrator trait
///
/// Orchestrators implement this trait to coordinate multiple agents
/// in various patterns (sequential, parallel, hierarchical, etc.).
#[async_trait::async_trait]
pub trait Orchestrator: Send + Sync {
    /// Orchestrator name (must be unique)
    fn name(&self) -> &str;

    /// Orchestrator description (what pattern it uses)
    fn description(&self) -> &str;

    /// Execute orchestration with the provided agents and input
    async fn orchestrate(
        &self,
        agents: Vec<Box<dyn Agent>>,
        input: OrchestratorInput,
    ) -> Result<OrchestratorOutput>;
}

/// Base orchestrator that provides common functionality
pub struct BaseOrchestrator {
    name: String,
    description: String,
}

impl BaseOrchestrator {
    /// Create a new base orchestrator
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }

    /// Get orchestrator name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get orchestrator description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Execute an agent with retry logic
    pub async fn execute_agent_with_retry(
        &self,
        agent: &dyn Agent,
        input: AgentInput,
        max_retries: usize,
    ) -> AgentOutput {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match agent.execute(input.clone()).await {
                Ok(output) => return output,
                Err(e) => {
                    last_error = Some(e.to_string());
                    if attempt < max_retries {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            100 * 2_u64.pow(attempt as u32),
                        ))
                        .await;
                    }
                },
            }
        }

        // All retries failed
        AgentOutput::new(format!(
            "Agent {} failed after {} retries: {}",
            agent.name(),
            max_retries,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        ))
        .with_confidence(0.0)
    }

    /// Convert orchestrator input to agent input
    pub fn input_to_agent_input(&self, input: &OrchestratorInput) -> AgentInput {
        AgentInput::new(&input.content)
            .with_context(input.context.clone())
            .with_metadata("orchestrator", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::agent::SimpleAgent;

    #[tokio::test]
    async fn test_orchestrator_input() {
        let input = OrchestratorInput::new("Test content")
            .with_context(serde_json::json!({"key": "value"}))
            .with_metadata("meta1", "value1");

        assert_eq!(input.content, "Test content");
        assert_eq!(input.context["key"], "value");
        assert_eq!(input.metadata["meta1"], "value1");
    }

    #[tokio::test]
    async fn test_orchestrator_output() {
        use crate::orchestration::context::ExecutionTrace;

        let trace = ExecutionTrace::new();
        let outputs = vec![AgentOutput::new("result1")];

        let success = OrchestratorOutput::success("Final result", outputs, trace.clone());
        assert!(success.is_successful());
        assert_eq!(success.result, "Final result");
        assert!(success.error.is_none());

        let failure = OrchestratorOutput::failure("Something went wrong", trace);
        assert!(!failure.is_successful());
        assert_eq!(failure.error, Some("Something went wrong".to_string()));
    }

    #[tokio::test]
    async fn test_base_orchestrator() {
        let orchestrator = BaseOrchestrator::new("TestOrchestrator", "A test orchestrator");

        assert_eq!(orchestrator.name(), "TestOrchestrator");
        assert_eq!(orchestrator.description(), "A test orchestrator");
    }

    #[tokio::test]
    async fn test_execute_agent_with_retry_success() {
        let orchestrator = BaseOrchestrator::new("Test", "Test");

        let agent = SimpleAgent::new("TestAgent", "Test", |input| {
            Ok(AgentOutput::new(format!("Processed: {}", input.content)))
        });

        let input = AgentInput::new("Hello");
        let output = orchestrator
            .execute_agent_with_retry(&agent, input, 3)
            .await;

        assert!(output.is_successful());
        assert_eq!(output.content, "Processed: Hello");
    }

    #[tokio::test]
    async fn test_execute_agent_with_retry_failure() {
        let orchestrator = BaseOrchestrator::new("Test", "Test");

        let agent = SimpleAgent::new("FailingAgent", "Always fails", |_input| {
            Err(anyhow::anyhow!("Always fails").into())
        });

        let input = AgentInput::new("Hello");
        let output = orchestrator
            .execute_agent_with_retry(&agent, input, 2)
            .await;

        assert!(!output.is_successful());
        assert!(output.content.contains("failed after"));
        assert_eq!(output.confidence, 0.0);
    }
}
