# Section 1: Getting Started & Core API

This section provides an introduction to the Claude Agent SDK for Rust, covering the SDK overview, architecture, quick start examples, and the one-shot query API.

## Overview

The Claude Agent SDK for Rust provides programmatic access to Claude Code CLI with full bidirectional streaming support and 100% feature parity with the official Python SDK.

### Key Features

| Feature | Description |
|---------|-------------|
| **Simple Query API** | One-shot queries with `query()` and `query_stream()` |
| **Bidirectional Streaming** | Real-time streaming communication with `ClaudeClient` |
| **Dynamic Control** | Interrupt, change permissions, switch models mid-execution |
| **Hooks System** | Intercept and control Claude's behavior with 6 hook types |
| **Custom Tools** | In-process MCP servers with ergonomic `tool!` macro |
| **Plugin System** | Load custom plugins to extend Claude's capabilities |
| **Permission Management** | Fine-grained control over tool execution |
| **Cost Control** | Budget limits and fallback models for production |
| **Extended Thinking** | Configure maximum thinking tokens for complex reasoning |
| **Session Management** | Resume, fork, and manage conversation sessions |
| **Multimodal Input** | Send images alongside text using base64 or URLs |

## Architecture

The SDK uses a layered architecture design:

```
┌─────────────────────────────────────────────────────────────┐
│                     Application Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │   V2 API     │  │   Query API  │  │  ClaudeClient    │   │
│  │  (simplified)│  │  (one-shot)  │  │  (bidirectional) │   │
│  └──────────────┘  └──────────────┘  └──────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Feature Layer                             │
│  ┌──────────┐ ┌───────────┐ ┌───────────┐ ┌─────────────┐   │
│  │  Skills  │ │  Agents   │ │Subagents  │ │  Commands   │   │
│  └──────────┘ └───────────┘ └───────────┘ └─────────────┘   │
│  ┌──────────┐ ┌───────────┐ ┌───────────┐ ┌─────────────┐   │
│  │   MCP    │ │   Todos   │ │Orchestration│Observability│  │
│  └──────────┘ └───────────┘ └───────────┘ └─────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Core Layer                                │
│  ┌──────────────────┐  ┌──────────────────────────────┐     │
│  │  Internal Client │  │    Transport (Subprocess)    │     │
│  └──────────────────┘  └──────────────────────────────┘     │
│  ┌──────────────────┐  ┌──────────────────────────────┐     │
│  │  Types & Config  │  │      Error Handling          │     │
│  └──────────────────┘  └──────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

### Module Structure

```
crates/claude-agent-sdk/src/
├── lib.rs          # Core SDK entry point, re-exports
├── client.rs       # ClaudeClient implementation
├── query.rs        # One-shot query API
├── errors.rs       # Error types
├── version.rs      # Version info
├── v2/             # V2 API (simplified, TypeScript-style)
├── types/          # Core type definitions
│   ├── config.rs   # Configuration options
│   ├── messages.rs # Message types
│   ├── hooks.rs    # Hook system
│   ├── mcp.rs      # MCP types
│   ├── permissions.rs
│   └── plugin.rs
├── skills/         # Skills system
├── orchestration/  # Agent orchestration
├── subagents/      # Subagent support
├── internal/       # Internal implementation
│   ├── client.rs   # Internal client
│   ├── transport/  # Subprocess transport
│   └── ...
├── observability/  # Logging and metrics
├── commands/       # Slash commands
├── mcp/            # MCP integration
├── partnership/    # Partnership features
└── todos/          # Todo list support
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
claude-agent-sdk = "0.1"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

## Quick Start

### Prerequisites

1. Install Claude Code CLI (or enable auto-install feature)
2. Set your Anthropic API key: `ANTHROPIC_API_KEY=your_key`

### Simple Query

The simplest way to interact with Claude:

```rust,no_run
use claude_agent_sdk::{query, Message, ContentBlock};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // One-shot query that collects all messages
    let messages = query("What is 2 + 2?", None).await?;

    for message in messages {
        if let Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
    }

    Ok(())
}
```

### Streaming Query

For memory-efficient processing of large responses:

```rust,no_run
use claude_agent_sdk::{query_stream, Message, ContentBlock};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Streaming query for memory-efficient processing
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

    Ok(())
}
```

### Bidirectional Client

For interactive, multi-turn conversations:

```rust,no_run
use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions, Message, PermissionMode};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .max_turns(5)
        .build();

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    // Send query
    client.query("What is Rust?").await?;

    // Receive responses
    {
        let mut stream = client.receive_response();
        while let Some(result) = stream.next().await {
            match result? {
                Message::Assistant(msg) => {
                    println!("Got assistant message");
                }
                Message::Result(_) => break,
                _ => {}
            }
        }
    } // stream is dropped here

    client.disconnect().await?;
    Ok(())
}
```

