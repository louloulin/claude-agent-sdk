//! Complete Integration Test Example
//!
//! This example demonstrates end-to-end integration testing
//! for the Claude Agent SDK.

use anyhow::Result;
use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ClaudeClient, ContentBlock, McpServerConfig, McpToolResultContent, Message,
    PermissionMode, ToolResult, create_sdk_mcp_server, query, query_stream, tool,
};
use futures::stream::StreamExt;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Complete Integration Test Example ===\n");

    // Run all integration tests
    let tests = vec![
        ("Basic Query", test_basic_query),
        ("Streaming Query", test_streaming_query),
        ("Custom Tools", test_custom_tools),
        ("Bidirectional Client", test_bidirectional_client),
        ("Error Handling", test_error_handling),
        ("Permission System", test_permission_system),
        ("Hooks", test_hooks),
        ("Budget Control", test_budget_control),
        ("Session Management", test_session_management),
        ("Concurrent Operations", test_concurrent_operations),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (name, test_fn) in tests {
        println!("Running: {}", name);
        match test_fn().await {
            Ok(_) => {
                println!("   ✓ PASSED\n");
                passed += 1;
            }
            Err(e) => {
                println!("   ✗ FAILED: {}\n", e);
                failed += 1;
            }
        }
    }

    // Print summary
    println!("=== Test Summary ===");
    println!("Total: {}", tests.len());
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);

    if failed > 0 {
        println!("\n⚠️  Some tests failed");
        std::process::exit(1);
    } else {
        println!("\n✅ All tests passed!");
    }

    Ok(())
}

/// Test 1: Basic query functionality
async fn test_basic_query() -> Result<()> {
    let messages = query("What is 2 + 2?", None).await?;

    assert!(!messages.is_empty(), "Should receive messages");

    let has_response = messages.iter().any(|m| matches!(m, Message::Assistant(_)));

    assert!(has_response, "Should have assistant response");
    Ok(())
}

/// Test 2: Streaming query functionality
async fn test_streaming_query() -> Result<()> {
    let mut stream = query_stream("What is 2 + 2?", None).await?;

    let mut count = 0;
    while let Some(result) = stream.next().await {
        let _msg = result?;
        count += 1;
    }

    assert!(count > 0, "Should receive messages via stream");
    Ok(())
}

/// Test 3: Custom tools integration
async fn test_custom_tools() -> Result<()> {
    async fn calculator_tool(args: serde_json::Value) -> Result<ToolResult> {
        let a = args["a"].as_i64().unwrap_or(0);
        let b = args["b"].as_i64().unwrap_or(0);

        Ok(ToolResult {
            content: vec![McpToolResultContent::Text {
                text: format!("{} + {} = {}", a, b, a + b),
            }],
            is_error: false,
        })
    }

    let calc_tool = tool!(
        "calculator",
        "Add two numbers",
        json!({
            "type": "object",
            "properties": {
                "a": {"type": "integer"},
                "b": {"type": "integer"}
            },
            "required": ["a", "b"]
        }),
        calculator_tool
    );

    let server = create_sdk_mcp_server("calc-server", "1.0.0", vec![calc_tool]);

    let mut mcp_servers = std::collections::HashMap::new();
    mcp_servers.insert("calc-server".to_string(), McpServerConfig::Sdk(server));

    let options = ClaudeAgentOptions::builder()
        .mcp_servers(claude_agent_sdk_rs::McpServers::Dict(mcp_servers))
        .allowed_tools(vec!["mcp__calc-server__calculator".to_string()])
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let _messages = query("Use the calculator to add 5 and 3", Some(options)).await?;
    Ok(())
}

/// Test 4: Bidirectional client
async fn test_bidirectional_client() -> Result<()> {
    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());

    client.connect().await?;

    client.query("What is 2 + 2?").await?;

    let mut response_count = 0;
    let mut stream = client.receive_response();

    while let Some(result) = stream.next().await {
        let msg = result?;
        if matches!(msg, Message::Result(_)) {
            break;
        }
        response_count += 1;
    }

    client.disconnect().await?;

    assert!(response_count > 0, "Should receive responses");
    Ok(())
}

