# Section 11: Examples Guide

This section provides a comprehensive guide to all 70+ examples in the Claude Agent SDK for Rust, organized by category with usage patterns and best practices.

## Overview

The SDK includes extensive examples demonstrating every feature and common usage patterns. All examples are located in `crates/claude-agent-sdk/examples/` and can be run with:

```bash
cargo run --example <example_name>
```

## Example Categories

| Category | Examples | Description |
|----------|----------|-------------|
| Getting Started | 01, 20, 56 | Basic usage patterns |
| Client & Streaming | 06, 07, 14, 20, 54 | ClaudeClient and streaming |
| Hooks & Permissions | 04, 05, 15 | Hook system and permissions |
| Configuration | 13, 17-19, 21-22, 46 | Options and setup |
| Session Management | 16 | Multi-turn conversations |
| Multimodal | 23, 61 | Images and mixed content |
| Skills | 30-42, 50, 55 | Skills system |
| MCP & Tasks | 08, 42 | MCP integration |
| Error Handling | 43, 46, 53 | Error patterns |
| Concurrency | 44, 47, 48 | Parallel processing |
| Testing & Production | 49-50 | Best practices |
| V2 API | 56, 62, 65 | Simplified API |
| Orchestration | 51, 52, 59, 60 | Multi-agent systems |

---

## 1. Getting Started

### Example 01: Hello World

The simplest example demonstrating basic query and file creation.

```bash
cargo run --example 01_hello_world
```

**What it does:**
1. Asks Claude to write a Python hello world script
2. Saves it to `./fixtures/hello.py`
3. Runs the script to verify it works

```rust
use claude_agent_sdk::{ClaudeAgentOptions, ContentBlock, Message, query};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions::builder()
        .allowed_tools(vec!["Write".to_string()])
        .permission_mode(claude_agent_sdk::PermissionMode::AcceptEdits)
        .max_turns(5)
        .build();

    let messages = query(
        "Write a simple Python hello world script to ./fixtures/hello.py",
        Some(options),
    ).await?;

    for message in &messages {
        match message {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                    }
                }
            },
            Message::Result(result) => {
                println!("Duration: {}ms, Cost: ${:.4}",
                    result.duration_ms,
                    result.total_cost_usd.unwrap_or(0.0));
            },
            _ => {},
        }
    }

    Ok(())
}
```

### Example 20: Streaming Query API

Memory-efficient streaming alternative to `query()`.

```bash
cargo run --example 20_query_stream
```

**Key differences from `query()`:**

| Function | Memory Usage | Processing |
|----------|-------------|------------|
| `query()` | O(n) - stores all messages | Waits for completion |
| `query_stream()` | O(1) per message | Real-time |

```rust
use claude_agent_sdk::{query_stream, Message, ContentBlock};
use futures::stream::StreamExt;

let mut stream = query_stream("Explain Rust ownership", None).await?;

while let Some(result) = stream.next().await {
    let message = result?;
    if let Message::Assistant(msg) = message {
        for block in &msg.message.content {
            if let ContentBlock::Text(text) = block {
                println!("Claude: {}", text.text);
            }
        }
    }
}
```

### Example 56: V2 API - Simplified Interface

TypeScript-style simplified API for quick interactions.

```bash
cargo run --example 56_v2_api
```

```rust
use claude_agent_sdk::v2::{prompt, create_session, SessionOptions};

// One-shot prompt
let result = prompt("What is 2 + 2?", Default::default()).await?;
println!("Answer: {}", result.content);
println!("Tokens: {}", result.total_tokens());

// Session-based conversation
let mut session = create_session(Default::default()).await?;
session.send("My favorite color is blue").await?;
let messages = session.receive().await?;
session.close().await?;
```

---

## 2. ClaudeClient & Bidirectional Streaming

### Example 06: Bidirectional Client

Multi-turn conversations with context retention.

```bash
cargo run --example 06_bidirectional_client
```

```rust
use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions, ContentBlock, Message};
use futures::StreamExt;

let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
client.connect().await?;

// First query
client.query("What is your name?").await?;
let mut stream = client.receive_response();
while let Some(message) = stream.next().await {
    // Process response...
}
drop(stream); // Release borrow before next query

// Second query - Claude remembers context!
client.query("Can you remember what I just asked?").await?;
// ... process response

client.disconnect().await?;
```

