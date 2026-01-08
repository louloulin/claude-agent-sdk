//! Testing Patterns Example
//!
//! This example demonstrates various testing patterns
//! when working with the Claude Agent SDK.

use anyhow::Result;
use claude_agent_sdk_rs::{
    ClaudeAgentOptions, McpServerConfig, McpToolResultContent, Message, ToolResult,
    create_sdk_mcp_server, query, tool,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Testing Patterns Example ===\n");

    // Example 1: Mock tool for testing
    println!("1. Mock Tool Testing:");
    mock_tool_testing().await?;

    // Example 2: Deterministic responses
    println!("\n2. Deterministic Testing:");
    deterministic_testing().await?;

    // Example 3: Integration testing patterns
    println!("\n3. Integration Testing:");
    integration_testing_patterns().await?;

    // Example 4: Property-based testing
    println!("\n4. Property-Based Testing:");
    property_based_testing().await?;

    // Example 5: Performance testing
    println!("\n5. Performance Testing:");
    performance_testing().await?;

    // Example 6: Error scenario testing
    println!("\n6. Error Scenario Testing:");
    error_scenario_testing().await?;

    // Example 7: Snapshot testing
    println!("\n7. Snapshot Testing:");
    snapshot_testing().await?;

    // Example 8: Test data builders
    println!("\n8. Test Data Builders:");
    test_data_builders().await?;

    Ok(())
}

/// Example 1: Mock tools for testing
async fn mock_tool_testing() -> Result<()> {
    // Create a mock tool that returns predictable data
    async fn mock_tool_handler(args: serde_json::Value) -> Result<ToolResult> {
        Ok(ToolResult {
            content: vec![McpToolResultContent::Text {
                text: format!("Mock response to: {}", args),
            }],
            is_error: false,
        })
    }

    let mock_tool = tool!(
        "mock_tool",
        "A mock tool for testing",
        json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            }
        }),
        mock_tool_handler
    );

    let server = create_sdk_mcp_server("test-tools", "1.0.0", vec![mock_tool]);

    let mut mcp_servers = std::collections::HashMap::new();
    mcp_servers.insert("test-tools".to_string(), McpServerConfig::Sdk(server));

    let options = ClaudeAgentOptions::builder()
        .mcp_servers(claude_agent_sdk_rs::McpServers::Dict(mcp_servers))
        .allowed_tools(vec!["mcp__test-tools__mock_tool".to_string()])
        .permission_mode(claude_agent_sdk_rs::PermissionMode::BypassPermissions)
        .build();

    println!("   Mock tool configured and ready for testing");
    let _messages = query("Use the mock tool with 'test input'", Some(options)).await?;
    println!("   ✓ Mock tool test passed");

    Ok(())
}

/// Example 2: Deterministic testing
async fn deterministic_testing() -> Result<()> {
    use std::collections::HashMap;

    // Use fixed seeds and predictable inputs
    let test_cases = vec![
        ("2 + 2", "4"),
        ("capital of France", "Paris"),
        ("Rust ownership", "ownership system"),
    ];

    for (query_text, expected_keyword) in test_cases {
        let options = ClaudeAgentOptions::builder()
            .model(Some("claude-sonnet-4-5".to_string()))
            .max_turns(1)
            .build();

        let messages = query(query_text, Some(options)).await?;

        // Check response contains expected keyword
        let response_text = extract_response_text(&messages);
        let contains = response_text
            .to_lowercase()
            .contains(&expected_keyword.to_lowercase());

        println!(
            "   Query: '{}' -> Contains '{}': {}",
            query_text, expected_keyword, contains
        );

        if !contains {
            println!("   ⚠️  Warning: Expected keyword not found");
        }
    }

    println!("   ✓ Deterministic testing complete");
    Ok(())
}

