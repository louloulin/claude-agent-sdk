# Section 7: Subagents System

## 7.1 Overview

The Subagents module provides a system for creating and managing specialized Claude instances with specific capabilities and instructions. This enables multi-agent architectures where different subagents handle different tasks.

**Module Location**: `crates/claude-agent-sdk/src/subagents/`

**Key Components**:
- **Subagent**: Configuration for a specialized agent instance
- **SubagentExecutor**: Runtime manager for executing subagents
- **DelegationStrategy**: Control flow for task delegation
- **SubagentConfig**: Multi-subagent configuration container

## 7.2 Core Types

### 7.2.1 Subagent

A `Subagent` represents a specialized Claude instance with specific capabilities.

```rust
pub struct Subagent {
    pub name: String,              // Unique identifier
    pub description: String,       // Purpose description
    pub instructions: String,      // Specific behavior instructions
    pub allowed_tools: Vec<String>, // Tool whitelist
    pub max_turns: Option<u32>,    // Turn limit (None = unlimited)
    pub model: Option<String>,     // Model override (None = default)
}
```

**Example**:
```rust
use claude_agent_sdk::subagents::Subagent;

let code_reviewer = Subagent {
    name: "code-reviewer".to_string(),
    description: "Expert code reviewer".to_string(),
    instructions: "Review code for bugs and best practices".to_string(),
    allowed_tools: vec!["Read".to_string(), "Grep".to_string()],
    max_turns: Some(5),
    model: Some("claude-sonnet-4".to_string()),
};
```

### 7.2.2 DelegationStrategy

Controls how tasks are delegated to subagents:

```rust
pub enum DelegationStrategy {
    Auto,      // Claude automatically decides when to delegate
    Manual,    // Requires explicit SubagentTool calls
    ToolCall,  // Delegate through tool calls
}
```

| Strategy | Description | Use Case |
|----------|-------------|----------|
| `Auto` | Claude decides autonomously | General-purpose delegation |
| `Manual` | Explicit control required | Deterministic workflows |
| `ToolCall` | Tool-mediated delegation | Integration with tool systems |

### 7.2.3 SubagentCall

Represents a single subagent execution request:

```rust
pub struct SubagentCall {
    pub subagent_name: String,
    pub input: String,
    pub output: Option<String>,
}
```

**Methods**:
- `new(name, input)` - Creates new call
- `is_executed()` - Checks if output exists

**Example**:
```rust
use claude_agent_sdk::subagents::SubagentCall;

let call = SubagentCall::new("code-reviewer", "Review src/main.rs");
assert!(!call.is_executed());
```

### 7.2.4 SubagentOutput

Result from subagent execution:

```rust
pub struct SubagentOutput {
    pub subagent_name: String,
    pub messages: Vec<serde_json::Value>,
}
```

> **Note**: Messages are serialized to `serde_json::Value` for flexibility.

### 7.2.5 SubagentError

Error types for subagent operations:

```rust
pub enum SubagentError {
    NotFound(String),       // Subagent not found
    AlreadyExists(String),  // Duplicate name on registration
    ExecutionFailed(String), // Query execution failed
    InvalidInput(String),   // Invalid input provided
}
```

## 7.3 SubagentConfig

Configuration container for managing multiple subagents:

```rust
pub struct SubagentConfig {
    pub subagents: Vec<Subagent>,
    pub delegation_strategy: DelegationStrategy,
}
```

**Methods**:
- `new(strategy)` - Creates empty configuration
- `add_subagent(subagent)` - Appends subagent to list
- `get_subagent(name)` - Linear search by name (O(n))
- `to_map()` - Converts to HashMap for O(1) lookup

**Example**:
```rust
use claude_agent_sdk::subagents::{SubagentConfig, Subagent, DelegationStrategy};

let mut config = SubagentConfig::new(DelegationStrategy::Auto);

config.add_subagent(Subagent {
    name: "reviewer".to_string(),
    description: "Code reviewer".to_string(),
    instructions: "Review code for quality".to_string(),
    allowed_tools: vec!["Read".to_string()],
    max_turns: Some(5),
    model: None,
});

// For frequent lookups, convert to map
let map = config.to_map(); // O(1) lookups
```

