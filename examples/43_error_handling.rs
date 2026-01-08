//! Error Handling Example
//!
//! This example demonstrates comprehensive error handling patterns
//! when working with the Claude Agent SDK.

use claude_agent_sdk_rs::{query, ClaudeAgentOptions, QueryError, Message};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Error Handling Example ===\n");

    // Example 1: Handle API errors gracefully
    println!("1. API Error Handling:");
    match handle_api_errors().await {
        Ok(_) => println!("   ✓ Query succeeded"),
        Err(e) => println!("   ✗ Query failed: {}", e),
    }

    // Example 2: Handle timeout errors
    println!("\n2. Timeout Handling:");
    match handle_timeout().await {
        Ok(_) => println!("   ✓ Query completed in time"),
        Err(e) => println!("   ✗ Query timed out: {}", e),
    }

    // Example 3: Handle permission errors
    println!("\n3. Permission Error Handling:");
    match handle_permission_denied().await {
        Ok(_) => println!("   ✓ Permission granted"),
        Err(e) => println!("   ✗ Permission denied: {}", e),
    }

    // Example 4: Handle tool execution errors
    println!("\n4. Tool Error Handling:");
    match handle_tool_errors().await {
        Ok(messages) => println!("   ✓ Tools executed successfully, {} messages", messages.len()),
        Err(e) => println!("   ✗ Tool execution failed: {}", e),
    }

    // Example 5: Custom error recovery
    println!("\n5. Custom Error Recovery:");
    match custom_recovery_strategy().await {
        Ok(result) => println!("   ✓ Recovered and got result: {}", result),
        Err(e) => println!("   ✗ Recovery failed: {}", e),
    }

    println!("\n=== Error Handling Complete ===");
    Ok(())
}

/// Example 1: Handle API-level errors
async fn handle_api_errors() -> Result<()> {
    let options = ClaudeAgentOptions::builder()
        .max_turns(3)
        .build();

    // This might fail due to network issues, API errors, etc.
    match query("What is 2 + 2?", Some(options)).await {
        Ok(messages) => {
            println!("   Got {} messages", messages.len());
            Ok(())
        }
        Err(QueryError::Api(e)) => {
            // Handle API-specific errors
            eprintln!("   API Error: {}", e);
            Err(e.into())
        }
        Err(QueryError::Transport(e)) => {
            // Handle transport/communication errors
            eprintln!("   Transport Error: {}", e);
            Err(e.into())
        }
        Err(e) => {
            // Handle other errors
            eprintln!("   Unexpected Error: {}", e);
            Err(e.into())
        }
    }
}

/// Example 2: Handle timeout scenarios
async fn handle_timeout() -> Result<()> {
    use std::time::Duration;

    // In a real scenario, you might use tokio::time::timeout
    let timeout_duration = Duration::from_secs(30);

    println!("   Setting timeout to {} seconds", timeout_duration.as_secs());

    // For this example, we just demonstrate the pattern
    // In production, you would wrap the query in tokio::time::timeout
    let _ = tokio::time::timeout(
        timeout_duration,
        query("Quick question: What's the capital of France?", None)
    ).await
    .map_err(|_| anyhow::anyhow!("Query timed out after {:?}", timeout_duration))??;

    Ok(())
}

/// Example 3: Handle permission-related errors
async fn handle_permission_denied() -> Result<()> {
    use claude_agent_sdk_rs::PermissionMode;

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    // With BypassPermissions, we shouldn't get permission errors
    let messages = query("Create a test file", Some(options)).await?;

    println!("   Successfully executed without permission prompts");
    println!("   Received {} messages", messages.len());

    Ok(())
}

/// Example 4: Handle tool execution errors
async fn handle_tool_errors() -> Result<Vec<Message>> {
    use claude_agent_sdk_rs::{ClaudeAgentOptions, tools};

    let options = ClaudeAgentOptions::builder()
        .allowed_tools(vec![
            "Read".to_string(),
            // Intentionally exclude Write to show handling
        ])
        .build();

    let messages = query("Try to read a file and then write to it", Some(options)).await?;

    // Check if any tool uses failed
    for msg in &messages {
        if let Message::Assistant(assistant_msg) = msg {
            for block in &assistant_msg.message.content {
                if let claude_agent_sdk_rs::ContentBlock::ToolUse(tool_use) = block {
                    if tool_use.name == "Bash" || tool_use.name == "Write" {
                        // Tool was attempted but might have failed
                        println!("   Tool {} was called", tool_use.name);
                    }
                }
            }
        }
    }

    Ok(messages)
}

/// Example 5: Custom error recovery strategies
async fn custom_recovery_strategy() -> Result<String> {
    use claude_agent_sdk_rs::{fallback_model, ClaudeAgentOptions};

    // Strategy 1: Use fallback model on failure
    let options = ClaudeAgentOptions::builder()
        .model(Some("claude-opus-4-5".to_string()))
        .fallback_model(Some("claude-sonnet-4-5".to_string()))
        .max_turns(2)
        .build();

    let messages = query("What is Rust?", Some(options)).await?;

    // Strategy 2: Parse and validate responses
    for msg in &messages {
        if let Message::Assistant(assistant_msg) = msg {
            for block in &assistant_msg.message.content {
                if let claude_agent_sdk_rs::ContentBlock::Text(text) = block {
                    if !text.text.is_empty() {
                        // Successfully got a response
                        return Ok(text.text.clone());
                    }
                }
            }
        }
    }

    // Strategy 3: Default recovery value
    Err(anyhow::anyhow!("No valid response received"))
}

/// Example 6: Retry logic with exponential backoff
async fn retry_with_backoff() -> Result<()> {
    use std::time::Duration;

    let max_retries = 3;
    let mut attempt = 0;

    loop {
        attempt += 1;

        match query("Test query", None).await {
            Ok(_) => {
                println!("   Success on attempt {}", attempt);
                return Ok(());
            }
            Err(e) if attempt < max_retries => {
                let backoff = Duration::from_millis(100 * 2_u64.pow(attempt as u32));
                println!("   Attempt {} failed, retrying in {:?}", attempt, backoff);
                tokio::time::sleep(backoff).await;
            }
            Err(e) => {
                println!("   All {} attempts failed", max_retries);
                return Err(e.into());
            }
        }
    }
}

/// Example 7: Structured error handling
#[derive(Debug)]
enum AppError {
    QueryFailed(QueryError),
    InvalidResponse(String),
    Timeout(Duration),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::QueryFailed(e) => write!(f, "Query failed: {}", e),
            AppError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            AppError::Timeout(d) => write!(f, "Operation timed out after {:?}", d),
        }
    }
}

impl std::error::Error for AppError {}

async fn structured_error_handling() -> Result<(), AppError> {
    query("Test", None)
        .await
        .map_err(AppError::QueryFailed)?;

    // Validate response
    // ...
    Ok(())
}