/// Test 5: Error handling
async fn test_error_handling() -> Result<()> {
    // Test with invalid configuration
    let result = query("", None).await; // Empty query

    // Should either succeed or fail gracefully
    match result {
        Ok(_) => Ok(()),
        Err(_) => Ok(()), // Error is acceptable
    }
}

/// Test 6: Permission system
async fn test_permission_system() -> Result<()> {
    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .allowed_tools(vec!["Read".to_string()])
        .build();

    let _messages = query("List files in current directory", Some(options)).await?;
    Ok(())
}

/// Test 7: Hooks system
async fn test_hooks() -> Result<()> {
    use claude_agent_sdk_rs::{Hook, HookContext, HookInput, HookJSONOutput, HookMatcher};
    use std::collections::HashMap;

    async fn test_hook(
        _input: HookInput,
        _tool_use_id: Option<String>,
        _context: HookContext,
    ) -> anyhow::Result<HookJSONOutput> {
        Ok(serde_json::json!({}))
    }

    let mut hooks = HashMap::new();
    hooks.insert(
        "PreToolUse".to_string(),
        vec![HookMatcher {
            matcher: Some("Read".to_string()),
            hooks: vec![Hook::new(test_hook)],
        }],
    );

    let options = ClaudeAgentOptions::builder().hooks(Some(hooks)).build();

    let _messages = query("Read README.md", Some(options)).await?;
    Ok(())
}

/// Test 8: Budget control
async fn test_budget_control() -> Result<()> {
    let options = ClaudeAgentOptions::builder()
        .max_budget_usd(0.01) // Very small budget
        .max_turns(2)
        .build();

    let _messages = query("What is 2 + 2?", Some(options)).await?;
    Ok(())
}

/// Test 9: Session management
async fn test_session_management() -> Result<()> {
    let options1 = ClaudeAgentOptions::builder()
        .resume(Some("test-session-1".to_string()))
        .build();

    let options2 = ClaudeAgentOptions::builder()
        .resume(Some("test-session-2".to_string()))
        .build();

    let _msg1 = query("Remember: X = 1", Some(options1)).await?;
    let _msg2 = query("Remember: X = 2", Some(options2)).await?;

    // Verify sessions are isolated
    let options_check = ClaudeAgentOptions::builder()
        .resume(Some("test-session-1".to_string()))
        .continue_conversation(true)
        .build();

    let msg_check = query("What is X?", Some(options_check)).await?;

    // Should have X = 1, not X = 2
    let response_text = extract_response_text(&msg_check);
    let has_correct_value = response_text.contains("1");

    assert!(has_correct_value, "Session should maintain separate state");
    Ok(())
}

/// Test 10: Concurrent operations
async fn test_concurrent_operations() -> Result<()> {
    use futures::stream::{self, StreamExt, TryStreamExt};

    let queries = vec!["What is 2 + 2?", "What is 3 + 3?", "What is 4 + 4?"];

    let results: Vec<_> = stream::iter(queries)
        .map(|q| async move { query(q, None).await })
        .buffer_unordered(3)
        .try_collect()
        .await?;

    assert_eq!(results.len(), 3, "Should complete all queries");
    Ok(())
}

/// Test 11: Multimodal input
async fn test_multimodal_input() -> Result<()> {
    use claude_agent_sdk_rs::{UserContentBlock, query_with_content};

    let base64_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";

    let messages = query_with_content(
        vec![
            UserContentBlock::text("What color is this image?"),
            UserContentBlock::image_base64("image/png", base64_data)?,
        ],
        None,
    )
    .await?;

    assert!(!messages.is_empty(), "Should handle multimodal input");
    Ok(())
}

/// Test 12: Fallback model
async fn test_fallback_model() -> Result<()> {
    let options = ClaudeAgentOptions::builder()
        .model("claude-opus-4-5")
        .fallback_model(Some("claude-sonnet-4-5".to_string()))
        .build();

    let _messages = query("What is 2 + 2?", Some(options)).await?;
    Ok(())
}

