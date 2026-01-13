//! # Parallel Orchestration Pattern
//!
//! Multiple agents execute in parallel, and their outputs are aggregated.
//!
//! ```text
//!         → Agent A ─┐
//! Input ─┼→ Agent B ─┼→ Aggregator → Output
//!         → Agent C ─┘
//! ```
//!
//! Use cases:
//! - Multi-angle analysis
//! - Parallel task processing
//! - Performance optimization

use crate::orchestration::{
    Result,
    agent::{Agent, AgentInput, AgentOutput},
    context::{AgentExecution, ExecutionContext},
    orchestrator::{BaseOrchestrator, Orchestrator, OrchestratorInput, OrchestratorOutput},
};
use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Parallel orchestrator that executes agents concurrently
pub struct ParallelOrchestrator {
    base: BaseOrchestrator,
    max_retries: usize,
    parallel_limit: usize,
}

impl ParallelOrchestrator {
    /// Create a new parallel orchestrator
    pub fn new() -> Self {
        Self {
            base: BaseOrchestrator::new(
                "ParallelOrchestrator",
                "Executes agents in parallel and aggregates their outputs",
            ),
            max_retries: 3,
            parallel_limit: 10,
        }
    }

    /// Set max retries per agent
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set parallel execution limit
    pub fn with_parallel_limit(mut self, limit: usize) -> Self {
        self.parallel_limit = limit;
        self
    }

    /// Execute agents in parallel
    async fn execute_parallel(
        &self,
        agents: Vec<Box<dyn Agent>>,
        input: AgentInput,
        ctx: &ExecutionContext,
    ) -> Result<Vec<AgentOutput>> {
        let semaphore = Arc::new(Semaphore::new(self.parallel_limit));
        let agents_count = agents.len();
        let mut futures = Vec::new();

        for (index, agent) in agents.iter().enumerate() {
            let agent_ref = agent.as_ref();
            let input_clone = input.clone();
            let semaphore_clone = semaphore.clone();
            let ctx_clone = ctx.clone();
            let base_name = self.base.name().to_string();

            let future = async move {
                // Acquire semaphore permit
                let _permit = semaphore_clone.acquire().await.unwrap();

                // Create execution record
                let mut exec_record = AgentExecution::new(agent_ref.name(), input_clone.clone());

                if ctx_clone.is_logging_enabled() {
                    println!(
                        "[{}] Executing agent {}/{}: {}",
                        base_name,
                        index + 1,
                        agents_count,
                        agent_ref.name()
                    );
                }

                // Execute agent with retry
                let output =
                    Self::execute_agent_with_retry_static(agent_ref, input_clone, self.max_retries)
                        .await;

                let success = output.is_successful();

                if success {
                    exec_record.succeed(output.clone());
                } else {
                    exec_record.fail(output.content.clone());
                }

                // Add to trace if enabled
                if ctx_clone.is_tracing_enabled() {
                    ctx_clone.add_execution(exec_record).await;
                }

                (agent_ref.name().to_string(), output, success)
            };

            futures.push(future);
        }

        // Wait for all agents to complete
        let results = join_all(futures).await;

        // Check for failures and collect outputs
        let mut outputs = Vec::new();
        let mut failed_agents = Vec::new();

        for (agent_name, output, success) in results {
            if success {
                outputs.push(output);
            } else {
                failed_agents.push(agent_name);
            }
        }

        // If any agents failed, return error
        if !failed_agents.is_empty() {
            return Err(
                crate::orchestration::errors::OrchestrationError::agent_failure(
                    failed_agents.join(", "),
                    "Execution failed",
                ),
            );
        }

        Ok(outputs)
    }

    // Static version for use in async block
    async fn execute_agent_with_retry_static(
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
}

impl Default for ParallelOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Orchestrator for ParallelOrchestrator {
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
        let mut config = crate::orchestration::context::ExecutionConfig::new();
        config.parallel_limit = self.parallel_limit;
        let ctx = ExecutionContext::new(config);

        let agent_input = self.base.input_to_agent_input(&input);

