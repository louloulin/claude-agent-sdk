//! Advanced Configuration Example
//!
//! This example demonstrates advanced configuration options
//! for fine-tuning Claude Agent behavior.

use claude_agent_sdk_rs::{
    query, ClaudeAgentOptions, SystemPrompt,
    PermissionMode, SdkBeta, tools
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Advanced Configuration Example ===\n");

    // Example 1: Beta Features
    println!("1. Beta Features:");
    beta_features_example().await?;

    // Example 2: Custom System Prompts
    println!("\n2. Custom System Prompts:");
    custom_system_prompts().await?;

    // Example 3: Advanced Tool Configuration
    println!("\n3. Advanced Tool Configuration:");
    advanced_tool_config().await?;

    // Example 4: Budget and Cost Control
    println!("\n4. Budget and Cost Control:");
    budget_control_example().await?;

    // Example 5: Model Selection and Fallback
    println!("\n5. Model Selection and Fallback:");
    model_selection_example().await?;

    // Example 6: Session Management
    println!("\n6. Session Management:");
    session_management_example().await?;

    // Example 7: Environment Variables Configuration
    println!("\n7. Environment Variables:");
    env_vars_example().await?;

    // Example 8: Debug Configuration
    println!("\n8. Debug Configuration:");
    debug_config_example().await?;

    Ok(())
}

/// Example 1: Enable Beta Features
async fn beta_features_example() -> Result<()> {
    let options = ClaudeAgentOptions::builder()
        .betas(vec![
            SdkBeta::MaxTokens3,           // Increased token limits
            SdkBeta::ComputerTools20250124, // Enhanced computer tools
            SdkBeta::PromptCaching,         // Prompt caching
        ])
        .build();

    println!("   Enabled beta features:");
    println!("   - MaxTokens3");
    println!("   - ComputerTools20250124");
    println!("   - PromptCaching");

    let _messages = query("List available beta features", Some(options)).await?;
    println!("   ✓ Beta features active");
    Ok(())
}

/// Example 2: Custom System Prompts
async fn custom_system_prompts() -> Result<()> {
    // Simple system prompt
    let simple_prompt = SystemPrompt::text(
        "You are a helpful Rust programming assistant."
    );

    // Multi-part system prompt
    let multi_part_prompt = SystemPrompt::text(
        "You are a Rust expert.\n\
         Focus on:\n\
         1. Idiomatic Rust code\n\
         2. Performance\n\
         3. Safety\n\
         4. Best practices"
    );

    // System prompt from file
    let file_prompt = SystemPrompt::file_path("path/to/prompt.txt");

    // Use system prompt in options
    let options = ClaudeAgentOptions::builder()
        .system_prompt(Some(multi_part_prompt))
        .build();

    println!("   Configured custom system prompt");
    let _messages = query("What is Rust ownership?", Some(options)).await?;
    println!("   ✓ Custom prompt used");

    Ok(())
}

/// Example 3: Advanced Tool Configuration
async fn advanced_tool_config() -> Result<()> {
    // Tools preset
    let all_tools = tools::preset::all();
    let coding_tools = tools::preset::coding();
    let filesystem_tools = tools::preset::filesystem();

    // Custom tool list
    let custom_tools = tools::custom(&[
        "Read",
        "Write",
        "Bash",
    ]);

    let options = ClaudeAgentOptions::builder()
        .tools(Some(coding_tools))
        .allowed_tools(vec![
            "Read".to_string(),
            "Write".to_string(),
            "Bash".to_string(),
        ])
        .disallowed_tools(vec![
            "Edit".to_string(),  // Disable for safety
        ])
        .build();

    println!("   Tool configuration:");
    println!("   - Preset: coding");
    println!("   - Allowed: Read, Write, Bash");
    println!("   - Disallowed: Edit");

    let _messages = query("List files in current directory", Some(options)).await?;
    println!("   ✓ Tools configured");
    Ok(())
}

/// Example 4: Budget and Cost Control
async fn budget_control_example() -> Result<()> {
    let options = ClaudeAgentOptions::builder()
        .max_budget_usd(0.50)  // Limit to $0.50
        .max_turns(5)
        .model(Some("claude-sonnet-4-5".to_string()))
        .build();

    println!("   Budget limit: $0.50 USD");
    println!("   Max turns: 5");
    println!("   Model: claude-sonnet-4-5");

    match query("Explain quantum computing in detail", Some(options)).await {
        Ok(messages) => {
            println!("   ✓ Query completed within budget");
            println!("   Received {} messages", messages.len());
        }
        Err(e) => {
            println!("   ✗ Query failed: {}", e);
            println!("   (Likely exceeded budget or max turns)");
        }
    }

    Ok(())
}

/// Example 5: Model Selection and Fallback
async fn model_selection_example() -> Result<()> {
    // Primary model with fallback
    let options = ClaudeAgentOptions::builder()
        .model(Some("claude-opus-4-5".to_string()))
        .fallback_model(Some("claude-sonnet-4-5".to_string()))
        .max_thinking_tokens(Some(50000))  // Extended thinking
        .build();

    println!("   Primary model: claude-opus-4-5");
    println!("   Fallback model: claude-sonnet-4-5");
    println!("   Max thinking tokens: 50000");

    let _messages = query("Solve this complex problem", Some(options)).await?;
    println!("   ✓ Model selection successful");

    Ok(())
}

