//! # Multi-Agent Orchestration Examples
//!
//! This example demonstrates how to use the multi-agent orchestration framework
//! to coordinate multiple AI agents with different patterns.
//!
//! ## Patterns Demonstrated
//!
//! 1. **Sequential Orchestration**: Agents execute one after another, with each output
//!    becoming the input for the next agent.
//!
//! 2. **Parallel Orchestration**: Agents execute simultaneously, and their outputs are
//!    aggregated.
//!
//! ## Running the Example
//!
//! ```bash
//! cargo run --example 51_orchestration
//! ```

use claude_agent_sdk::orchestration::{
    Agent, AgentOutput, Orchestrator, OrchestratorInput, ParallelOrchestrator,
    SequentialOrchestrator,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// ============================================================================
// Simple Agent Creators
// ============================================================================

/// Create a researcher agent that gathers information
fn create_researcher() -> Box<dyn Agent> {
    Box::new(claude_agent_sdk::orchestration::agent::SimpleAgent::new(
        "Researcher",
        "Gathers information on topics",
        |input| {
            Ok(AgentOutput::new(format!(
                "📚 Research on '{}': Found 5 relevant articles, 3 books, and 2 research papers.",
                input.content
            ))
            .with_confidence(0.95)
            .with_metadata("sources_count", "10"))
        },
    ))
}

/// Create a writer agent that creates content
fn create_writer() -> Box<dyn Agent> {
    Box::new(claude_agent_sdk::orchestration::agent::SimpleAgent::new(
        "Writer",
        "Creates structured content",
        |input| {
            Ok(AgentOutput::new(format!(
                "✍️ Article Draft: '{}'\n\nBased on the research, here's a well-structured article...",
                input.content
            ))
            .with_confidence(0.90)
            .with_metadata("word_count", "850"))
        },
    ))
}

/// Create an editor agent that refines content
fn create_editor() -> Box<dyn Agent> {
    Box::new(claude_agent_sdk::orchestration::agent::SimpleAgent::new(
        "Editor",
        "Reviews and improves content",
        |input| {
            Ok(AgentOutput::new(format!(
                "✅ Edited Version: '{}'\n\nImproved grammar, flow, and clarity. Ready for publication!",
                input.content
            ))
            .with_confidence(0.92)
            .with_metadata("changes_made", "15"))
        },
    ))
}

/// Create a critic agent that provides feedback
fn create_critic(name: &'static str, perspective: &'static str) -> Box<dyn Agent> {
    Box::new(claude_agent_sdk::orchestration::agent::SimpleAgent::new(
        name,
        format!("Provides {} perspective", perspective),
        move |input| {
            Ok(AgentOutput::new(format!(
                "👤 {} ({}) Opinion: '{}'\n\nFrom a {} standpoint, this looks promising with room for growth.",
                name, perspective, input.content, perspective
            ))
            .with_confidence(0.85))
        },
    ))
}

/// Create an analyzer agent that evaluates different aspects
fn create_analyzer(aspect: &'static str) -> Box<dyn Agent> {
    Box::new(claude_agent_sdk::orchestration::agent::SimpleAgent::new(
        format!("{}_Analyzer", aspect),
        format!("Analyzes {}", aspect),
        move |input| {
            Ok(AgentOutput::new(format!(
                "🔍 {} Analysis: '{}'\n\nThe {} aspects are well-developed.",
                aspect, input.content, aspect
            ))
            .with_confidence(0.88))
        },
    ))
}

// ============================================================================
// Example 1: Sequential Orchestration - Content Creation Pipeline
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     Multi-Agent Orchestration Framework Examples            ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Example 1: Sequential Pipeline
    println!("📌 Example 1: Sequential Content Creation Pipeline");
    println!("{}", "─".repeat(60));
    sequential_pipeline_example().await?;
    println!();

    // Example 2: Parallel Analysis
    println!("\n📌 Example 2: Parallel Multi-Perspective Analysis");
    println!("{}", "─".repeat(60));
    parallel_analysis_example().await?;
    println!();

    // Example 3: Parallel with Concurrency Control
    println!("\n📌 Example 3: Parallel Execution with Concurrency Control");
    println!("{}", "─".repeat(60));
    parallel_with_limit_example().await?;
    println!();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     All examples completed successfully! ✅               ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    Ok(())
}