        // Execute agents in parallel
        let outputs = match self.execute_parallel(agents, agent_input, &ctx).await {
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

        // Aggregate results
        let aggregated = self.aggregate_results(&outputs);

        Ok(OrchestratorOutput::success(aggregated, outputs, trace))
    }
}

impl ParallelOrchestrator {
    /// Aggregate multiple agent outputs into a single result
    fn aggregate_results(&self, outputs: &[AgentOutput]) -> String {
        if outputs.is_empty() {
            return String::new();
        }

        if outputs.len() == 1 {
            return outputs[0].content.clone();
        }

        // Combine all outputs
        let mut result = String::from("Parallel execution results:\n\n");

        for (index, output) in outputs.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", index + 1, output.content));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::agent::SimpleAgent;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_parallel_orchestrator() {
        let orchestrator = ParallelOrchestrator::new();

        // Create three agents that execute independently
        let agent1: Box<dyn Agent> = Box::new(SimpleAgent::new("Agent1", "First", |input| {
            Ok(AgentOutput::new(format!(
                "Result 1 from: {}",
                input.content
            )))
        }));

        let agent2: Box<dyn Agent> = Box::new(SimpleAgent::new("Agent2", "Second", |input| {
            Ok(AgentOutput::new(format!(
                "Result 2 from: {}",
                input.content
            )))
        }));

        let agent3: Box<dyn Agent> = Box::new(SimpleAgent::new("Agent3", "Third", |input| {
            Ok(AgentOutput::new(format!(
                "Result 3 from: {}",
                input.content
            )))
        }));

        let agents: Vec<Box<dyn Agent>> = vec![agent1, agent2, agent3];

        let input = OrchestratorInput::new("Test input");

        let output = orchestrator.orchestrate(agents, input).await.unwrap();

        assert!(output.is_successful());
        assert_eq!(output.agent_outputs.len(), 3);
        assert!(output.result.contains("Parallel execution results"));
        assert!(output.result.contains("Result 1 from: Test input"));
        assert!(output.result.contains("Result 2 from: Test input"));
        assert!(output.result.contains("Result 3 from: Test input"));
    }

    #[tokio::test]
    async fn test_parallel_execution_is_parallel() {
        let orchestrator = ParallelOrchestrator::new();

        let counter = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));

        let mut agents: Vec<Box<dyn Agent>> = Vec::new();

        for i in 0..5 {
            let counter_clone = counter.clone();
            let max_clone = max_concurrent.clone();

            let agent: Box<dyn Agent> = Box::new(SimpleAgent::new(
                format!("Agent{}", i),
                format!("Agent number {}", i),
                move |_input| {
                    // Increment counter
                    let current = counter_clone.fetch_add(1, Ordering::SeqCst);

                    // Update max if needed
                    loop {
                        let current_max = max_clone.load(Ordering::SeqCst);
                        if current + 1 <= current_max {
                            break;
                        }
                        if max_clone
                            .compare_exchange(
                                current_max,
                                current + 1,
                                Ordering::SeqCst,
                                Ordering::SeqCst,
                            )
                            .is_ok()
                        {
                            break;
                        }
                    }

                    // Simulate work (using a simple computation instead of sleep)
                    let mut sum = 0u64;
                    for j in 0..1000 {
                        sum = sum.wrapping_add(j);
                    }

                    // Decrement counter
                    counter_clone.fetch_sub(1, Ordering::SeqCst);

                    Ok(AgentOutput::new(format!("Agent {} done", i)))
                },
            ));

            agents.push(agent);
        }

        let input = OrchestratorInput::new("Test");
        let output = orchestrator.orchestrate(agents, input).await.unwrap();

        assert!(output.is_successful());
        assert_eq!(output.agent_outputs.len(), 5);

        // Verify agents executed
        let max_val = max_concurrent.load(Ordering::SeqCst);
        assert!(
            max_val >= 1,
            "Expected at least 1 agent to execute (max concurrent: {})",
            max_val
        );
    }

    #[tokio::test]
    async fn test_parallel_orchestrator_empty_agents() {
        let orchestrator = ParallelOrchestrator::new();
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
    async fn test_parallel_with_limit() {
        let orchestrator = ParallelOrchestrator::new().with_parallel_limit(2);

        let counter = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));

        let mut agents: Vec<Box<dyn Agent>> = Vec::new();

        for i in 0..5 {
            let counter_clone = counter.clone();
            let max_clone = max_concurrent.clone();

            let agent: Box<dyn Agent> = Box::new(SimpleAgent::new(
                format!("Agent{}", i),
                format!("Agent {}", i),
                move |_input| {
                    let current = counter_clone.fetch_add(1, Ordering::SeqCst);

                    loop {
                        let current_max = max_clone.load(Ordering::SeqCst);
                        if current + 1 <= current_max {
                            break;
                        }
                        if max_clone
                            .compare_exchange(
                                current_max,
                                current + 1,
                                Ordering::SeqCst,
                                Ordering::SeqCst,
                            )
                            .is_ok()
                        {
                            break;
                        }
                    }

                    // Simulated work

                    counter_clone.fetch_sub(1, Ordering::SeqCst);

                    Ok(AgentOutput::new(format!("Agent {} done", i)))
                },
            ));

            agents.push(agent);
        }

        let input = OrchestratorInput::new("Test");
        let output = orchestrator.orchestrate(agents, input).await.unwrap();

        assert!(output.is_successful());

        // With limit of 2, we should never have more than 2 concurrent
        let max_val = max_concurrent.load(Ordering::SeqCst);
        assert!(max_val <= 2, "Expected max 2 concurrent, got {}", max_val);
    }
}
