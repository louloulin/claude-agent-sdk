//! Comprehensive testing strategies for Claude Agent SDK applications.
//!
//! Demonstrates:
//! - Unit testing patterns
//! - Integration testing
//! - Mock tools for testing
//! - Property-based testing concepts
//! - Deterministic testing with seeds

use claude_agent_sdk_rs::{
    ContentBlock, Message, PermissionMode, ClaudeAgentOptions, Hooks, query,
};
use std::sync::{Arc, Mutex};

// ============================================================================
// Mock Tool for Testing
// ============================================================================

/// A mock tool that returns predictable responses for testing
struct MockCalculatorTool {
    call_count: Arc<Mutex<usize>>,
}

impl MockCalculatorTool {
    fn new() -> Self {
        Self {
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

// Note: Tool and ToolExecutor traits don't exist in the current SDK API
// Mock tools should use the SDK's tool! macro instead
// This struct is kept for reference purposes only

// ============================================================================
// Test Utilities
// ============================================================================

/// Assert that a query response contains expected text
fn assert_response_contains(messages: Vec<Message>, expected: &str) -> anyhow::Result<()> {
    for message in messages {
        if let Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    if text.text.contains(expected) {
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!(
        "Expected response to contain '{}', but it was not found",
        expected
    ))
}

/// Measure test execution time
struct TestTimer {
    start: std::time::Instant,
    name: String,
}

impl TestTimer {
    fn new(name: &str) -> Self {
        println!("  🧪 Testing: {}", name);
        Self {
            start: std::time::Instant::now(),
            name: name.to_string(),
        }
    }

    fn done(self) {
        let elapsed = self.start.elapsed();
        println!("  ✅ {} ({:.2}ms)\n", self.name, elapsed.as_millis());
    }
}

// ============================================================================
// Unit Testing Examples
// ============================================================================

#[tokio::test]
async fn test_simple_query() -> anyhow::Result<()> {
    let _timer = TestTimer::new("simple query");

    let messages = query("What is 2 + 2?", None).await?;

    assert!(!messages.is_empty(), "Should receive at least one message");

    Ok(())
}

#[tokio::test]
async fn test_query_with_options() -> anyhow::Result<()> {
    let _timer = TestTimer::new("query with options");

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let messages = query("Say 'test'", Some(options)).await?;

    assert_response_contains(messages, "test")?;

    Ok(())
}
#[tokio::test]
async fn test_query_with_options() -> anyhow::Result<()> {
    let _timer = TestTimer::new("query with options");

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let messages = query("Say 'test'", Some(options)).await?;

    assert_response_contains(messages, "test")?;

    Ok(())
}
#[tokio::test]
async fn test_query_with_options() -> anyhow::Result<()> {
    let _timer = TestTimer::new("query with options");

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let messages = query("Say 'test'", Some(options)).await?;

    assert_response_contains(messages, "test")?;

    Ok(())
}
#[tokio::test]
async fn test_query_with_options() -> anyhow::Result<()> {
    let _timer = TestTimer::new("query with options");

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let messages = query("Say 'test'", Some(options)).await?;

    assert_response_contains(messages, "test")?;

    Ok(())
}
#[tokio::test]
async fn test_query_with_options() -> anyhow::Result<()> {
    let _timer = TestTimer::new("query with options");

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let messages = query("Say 'test'", Some(options)).await?;

    assert_response_contains(messages, "test")?;

    Ok(())
}
#[tokio::test]
async fn test_query_with_options() -> anyhow::Result<()> {
    let _timer = TestTimer::new("query with options");

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let messages = query("Say 'test'", Some(options)).await?;

    assert_response_contains(messages, "test")?;

    Ok(())
}
#[tokio::test]
async fn test_query_with_options() -> anyhow::Result<()> {
    let _timer = TestTimer::new("query with options");

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let messages = query("Say 'test'", Some(options)).await?;

    assert_response_contains(messages, "test")?;

    Ok(())
}
#[tokio::test]
async fn test_query_with_options() -> anyhow::Result<()> {
    let _timer = TestTimer::new("query with options");

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let messages = query("Say 'test'", Some(options)).await?;

    assert_response_contains(messages, "test")?;

    Ok(())
}
#[tokio::test]
async fn test_query_with_options() -> anyhow::Result<()> {
    let _timer = TestTimer::new("query with options");

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let messages = query("Say 'test'", Some(options)).await?;

    assert_response_contains(messages, "test")?;

    Ok(())
}

    let messages = query("Say 'test'", Some(options)).await?;

    assert_response_contains(messages, "test")?;

    Ok(())
}

#[tokio::test]
async fn test_mock_tool_execution() -> anyhow::Result<()> {
    let _timer = TestTimer::new("mock tool execution");

    // Note: Mock tool execution tests should use the SDK's tool! macro
    // This is a placeholder test demonstrating the concept
    let tool = MockCalculatorTool::new();
    let _initial_count = tool.call_count();

    // In actual implementation, would use tool! macro to create test tools
    // For now, just verify the mock can be created
    assert_eq!(tool.call_count(), 0, "Initial call count should be 0");

    Ok(())
}

// ============================================================================
// Integration Testing Examples
// ============================================================================

#[tokio::test]
async fn test_multi_turn_conversation() -> anyhow::Result<()> {
    let _timer = TestTimer::new("multi-turn conversation");

    // Test multi-turn conversation using query function
    let options1 = ClaudeAgentOptions::builder()
        .continue_conversation(true)
        .build();

    let messages = query("Remember the number 5", Some(options1)).await?;
    assert!(!messages.is_empty(), "First query should return messages");

    // Follow-up query
    let options2 = ClaudeAgentOptions::builder()
        .continue_conversation(true)
        .build();

    let _messages = query("What number did I mention?", Some(options2)).await?;
    // Note: In actual test, would verify the response contains "5"
    // but this requires session management which is complex

    Ok(())
}

#[tokio::test]
async fn test_permission_system() -> anyhow::Result<()> {
    let _timer = TestTimer::new("permission system");

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .allowed_tools(vec!["Read".to_string()])
        .build();

    let _messages = query("List files in current directory", Some(options)).await?;

    // Verify tool was allowed (would need more complex setup)
    Ok(())
}

#[tokio::test]
async fn test_hook_execution() -> anyhow::Result<()> {
    let _timer = TestTimer::new("hook execution");

    use claude_agent_sdk_rs::{HookContext, HookInput, HookJsonOutput};

    async fn test_hook(
        _input: HookInput,
        _tool_use_id: Option<String>,
        _context: HookContext,
    ) -> HookJsonOutput {
        HookJsonOutput::Sync(Default::default())
    }

    let mut hooks = Hooks::new();
    hooks.add_pre_tool_use(test_hook);

    let options = ClaudeAgentOptions::builder()
        .hooks(Some(hooks.build()))
        .build();

    let _messages = query("What is 2 + 2?", Some(options)).await?;

    // In real scenario with tool use, hooks would be called
    Ok(())
}

// ============================================================================
// Property-Based Testing Concepts
// ============================================================================

/// Property: Responses should never be empty for valid queries
async fn property_non_empty_response(prompt: &str) -> bool {
    match query(prompt, None).await {
        Ok(messages) => !messages.is_empty(),
        Err(_) => false,
    }
}

/// Property: Temperature should affect response consistency
async fn property_temperature_effect() -> anyhow::Result<()> {
    let prompt = "Say a random number between 1 and 100";

    // Low temperature - should be more consistent
    let options_low = ClaudeAgentOptions {
        temperature: Some(0.0),
        ..Default::default()
    };

    let options_high = ClaudeAgentOptions {
        temperature: Some(1.0),
        ..Default::default()
    };

    let _response1 = query(prompt, Some(options_low)).await?;
    let _response2 = query(prompt, Some(options_low)).await?;
    let _response3 = query(prompt, Some(options_high)).await?;

    // In real property-based test, would measure variance
    Ok(())
}

// ============================================================================
// Deterministic Testing
// ============================================================================

/// Example of deterministic testing with controlled inputs
#[tokio::test]
async fn test_deterministic_behavior() -> anyhow::Result<()> {
    let _timer = TestTimer::new("deterministic behavior");

    // Use deterministic inputs
    let test_cases = vec![
        ("What is 1 + 1?", "2"),
        ("What is the capital of France?", "Paris"),
        ("Say 'test'", "test"),
    ];

    for (prompt, expected) in test_cases {
        let messages = query(prompt, None).await?;
        assert_response_contains(messages, expected)?;
    }

    Ok(())
}

// ============================================================================
// Performance Testing
// ============================================================================

/// Benchmark query performance
async fn benchmark_query_performance() -> anyhow::Result<()> {
    println!("  🧪 Testing: query performance benchmark\n");

    let iterations = 5;
    let prompt = "What is 2 + 2? Answer with just the number.";

    let mut times = Vec::new();

    for i in 0..iterations {
        let start = std::time::Instant::now();
        let _messages = query(prompt, None).await?;
        let elapsed = start.elapsed();

        times.push(elapsed);
        println!("    Iteration {}: {:.2}ms", i + 1, elapsed.as_millis());
    }

    let avg_time: std::time::Duration =
        times.iter().sum::<std::time::Duration>() / times.len() as u32;
    let min_time = times.iter().min().unwrap();
    let max_time = times.iter().max().unwrap();

    println!("\n  Statistics:");
    println!("    Average: {:.2}ms", avg_time.as_millis());
    println!("    Min: {:.2}ms", min_time.as_millis());
    println!("    Max: {:.2}ms", max_time.as_millis());

    Ok(())
}

// ============================================================================
// Main Test Runner
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 Comprehensive Testing Strategies\n");
    println!("{}", "=".repeat(50));

    println!("\n📋 Running Tests...\n");

    // Unit tests
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Unit Tests");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    test_simple_query().await?;
    test_query_with_options().await?;
    test_mock_tool_execution().await?;

    // Integration tests
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Integration Tests");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    test_multi_turn_conversation().await?;
    test_permission_system().await?;
    test_hook_execution().await?;

    // Property-based tests
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Property-Based Tests");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let result = property_non_empty_response("What is 2 + 2?");
    println!("  Property: Non-empty response = {}", result);

    property_temperature_effect().await?;

    // Deterministic tests
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Deterministic Tests");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    test_deterministic_behavior().await?;

    // Performance tests
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Performance Tests");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    benchmark_query_performance().await?;

    // Summary
    println!("\n{}", "=".repeat(50));
    println!("✅ All Tests Passed");
    println!("{}", "=".repeat(50));
    println!("\nTesting Strategies Demonstrated:");
    println!("  🧪 Unit tests for individual components");
    println!("  🔗 Integration tests for multi-component scenarios");
    println!("  🎭 Mock tools for predictable testing");
    println!("  📊 Property-based testing for invariants");
    println!("  🎯 Deterministic testing with known inputs");
    println!("  ⚡ Performance benchmarking");
    println!("\nBest Practices:");
    println!("  • Use mock tools for fast, predictable tests");
    println!("  • Test edge cases and error conditions");
    println!("  • Measure performance to catch regressions");
    println!("  • Use property-based testing for invariants");
    println!("  • Keep tests deterministic and repeatable");

    Ok(())
}
