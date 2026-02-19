# Section 3: V2 Session API

This section documents the V2 API, which provides a simplified, TypeScript-inspired interface for interacting with Claude.

## Overview

The V2 API offers a more ergonomic interface compared to V1, with:
- **One-shot prompts**: Simple `prompt()` function for single queries
- **Session-based API**: `create_session()` and `resume_session()` for multi-turn conversations
- **Simplified options**: `SessionOptions` with only commonly used parameters
- **TypeScript-style naming**: `prompt`, `send`, `receive` instead of `query`, `query_with_prompt`

## Module Structure

```
v2/
├── mod.rs      # One-shot prompt() function + module exports (264 lines, 3 tests)
├── session.rs  # Session-based API (322 lines, 1 test)
└── types.rs    # Simplified type definitions (454 lines, 9 tests)
```

**Total**: 1,040 lines, 13 tests

## Quick Start

```rust,no_run
use claude_agent_sdk::v2::{prompt, create_session};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // One-shot prompt
    let result = prompt("What is 2 + 2?", Default::default()).await?;
    println!("Answer: {}", result.content);

    // Session-based conversation
    let mut session = create_session(Default::default()).await?;
    session.send("Hello, Claude!").await?;

    for message in session.receive().await? {
        println!("Message: {:?}", message);
    }

    Ok(())
}
```

## V1 vs V2 Comparison

### One-shot Queries

```rust,ignore
// V1
let messages = query("Question", None).await?;
for msg in messages {
    if let Message::Assistant(assist_msg) = msg {
        // process...
    }
}

// V2
let result = prompt("Question", Default::default()).await?;
// result.content has the answer text
```

### Session-based

```rust,ignore
// V1
let mut client = ClaudeClient::new(options);
client.connect().await?;
client.query("Hello").await?;
let stream = client.receive_response();

// V2
let mut session = create_session(Default::default()).await?;
session.send("Hello").await?;
let messages = session.receive().await?;
```

## API Reference

### 3.1 One-shot Prompt

```rust
pub async fn prompt(
    prompt: impl Into<String>,
    options: SessionOptions,
) -> Result<PromptResult>
```

Sends a single prompt to Claude and returns the complete response.

**Parameters**:
- `prompt` - The prompt text (accepts `&str`, `String`, etc.)
- `options` - Session configuration options

**Returns**: `PromptResult` with:
- `content: String` - Response text
- `input_tokens: u64` - Input token count
- `output_tokens: u64` - Output token count
- `model: Option<String>` - Model used (if available)

**Example**:
```rust,no_run
use claude_agent_sdk::v2::prompt;

let result = prompt("What is 2 + 2?", Default::default()).await?;
println!("Response: {}", result.content);
println!("Tokens: {}", result.total_tokens());
println!("Est. Cost: ${:.4}", result.estimated_cost_usd());
```

### 3.2 Session Management

#### create_session

```rust
pub async fn create_session(options: SessionOptions) -> Result<Session>
```

Creates a new session with auto-generated UUID and connects to Claude.

**Example**:
```rust,no_run
use claude_agent_sdk::v2::{create_session, SessionOptions};

let session = create_session(SessionOptions::default()).await?;
println!("Session ID: {}", session.id);
```

#### resume_session

```rust
pub async fn resume_session(
    session_id: &str,
    options: SessionOptions,
) -> Result<Session>
```

Resumes an existing session by ID. Note: Full session persistence is not yet implemented; currently creates a new session with the provided ID.

**Example**:
```rust,no_run
use claude_agent_sdk::v2::{resume_session, SessionOptions};

let session = resume_session("existing-session-id", SessionOptions::default()).await?;
```

### 3.3 Session Struct

```rust
pub struct Session {
    pub id: String,
    pub options: SessionOptions,
    // internal client...
}
```

