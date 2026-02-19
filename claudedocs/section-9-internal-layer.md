# Section 9: Internal Layer

This section documents the internal implementation layer of the Claude Agent SDK for Rust, which handles low-level communication with the Claude Code CLI.

## 9.1 Overview

The internal layer provides the core infrastructure for SDK-CLI communication:

```
internal/
├── mod.rs            # Module exports
├── client.rs         # Simple client wrapper
├── message_parser.rs # JSON-to-Message parsing
├── query_full.rs     # Full bidirectional control protocol
├── cli_installer.rs  # Auto-installation system
└── transport/
    ├── mod.rs        # Transport module exports
    ├── trait_def.rs  # Transport trait definition
    └── subprocess.rs # Subprocess transport implementation
```

**Key Components**:
- **Transport Layer**: Abstract interface and subprocess implementation for CLI communication
- **Message Parser**: Converts JSON output to typed Message structures
- **QueryFull**: Bidirectional control protocol with hooks and MCP support
- **CLI Installer**: Automatic CLI download and installation

## 9.2 Transport Trait

### 9.2.1 Transport Definition

The `Transport` trait defines the interface for CLI communication:

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect the transport
    async fn connect(&mut self) -> Result<()>;

    /// Write raw data to the transport
    async fn write(&mut self, data: &str) -> Result<()>;

    /// Read messages as a stream of JSON values
    fn read_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>>;

    /// Close the transport
    async fn close(&mut self) -> Result<()>;

    /// Check if the transport is ready
    fn is_ready(&self) -> bool;

    /// End input stream (close stdin)
    async fn end_input(&mut self) -> Result<()>;
}
```

### 9.2.2 Transport Lifecycle

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   new()     │────▶│  connect()  │────▶│   ready     │
└─────────────┘     └─────────────┘     └─────────────┘
                                               │
                                               ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Drop      │◀────│   close()   │◀────│read_messages│
└─────────────┘     └─────────────┘     └─────────────┘
```

## 9.3 SubprocessTransport

### 9.3.1 Structure

```rust
pub struct SubprocessTransport {
    cli_path: PathBuf,
    cwd: Option<PathBuf>,
    options: ClaudeAgentOptions,
    prompt: QueryPrompt,
    process: Option<Child>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stdout: Arc<Mutex<Option<BufReader<ChildStdout>>>>,
    max_buffer_size: usize,
    ready: bool,
}
```

### 9.3.2 QueryPrompt Types

```rust
pub enum QueryPrompt {
    /// Text prompt (one-shot mode)
    Text(String),
    /// Structured content blocks (supports images and text)
    Content(Vec<UserContentBlock>),
    /// Streaming mode (no initial prompt)
    Streaming,
}
```

**Conversion Traits**:
```rust
// From string
let prompt: QueryPrompt = "Hello".into();
let prompt: QueryPrompt = String::from("Hello").into();

// From content blocks
let blocks = vec![UserContentBlock::text("Hello")];
let prompt: QueryPrompt = blocks.into();
```

### 9.3.3 CLI Discovery Strategy

The transport uses a 5-tier CLI discovery strategy:

| Priority | Strategy | Notes |
|----------|----------|-------|
| 1 | Execute `claude --version` | Most reliable, uses shell PATH |
| 2 | `which claude` (Unix) | Explicit PATH lookup |
| 3 | `where claude` (Windows) | Windows equivalent |
| 4 | Common paths | `/usr/local/bin`, `~/.local/bin`, etc. |
| 5 | `CLAUDE_CLI_PATH` env | Manual override |

**Common Installation Paths**:
```rust
// Unix-like systems
/usr/local/bin/claude
/opt/homebrew/bin/claude
/usr/bin/claude
~/.local/bin/claude
~/bin/claude

// Windows
%USERPROFILE%\AppData\Local\Programs\Claude\claude.exe
%USERPROFILE%\AppData\Roaming\npm\claude.cmd
C:\Program Files\Claude\claude.exe
```

### 9.3.4 Command Building

The transport builds CLI arguments from `ClaudeAgentOptions`:

```rust
fn build_command(&self) -> Vec<String>
```

**Generated Arguments**:

| Option | CLI Argument |
|--------|--------------|
| `output_format` | `--output-format stream-json --verbose` |
| `system_prompt` | `--system-prompt "text"` or `--append-system-prompt` |
| `tools` | `--tools tool1,tool2` or `--tools default` |
| `permission_mode` | `--permission-mode bypassPermissions` |
| `allowed_tools` | `--allowedTools Bash,Read` |
| `model` | `--model claude-sonnet-4-20250514` |
| `max_turns` | `--max-turns 10` |
| `resume` | `--resume session-id` |
| `continue_conversation` | `--continue` |
| `max_budget_usd` | `--max-budget-usd 1.0` |
| `betas` | `--betas context-1m-2025-08-07` |