### Example 07: Dynamic Control

Runtime control of Claude's behavior.

```bash
cargo run --example 07_dynamic_control
```

**Dynamic control methods:**
- `interrupt()` - Stop current operation
- `set_permission_mode(mode)` - Change permissions dynamically
- `set_model(model)` - Switch AI models mid-session

```rust
// Change permission mode
client.set_permission_mode(PermissionMode::AcceptEdits).await?;

// Switch model
client.set_model(Some("claude-sonnet-4-20250514")).await?;

// Interrupt long-running task
client.interrupt().await?;
```

---

## 3. Hooks System

### Example 05: PreToolUse Hooks

Block or allow tool execution before it happens.

```bash
cargo run --example 05_hooks_pretooluse
```

```rust
use claude_agent_sdk::{Hooks, HookInput, HookJsonOutput, SyncHookJsonOutput,
    HookSpecificOutput, PreToolUseHookSpecificOutput};

async fn block_dangerous_bash(
    input: HookInput,
    _tool_use_id: Option<String>,
    _context: HookContext,
) -> HookJsonOutput {
    match input {
        HookInput::PreToolUse(pre_tool) if pre_tool.tool_name == "Bash" => {
            let command = pre_tool.tool_input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if command.contains("rm -rf") {
                return HookJsonOutput::Sync(SyncHookJsonOutput {
                    hook_specific_output: Some(HookSpecificOutput::PreToolUse(
                        PreToolUseHookSpecificOutput {
                            permission_decision: Some("deny".to_string()),
                            permission_decision_reason: Some(
                                "Dangerous command blocked".to_string()
                            ),
                            ..Default::default()
                        }
                    )),
                    ..Default::default()
                });
            }
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        },
        _ => HookJsonOutput::Sync(SyncHookJsonOutput::default()),
    }
}

// Register hook
let mut hooks = Hooks::new();
hooks.add_pre_tool_use_with_matcher("Bash", block_dangerous_bash);

let options = ClaudeAgentOptions::builder()
    .hooks(hooks.build())
    .build();
```

### Example 15: Comprehensive Hooks

All hook types with detailed examples.

```bash
cargo run --example 15_hooks_comprehensive all
cargo run --example 15_hooks_comprehensive PreToolUse
cargo run --example 15_hooks_comprehensive PostToolUse
cargo run --example 15_hooks_comprehensive UserPromptSubmit
```

**Hook types:**

| Hook | When Called | Use Case |
|------|-------------|----------|
| PreToolUse | Before tool execution | Block/allow tools |
| PostToolUse | After tool execution | Review/modify output |
| UserPromptSubmit | When user sends prompt | Add context |

---

## 4. Configuration

### Example 17: Fallback Model

Automatic failover to backup model.

```bash
cargo run --example 17_fallback_model
```

```rust
let options = ClaudeAgentOptions::builder()
    .model("claude-opus-4-5")
    .fallback_model("claude-sonnet-4-5")
    .build();
```

### Example 21: Custom Plugins

Load custom plugins to extend Claude's capabilities.

```bash
cargo run --example 21_custom_plugins
```

```rust
use claude_agent_sdk::SdkPluginConfig;

let options = ClaudeAgentOptions::builder()
    .plugins(vec![
        SdkPluginConfig::local("./plugins/database-tools"),
        SdkPluginConfig::local("~/.claude/plugins/company-tools"),
    ])
    .build();
```

### Example 18 & 19: Budget and Thinking Tokens

Cost control and extended thinking.

```bash
cargo run --example 18_max_budget_usd
cargo run --example 19_max_thinking_tokens
```

```rust
let options = ClaudeAgentOptions::builder()
    .max_budget_usd(10.0)           // Limit spending
    .max_thinking_tokens(2000)      // Extended thinking
    .max_turns(10)                  // Turn limit
    .build();
```

---

## 5. Session Management

### Example 16: Session Management

Multi-turn conversations with separate contexts.

