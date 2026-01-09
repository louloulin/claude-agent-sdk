//! # Sequential Orchestration Pattern
//!
//! Agents execute one after another, with each agent's output becoming the input
//! for the next agent in the sequence.
//!
//! ```text
//! Input → Agent A → Agent B → Agent C → Output
//! ```
//!
//! Use cases:
//! - Data processing pipelines
//! - Multi-step reasoning
//! - Content generation and refinement

use crate::orchestration::{
    Result,
    agent::{Agent, AgentInput, AgentOutput},
    context::{AgentExecution, ExecutionContext},
    orchestrator::{BaseOrchestrator, Orchestrator, OrchestratorInput, OrchestratorOutput},
};

/// Sequential orchestrator that executes agents one after another
pub struct SequentialOrchestrator {
    base: BaseOrchestrator,
    max_retries: usize,
}

impl SequentialOrchestrator {
    /// Create a new sequential orchestrator
    pub fn new() -> Self {
        Self {
            base: BaseOrchestrator::new(
                "SequentialOrchestrator",
                "Executes agents sequentially, passing each output to the next input",
            ),
            max_retries: 3,
        }
    }

    /// Set max retries per agent
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Execute agents sequentially
    async fn execute_sequential(
        &self,
        agents: Vec<Box<dyn Agent>>,
        mut input: AgentInput,
        ctx: &ExecutionContext,
    ) -> Result<Vec<AgentOutput>> {
        let mut outputs = Vec::new();

        for (index, agent) in agents.iter().enumerate() {
            // Create execution record
            let mut exec_record = AgentExecution::new(agent.name(), input.clone());

            if ctx.is_logging_enabled() {
                println!(
                    "[{}] Executing agent {}/{}: {}",
                    self.base.name(),
                    index + 1,
                    agents.len(),
                    agent.name()
                );
            }

            // Execute agent with retry
            let output = self
                .base
                .execute_agent_with_retry(agent.as_ref(), input.clone(), self.max_retries)
                .await;

            let success = output.is_successful();

            if success {
                exec_record.succeed(output.clone());
                outputs.push(output.clone());

                // Use this output as input for next agent
                input = AgentInput::new(&output.content)
                    .with_context(output.data.clone())
                    .with_metadata("previous_agent", agent.name());
            } else {
                exec_record.fail(output.content.clone());
                return Err(
                    crate::orchestration::errors::OrchestrationError::agent_failure(
                        agent.name(),
                        output.content,
                    ),
                );
            }

            // Add to trace if enabled
            if ctx.is_tracing_enabled() {
                ctx.add_execution(exec_record).await;
            }
        }

        Ok(outputs)
    }
}

impl Default for SequentialOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Orchestrator for SequentialOrchestrator {
    fn name(&self) -> &str {
        self.base.name()
    }

    fn description(&self) -> &str {
        self.base.description()
    }

    async fn orchestrate(
        &self,
        agents: Vec<Box<dyn Agent>>,
        input: OrchestratorInput,
    ) -> Result<OrchestratorOutput> {
        if agents.is_empty() {
            return Err(
                crate::orchestration::errors::OrchestrationError::invalid_config(
                    "At least one agent is required",
                ),
            );
        }

        // Create execution context
        let config = crate::orchestration::context::ExecutionConfig::new();
        let ctx = ExecutionContext::new(config);

        let agent_input = self.base.input_to_agent_input(&input);

        // Execute agents sequentially
        let outputs = match self.execute_sequential(agents, agent_input, &ctx).await {
            Ok(outputs) => outputs,
            Err(e) => {
                ctx.complete_trace().await;
                let trace = ctx.get_trace().await;
                return Ok(OrchestratorOutput::failure(e.to_string(), trace));
            },
        };

        // Complete trace
        ctx.complete_trace().await;
        let trace = ctx.get_trace().await;

        // Get final result
        let final_output = outputs.last().unwrap();
        let result = final_output.content.clone();

        Ok(OrchestratorOutput::success(result, outputs, trace))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::agent::SimpleAgent;

    #[tokio::test]
    async fn test_sequential_orchestrator() {
        let orchestrator = SequentialOrchestrator::new();

        // Create three simple agents
        let agent1 = SimpleAgent::new("Agent1", "First agent", |input| {
            Ok(AgentOutput::new(format!("Step 1: {}", input.content)).with_metadata("step", "1"))
        });

        let agent2 = SimpleAgent::new("Agent2", "Second agent", |input| {
            Ok(AgentOutput::new(format!("Step 2: {}", input.content)).with_metadata("step", "2"))
        });

        let agent3 = SimpleAgent::new("Agent3", "Third agent", |input| {
            Ok(AgentOutput::new(format!("Step 3: {}", input.content)).with_metadata("step", "3"))
        });

        let agents: Vec<Box<dyn Agent>> =
            vec![Box::new(agent1), Box::new(agent2), Box::new(agent3)];

        let input = OrchestratorInput::new("Initial input");

        let output = orchestrator.orchestrate(agents, input).await.unwrap();

        assert!(output.is_successful());
        assert_eq!(output.agent_outputs.len(), 3);
        assert_eq!(output.agent_outputs[0].content, "Step 1: Initial input");
        assert_eq!(
            output.agent_outputs[1].content,
            "Step 2: Step 1: Initial input"
        );
        assert_eq!(
            output.agent_outputs[2].content,
            "Step 3: Step 2: Step 1: Initial input"
        );
        assert_eq!(output.result, "Step 3: Step 2: Step 1: Initial input");
    }

    #[tokio::test]
    async fn test_sequential_orchestrator_empty_agents() {
        let orchestrator = SequentialOrchestrator::new();
        let agents: Vec<Box<dyn Agent>> = vec![];
        let input = OrchestratorInput::new("Test");

        let result = orchestrator.orchestrate(agents, input).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::orchestration::errors::OrchestrationError::InvalidConfig(_)
        ));
    }

    #[tokio::test]
    async fn test_sequential_orchestrator_with_retry() {
        let orchestrator = SequentialOrchestrator::new().with_max_retries(2);

        let call_count = std::sync::atomic::AtomicUsize::new(0);

        let agent = SimpleAgent::new("FlakyAgent", "Sometimes fails", move |input| {
            let count = call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if count < 2 {
                // Fail first two times
                Err(anyhow::anyhow!("Temporary failure").into())
            } else {
                Ok(AgentOutput::new(format!("Success: {}", input.content)))
            }
        });

        let agents: Vec<Box<dyn Agent>> = vec![Box::new(agent)];
        let input = OrchestratorInput::new("Test");

        let output = orchestrator.orchestrate(agents, input).await.unwrap();

        assert!(output.is_successful());
        assert_eq!(output.agent_outputs[0].content, "Success: Test");
    }
}