**Environment Variables**:
```rust
CLAUDE_CODE_ENTRYPOINT=rust-sdk
CLAUDE_AGENT_SDK_VERSION=0.1.0
CLAUDE_CODE_ENABLE_SDK_FILE_CHECKPOINTING=true  // If enabled
```

### 9.3.5 Version Checking

```rust
async fn check_claude_version(&self) -> Result<()>
```

- Compares CLI version against `MIN_CLI_VERSION`
- Skips check if `CLAUDE_SKIP_VERSION_CHECK=1` is set
- Logs warning if version is below minimum

### 9.3.6 Message Streaming

```rust
fn read_messages(
    &mut self,
) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>>
```

**Implementation**:
1. Uses `async_stream::stream!` for lazy evaluation
2. Reads line-by-line from stdout
3. Tracks buffer size against `max_buffer_size` (default: 10MB)
4. Parses each line as JSON
5. Yields `Result<serde_json::Value>`

## 9.4 CLI Auto-Installer

### 9.4.1 InstallProgress Events

```rust
pub enum InstallProgress {
    /// Checking installation status
    Checking(String),
    /// Download progress
    Downloading {
        current: u64,
        total: Option<u64>,
    },
    /// Installation in progress
    Installing(String),
    /// Installation complete
    Done(PathBuf),
    /// Installation failed
    Failed(String),
}
```

### 9.4.2 CliInstaller

```rust
pub struct CliInstaller {
    pub auto_install: bool,
    progress_callback: Option<Arc<dyn Fn(InstallProgress) + Send + Sync>>,
}
```

**Methods**:
```rust
// Create installer
let installer = CliInstaller::new(true);

// With progress callback
let installer = CliInstaller::new(true)
    .with_progress_callback(Arc::new(|event| {
        match event {
            InstallProgress::Downloading { current, total } => {
                println!("Downloaded: {} bytes", current);
            }
            // ...
        }
    }));

// Install if needed
let cli_path = installer.install_if_needed().await?;
```

### 9.4.3 Installation Strategy

```
┌───────────────────────┐
│  Check existing CLI   │
└──────────┬────────────┘
           │ Not found
           ▼
┌───────────────────────┐
│   Try npm install     │
│  (npm install -g      │
│   @anthropic-ai/      │
│   claude-code)        │
└──────────┬────────────┘
           │ Failed
           ▼
┌───────────────────────┐
│  Try direct download  │
│  (GitHub Releases)    │
└──────────┬────────────┘
           │ Failed
           ▼
┌───────────────────────┐
│     Return error      │
└───────────────────────┘
```

### 9.4.4 Platform Detection

```rust
fn detect_platform() -> (&'static str, &'static str)

// Returns (platform, arch):
// - ("darwin", "x64")    - macOS Intel
// - ("darwin", "arm64")  - macOS Apple Silicon
// - ("linux", "x64")     - Linux x86_64
// - ("linux", "arm64")   - Linux ARM64
// - ("windows", "x64")   - Windows x86_64
```

### 9.4.5 Installation Directories

| Platform | Installation Path |
|----------|------------------|
| Unix | `~/.local/bin/claude` |
| Windows | `%USERPROFILE%\AppData\Local\Programs\Claude\claude.exe` |

## 9.5 MessageParser

### 9.5.1 Structure

```rust
pub struct MessageParser;
```

### 9.5.2 Parsing

```rust
impl MessageParser {
    /// Parse a JSON value into a Message
    pub fn parse(data: serde_json::Value) -> Result<Message> {
        serde_json::from_value(data.clone()).map_err(|e| {
            MessageParseError::new(
                format!("Failed to parse message: {}", e),
                Some(data),
            ).into()
        })
    }
}
```

**Usage**:
```rust
let json: serde_json::Value = /* ... */;
let message = MessageParser::parse(json)?;
```

## 9.6 InternalClient

### 9.6.1 Structure

```rust
pub struct InternalClient {
    transport: SubprocessTransport,
}
```

### 9.6.2 Simple Execution Flow

```rust
impl InternalClient {
    /// Create a new client
    pub fn new(prompt: QueryPrompt, options: ClaudeAgentOptions) -> Result<Self>

    /// Connect and get messages
    pub async fn execute(mut self) -> Result<Vec<Message>>
}
```