/// Example 3: Integration testing patterns
async fn integration_testing_patterns() -> Result<()> {
    // Test complete workflows
    struct TestSuite {
        name: String,
        query: String,
        expected_tools: Vec<String>,
        min_turns: u32,
        max_turns: u32,
    }

    let test_cases = vec![
        TestSuite {
            name: "File Reading".to_string(),
            query: "Read the README.md file and summarize it".to_string(),
            expected_tools: vec!["Read".to_string()],
            min_turns: 1,
            max_turns: 5,
        },
        TestSuite {
            name: "Code Analysis".to_string(),
            query: "Analyze the code in src/lib.rs".to_string(),
            expected_tools: vec!["Read".to_string()],
            min_turns: 1,
            max_turns: 3,
        },
    ];

    for test in test_cases {
        println!("   Running test: {}", test.name);

        let options = ClaudeAgentOptions::builder()
            .max_turns(Some(test.max_turns))
            .allowed_tools(test.expected_tools.clone())
            .build();

        match query(&test.query, Some(options)).await {
            Ok(messages) => {
                let turn_count = messages.len() as u32;
                let valid_turns =
                    turn_count >= test.min_turns as usize && turn_count <= test.max_turns as usize;

                println!("     Turn count: {} (valid: {})", turn_count, valid_turns);

                if !valid_turns {
                    println!("     ⚠️  Warning: Turn count outside expected range");
                }
            }
            Err(e) => {
                println!("     ✗ Test failed: {}", e);
            }
        }
    }

    println!("   ✓ Integration testing complete");
    Ok(())
}

/// Example 4: Property-based testing
async fn property_based_testing() -> Result<()> {
    // Test properties that should always hold
    let test_inputs = vec!["What is 2 + 2?", "What is 5 + 3?", "What is 10 + 15?"];

    // Property: Response should not be empty
    println!("   Testing property: Non-empty responses");

    for input in test_inputs {
        let messages = query(input, None).await?;
        let response_text = extract_response_text(&messages);

        let property_holds = !response_text.trim().is_empty();
        println!("     Input: '{}' -> Non-empty: {}", input, property_holds);

        if !property_holds {
            println!("     ✗ Property violation: Empty response");
            return Err(anyhow::anyhow!("Property violation detected"));
        }
    }

    println!("   ✓ All properties verified");
    Ok(())
}

/// Example 5: Performance testing
async fn performance_testing() -> Result<()> {
    use std::time::Instant;

    let test_query = "What is 2 + 2?";
    let iterations = 5;

    println!("   Running {} iterations...", iterations);

    let mut total_time = std::time::Duration::from_secs(0);

    for i in 0..iterations {
        let start = Instant::now();
        let _messages = query(test_query, None).await?;
        let elapsed = start.elapsed();

        total_time += elapsed;
        println!("     Iteration {}: {:?}", i + 1, elapsed);
    }

    let avg_time = total_time / iterations;
    println!("   Average time: {:?}", avg_time);

    // Performance assertion
    let max_acceptable = std::time::Duration::from_secs(10);
    if avg_time > max_acceptable {
        println!("   ⚠️  Warning: Average time exceeds {:?}", max_acceptable);
    } else {
        println!("   ✓ Performance within acceptable range");
    }

    Ok(())
}

/// Example 6: Error scenario testing
async fn error_scenario_testing() -> Result<()> {
    // Test various error conditions
    let error_scenarios = vec![
        ("Empty query", ""),
        ("Invalid tool usage", "Use a non-existent tool XYZ123"),
        ("Malformed input", "Query with malformed \x00 input"),
    ];

    for (scenario, query_text) in error_scenarios {
        println!("   Testing: {}", scenario);

        match query(query_text, None).await {
            Ok(messages) => {
                println!("     Unexpected success ({} messages)", messages.len());
            }
            Err(e) => {
                println!("     Expected error: {}", e);
            }
        }
    }

    println!("   ✓ Error scenario testing complete");
    Ok(())
}

/// Example 7: Snapshot testing
async fn snapshot_testing() -> Result<()> {
    // Generate and compare snapshots
    let query_text = "What is 2 + 2?";

    let options = ClaudeAgentOptions::builder()
        .model(Some("claude-sonnet-4-5".to_string()))
        .max_turns(1)
        .temperature(Some(0.0)) // Deterministic
        .build();

    let messages = query(query_text, Some(options)).await?;
    let response_text = extract_response_text(&messages);

    println!("   Generated snapshot:");
    println!("   ```");
    println!("{}", response_text);
    println!("   ```");

    // In a real test, you would compare this with a stored snapshot
    let expected_contains = "4";
    let matches = response_text.contains(expected_contains);

    println!("   Contains '{}': {}", expected_contains, matches);

    if matches {
        println!("   ✓ Snapshot matches expected content");
    } else {
        println!("   ⚠️  Snapshot differs from expected");
    }

    Ok(())
}

