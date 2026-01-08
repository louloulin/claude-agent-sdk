//! # Orchestration patterns
//!
//! This module contains various orchestration patterns for coordinating multiple agents.

pub mod sequential;
pub mod parallel;

// Re-export orchestrators
pub use sequential::SequentialOrchestrator;
pub use parallel::ParallelOrchestrator;