**Execution Flow**:
```
┌─────────────────┐
│   new(prompt,   │
│    options)     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    execute()    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────────┐
│ transport.      │────▶│ read_messages() │
│   connect()     │     │    Stream       │
└─────────────────┘     └────────┬────────┘
                                 │
                                 ▼
                        ┌─────────────────┐
                        │ MessageParser:: │
                        │    parse()      │
                        └────────┬────────┘
                                 │
                                 ▼
                        ┌─────────────────┐
                        │ Vec<Message>    │
                        └─────────────────┘
```

## 9.7 QueryFull

### 9.7.1 Structure

```rust
pub struct QueryFull {
    pub(crate) transport: Arc<Mutex<Box<dyn Transport>>>,
    hook_callbacks: Arc<Mutex<HashMap<String, HookCallback>>>,
    sdk_mcp_servers: Arc<Mutex<HashMap<String, McpSdkServerConfig>>>,
    next_callback_id: Arc<AtomicU64>,
    request_counter: Arc<AtomicU64>,
    pending_responses: Arc<Mutex<HashMap<String, oneshot::Sender<serde_json::Value>>>>,
    message_tx: mpsc::UnboundedSender<serde_json::Value>,
    pub(crate) message_rx: Arc<Mutex<mpsc::UnboundedReceiver<serde_json::Value>>>,
    stdin: Option<Arc<Mutex<Option<tokio::process::ChildStdin>>>>,
    initialization_result: Arc<Mutex<Option<serde_json::Value>>>,
}
```

### 9.7.2 Control Protocol

**Request Format** (SDK → CLI):
```json
{
    "type": "control_request",
    "request_id": "req_123_abc",
    "request": {
        "subtype": "initialize",
        "hooks": { ... }
    }
}
```

**Response Format** (CLI → SDK):
```json
{
    "type": "control_response",
    "response": {
        "subtype": "success",
        "request_id": "req_123_abc",
        "response": { ... }
    }
}
```

### 9.7.3 Initialization

```rust
pub async fn initialize(
    &self,
    hooks: Option<HashMap<String, Vec<HookMatcher>>>,
) -> Result<serde_json::Value>
```

**Hook Registration**:
1. Assigns unique callback IDs (`hook_0`, `hook_1`, ...)
2. Stores callbacks in `hook_callbacks` map
3. Sends matcher configuration to CLI
4. Returns initialization result

### 9.7.4 Background Message Handling

```rust
pub async fn start(&self) -> Result<()>
```

**Spawns background task**:
```
┌─────────────────────────────────────────────────┐
│              Background Task                      │
├─────────────────────────────────────────────────┤
│  loop {                                          │
│    match message.type {                          │
│      "control_response" => Handle response       │
│      "control_request"  => Handle incoming req   │
│      _                 => Forward to message_tx  │
│    }                                             │
│  }                                               │
└─────────────────────────────────────────────────┘
```

### 9.7.5 Control Request Handlers

**Hook Callback** (`subtype: "hook_callback"`):
```rust
// 1. Look up callback by ID
// 2. Parse HookInput from request
// 3. Execute callback
// 4. Return HookJsonOutput
```

**MCP Message** (`subtype: "mcp_message"`):
```rust
// 1. Look up SDK MCP server by name
// 2. Call server.handle_message()
// 3. Return response
```

### 9.7.6 Control Methods

**Interrupt**:
```rust
pub async fn interrupt(&self) -> Result<()>
```

**Change Permission Mode**:
```rust
pub async fn set_permission_mode(
    &self,
    mode: PermissionMode,
) -> Result<()>
```

**Change Model**:
```rust
pub async fn set_model(&self, model: Option<&str>) -> Result<()>
```

**Rewind Files**:
```rust
/// Rewind tracked files to their state at a specific user message.
///
/// Requires:
/// - `enable_file_checkpointing=true`
/// - `extra_args={"replay-user-messages": None}`
pub async fn rewind_files(&self, user_message_id: &str) -> Result<()>
```

**Get Server Info**:
```rust
pub async fn get_initialization_result(&self) -> Option<serde_json::Value>
```

## 9.8 Message Flow

### 9.8.1 One-Shot Query Flow

```
User Code
    │
    ▼
InternalClient::execute()
    │
    ▼
SubprocessTransport::connect()
    │
    ├──────────────────────────┐
    │                          │
    ▼                          ▼
write(prompt)            read_messages()
    │                          │
    ▼                          ▼
end_input()              MessageParser::parse()
    │                          │
    └──────────────────────────┘
               │
               ▼
         Vec<Message>
```

### 9.8.2 Bidirectional Control Flow