## Query API Reference

### 1.1 `query()` - Collecting Query

```rust
pub async fn query(
    prompt: impl Into<String>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Vec<Message>>
```

Sends a one-shot query and collects all messages into memory before returning.

**Use When:**
- You need all messages at once for analysis
- Response size is manageable
- Simplicity is preferred over memory efficiency

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `prompt` | `impl Into<String>` | The prompt text |
| `options` | `Option<ClaudeAgentOptions>` | Optional configuration |

**Returns:** `Result<Vec<Message>>` - All messages from the conversation

**Example:**
```rust,no_run
use claude_agent_sdk::{query, Message, ContentBlock};

let messages = query("What is 2 + 2?", None).await?;

for message in messages {
    match message {
        Message::Assistant(msg) => {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
        Message::Result(result) => {
            println!("Cost: ${:.4}", result.total_cost_usd.unwrap_or(0.0));
        }
        _ => {}
    }
}
```

### 1.2 `query_stream()` - Streaming Query

```rust
pub async fn query_stream(
    prompt: impl Into<String>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>>
```

Returns a stream that yields messages as they arrive from Claude.

**Use When:**
- Processing large responses
- Real-time message display needed
- Memory efficiency is important

**Performance Comparison:**
| Function | Memory Usage | Processing |
|----------|-------------|------------|
| `query()` | O(n) - stores all messages | Waits for completion |
| `query_stream()` | O(1) per message | Real-time |

**Example:**
```rust,no_run
use claude_agent_sdk::{query_stream, Message, ContentBlock};
use futures::stream::StreamExt;

let mut stream = query_stream("What is 2 + 2?", None).await?;

while let Some(result) = stream.next().await {
    match result? {
        Message::Assistant(msg) => {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
        _ => {}
    }
}
```

### 1.3 `query_with_content()` - Multimodal Query

```rust
pub async fn query_with_content(
    content: impl Into<Vec<UserContentBlock>>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Vec<Message>>
```

Sends mixed content including text and images.

**Supported Image Formats:**
- JPEG (`image/jpeg`)
- PNG (`image/png`)
- GIF (`image/gif`)
- WebP (`image/webp`)

**Size Limits:**
- Maximum base64 data size: 15MB

**Example with Image:**
```rust,no_run
use claude_agent_sdk::{query_with_content, UserContentBlock, Message, ContentBlock};

// Load and encode your image (this example uses a 1x1 red pixel)
let base64_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";

let messages = query_with_content(vec![
    UserContentBlock::text("What color is this image?"),
    UserContentBlock::image_base64("image/png", base64_data)?,
], None).await?;

for message in messages {
    if let Message::Assistant(msg) = message {
        for block in &msg.message.content {
            if let ContentBlock::Text(text) = block {
                println!("Claude: {}", text.text);
            }
        }
    }
}
```

**Example with Image URL:**
```rust,no_run
use claude_agent_sdk::{query_with_content, UserContentBlock};

let messages = query_with_content(vec![
    UserContentBlock::text("Describe this architecture diagram"),
    UserContentBlock::image_url("https://example.com/diagram.png"),
], None).await?;
```

### 1.4 `query_stream_with_content()` - Streaming Multimodal

```rust
pub async fn query_stream_with_content(
    content: impl Into<Vec<UserContentBlock>>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>>
```

Combines streaming benefits with multimodal input support.

**Example:**
```rust,no_run
use claude_agent_sdk::{query_stream_with_content, UserContentBlock, Message, ContentBlock};
use futures::stream::StreamExt;

let png_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

let mut stream = query_stream_with_content(vec![
    UserContentBlock::image_base64("image/png", png_base64)?,
    UserContentBlock::text("What's in this image?"),
], None).await?;

while let Some(result) = stream.next().await {
    let message = result?;
    if let Message::Assistant(msg) = message {
        for block in &msg.message.content {
            if let ContentBlock::Text(text) = block {
                print!("{}", text.text);
            }
        }
    }
}
```

## Configuration

### ClaudeAgentOptions

The main configuration structure using the builder pattern:

```rust,no_run
use claude_agent_sdk::{ClaudeAgentOptions, PermissionMode, SdkPluginConfig};

let options = ClaudeAgentOptions::builder()
    // Model selection
    .model("claude-opus-4")
    .fallback_model("claude-sonnet-4")

    // Cost control
    .max_budget_usd(10.0)
    .max_thinking_tokens(2000)
    .max_turns(10)

    // Permissions
    .permission_mode(PermissionMode::Default)

    // Plugins
    .plugins(vec![SdkPluginConfig::local("./my-plugin")])

    .build();
```

### Key Configuration Options

