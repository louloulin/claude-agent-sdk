//! # Multi-Agent Orchestration Framework
//!
//! This module provides a comprehensive framework for orchestrating multiple AI agents
//! to collaborate on complex tasks. It supports various orchestration patterns including
//! sequential, parallel, hierarchical, debate, and routing modes.
//!
//! ## Features
//!
//! - **Flexible Orchestration Patterns**: 5+ built-in patterns for different use cases
//! - **Type-Safe Agent Interface**: Strongly typed agent definitions with Rust's trait system
//! - **Async-First Design**: Full async/await support with Tokio
//! - **Execution Tracking**: Comprehensive execution traces for debugging and monitoring
//! - **Error Recovery**: Built-in retry logic and graceful degradation
//! - **Extensible**: Easy to add custom agents and orchestrators
//!
//! ## Quick Start
//!
//! ```rust,no_run,ignore
//! use claude_agent_sdk_rs::orchestration::{
//!     Agent, AgentInput, AgentOutput,
//!     SequentialOrchestrator, Orchestrator,
//! };
//! use async_trait::async_trait;
//!
//! // Define a simple agent
//! struct Researcher;
//!
//! #[async_trait]
//! impl Agent for Researcher {
//!     fn name(&self) -> &str {
//!         "Researcher"
//!     }
//!
//!     fn description(&self) -> &str {
//!         "Researches topics and gathers information"
//!     }
//!
//!     async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
//!         // Agent implementation...
//!         Ok(AgentOutput {
//!             content: "Research complete".to_string(),
//!             data: serde_json::json!({}),
//!             confidence: 0.9,
//!             metadata: std::collections::HashMap::new(),
//!         })
//!     }
//! }
//!
//! // Use orchestrator
//! # async fn example() -> anyhow::Result<()> {
//! let agents: Vec<Box<dyn Agent>> = vec
//![Box::new(Researcher)];
//! let orchestrator = SequentialOrchestrator::new(agents);
//! let output = orchestrator.orchestrate(input).await?;
//! # Ok(())
//! # }
//! ```

pub mod agent;
pub mod context;
pub mod errors;
pub mod orchestrator;
pub mod patterns;
pub mod registry;

// Re-export commonly used types
pub use agent::{Agent, AgentInput, AgentOutput};
pub use context::{ExecutionConfig, ExecutionContext, ExecutionTrace};
pub use errors::{OrchestrationError, Result};
pub use orchestrator::{Orchestrator, OrchestratorInput, OrchestratorOutput};
pub use registry::{AgentFilter, AgentMetadata, AgentRegistry, AgentRegistryBuilder, RegistryError};

pub use patterns::{parallel::ParallelOrchestrator, sequential::SequentialOrchestrator};
