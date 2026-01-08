//! # Orchestration patterns
//!
//! This module contains various orchestration patterns for coordinating multiple agents.

pub mod parallel;
pub mod sequential;

// Re-export orchestrators
pub use parallel::ParallelOrchestrator;
pub use sequential::SequentialOrchestrator;