**Methods**:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `send(&mut self, message)` | `Result<()>` | Send message to Claude |
| `receive(&self)` | `Result<Vec<V2Message>>` | Receive Claude's responses |
| `model(&self)` | `Option<String>` | Get model being used |
| `is_connected(&self)` | `bool` | Check connection status |
| `close(self)` | `Result<()>` | Close and release resources |

**Example**:
```rust,no_run
use claude_agent_sdk::v2::{create_session, SessionOptions};

let mut session = create_session(SessionOptions::default()).await?;

// Send a message
session.send("What is 2 + 2?").await?;

// Receive responses
let messages = session.receive().await?;
for msg in messages {
    if let Some(text) = msg.as_text() {
        println!("Claude: {}", text);
    }
}

// Close session
session.close().await?;
```

### 3.4 SessionOptions

```rust
#[derive(Debug, Clone, TypedBuilder, Serialize, Deserialize, Default)]
pub struct SessionOptions {
    pub model: Option<String>,
    pub permission_mode: Option<PermissionMode>,
    pub max_budget_usd: Option<f64>,
    pub max_turns: Option<u32>,
    pub max_thinking_tokens: Option<u32>,
    pub system_prompt: Option<String>,
    pub include_partial_messages: bool,
}
```

**Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `model` | `Option<String>` | `None` | Model to use (None = system default) |
| `permission_mode` | `Option<PermissionMode>` | `None` | Tool permission handling |
| `max_budget_usd` | `Option<f64>` | `None` | Budget limit in USD |
| `max_turns` | `Option<u32>` | `None` | Max conversation turns |
| `max_thinking_tokens` | `Option<u32>` | `None` | Extended thinking token limit |
| `system_prompt` | `Option<String>` | `None` | Custom system prompt |
| `include_partial_messages` | `bool` | `false` | Include partial stream messages |

**Builder Example**:
```rust
use claude_agent_sdk::v2::{SessionOptions, PermissionMode};

let options = SessionOptions::builder()
    .model("claude-sonnet-4-20250514".to_string())
    .max_turns(10)
    .permission_mode(PermissionMode::BypassPermissions)
    .build();
```

### 3.5 PromptResult

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResult {
    pub content: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model: Option<String>,
}
```

**Methods**:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `total_tokens(&self)` | `u64` | Sum of input + output tokens |
| `estimated_cost_usd(&self)` | `f64` | Approximate cost (Input: $3/M, Output: $15/M) |

**Example**:
```rust
let result = PromptResult {
    content: "The answer is 4".to_string(),
    input_tokens: 15,
    output_tokens: 5,
    model: Some("claude-sonnet-4-20250514".to_string()),
};

println!("Response: {}", result.content);
println!("Total tokens: {}", result.total_tokens());
println!("Est. cost: ${:.6}", result.estimated_cost_usd());
```

### 3.6 V2Message (Session)

```rust
#[derive(Debug, Clone)]
pub enum V2Message {
    Assistant {
        content: String,
    },
}
```

Simplified message type for session `receive()` method.

**Methods**:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `as_text(&self)` | `Option<&str>` | Get text content |

### 3.7 Message (Types)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    User { content: String },
    Assistant { content: String },
    ToolResult { tool_name: String, result: String },
}
```

Extended message type for V2 API with user, assistant, and tool result variants.

**Methods**:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `as_text(&self)` | `Option<&str>` | Get text (None for ToolResult) |
| `is_user(&self)` | `bool` | Check if user message |
| `is_assistant(&self)` | `bool` | Check if assistant message |
| `is_tool_result(&self)` | `bool` | Check if tool result |

