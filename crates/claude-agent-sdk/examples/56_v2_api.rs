//! Example: V2 API - Simplified TypeScript-style API
//!
//! This example demonstrates the V2 API which provides a simplified,
//! TypeScript-inspired interface for interacting with Claude.
//!
//! What it demonstrates:
//! 1. One-shot prompt() function for single queries
//! 2. Session-based API with create_session() and resume_session()
//! 3. Simplified SessionOptions with common parameters
//! 4. TypeScript-style naming: prompt, send, receive

use claude_agent_sdk::v2::{create_session, prompt, resume_session, SessionOptions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== V2 API Examples ===\n");

    one_shot_prompt_example().await?;
    session_example().await?;
    session_options_example().await?;

    Ok(())
}

/// Demonstrates the simplest way to query Claude - one-shot prompt
async fn one_shot_prompt_example() -> anyhow::Result<()> {
    println!("=== One-Shot Prompt Example ===\n");

    // Simple one-shot prompt with default options
    let result = prompt("What is 2 + 2? Answer briefly.", Default::default()).await?;

    println!("Question: What is 2 + 2?");
    println!("Answer: {}", result.content);
    println!(
        "Tokens used: {} input + {} output = {} total",
        result.input_tokens,
        result.output_tokens,
        result.total_tokens()
    );

    if let Some(model) = &result.model {
        println!("Model: {}", model);
    }

    println!();
    Ok(())
}

/// Demonstrates session-based multi-turn conversations
async fn session_example() -> anyhow::Result<()> {
    println!("=== Session Example ===\n");

    // Create a new session with default options
    let mut session = create_session(Default::default()).await?;
    println!("Created session: {}", session.id);

    // First turn
    session.send("My favorite color is blue. Remember this.").await?;
    let messages = session.receive().await?;

    for msg in messages {
        if let Some(text) = msg.as_text() {
            println!("Claude: {}", text);
        }
    }

    // Second turn - test memory
    session.send("What is my favorite color?").await?;
    let messages = session.receive().await?;

    for msg in messages {
        if let Some(text) = msg.as_text() {
            println!("Claude: {}", text);
        }
    }

    // Close the session
    session.close().await?;
    println!("Session closed.\n");

    Ok(())
}

/// Demonstrates using SessionOptions for configuration
async fn session_options_example() -> anyhow::Result<()> {
    println!("=== Session Options Example ===\n");

    // Create options using the builder pattern
    let options = SessionOptions::builder()
        .max_turns(3)
        .permission_mode(claude_agent_sdk::v2::PermissionMode::BypassPermissions)
        .build();

    // Create session with custom options
    let mut session = create_session(options).await?;
    println!("Created session with custom options: {}", session.id);

    // Send a message
    session.send("Tell me a very short joke in one sentence.").await?;

    // Receive response
    let messages = session.receive().await?;
    for msg in messages {
        if let Some(text) = msg.as_text() {
            println!("Claude: {}", text);
        }
    }

    session.close().await?;

    // Demonstrate session resumption
    println!("\n--- Session Resumption ---");
    let session_id = "custom-session-123";
    let resumed_session = resume_session(session_id, Default::default()).await?;
    println!("Resumed session with ID: {}", resumed_session.id);

    resumed_session.close().await?;

    println!();
    Ok(())
}