/// Example 1: Sequential content creation pipeline
async fn sequential_pipeline_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating a content pipeline: Research → Write → Edit\n");

    // Create agents
    let agents: Vec<Box<dyn Agent>> = vec![
        create_researcher(),
        create_writer(),
        create_editor(),
    ];

    // Create orchestrator
    let orchestrator = SequentialOrchestrator::new().with_max_retries(2);

    // Execute
    let input = OrchestratorInput::new("Climate Change Solutions");
    let output = orchestrator.orchestrate(agents, input).await?;

    // Display results
    println!("Pipeline Results:");
    println!("  Success: {}", output.is_successful());
    println!("  Agents executed: {}", output.agent_outputs.len());
    println!(
        "  Execution time: {:?}\n",
        output.execution_trace.duration()
    );

    for (index, agent_output) in output.agent_outputs.iter().enumerate() {
        println!(
            "  Step {} - Confidence: {:.1}%",
            index + 1,
            agent_output.confidence * 100.0
        );
        println!("  {}", agent_output.content.lines().next().unwrap_or(""));
        println!();
    }

    println!("  Final Result:");
    println!("  {}", output.result.lines().next().unwrap_or(""));

    Ok(())
}

/// Example 2: Parallel multi-perspective analysis
async fn parallel_analysis_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("Analyzing from multiple perspectives in parallel\n");

    // Create agents with different perspectives
    let agents: Vec<Box<dyn Agent>> = vec![
        create_critic("Alice", "technical"),
        create_critic("Bob", "business"),
        create_critic("Charlie", "user-experience"),
        create_critic("Diana", "security"),
    ];

    // Create orchestrator
    let orchestrator = ParallelOrchestrator::new();

    // Execute
    let input = OrchestratorInput::new("New Cloud Architecture Proposal");
    let output = orchestrator.orchestrate(agents, input).await?;

    // Display results
    println!("Analysis Results:");
    println!("  Success: {}", output.is_successful());
    println!("  Parallel agents: {}", output.agent_outputs.len());
    println!(
        "  Execution time: {:?}\n",
        output.execution_trace.duration()
    );

    for agent_output in &output.agent_outputs {
        println!("  {}", agent_output.content.lines().next().unwrap_or(""));
    }

    Ok(())
}

/// Example 3: Parallel execution with concurrency limit
async fn parallel_with_limit_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running multiple agents with controlled concurrency (max 3)\n");

    // Track concurrent execution
    let concurrent_count = Arc::new(AtomicUsize::new(0));
    let max_concurrent = Arc::new(AtomicUsize::new(0));
    let mut agents: Vec<Box<dyn Agent>> = Vec::new();

    // Create 10 agents that track concurrency
    for i in 0..10 {
        let concurrent_clone = concurrent_count.clone();
        let max_clone = max_concurrent.clone();

        let agent = claude_agent_sdk::orchestration::agent::SimpleAgent::new(
            format!("Agent_{}", i),
            format!("Agent number {}", i),
            move |input| {
                // Increment concurrent count
                let current = concurrent_clone.fetch_add(1, Ordering::SeqCst);

                // Update max
                loop {
                    let current_max = max_clone.load(Ordering::SeqCst);
                    if current < current_max {
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

                // Simulate work
                std::thread::sleep(std::time::Duration::from_millis(50));

                // Decrement concurrent count
                concurrent_clone.fetch_sub(1, Ordering::SeqCst);

                Ok(AgentOutput::new(format!(
                    "Agent {} processed: {}",
                    i,
                    input.content.chars().take(30).collect::<String>()
                )))
            },
        );

        agents.push(Box::new(agent));
    }

    // Create orchestrator with limit
    let orchestrator = ParallelOrchestrator::new().with_parallel_limit(3);

    // Execute
    let input = OrchestratorInput::new("Parallel Task Batch");
    let output = orchestrator.orchestrate(agents, input).await?;

    // Display results
    println!("Execution Results:");
    println!("  Success: {}", output.is_successful());
    println!("  Total agents: {}", output.agent_outputs.len());
    println!(
        "  Max concurrent: {}",
        max_concurrent.load(Ordering::SeqCst)
    );
    println!(
        "  Execution time: {:?}\n",
        output.execution_trace.duration()
    );

    // Show sample outputs
    println!("Sample outputs (first 3):");
    for (index, agent_output) in output.agent_outputs.iter().take(3).enumerate() {
        println!("  {}. {}", index + 1, agent_output.content);
    }

    Ok(())
}