> **Performance Note**: `get_subagent()` uses O(n) linear search. Use `to_map()` for frequent lookups.

## 7.4 SubagentExecutor

The runtime executor that manages and executes subagents.

### 7.4.1 Creation

```rust
use claude_agent_sdk::subagents::{SubagentExecutor, DelegationStrategy};

let executor = SubagentExecutor::new(DelegationStrategy::Auto);
```

### 7.4.2 Registration

```rust
let mut executor = SubagentExecutor::new(DelegationStrategy::Auto);

let subagent = Subagent {
    name: "my-agent".to_string(),
    description: "Description".to_string(),
    instructions: "Instructions".to_string(),
    allowed_tools: vec!["Read".to_string()],
    max_turns: Some(5),
    model: None,
};

executor.register(subagent)?; // Returns AlreadyExists error if duplicate
```

### 7.4.3 Execution

```rust
async fn run_subagent(executor: &SubagentExecutor) -> Result<(), Box<dyn std::error::Error>> {
    let output = executor.execute("my-agent", "Process this input").await?;
    println!("Subagent {} returned {} messages",
        output.subagent_name,
        output.messages.len()
    );
    Ok(())
}
```

### 7.4.4 Query Methods

```rust
// List all registered subagents
let names: Vec<String> = executor.list_subagents();

// Check if subagent exists
if executor.has_subagent("my-agent") {
    println!("Agent exists");
}

// Get current strategy
let strategy: &DelegationStrategy = executor.strategy();
```

## 7.5 Execution Flow

The `execute()` method follows this flow:

```
execute(name, input)
    │
    ├─► Lookup subagent by name (O(1) HashMap)
    │       └─► Return NotFound error if missing
    │
    ├─► Build system prompt
    │       description + "\n\nInstructions:\n" + instructions
    │
    ├─► Build ClaudeAgentOptions
    │       ├─► Both model + max_turns specified
    │       ├─► Only model specified
    │       ├─► Only max_turns specified
    │       └─► Neither specified (defaults)
    │
    ├─► Call crate::query::query()
    │       └─► Wrap errors in ExecutionFailed
    │
    └─► Serialize messages to JSON
            └─► Return SubagentOutput
```

## 7.6 Error Handling

| Error | Trigger | Recovery |
|-------|---------|----------|
| `NotFound` | Subagent name not registered | Register subagent first |
| `AlreadyExists` | Duplicate name on registration | Use unique names |
| `ExecutionFailed` | Query API error | Check API status, credentials |
| `InvalidInput` | Invalid input (currently unused) | Validate input before calling |