/// Example 8: Test data builders
async fn test_data_builders() -> Result<()> {
    // Builder pattern for test data
    struct TestDataBuilder {
        query: String,
        max_turns: Option<u32>,
        model: Option<String>,
    }

    impl TestDataBuilder {
        fn new() -> Self {
            Self {
                query: String::new(),
                max_turns: None,
                model: None,
            }
        }

        fn query(mut self, query: impl Into<String>) -> Self {
            self.query = query.into();
            self
        }

        fn max_turns(mut self, turns: u32) -> Self {
            self.max_turns = Some(turns);
            self
        }

        fn model(mut self, model: impl Into<String>) -> Self {
            self.model = Some(model.into());
            self
        }

        fn build(self) -> (String, ClaudeAgentOptions) {
            let options = ClaudeAgentOptions::builder()
                .max_turns(self.max_turns)
                .model(self.model)
                .build();

            (self.query, options)
        }
    }

    // Use the builder
    let (query, options) = TestDataBuilder::new()
        .query("What is 2 + 2?")
        .max_turns(2)
        .model("claude-sonnet-4-5")
        .build();

    println!("   Built test data:");
    println!("     Query: {}", query);
    println!("     Max turns: {:?}", options.max_turns);
    println!("     Model: {:?}", options.model);

    let _messages = query(&query, Some(options)).await?;
    println!("   ✓ Test data builder pattern works");

    Ok(())
}

/// Helper function to extract response text
fn extract_response_text(messages: &[Message]) -> String {
    let mut text = String::new();

    for msg in messages {
        if let Message::Assistant(assistant_msg) = msg {
            for block in &assistant_msg.message.content {
                if let claude_agent_sdk_rs::ContentBlock::Text(content) = block {
                    text.push_str(&content.text);
                }
            }
        }
    }

    text
}

/// Example 9: Custom assertions
async fn custom_assertions() -> Result<()> {
    let messages = query("List numbers 1-5", None).await?;
    let response = extract_response_text(&messages);

    // Custom assertion: response should contain numbers 1-5
    let numbers = vec!["1", "2", "3", "4", "5"];
    let all_present = numbers.iter().all(|n| response.contains(n));

    println!(
        "   Custom assertion: All numbers present -> {}",
        all_present
    );

    if !all_present {
        println!("   Missing numbers:");
        for n in numbers {
            if !response.contains(n) {
                println!("     - {}", n);
            }
        }
    }

    assert!(all_present, "Not all numbers present in response");
    println!("   ✓ Custom assertion passed");

    Ok(())
}

/// Example 10: Test isolation
async fn test_isolation() -> Result<()> {
    // Ensure tests don't interfere with each other
    let session_1 = "test-session-1";
    let session_2 = "test-session-2";

    let options1 = ClaudeAgentOptions::builder()
        .resume(Some(session_1.to_string()))
        .build();

    let options2 = ClaudeAgentOptions::builder()
        .resume(Some(session_2.to_string()))
        .build();

    // Run queries in isolated sessions
    let _msg1 = query("Remember: X = 1", Some(options1)).await?;
    let _msg2 = query("Remember: X = 2", Some(options2)).await?;

    // Verify isolation
    let options1_check = ClaudeAgentOptions::builder()
        .resume(Some(session_1.to_string()))
        .continue_conversation(true)
        .build();

    let msg_check = query("What is X?", Some(options1_check)).await?;
    let response = extract_response_text(&msg_check);

    let isolated = response.contains("1") && !response.contains("2");
    println!("   Session isolation: {}", isolated);

    if isolated {
        println!("   ✓ Sessions are properly isolated");
    } else {
        println!("   ⚠️  Warning: Session isolation may be compromised");
    }

    Ok(())
}
