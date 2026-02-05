//! # Execution context and configuration
//!
//! This module provides the execution context for managing orchestration state,
//! including agent management, state tracking, and execution traces.

use crate::orchestration::agent::AgentOutput;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;

/// Execution configuration for orchestrators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Maximum time for entire orchestration
    pub timeout: Duration,

    /// Maximum number of retries per agent
    pub max_retries: usize,

    /// Maximum parallel agent executions
    pub parallel_limit: usize,

    /// Enable detailed logging
    pub enable_logging: bool,

    /// Enable execution tracing
    pub enable_tracing: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300), // 5 minutes
            max_retries: 3,
            parallel_limit: 10,
            enable_logging: true,
            enable_tracing: true,
        }
    }
}

impl ExecutionConfig {
    /// Create a new execution config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set parallel limit
    pub fn with_parallel_limit(mut self, parallel_limit: usize) -> Self {
        self.parallel_limit = parallel_limit;
        self
    }

    /// Enable logging
    pub fn with_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }

    /// Enable tracing
    pub fn with_tracing(mut self, enable: bool) -> Self {
        self.enable_tracing = enable;
        self
    }
}

/// Execution trace for tracking orchestration runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,

    /// End time
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,

    /// Agent executions
    pub agent_executions: Vec<AgentExecution>,

    /// Total execution duration in milliseconds
    pub duration_ms: Option<u64>,
}

impl Default for ExecutionTrace {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionTrace {
    /// Create a new execution trace
    pub fn new() -> Self {
        Self {
            start_time: chrono::Utc::now(),
            end_time: None,
            agent_executions: Vec::new(),
            duration_ms: None,
        }
    }

    /// Add an agent execution record
    pub fn add_execution(&mut self, execution: AgentExecution) {
        self.agent_executions.push(execution);
    }

    /// Mark the trace as complete
    pub fn complete(&mut self) {
        self.end_time = Some(chrono::Utc::now());
        self.duration_ms = Some(
            self.end_time
                .unwrap()
                .signed_duration_since(self.start_time)
                .num_milliseconds() as u64,
        );
    }

    /// Get total duration if completed
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.duration_ms
            .map(|ms| chrono::Duration::milliseconds(ms as i64))
    }
}

/// Record of a single agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecution {
    /// Agent name
    pub agent_name: String,

    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,

    /// End time
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,

    /// Input to agent
    pub input: crate::orchestration::agent::AgentInput,

    /// Output from agent
    pub output: Option<AgentOutput>,

    /// Whether execution succeeded
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,

    /// Execution duration in milliseconds
    pub duration_ms: Option<u64>,
}

impl AgentExecution {
    /// Create a new agent execution record
    pub fn new(
        agent_name: impl Into<String>,
        input: crate::orchestration::agent::AgentInput,
    ) -> Self {
        Self {
            agent_name: agent_name.into(),
            start_time: chrono::Utc::now(),
            end_time: None,
            input,
            output: None,
            success: false,
            error: None,
            duration_ms: None,
        }
    }

    /// Mark execution as successful with output
    pub fn succeed(&mut self, output: AgentOutput) {
        self.success = true;
        self.output = Some(output);
        self.end_time = Some(chrono::Utc::now());
        self.duration_ms = Some(
            self.end_time
                .unwrap()
                .signed_duration_since(self.start_time)
                .num_milliseconds() as u64,
        );
    }

    /// Mark execution as failed with error
    pub fn fail(&mut self, error: impl Into<String>) {
        self.success = false;
        self.error = Some(error.into());
        self.end_time = Some(chrono::Utc::now());
        self.duration_ms = Some(
            self.end_time
                .unwrap()
                .signed_duration_since(self.start_time)
                .num_milliseconds() as u64,
        );
    }
}

/// Execution context for managing orchestration state
pub struct ExecutionContext {
    /// Configuration
    config: ExecutionConfig,

    /// State storage
    state: RwLock<HashMap<String, serde_json::Value>>,

    /// Execution trace
    trace: RwLock<ExecutionTrace>,
}