```
User Code
    │
    ▼
QueryFull::initialize(hooks)
    │
    ▼
QueryFull::start()  ──────────────────┐
    │                                 │
    │                          ┌──────▼──────┐
    │                          │ Background  │
    │                          │ Task Loop   │
    │                          └──────┬──────┘
    │                                 │
    ▼                                 │
message_rx.recv() ◀───────────────────┘
    │
    ▼
Process messages

Control operations:
    interrupt() ──────▶ send_control_request()
    set_model() ──────▶ send_control_request()
    rewind_files() ───▶ send_control_request()
```

## 9.9 Error Handling

### 9.9.1 Error Types

| Error Type | Source | Example |
|------------|--------|---------|
| `CliNotFound` | CLI not in PATH | "Claude Code CLI not found" |
| `ConnectionError` | Failed to connect | "Failed to get stdin" |
| `ProcessError` | CLI exit failure | "Non-zero exit status" |
| `JsonDecodeError` | Parse failure | "Failed to parse JSON" |
| `MessageParseError` | Message type error | "Failed to parse message" |
| `Transport` | I/O error | "Failed to write to stdin" |
| `ControlProtocol` | Protocol error | "Missing subtype" |

### 9.9.2 Recovery Patterns

**CLI Not Found**:
```rust
// Option 1: Manual installation message
// Option 2: Enable auto-install
let options = ClaudeAgentOptions::builder()
    .auto_install_cli(true)
    .cli_install_callback(Arc::new(|event| {
        // Handle progress
    }))
    .build();
```

**Version Mismatch**:
```rust
// Skip version check (not recommended)
std::env::set_var("CLAUDE_SKIP_VERSION_CHECK", "1");
```

**Buffer Overflow**:
```rust
// Increase buffer size
let options = ClaudeAgentOptions::builder()
    .max_buffer_size(50 * 1024 * 1024) // 50MB
    .build();
```

## 9.10 Test Coverage

| Module | Tests | Focus |
|--------|-------|-------|
| cli_installer.rs | 2 | Platform detection, install directory |
| message_parser.rs | 0 | (Covered by types module tests) |
| client.rs | 0 | (Simple wrapper, tested via integration) |
| query_full.rs | 0 | (Complex async, tested via integration) |
| transport/subprocess.rs | 0 | (Process-based, tested via integration) |

**Note**: Internal layer is primarily tested through integration tests and example code.

## 9.11 API Reference

### Re-exports from internal/mod.rs

```rust
pub mod cli_installer;
pub mod client;
pub mod message_parser;
pub mod query_full;
pub mod transport;
```

### Re-exports from transport/mod.rs

```rust
pub use subprocess::SubprocessTransport;
pub use trait_def::Transport;
```

### Common Imports

```rust
use claude_agent_sdk::internal::{
    client::InternalClient,
    message_parser::MessageParser,
    query_full::QueryFull,
    cli_installer::{CliInstaller, InstallProgress},
    transport::{Transport, SubprocessTransport, QueryPrompt},
};
```

## 9.12 Design Patterns

### 9.12.1 Async Stream Pattern

```rust
fn read_messages(&mut self) -> Pin<Box<dyn Stream<Item = Result<Value>> + Send + '_>> {
    Box::pin(async_stream::stream! {
        // Yield values
        yield Ok(json);
    })
}
```

### 9.12.2 Background Task Pattern

```rust
tokio::spawn(async move {
    while let Some(result) = stream.next().await {
        // Process in background
    }
});
```

### 9.12.3 Direct Stdin Access Pattern

```rust
// Bypass transport lock for concurrent writes
if let Some(ref stdin) = self.stdin {
    let mut guard = stdin.lock().await;
    if let Some(ref mut stream) = *guard {
        stream.write_all(data.as_bytes()).await?;
    }
}
```

## 9.13 Performance Considerations

### 9.13.1 Process Spawn Overhead

- Each query spawns a new CLI process (~50-100ms)
- Consider connection pooling for high-frequency queries
- V2 Session API maintains persistent connection

### 9.13.2 Buffer Management

- Default 10MB buffer size
- Configure based on expected message size
- Overflow causes stream termination

### 9.13.3 Lock Contention

- `Arc<Mutex<>>` used for shared state
- Background task holds transport lock
- Direct stdin access bypasses lock

## 9.14 Security Considerations

| Concern | Mitigation |
|---------|------------|
| Path injection | CLI path validation before spawn |
| Command injection | No shell interpretation |
| Process isolation | Separate process, controlled environment |
| Stdin/stdout access | Mutex-protected streams |
| Auto-install | Downloads from official sources only |