/// Example 6: Advanced Session Management
async fn session_management_example() -> Result<()> {
    // Fork session for fresh start
    let options_fork = ClaudeAgentOptions::builder()
        .fork_session(true)  // Each session starts fresh
        .build();

    // Resume existing session
    let options_resume = ClaudeAgentOptions::builder()
        .resume(Some("my-session-id".to_string()))
        .continue_conversation(true)
        .build();

    println!("   Session configurations:");
    println!("   - Fork mode: fresh start each time");
    println!("   - Resume mode: continue existing session");

    let _messages = query("What is 2 + 2?", Some(options_fork)).await?;
    println!("   ✓ Fork session created");

    Ok(())
}

/// Example 7: Environment Variables
async fn env_vars_example() -> Result<()> {
    use std::collections::HashMap;

    let mut env = HashMap::new();
    env.insert("RUST_LOG".to_string(), "debug".to_string());
    env.insert("API_KEY".to_string(), "sk-xxx".to_string());

    let options = ClaudeAgentOptions::builder()
        .env(env)
        .build();

    println!("   Environment variables:");
    println!("   - RUST_LOG=debug");
    println!("   - API_KEY=sk-xxx");

    let _messages = query("Check environment", Some(options)).await?;
    println!("   ✓ Environment configured");

    Ok(())
}

/// Example 8: Debug Configuration
async fn debug_config_example() -> Result<()> {
    use std::sync::Arc;

    let stderr_callback = Arc::new(|msg: String| {
        eprintln!("🔍 DEBUG: {}", msg);
    });

    let mut extra_args = std::collections::HashMap::new();
    extra_args.insert("debug-to-stderr".to_string(), None);
    extra_args.insert("verbose".to_string(), None);

    let options = ClaudeAgentOptions::builder()
        .stderr_callback(Some(stderr_callback))
        .extra_args(Some(extra_args))
        .build();

    println!("   Debug mode enabled");
    println!("   - Stderr callback active");
    println!("   - Extra arguments: debug-to-stderr, verbose");

    let _messages = query("Simple test query", Some(options)).await?;
    println!("   ✓ Debug output captured");

    Ok(())
}

/// Example 9: Working Directory Configuration
async fn working_directory_example() -> Result<()> {
    use std::path::PathBuf;

    let options = ClaudeAgentOptions::builder()
        .cwd(Some(PathBuf::from("/tmp")))
        .add_dirs(vec![
            PathBuf::from("/home/user/projects"),
            PathBuf::from("/shared"),
        ])
        .build();

    println!("   Working directory: /tmp");
    println!("   Additional directories:");
    println!("   - /home/user/projects");
    println!("   - /shared");

    let _messages = query("List files in working directory", Some(options)).await?;
    println!("   ✓ Working directory configured");

    Ok(())
}

/// Example 10: User Identifier and Metadata
async fn user_metadata_example() -> Result<()> {
    let options = ClaudeAgentOptions::builder()
        .user(Some("user-12345".to_string()))
        .permission_prompt_tool_name(Some("admin_approval".to_string()))
        .build();

    println!("   User ID: user-12345");
    println!("   Permission tool: admin_approval");

    let _messages = query("Who am I?", Some(options)).await?;
    println!("   ✓ User metadata configured");

    Ok(())
}

/// Example 11: Stream vs Non-Stream Configuration
async fn stream_config_example() -> Result<()> {
    let options = ClaudeAgentOptions::builder()
        .include_partial_messages(true)  // Include partial in stream
        .max_buffer_size(Some(1024 * 1024))  // 1MB buffer
        .build();

    println!("   Stream configuration:");
    println!("   - Include partial messages: true");
    println!("   - Max buffer size: 1MB");

    let _messages = query("Explain streams", Some(options)).await?;
    println!("   ✓ Stream configuration applied");

    Ok(())
}

/// Example 12: Complete Production Configuration
async fn production_config_example() -> Result<()> {
    let options = ClaudeAgentOptions::builder()
        // Model selection
        .model(Some("claude-sonnet-4-5".to_string()))
        .fallback_model(Some("claude-haiku-4-5".to_string()))

        // Cost control
        .max_budget_usd(1.00)
        .max_turns(10)

        // Permissions
        .permission_mode(PermissionMode::AcceptEdits)

        // Tools
        .tools(Some(tools::preset::production()))
        .allowed_tools(vec![
            "Read".to_string(),
            "Write".to_string(),
            "Bash".to_string(),
        ])

        // System prompt
        .system_prompt(Some(SystemPrompt::text(
            "You are a production assistant focused on \
             reliability and correctness."
        )))

        // Beta features
        .betas(vec![
            SdkBeta::MaxTokens3,
            SdkBeta::PromptCaching,
        ])

        // Performance
        .max_thinking_tokens(Some(20000))

        // Build
        .build();

    println!("   ✓ Production configuration complete");
    println!("   Features:");
    println!("   - Model with fallback");
    println!("   - Budget limit: $1.00");
    println!("   - Production tools preset");
    println!("   - Beta features enabled");
    println!("   - Extended thinking: 20k tokens");

    let _messages = query("Production test query", Some(options)).await?;
    println!("   ✓ Production-ready configuration");

    Ok(())
}