**Example Error Handling**:
```rust
use claude_agent_sdk::subagents::{SubagentExecutor, SubagentError, DelegationStrategy};

async fn safe_execute(executor: &SubagentExecutor, name: &str, input: &str) {
    match executor.execute(name, input).await {
        Ok(output) => println!("Success: {} messages", output.messages.len()),
        Err(SubagentError::NotFound(name)) => {
            eprintln!("Subagent '{}' not registered", name);
        }
        Err(SubagentError::ExecutionFailed(msg)) => {
            eprintln!("Execution failed: {}", msg);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## 7.7 Security Considerations

### Tool Access Control

Subagents use `allowed_tools` as a whitelist for tool access:

```rust
let restricted_agent = Subagent {
    name: "read-only".to_string(),
    description: "Read-only analysis".to_string(),
    instructions: "Analyze without modifications".to_string(),
    allowed_tools: vec!["Read".to_string(), "Grep".to_string()],
    // No Write, Edit, Bash tools
    max_turns: None,
    model: None,
};
```

### Known Security Considerations

| Concern | Status | Notes |
|---------|--------|-------|
| Tool access control | ✅ Protected | `allowed_tools` whitelist |
| Input validation | ⚠️ None | No validation of input string |
| Name injection | ⚠️ None | No sanitization of subagent name |
| Instruction injection | ⚠️ None | User-provided instructions passed directly |

**Recommendation**: Validate subagent names (alphanumeric, length limits) and sanitize user inputs in production use.

## 7.8 Performance Characteristics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| `register()` | O(1) amortized | HashMap insert |
| `execute()` lookup | O(1) | HashMap get |
| `list_subagents()` | O(n) | Keys iteration |
| `has_subagent()` | O(1) | HashMap contains_key |
| `SubagentConfig::get_subagent()` | O(n) | Linear search ⚠️ |
| `SubagentConfig::to_map()` | O(n) | One-time conversion |

## 7.9 API Reference

### Re-exports (mod.rs)

```rust
pub use types::{
    DelegationStrategy,
    Subagent,
    SubagentCall,
    SubagentConfig,
    SubagentError,
    SubagentOutput,
};
```

### SubagentExecutor Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `(strategy: DelegationStrategy) -> Self` | Create executor |
| `register` | `(&mut self, subagent: Subagent) -> Result<(), SubagentError>` | Register subagent |
| `execute` | `(&self, name: &str, input: &str) -> Result<SubagentOutput, SubagentError>` | Execute async |
| `list_subagents` | `(&self) -> Vec<String>` | Get all names |
| `has_subagent` | `(&self, name: &str) -> bool` | Check existence |
| `strategy` | `(&self) -> &DelegationStrategy` | Get strategy |

## 7.10 Complete Example

```rust
use claude_agent_sdk::subagents::{
    Subagent, SubagentConfig, SubagentExecutor, DelegationStrategy
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create executor with auto delegation
    let mut executor = SubagentExecutor::new(DelegationStrategy::Auto);

    // Define specialized subagents
    let code_reviewer = Subagent {
        name: "code-reviewer".to_string(),
        description: "Expert code reviewer focusing on quality and best practices".to_string(),
        instructions: r#"
            Analyze code for:
            - Bugs and potential errors
            - Security vulnerabilities
            - Performance issues
            - Code style and maintainability
        "#.to_string(),
        allowed_tools: vec!["Read".to_string(), "Grep".to_string()],
        max_turns: Some(5),
        model: Some("claude-sonnet-4".to_string()),
    };

    let doc_writer = Subagent {
        name: "doc-writer".to_string(),
        description: "Technical documentation specialist".to_string(),
        instructions: "Write clear, comprehensive documentation".to_string(),
        allowed_tools: vec!["Read".to_string(), "Write".to_string()],
        max_turns: Some(3),
        model: None, // Use default model
    };

    // Register subagents
    executor.register(code_reviewer)?;
    executor.register(doc_writer)?;

    // List available subagents
    println!("Available subagents: {:?}", executor.list_subagents());

    // Execute a subagent
    let output = executor.execute("code-reviewer", "Review src/lib.rs").await?;
    println!("Received {} messages from {}", output.messages.len(), output.subagent_name);

    Ok(())
}
```

## 7.11 Test Coverage

**Unit Tests** (17 total):
- types.rs: 9 tests covering struct creation, config operations, error display
- mod.rs: 5 tests covering executor operations and error cases
- All tests passing

**Coverage Gaps**:
- No integration test for successful `execute()` path (requires API credentials)
- No tests for `InvalidInput` error variant

## 7.12 Design Notes

### Design Decisions

1. **HashMap Storage**: `SubagentExecutor` uses `HashMap<String, Subagent>` for O(1) lookup
2. **Serde Support**: All types support serialization for configuration files
3. **Builder Pattern**: `ClaudeAgentOptions` built conditionally based on optional fields
4. **Async-First**: `execute()` is async to support non-blocking operations

### Consistency Notes

- `SubagentError` manually implements `Display` and `Error` traits
- Other modules use `thiserror` crate for error types
- Consider migrating to `thiserror` for consistency (suggestion)
