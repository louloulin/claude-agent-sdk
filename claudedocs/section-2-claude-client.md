# Section 2: ClaudeClient Deep Dive

This section provides an in-depth exploration of `ClaudeClient`, the core component for bidirectional streaming communication with Claude Code CLI.

## Overview

`ClaudeClient` provides the same functionality as Python's `ClaudeSDKClient`, supporting real-time bidirectional communication, streaming responses, and dynamic control over the Claude session.

### Core Structure

```rust
pub struct ClaudeClient {
    options: ClaudeAgentOptions,
    query: Option<Arc<Mutex<QueryFull>>>,
    connected: bool,
}
```

### Key Capabilities

| Feature | Description |
|---------|-------------|
| **Bidirectional Streaming** | Real-time two-way communication with Claude |
| **Session Management** | Multiple independent conversation contexts |
| **Dynamic Control** | Interrupt, change permissions, switch models mid-execution |
| **Hook Support** | Register hooks for PreToolUse, PostToolUse, and more |
| **Multimodal Input** | Send images alongside text in streaming mode |
| **File Checkpointing** | Rewind tracked files to previous states |

## Connection Lifecycle

### Creating a Client

Two constructors are available:

```rust,no_run
use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions};

// Simple creation (lazy validation)
let client = ClaudeClient::new(ClaudeAgentOptions::default());

// Early validation (catches config errors immediately)
let client = ClaudeClient::try_new(ClaudeAgentOptions::default())?;
```

**When to use `try_new()`:**
- Validating configuration before async context
- Early detection of invalid working directory
- Catching missing CLI before `connect()`

### Connection Flow

```
┌─────────────────┐
│   ClaudeClient  │
│    new/try_new  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    connect()    │  ◄── Spawns CLI subprocess
└────────┬────────┘
         │        ├── Creates SubprocessTransport
         │        ├── Initializes QueryFull
         │        ├── Starts background reader
         │        └── Sends initialization request
         ▼
┌─────────────────┐
│    Connected    │  ◄── Ready for queries
│     State       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   disconnect()  │  ◄── Clean shutdown
└─────────────────┘
```

### Complete Connection Example

```rust,no_run
use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());

    // Establish connection
    client.connect().await?;

    // Use the client...

    // Clean disconnect
    client.disconnect().await?;

    Ok(())
}
```

### Drop Warning

If `ClaudeClient` is dropped without calling `disconnect()`, a warning is printed:

```
Warning: ClaudeClient dropped without calling disconnect(). Resources may not be cleaned up properly.
```

**Best Practice:** Always call `disconnect()` before the client goes out of scope.

## Query Methods

### 2.1 `query()` - Simple Text Query

```rust
pub async fn query(&mut self, prompt: impl Into<String>) -> Result<()>
```

Sends a text prompt using the default session ("default").

**Example:**
```rust,no_run
client.query("What is the capital of France?").await?;
```

### 2.2 `query_with_session()` - Session-Specific Query

```rust
pub async fn query_with_session(
    &mut self,
    prompt: impl Into<String>,
    session_id: impl Into<String>,
) -> Result<()>
```

Sends a query to a specific conversation context. Different sessions maintain separate conversation histories.

**Example:**
```rust,no_run
// Two independent conversations
client.query_with_session("About Python", "python-session").await?;
client.query_with_session("About Rust", "rust-session").await?;
```

### 2.3 `query_with_content()` - Multimodal Query

```rust
pub async fn query_with_content(
    &mut self,
    content: impl Into<Vec<UserContentBlock>>,
) -> Result<()>
```

Sends structured content including text and images for vision-related tasks.

**Example:**
```rust,no_run
use claude_agent_sdk::UserContentBlock;

let base64_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";

client.query_with_content(vec![
    UserContentBlock::text("What's in this image?"),
    UserContentBlock::image_base64("image/png", base64_data)?,
]).await?;
```

### 2.4 `query_with_content_and_session()` - Full Control

```rust
pub async fn query_with_content_and_session(
    &mut self,
    content: impl Into<Vec<UserContentBlock>>,
    session_id: impl Into<String>,
) -> Result<()>
```

Combines multimodal input with session management.

**Example:**
```rust,no_run
use claude_agent_sdk::UserContentBlock;

client.query_with_content_and_session(
    vec![
        UserContentBlock::text("Analyze this chart"),
        UserContentBlock::image_url("https://example.com/chart.png"),
    ],
    "analysis-session",
).await?;
```

## Response Streaming

### 2.5 `receive_messages()` - Continuous Stream

```rust
pub fn receive_messages(&self) -> Pin<Box<dyn Stream<Item = Result<Message>> + Send + '_>>
```

Returns a stream that yields all messages indefinitely until the connection closes.

**Use When:**
- Processing multiple conversation turns
- Handling system events
- Long-running sessions

**Example:**
```rust,no_run
use futures::StreamExt;

let mut stream = client.receive_messages();
while let Some(message) = stream.next().await {
    match message? {
        Message::Assistant(msg) => println!("Assistant: {:?}", msg),
        Message::System(sys) => println!("System: {:?}", sys),
        Message::Result(result) => println!("Completed: ${:?}", result.total_cost_usd),
        _ => {}
    }
}
```