```bash
cargo run --example 16_session_management
```

```rust
// Separate sessions for different contexts
client.query_with_session("Math question", "math-session").await?;
client.query_with_session("Programming question", "coding-session").await?;

// Fork session for fresh start
let options = ClaudeAgentOptions::builder()
    .fork_session(true)
    .build();

// New session with convenience method
client.new_session("new-id", "Starting fresh").await?;
```

---

## 6. Multimodal Input

### Example 23: Image Input

Send images alongside text.

```bash
cargo run --example 23_image_input
```

```rust
use claude_agent_sdk::{query_with_content, UserContentBlock};

// Base64-encoded image
let content = vec![
    UserContentBlock::text("What color is this image?"),
    UserContentBlock::image_base64("image/png", base64_data)?,
];

let messages = query_with_content(content, None).await?;

// Image URL
let content = vec![
    UserContentBlock::text("Describe this diagram"),
    UserContentBlock::image_url("https://example.com/diagram.png"),
];
```

**Supported formats:** JPEG, PNG, GIF, WebP

---

## 7. Skills System

### Example 30: Basic Skills

Create modular, reusable AI capabilities.

```bash
cargo run --example 30_agent_skills
```

```rust
use claude_agent_sdk::skills::*;
use async_trait::async_trait;

struct FibonacciSkill;

#[async_trait]
impl Skill for FibonacciSkill {
    fn name(&self) -> String { "fibonacci".to_string() }
    fn description(&self) -> String { "Calculates Fibonacci numbers".to_string() }

    async fn execute(&self, _input: SkillInput) -> SkillResult {
        let result = fibonacci(10);
        Ok(SkillOutput::ok(serde_json::json!({"result": result})))
    }
}

// Register and execute
let mut registry = SkillRegistry::new();
registry.register(Box::new(FibonacciSkill))?;
```

### Skill Examples Index

| Example | Topic |
|---------|-------|
| 30 | Basic skill creation |
| 31 | Skill persistence |
| 32 | Skill discovery |
| 33 | Resource handling |
| 34 | Dependencies |
| 35 | Version management |
| 36 | Tagging |
| 37 | YAML configuration |
| 38 | Hot reload |
| 39 | Sandbox mode |
| 40 | Performance |
| 41 | VS Code integration |
| 42 | SKILL.md files |

---

## 8. MCP Integration

### Example 42: Async Tasks (MCP 2025-11-25)

Call-now, fetch-later asynchronous workflows.

```bash
cargo run --example 42_mcp_async_tasks
```

```rust
use claude_agent_sdk::mcp::tasks::{
    TaskManager, TaskRequest, TaskHint, TaskPriority, TaskProgress,
};

let manager = TaskManager::new();

// Create task with hints
let request = TaskRequest {
    method: "tools/call".to_string(),
    params: json!({"name": "long_running_task"}),
    task_hint: Some(TaskHint {
        estimated_duration_secs: Some(10),
        supports_progress: true,
        cancellable: true,
    }),
    priority: Some(TaskPriority::High),
};

let task = manager.create_task(request).await?;

// Update progress
manager.mark_working(&task.id).await?;
manager.update_progress(&task.id,
    TaskProgress::new(0.5).with_message("50% complete")
).await?;

// Complete
manager.mark_completed(&task.id, json!({"result": "done"})).await?;
```

---

## 9. Error Handling

### Example 43: Error Handling Patterns

Comprehensive error handling strategies.

```bash
cargo run --example 43_error_handling
```

```rust
use claude_agent_sdk::{query, ClaudeError};

match query("Hello", None).await {
    Ok(messages) => { /* process */ },
    Err(ClaudeError::CliNotFound(e)) => {
        eprintln!("CLI not found: {:?}", e.cli_path);
    },
    Err(ClaudeError::Connection(e)) => {
        eprintln!("Connection failed: {}", e.message);
    },
    Err(ClaudeError::Process(e)) => {
        eprintln!("Process failed: {:?}", e.exit_code);
    },
    Err(e) => eprintln!("Error: {}", e),
}
```

### Retry with Backoff

