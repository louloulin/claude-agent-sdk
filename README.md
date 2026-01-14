# Claude Agent SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/cc-agent-sdk.svg)](https://crates.io/crates/cc-agent-sdk)
[![Documentation](https://docs.rs/cc-agent-sdk/badge.svg)](https://docs.rs/cc-agent-sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE.md)

[English](README.md) | [中文文档](README.zh-CN.md)

> 🦀 **Production-Ready Rust SDK** for Claude Agent with type-safe, high-performance API.

The Claude Agent SDK for Rust provides comprehensive programmatic access to Claude's capabilities with zero-cost abstractions, compile-time memory safety, and true concurrent processing.

---

## 📖 Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Core APIs](#core-apis)
  - [Simple Query API](#1-simple-query-api)
  - [Streaming API](#2-streaming-api)
  - [Bidirectional Client](#3-bidirectional-client)
  - [V2 Session API](#4-v2-session-api)
- [Advanced Features](#advanced-features)
  - [Hooks System](#hooks-system)
  - [Skills System](#skills-system)
  - [MCP Integration](#mcp-integration)
  - [Subagents](#subagents)
- [Examples](#examples)

---

## ✨ Features

- **🚀 Complete V2 API** - Full TypeScript-inspired session-based API
- **🔄 Bidirectional Streaming** - Real-time communication with Claude
- **🪝 Hooks System** - Intercept and control Claude's behavior
- **🧠 Skills System** - Enhanced with validation, security audit, and progressive disclosure
- **🤖 Subagents** - Full agent delegation and orchestration support
- **🔌 MCP Integration** - Model Context Protocol server support
- **⚡ Slash Commands** - Command registration and execution framework
- **📊 Observability** - Comprehensive logging and metrics collection
- **🛡️ Type Safety** - Compile-time guarantees for agent configurations
- **⚡ High Performance** - Zero-cost abstractions and lock-free architecture

---

## 🚀 Quick Start

### Prerequisites

- **Rust**: 1.90 or higher
- **Claude Code CLI**: Version 2.0.0 or higher
- **API Key**: `ANTHROPIC_API_KEY` environment variable set

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cc-agent-sdk = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Basic Usage

```rust
use claude_agent_sdk::{query, Message, ContentBlock};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Simple one-shot query
    let messages = query("What is 2 + 2?", None).await?;

    for message in messages {
        if let Message::Assistant(msg) = message {
            for block in msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
    }

    Ok(())
}
```

---

## 🔧 Core APIs

The SDK provides four main API styles for different use cases:

### 1. Simple Query API

**Best for**: One-shot queries, quick prototypes, simple use cases

```rust
use claude_agent_sdk::{query, Message, ContentBlock};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let messages = query("What is 2 + 2?", None).await?;
    // ... process messages ...
    Ok(())
}
```

**Key Functions**:
- `query(prompt, options)` - Collect all messages into a Vec
- `query_with_content(content_blocks, options)` - Send structured content (images + text)

### 2. Streaming API

**Best for**: Memory-efficient processing, real-time responses

```rust
use claude_agent_sdk::{query_stream, Message, ContentBlock};
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut stream = query_stream("Explain Rust ownership", None).await?;

    while let Some(result) = stream.next().await {
        let message = result?;
        if let Message::Assistant(msg) = message {
             for block in msg.message.content {
                if let ContentBlock::Text(text) = block {
                    print!("{}", text.text);
                }
            }
        }
    }
    Ok(())
}
```

**Key Functions**:
- `query_stream(prompt, options)` - Returns a stream of messages
- `query_stream_with_content(content_blocks, options)` - Stream with structured content

### 3. Bidirectional Client

**Best for**: Full control, multi-turn conversations, dynamic control flow

```rust
use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions, Message};
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions::default();
    let mut client = ClaudeClient::new(options);

    client.connect().await?;
    client.query("What is Rust?").await?;

    {
        let mut stream = client.receive_response();
        while let Some(result) = stream.next().await {
            // Process real-time messages
            if let Ok(Message::Result(_)) = result { break; }
        }
    }

    client.disconnect().await?;
    Ok(())
}
```

### 4. V2 Session API

**Best for**: TypeScript-style sessions, clean send/receive pattern

```rust
use claude_agent_sdk::v2::{create_session, SessionConfigBuilder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = SessionConfigBuilder::default()
        .model("claude-sonnet-4-5")
        .build()?;

    let mut session = create_session(config).await?;

    // Send message and receive response
    session.send("What is Rust?").await?;
    let messages = session.receive().await?;
    
    // Context is automatically maintained
    session.send("What are its key features?").await?;
    let messages = session.receive().await?;

    Ok(())
}
```

---

## 🏗️ Advanced Features

### Hooks System

Hooks allow you to intercept and control Claude's behavior at 8 key points in the execution lifecycle.

### Available Hooks

| Hook Type | Description | Use Case |
|-----------|-------------|----------|
| `PreToolUse` | Before tool execution | Log/modify tool usage |
| `PostToolUse` | After tool execution | Process tool results |
| `PreMessage` | Before sending message | Filter/transform messages |
| `PostMessage` | After receiving message | Log incoming messages |
| `PromptStart` | When prompt starts | Initialize context |
| `PromptEnd` | When prompt ends | Cleanup context |
| `SubagentStop` | When subagent stops | Process subagent results |
| `PreCompact` | Before conversation compaction | Preserve important context |

### Example: Pre-Tool Hook

```rust
use claude_agent_sdk::{
    HookEvent, HookMatcher, ClaudeAgentOptionsBuilder
};
use std::sync::Arc;

let pre_tool_hook = |input, tool_use_id, context| {
    Box::pin(async move {
        // Log tool usage
        println!("Tool {} called with: {:?}", tool_use_id, input);

        // Optionally modify input or add context
        Ok(serde_json::json!({
            "logged": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    })
};

let hooks = vec![
    HookMatcher::builder()
        .hook_event(HookEvent::PreToolUse)
        .hook(Arc::new(pre_tool_hook))
        .build()
];

let options = ClaudeAgentOptionsBuilder::default()
    .hooks(hooks)
    .build()?;
```

### Example: Post-Message Hook

```rust
let post_message_hook = |message, context| {
    Box::pin(async move {
        // Process received message
        if let Some(text) = message.get("content") {
            println!("Received: {}", text);
        }

        Ok(serde_json::json!({}))
    })
};

let hooks = vec![
    HookMatcher::builder()
        .hook_event(HookEvent::PostMessage)
        .hook(Arc::new(post_message_hook))
        .build()
];
```

### Hook Context

All hooks receive a context object with:

```rust
pub struct HookContext {
    pub turn_id: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub custom_data: HashMap<String, serde_json::Value>,
}
```

### Skills System

The Skills System provides enhanced capabilities with validation, security auditing, and progressive disclosure.

```rust
use claude_agent_sdk::skills::{SkillMdFile, SkillMdValidator, SkillAuditor};

// Load and validate SKILL.md
let validator = SkillMdValidator::new();
let skill_file = SkillMdFile::load("skills/my-skill/SKILL.md")?;
let result = validator.validate(&skill_file)?;

// Audit for security risks
let auditor = SkillAuditor::new();
let audit = auditor.audit_skill(&skill_file)?;

if audit.has_risky_patterns() {
    println!("Security risks detected: {:?}", audit.risks());
}
```

### MCP Integration

Support for Model Context Protocol (MCP) servers and tools.

```rust
use claude_agent_sdk::{tool, create_sdk_mcp_server, ToolResult};

// Define tool handler and create server
let my_tool = tool!(
    "my-tool", "Description",
    json!({ "type": "object", "properties": { "name": {"type": "string"} } }),
    |args| async move {
        Ok(ToolResult { content: vec![], is_error: false })
    }
);

let server = create_sdk_mcp_server("my-server", "1.0.0", vec![my_tool]);
```

### Subagents

Delegate tasks to specialized sub-agents.

```rust
use claude_agent_sdk::{AgentRegistry, SimpleAgent, AgentMetadata, AgentOutput};
use claude_agent_sdk::orchestration::SequentialOrchestrator;

let researcher = SimpleAgent::new("researcher", "Researcher", |input| async move {
    Ok(AgentOutput::new(format!("Researched: {}", input.content)))
});

let mut registry = AgentRegistry::new();
registry.register(Box::new(researcher), AgentMetadata::new("researcher", "Researcher", "Academic research", "research")).await?;
```

### Multimodal Support

Query with images and text.

```rust
use claude_agent_sdk::{query_with_content, UserContentBlock};

let image_data = std::fs::read("image.png")?;
let base64_image = base64::encode(&image_data);

let messages = query_with_content(vec![
    UserContentBlock::text("What's in this image?"),
    UserContentBlock::image_base64("image/png", &base64_image)?,
], None).await?;
```

---

## Examples

Check out the [examples/](examples/) directory for complete working examples:

- [Simple Query](examples/simple_query.rs)
- [Streaming Query](examples/streaming_query.rs)
- [V2 Session](examples/v2_session.rs)
- [Hooks Usage](examples/hooks_demo.rs)
- [Skills Demo](examples/skills_demo.rs)

---

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