### 2.6 `receive_response()` - Single Turn Stream

```rust
pub fn receive_response(&self) -> Pin<Box<dyn Stream<Item = Result<Message>> + Send + '_>>
```

Returns a stream that yields messages until a `ResultMessage` is received, signaling one complete turn.

**Use When:**
- Processing one query at a time
- Waiting for complete responses
- Cost tracking per turn

**Example:**
```rust,no_run
use futures::StreamExt;

client.query("Hello Claude!").await?;

let mut stream = client.receive_response();
while let Some(message) = stream.next().await {
    match message? {
        Message::Assistant(msg) => {
            println!("Assistant response received");
        }
        Message::Result(result) => {
            println!("Turn complete! Cost: ${:.4}", result.total_cost_usd.unwrap_or(0.0));
            break;
        }
        _ => {}
    }
}
```

### Important: Stream Ownership

The stream borrows the client immutably. Drop the stream before calling query methods:

```rust,no_run
// Correct: stream goes out of scope before next query
{
    let mut stream = client.receive_response();
    while let Some(msg) = stream.next().await {
        // Process message
    }
} // stream dropped here

client.query("Next question").await?; // Now OK
```

## Dynamic Control

### 2.7 `interrupt()` - Stop Current Operation

```rust
pub async fn interrupt(&self) -> Result<()>
```

Sends an interrupt signal to stop the current Claude operation immediately.

**Use When:**
- User cancels a request
- Timeout exceeded
- Need to stop a long-running operation

**Example:**
```rust,no_run
use tokio::time::{timeout, Duration};

// Interrupt after 30 seconds
match timeout(Duration::from_secs(30), async {
    client.query("Complex analysis...").await?;
    // Process response...
    Ok::<_, claude_agent_sdk::ClaudeError>(())
})
.await
{
    Ok(result) => result?,
    Err(_) => {
        println!("Timeout, interrupting...");
        client.interrupt().await?;
    }
}
```

### 2.8 `set_permission_mode()` - Change Permissions

```rust
pub async fn set_permission_mode(&self, mode: PermissionMode) -> Result<()>
```

Dynamically changes the permission mode during an active session.

**Permission Modes:**
| Mode | Behavior |
|------|----------|
| `Default` | Standard permission prompts |
| `AcceptEdits` | Auto-accept file edits |
| `Plan` | Planning mode |
| `BypassPermissions` | No prompts (use carefully) |

**Example:**
```rust,no_run
use claude_agent_sdk::PermissionMode;

// Start with default permissions
client.query("Read my files").await?;

// Switch to auto-accept for edits
client.set_permission_mode(PermissionMode::AcceptEdits).await?;
client.query("Refactor all files").await?;

// Return to safe mode
client.set_permission_mode(PermissionMode::Default).await?;
```

### 2.9 `set_model()` - Switch AI Model

```rust
pub async fn set_model(&self, model: Option<&str>) -> Result<()>
```

Changes the AI model mid-session. Pass `None` to use the default model.

**Example:**
```rust,no_run
// Use fast model for simple tasks
client.set_model(Some("claude-sonnet-4")).await?;
client.query("Quick summary").await?;

// Switch to powerful model for complex analysis
client.set_model(Some("claude-opus-4")).await?;
client.query("Deep analysis of this codebase").await?;

// Reset to default
client.set_model(None).await?;
```

### 2.10 `rewind_files()` - File Checkpoint

```rust
pub async fn rewind_files(&self, user_message_id: &str) -> Result<()>
```

Rewinds tracked files to their state at a specific user message checkpoint.

**Requirements:**
- Enable `enable_file_checkpointing(true)` in options
- Add `extra_args={"replay-user-messages": None}` to receive `UserMessage` with UUIDs

**Example:**
```rust,no_run
use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions, Message};
use std::collections::HashMap;
use futures::StreamExt;

let options = ClaudeAgentOptions::builder()
    .enable_file_checkpointing(true)
    .extra_args(HashMap::from([("replay-user-messages".to_string(), None)]))
    .build();

let mut client = ClaudeClient::new(options);
client.connect().await?;

// Make changes and capture checkpoint
client.query("Create a new module").await?;
let mut checkpoint_id = None;

{
    let mut stream = client.receive_response();
    while let Some(Ok(msg)) = stream.next().await {
        if let Message::User(user_msg) = &msg {
            if let Some(uuid) = &user_msg.uuid {
                checkpoint_id = Some(uuid.clone());
            }
        }
    }
}

// Make more changes
client.query("Add more features").await?;

// Later, undo changes back to checkpoint
if let Some(id) = checkpoint_id {
    client.rewind_files(&id).await?;
}
```

## Session Management

### 2.11 `new_session()` - Start New Session

```rust
pub async fn new_session(
    &mut self,
    session_id: impl Into<String>,
    prompt: impl Into<String>,
) -> Result<()>
```