```rust
async fn retry_with_backoff() -> Result<()> {
    let max_retries = 3;
    let mut attempt = 0;

    loop {
        attempt += 1;
        match query("Test", None).await {
            Ok(_) => return Ok(()),
            Err(_) if attempt < max_retries => {
                let backoff = Duration::from_millis(100 * 2_u64.pow(attempt));
                tokio::time::sleep(backoff).await;
            },
            Err(e) => return Err(e.into()),
        }
    }
}
```

---

## 10. Concurrency

### Example 44: Concurrent Queries

Parallel execution for improved performance.

```bash
cargo run --example 44_concurrent_queries
```

```rust
use futures::future::join_all;

// Run queries concurrently
let futures: Vec<_> = questions.into_iter().map(|q| {
    query(q.to_string(), None)
}).collect();

let results = join_all(futures).await;
```

### Rate-Limited Concurrency

```rust
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(3)); // Max 3 concurrent

let results: Vec<_> = futures::stream::iter(queries)
    .map(|q| {
        let semaphore = semaphore.clone();
        async move {
            let _permit = semaphore.acquire().await.unwrap();
            query(q, None).await
        }
    })
    .buffer_unordered(3)
    .collect()
    .await;
```

---

## 11. Orchestration

### Example 51: Multi-Agent Orchestration

Coordinate multiple AI agents.

```bash
cargo run --example 51_orchestration
```

```rust
use claude_agent_sdk::orchestration::{
    SequentialOrchestrator, ParallelOrchestrator, Agent,
};

// Sequential pipeline: Research → Write → Edit
let agents: Vec<Box<dyn Agent>> = vec![
    create_researcher(),
    create_writer(),
    create_editor(),
];

let orchestrator = SequentialOrchestrator::new();
let output = orchestrator.orchestrate(agents, input).await?;

// Parallel analysis from multiple perspectives
let agents: Vec<Box<dyn Agent>> = vec![
    create_critic("Technical"),
    create_critic("Business"),
    create_critic("UX"),
];

let orchestrator = ParallelOrchestrator::new()
    .with_parallel_limit(3);
let output = orchestrator.orchestrate(agents, input).await?;
```

---

## 12. Auto-Install CLI

### Auto-Install Example

Automatic Claude CLI installation.

```bash
cargo run --example auto_install_cli
```

```rust
let options = ClaudeAgentOptions::builder()
    .auto_install_cli(true)
    .build();

// Client creation triggers auto-install if CLI not found
let mut client = ClaudeClient::try_new(options)?;
client.connect().await?;
```

---

## Example Quick Reference

### By Use Case

| Use Case | Recommended Examples |
|----------|---------------------|
| First time | 01, 56 |
| One-shot queries | 01, 20, 56 |
| Multi-turn conversations | 06, 16 |
| Tool control | 02, 03, 04, 05 |
| Custom behavior | 05, 15, 21 |
| Production apps | 17, 18, 43, 50 |
| High performance | 20, 44, 48 |
| Complex workflows | 51, 52 |
| Testing | 49, 50 |

### By Complexity

| Level | Examples |
|-------|----------|
| Beginner | 01, 20, 56, auto_install_cli |
| Intermediate | 06, 07, 16, 23, 43 |
| Advanced | 15, 42, 44, 51, 62 |

---

## Running Examples

### Prerequisites

1. Install Claude Code CLI (or enable auto-install)
2. Set `ANTHROPIC_API_KEY` environment variable

### Commands

```bash
# List all examples
ls crates/claude-agent-sdk/examples/

# Run specific example
cargo run --example 01_hello_world

# Run with release optimizations
cargo run --release --example 44_concurrent_queries

# Run hooks example with specific scenario
cargo run --example 15_hooks_comprehensive PreToolUse
```

---

## Related Sections

- **Section 1**: Getting Started & Core API - SDK overview
- **Section 2**: ClaudeClient Deep Dive - Client details
- **Section 3**: V2 Session API - Simplified interface
- **Section 4**: Skills System - Skill development
- **Section 5**: MCP Integration - Custom tools
- **Section 6**: Agent Orchestration - Multi-agent systems
- **Section 8**: Types Reference - Type documentation