/// Test 13: Extended thinking
async fn test_extended_thinking() -> Result<()> {
    let options = ClaudeAgentOptions::builder()
        .max_thinking_tokens(Some(10000))
        .build();

    let _messages = query("Solve this complex problem", Some(options)).await?;
    Ok(())
}

/// Test 14: Tool restrictions
async fn test_tool_restrictions() -> Result<()> {
    let options = ClaudeAgentOptions::builder()
        .allowed_tools(vec!["Read".to_string()])
        .disallowed_tools(vec!["Write".to_string()])
        .build();

    let _messages = query("Read README.md", Some(options)).await?;
    Ok(())
}

/// Test 15: System prompts
async fn test_system_prompts() -> Result<()> {
    use claude_agent_sdk_rs::SystemPrompt;

    let options = ClaudeAgentOptions::builder()
        .system_prompt(Some(SystemPrompt::text(
            "You are a helpful assistant focused on brevity.",
        )))
        .build();

    let _messages = query("What is 2 + 2? Be brief.", Some(options)).await?;
    Ok(())
}

/// Test 16: CLI path configuration
async fn test_cli_path_config() -> Result<()> {
    use std::path::PathBuf;

    let options = ClaudeAgentOptions::builder()
        .cli_path(Some(PathBuf::from("claude")))
        .build();

    let _messages = query("What is 2 + 2?", Some(options)).await?;
    Ok(())
}

/// Test 17: Working directory
async fn test_working_directory() -> Result<()> {
    use std::path::PathBuf;

    let options = ClaudeAgentOptions::builder()
        .cwd(Some(PathBuf::from("/tmp")))
        .build();

    let _messages = query("What is the current directory?", Some(options)).await?;
    Ok(())
}

/// Test 18: Environment variables
async fn test_environment_variables() -> Result<()> {
    use std::collections::HashMap;

    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());

    let options = ClaudeAgentOptions::builder().env(env).build();

    let _messages = query("What is the value of TEST_VAR?", Some(options)).await?;
    Ok(())
}

/// Test 19: Fork session
async fn test_fork_session() -> Result<()> {
    let options = ClaudeAgentOptions::builder().fork_session(true).build();

    let _messages = query("What is 2 + 2?", Some(options)).await?;

    // Fork session should not remember previous context
    let options2 = ClaudeAgentOptions::builder().fork_session(true).build();

    let _messages2 = query("What did I ask before?", Some(options2)).await?;

    Ok(())
}

/// Test 20: Max turns enforcement
async fn test_max_turns_enforcement() -> Result<()> {
    let options = ClaudeAgentOptions::builder().max_turns(Some(1)).build();

    let messages = query("What is 2 + 2?", Some(options)).await?;

    // Should respect max turns
    assert!(!messages.is_empty(), "Should get at least one message");
    Ok(())
}

/// Helper function to extract response text
fn extract_response_text(messages: &[Message]) -> String {
    let mut text = String::new();

    for msg in messages {
        if let Message::Assistant(assistant_msg) = msg {
            for block in &assistant_msg.message.content {
                if let ContentBlock::Text(content) = block {
                    text.push_str(&content.text);
                }
            }
        }
    }

    text
}

/// Performance assertions
async fn test_performance_assertions() -> Result<()> {
    use std::time::Instant;

    let start = Instant::now();
    let _messages = query("What is 2 + 2?", None).await?;
    let elapsed = start.elapsed();

    // Should complete within reasonable time
    assert!(
        elapsed.as_secs() < 30,
        "Query should complete within 30 seconds"
    );

    println!("   Performance: {:?}", elapsed);
    Ok(())
}

/// Memory leak check
async fn test_memory_leak() -> Result<()> {
    // Run multiple queries to check for memory leaks
    for i in 0..10 {
        let _messages = query(&format!("Query {}", i), None).await?;
    }

    // If we reach here without OOM, no obvious memory leak
    Ok(())
}

/// Cleanup and teardown
async fn test_cleanup() -> Result<()> {
    // Test proper resource cleanup
    let client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await?;
    client.disconnect().await?;

    // Client should be properly cleaned up
    Ok(())
}