impl Clone for ExecutionContext {
    fn clone(&self) -> Self {
        // Create a new context with same config but empty state and trace
        Self {
            config: self.config.clone(),
            state: RwLock::new(HashMap::new()),
            trace: RwLock::new(ExecutionTrace::new()),
        }
    }
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(config: ExecutionConfig) -> Self {
        Self {
            config,
            state: RwLock::new(HashMap::new()),
            trace: RwLock::new(ExecutionTrace::new()),
        }
    }

    /// Get configuration
    pub fn config(&self) -> &ExecutionConfig {
        &self.config
    }

    /// Get state value
    pub async fn get_state(&self, key: &str) -> Option<serde_json::Value> {
        let state = self.state.read().await;
        state.get(key).cloned()
    }

    /// Set state value
    pub async fn set_state(&self, key: impl Into<String>, value: serde_json::Value) {
        let mut state = self.state.write().await;
        state.insert(key.into(), value);
    }

    /// Remove state value
    pub async fn remove_state(&self, key: &str) -> Option<serde_json::Value> {
        let mut state = self.state.write().await;
        state.remove(key)
    }

    /// Clear all state
    pub async fn clear_state(&self) {
        let mut state = self.state.write().await;
        state.clear();
    }

    /// Get execution trace
    pub async fn get_trace(&self) -> ExecutionTrace {
        self.trace.read().await.clone()
    }

    /// Add agent execution to trace
    pub async fn add_execution(&self, execution: AgentExecution) {
        let mut trace = self.trace.write().await;
        trace.add_execution(execution);
    }

    /// Complete execution trace
    pub async fn complete_trace(&self) {
        let mut trace = self.trace.write().await;
        trace.complete();
    }

    /// Check if logging is enabled
    pub fn is_logging_enabled(&self) -> bool {
        self.config.enable_logging
    }

    /// Check if tracing is enabled
    pub fn is_tracing_enabled(&self) -> bool {
        self.config.enable_tracing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_config() {
        let config = ExecutionConfig::new()
            .with_timeout(Duration::from_secs(60))
            .with_max_retries(5)
            .with_parallel_limit(20)
            .with_logging(false)
            .with_tracing(false);

        assert_eq!(config.timeout.as_secs(), 60);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.parallel_limit, 20);
        assert!(!config.enable_logging);
        assert!(!config.enable_tracing);
    }

    #[tokio::test]
    async fn test_execution_context() {
        let config = ExecutionConfig::new();
        let ctx = ExecutionContext::new(config);

        // Test state management
        ctx.set_state("key1", serde_json::json!("value1")).await;
        assert_eq!(
            ctx.get_state("key1").await,
            Some(serde_json::json!("value1"))
        );

        ctx.set_state("key2", serde_json::json!(42)).await;
        assert_eq!(ctx.get_state("key2").await, Some(serde_json::json!(42)));

        assert_eq!(
            ctx.remove_state("key1").await,
            Some(serde_json::json!("value1"))
        );
        assert!(ctx.get_state("key1").await.is_none());

        ctx.clear_state().await;
        assert!(ctx.get_state("key2").await.is_none());
    }

    #[test]
    fn test_execution_trace() {
        let mut trace = ExecutionTrace::new();
        assert!(trace.end_time.is_none());
        assert!(trace.duration_ms.is_none());

        trace.complete();
        assert!(trace.end_time.is_some());
        assert!(trace.duration_ms.is_some());
    }

    #[test]
    fn test_agent_execution() {
        let input = crate::orchestration::agent::AgentInput::new("test");
        let mut exec = AgentExecution::new("TestAgent", input);

        assert!(!exec.success);
        assert!(exec.output.is_none());
        assert!(exec.end_time.is_none());

        let output = AgentOutput::new("result").with_confidence(0.9);
        exec.succeed(output);

        assert!(exec.success);
        assert!(exec.output.is_some());
        assert!(exec.end_time.is_some());
        assert!(exec.duration_ms.is_some());
    }
}