| Option | Type | Description |
|--------|------|-------------|
| `model` | `Option<String>` | Primary model (e.g., "claude-sonnet-4", "claude-opus-4") |
| `fallback_model` | `Option<String>` | Fallback model if primary fails |
| `max_budget_usd` | `Option<f64>` | Budget limit in USD |
| `max_thinking_tokens` | `Option<u32>` | Maximum tokens for extended thinking |
| `max_turns` | `Option<u32>` | Maximum conversation turns |
| `permission_mode` | `Option<PermissionMode>` | Permission handling mode |
| `system_prompt` | `Option<SystemPrompt>` | Custom system prompt |
| `tools` | `Option<Tools>` | Tools configuration |
| `allowed_tools` | `Vec<String>` | List of allowed tool names |
| `mcp_servers` | `McpServers` | MCP server configurations |
| `cwd` | `Option<PathBuf>` | Working directory |
| `resume` | `Option<String>` | Session ID to resume |
| `auto_install_cli` | `bool` | Auto-install CLI if missing |

### Permission Modes

```rust
pub enum PermissionMode {
    Default,           // Standard permission prompts
    AcceptEdits,       // Auto-accept file edits
    Plan,              // Planning mode
    BypassPermissions, // No permission prompts (use carefully)
}
```

## Error Handling

The SDK provides comprehensive error types:

```rust
pub enum ClaudeError {
    Connection(ConnectionError),     // CLI connection issues
    Process(ProcessError),           // Process execution errors
    JsonDecode(JsonDecodeError),     // JSON parsing errors
    MessageParse(MessageParseError), // Message format errors
    Transport(String),               // Transport layer errors
    ControlProtocol(String),         // Control protocol errors
    InvalidConfig(String),           // Configuration errors
    CliNotFound(CliNotFoundError),   // CLI not installed
    ImageValidation(ImageValidationError), // Image validation errors
    Io(std::io::Error),              // IO errors
    Other(anyhow::Error),            // Other errors
    NotFound(String),                // Resource not found
    InvalidInput(String),            // Invalid input
    InternalError(String),           // Internal SDK errors
}
```

### Error Handling Example

```rust,no_run
use claude_agent_sdk::{query, ClaudeError};

match query("Hello", None).await {
    Ok(messages) => {
        // Process messages
    }
    Err(ClaudeError::CliNotFound(e)) => {
        eprintln!("Claude CLI not found: {}", e.message);
        eprintln!("Path checked: {:?}", e.cli_path);
    }
    Err(ClaudeError::Connection(e)) => {
        eprintln!("Connection failed: {}", e.message);
    }
    Err(ClaudeError::Process(e)) => {
        eprintln!("Process failed with exit code {:?}", e.exit_code);
        eprintln!("stderr: {:?}", e.stderr);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Message Types

### Message Enum

```rust
pub enum Message {
    Assistant(AssistantMessage), // Claude's response
    System(SystemMessage),       // System notifications
    Result(ResultMessage),       // Query completion
    StreamEvent(StreamEvent),    // Stream events
    User(UserMessage),           // User messages
}
```

### ContentBlock Types

```rust
pub enum ContentBlock {
    Text(TextBlock),           // Text content
    Thinking(ThinkingBlock),   // Extended thinking
    ToolUse(ToolUseBlock),     // Tool invocation
    ToolResult(ToolResultBlock), // Tool result
    Image(ImageBlock),         // Image content
}
```

### Processing Messages

```rust,no_run
use claude_agent_sdk::{query, Message, ContentBlock};

let messages = query("What is 2 + 2?", None).await?;

for message in messages {
    match message {
        Message::Assistant(msg) => {
            println!("Session: {:?}", msg.session_id);
            for block in &msg.message.content {
                match block {
                    ContentBlock::Text(text) => {
                        println!("Text: {}", text.text);
                    }
                    ContentBlock::Thinking(thinking) => {
                        println!("Thinking: {}", thinking.thinking);
                    }
                    ContentBlock::ToolUse(tool) => {
                        println!("Tool: {} ({})", tool.name, tool.id);
                    }
                    _ => {}
                }
            }
        }
        Message::Result(result) => {
            println!("Completed in {}ms", result.duration_ms);
            println!("Cost: ${:.4}", result.total_cost_usd.unwrap_or(0.0));
            println!("Turns: {}", result.num_turns);
        }
        Message::System(sys) => {
            println!("System: {} - {:?}", sys.subtype, sys.session_id);
        }
        _ => {}
    }
}
```

## Related Sections

- **Section 2**: ClaudeClient Deep Dive - Bidirectional streaming and dynamic control
- **Section 3**: V2 Session API - Simplified TypeScript-style interface
- **Section 4**: Skills System - Custom capabilities
- **Section 5**: MCP Integration - Custom tools
- **Section 8**: Types Reference - Complete type documentation