Convenience method to start a new conversation context.

**Example:**
```rust,no_run
// First conversation
client.query("About Python").await?;

// Start fresh conversation
client.new_session("new-topic", "About Rust").await?;
```

### 2.12 `get_server_info()` - Server Capabilities

```rust
pub async fn get_server_info(&self) -> Option<serde_json::Value>
```

Returns initialization information from Claude Code CLI including available commands and output styles.

**Example:**
```rust,no_run
if let Some(info) = client.get_server_info().await {
    if let Some(commands) = info.get("commands").and_then(|c| c.as_array()) {
        println!("Available commands: {}", commands.len());
    }
    if let Some(style) = info.get("output_style") {
        println!("Output style: {:?}", style);
    }
}
```

## Internal Architecture

### Communication Flow

```
┌──────────────────────────────────────────────────────────────┐
│                       ClaudeClient                            │
│  ┌────────────────┐  ┌─────────────────┐  ┌───────────────┐  │
│  │ query()        │  │ receive_*()     │  │ interrupt()   │  │
│  │ query_with_*() │  │                 │  │ set_*()       │  │
│  └───────┬────────┘  └────────┬────────┘  └───────┬───────┘  │
│          │                    │                   │          │
│          ▼                    ▼                   ▼          │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              QueryFull (Arc<Mutex<QueryFull>>)          │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │ │
│  │  │ stdin       │  │ message_rx  │  │ transport       │  │ │
│  │  │ (direct)    │  │ (channel)   │  │ (shared)        │  │ │
│  │  └──────┬──────┘  └──────┬──────┘  └────────┬────────┘  │ │
│  └─────────┼────────────────┼──────────────────┼───────────┘ │
└────────────┼────────────────┼──────────────────┼─────────────┘
             │                │                  │
             ▼                ▼                  ▼
      ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐
      │   stdin     │  │  Receiver  │  │ Subprocess      │
      │   stream    │  │  channel   │  │ Transport       │
      └─────────────┘  └─────────────┘  └────────┬────────┘
                                                   │
                                                   ▼
                                          ┌─────────────────┐
                                          │ Claude Code CLI │
                                          │ (subprocess)    │
                                          └─────────────────┘
```

### Lock Management

The client carefully manages locks to avoid deadlocks:

1. **Direct stdin access**: Stores `stdin` separately to bypass transport lock
2. **Arc<Mutex>** pattern: Allows concurrent read/write operations
3. **Guard dropping**: Explicitly drops guards before acquiring new ones

```rust
// Lock acquisition pattern used internally
let query_guard = query.lock().await;
let stdin = query_guard.stdin.clone();
drop(query_guard);  // Release before next lock

if let Some(stdin_arc) = stdin {
    let mut stdin_guard = stdin_arc.lock().await;
    // Use stdin...
}
```

## Comparison with Python SDK

| Feature | Python (`ClaudeSDKClient`) | Rust (`ClaudeClient`) |
|---------|---------------------------|----------------------|
| Connect | `async with client:` | `client.connect().await` |
| Disconnect | `__aexit__` | `client.disconnect().await` |
| Query | `client.query(prompt)` | `client.query(prompt).await` |
| Stream | `async for msg in client:` | `client.receive_response().next().await` |
| Interrupt | `client.interrupt()` | `client.interrupt().await` |
| Set Model | `client.set_model(model)` | `client.set_model(model).await` |
| Set Permissions | `client.set_permission_mode(mode)` | `client.set_permission_mode(mode).await` |
| Rewind Files | `client.rewind_files(id)` | `client.rewind_files(id).await` |

## Best Practices

### 1. Always Disconnect

```rust,no_run
// Use RAII pattern with a wrapper
struct ManagedClient {
    client: ClaudeClient,
}

impl Drop for ManagedClient {
    fn drop(&mut self) {
        // Note: Can't await in Drop, use tokio::task::block_in_place
        // or design your code to call disconnect() explicitly
    }
}
```

### 2. Handle Stream Lifecycle

```rust,no_run
// Process response, then drop stream
{
    let mut stream = client.receive_response();
    while let Some(msg) = stream.next().await {
        // Handle message
    }
} // Stream dropped, client available again

// Now safe to send next query
client.query("Next question").await?;
```

### 3. Use Session IDs for Context Separation

```rust,no_run
// Isolate different tasks
client.query_with_session("Project A question", "project-a").await?;
client.query_with_session("Project B question", "project-b").await?;
```

### 4. Enable Checkpointing for Destructive Operations

```rust,no_run
let options = ClaudeAgentOptions::builder()
    .enable_file_checkpointing(true)
    .extra_args(HashMap::from([("replay-user-messages".to_string(), None)]))
    .build();
```

## Related Sections

- **Section 1**: Getting Started & Core API - SDK overview and one-shot queries
- **Section 3**: V2 Session API - Simplified TypeScript-style interface
- **Section 5**: MCP Integration - Custom tools and MCP servers
- **Section 9**: Internal Layer - Transport and QueryFull implementation