### 3.8 PermissionMode

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionMode {
    Default,
    AcceptEdits,
    Plan,
    BypassPermissions,
}
```

Controls how Claude requests permission to use tools.

| Variant | Description |
|---------|-------------|
| `Default` | Standard permission prompting |
| `AcceptEdits` | Auto-accept edit operations |
| `Plan` | Planning mode (no execution) |
| `BypassPermissions` | Auto-approve all tool usage |

## Implementation Details

### prompt() Implementation

```rust
pub async fn prompt(prompt: impl Into<String>, options: SessionOptions) -> Result<PromptResult>
```

**Flow**:
1. Convert `SessionOptions` to `ClaudeAgentOptions`
2. Create new `ClaudeClient` with options
3. Call `client.connect().await`
4. Call `client.query(&prompt_text).await`
5. Stream messages via `receive_response()`
6. Extract text from `Assistant` messages
7. Parse token usage from `usage` JSON field
8. Return `PromptResult`

**Key Analysis**:
- ✅ Simple, focused implementation
- ✅ Proper token extraction from usage JSON
- ⚠️ Creates new client per call (no connection pooling)
- ✅ Handles all message types gracefully (ignores non-text)

### Session Implementation

**Session Structure**:
```rust
pub struct Session {
    pub id: String,
    pub options: SessionOptions,
    client: Arc<Mutex<ClaudeClient>>,
}
```

- Uses `Arc<Mutex<...>>` for thread-safe client sharing
- UUID-based session IDs via `uuid::Uuid::new_v4()`

**send() Method**:
1. Validates message is not empty (returns `InvalidInput` error)
2. Locks client mutex
3. Calls `client.query(&message_text).await`

**receive() Method**:
1. Locks client (doesn't await while holding lock - potential issue)
2. Gets stream from `receive_response()`
3. Iterates stream until `Result` message (end of turn)
4. Extracts text from `Assistant` content blocks
5. Returns `Vec<V2Message>`

**Analysis**:
- ⚠️ `receive()` holds mutex during stream iteration - could block concurrent sends
- ✅ Empty message validation with clear error
- ✅ Clean end-of-turn detection via `Result` message

### SessionOptions Conversion

The `From<SessionOptions> for ClaudeAgentOptions` implementation uses an 8-branch match expression to handle all combinations of optional fields:

```rust
match (options.model, permission_mode, options.max_budget_usd) {
    (Some(model), Some(pm), Some(max_budget)) => { /* ... */ }
    (Some(model), Some(pm), None) => { /* ... */ }
    // ... 6 more branches
}
```

**Analysis**:
- ⚠️ Verbose - 8 nearly identical branches
- ⚠️ Uses `unwrap_or(0)` for `max_turns` and `max_thinking_tokens` - should preserve None
- ⚠️ Creates empty `SystemPrompt::Text(String::new())` when None - may not be correct
- 💡 Could be simplified with `..Default::default()` pattern or custom builder

## Error Handling

| Scenario | Error Type | Message |
|----------|------------|---------|
| Empty message | `ClaudeError::InvalidInput` | "Message cannot be empty" |
| Connection failure | `ClaudeError::ConnectionError` | From underlying client |
| Query failure | `ClaudeError::QueryError` | From underlying client |
| Response parsing | `ClaudeError::ParseError` | From underlying client |

## Test Coverage

**mod.rs** (3 tests):
- `test_prompt_result_structure` - Struct creation and `total_tokens()`
- `test_session_options_default` - Default options creation
- `test_session_options_builder` - Builder pattern

**session.rs** (1 test):
- `test_v2_message_as_text` - V2Message text extraction

**types.rs** (9 tests):
- `test_session_options_builder` - Builder pattern
- `test_permission_mode_conversion` - V2 to V1 conversion
- `test_prompt_result_total_tokens` - Token calculation
- `test_message_is_user` - User variant check
- `test_message_is_assistant` - Assistant variant check
- `test_message_is_tool_result` - ToolResult variant check
- `test_prompt_result_cost_estimation` - Cost calculation

**Total**: 13 tests

**Coverage Assessment**:
- ✅ Core functionality tested
- ⚠️ No integration tests for actual API calls
- ⚠️ No tests for `prompt()` function behavior
- ⚠️ No tests for session `send()`/`receive()` flow
- ⚠️ No tests for `resume_session()`

## Security Analysis

| Concern | Status | Notes |
|---------|--------|-------|
| Message validation | ✅ | Empty message check |
| Input sanitization | ⚠️ | No sanitization of prompt text |
| Session ID generation | ✅ | UUID via `uuid` crate |
| Concurrent access | ✅ | Arc<Mutex<...>> pattern |

## Performance Analysis

| Operation | Complexity | Notes |
|-----------|------------|-------|
| `prompt()` | O(n) | n = response size (streaming) |
| `send()` | O(1) | Async query dispatch |
| `receive()` | O(n) | n = message count |
| Session creation | O(1) | UUID generation + connect |

**Performance Considerations**:
1. **No connection pooling**: Each `prompt()` call creates new client
2. **Mutex contention**: `receive()` holds lock during stream iteration
3. **Memory**: Full message content stored in memory

## Feature Parity

V2 provides the same functionality as V1 with a simpler API:

| Feature | V1 | V2 |
|---------|----|----|
| One-shot queries | `query()` | `prompt()` |
| Multi-turn sessions | `ClaudeClient` | `Session` |
| Streaming | `receive_response()` | `receive()` |
| Permission management | `PermissionMode` | `PermissionMode` |
| Cost control | `max_budget_usd` | `max_budget_usd` |
| Custom tools | Full support | Via options |
| Hooks | Full support | Via options |
| Session resumption | Partial | `resume_session()` |

## API Quality Assessment

| Aspect | Rating | Notes |
|--------|--------|-------|
| Documentation | ✅ Excellent | Comprehensive doc comments |
| Examples | ✅ Good | In-doc examples + quick start |
| Error handling | ✅ Good | Clear errors |
| Type safety | ✅ Good | TypedBuilder + Serde |
| Ergonomics | ✅ Excellent | Very simple API |
| Consistency | ✅ Good | Follows Rust conventions |

## Findings Summary

**Critical Issues**: 0

**Important Issues**: 0

**Suggestions**:
1. **Performance**: Add connection pooling for `prompt()` to avoid creating new client each call
2. **Concurrency**: Consider releasing mutex before stream iteration in `receive()`
3. **Refactor**: Simplify `From<SessionOptions>` with builder defaults or pattern matching
4. **Defaults**: Preserve `None` for optional fields instead of `unwrap_or(0)`
5. **Testing**: Add integration tests for session flow
6. **SystemPrompt**: Handle None case better than empty string

**Positive Notes**:
- Very clean, ergonomic API design
- Excellent documentation with examples
- TypeScript-friendly naming conventions
- TypedBuilder for compile-time safety
- Good separation of V1 and V2 concerns
- Simplified options reduce cognitive load
- Auto-connect on session creation improves DX

## Migration Guide

### From V1 to V2

**One-shot queries**:
```rust,ignore
// V1
let messages = query("Question", None).await?;
for msg in messages {
    if let Message::Assistant(assist_msg) = {
        // process...
    }
}

// V2
let result = prompt("Question", Default::default()).await?;
// result.content has the answer text
```

**Session-based**:
```rust,ignore
// V1
let mut client = ClaudeClient::new(options);
client.connect().await?;
client.query("Hello").await?;
let stream = client.receive_response();

// V2
let mut session = create_session(Default::default()).await?;
session.send("Hello").await?;
let messages = session.receive().await?;
```

**Options conversion**:
```rust,ignore
// V1
let options = ClaudeAgentOptions::builder()
    .model("claude-sonnet-4-20250514".to_string())
    .permission_mode(PermissionMode::BypassPermissions)
    .build();

// V2
let options = SessionOptions::builder()
    .model("claude-sonnet-4-20250514".to_string())
    .permission_mode(PermissionMode::BypassPermissions)
    .build();
```
