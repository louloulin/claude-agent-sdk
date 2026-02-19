# Section 8: Types Reference

This section provides comprehensive documentation of all types in the Claude Agent SDK for Rust.

## 8.1 Overview

The SDK's type system is organized into several modules:

```
types/
├── mod.rs        # Re-exports 6 submodules
├── messages.rs   # Message types (774 lines, 26 tests)
├── config.rs     # Configuration types with TypedBuilder
├── mcp.rs        # MCP server types and traits
├── hooks.rs      # Hook system (1030 lines, 35 tests)
├── permissions.rs # Permission types (263 lines, 7 tests)
└── plugin.rs     # Plugin config (157 lines, 7 tests)

v2/
└── types.rs      # V2 simplified types (454 lines, 9 tests)

errors.rs         # Error types with thiserror
```

## 8.2 Message Types

### 8.2.1 Message Enum

The main message type containing all variants from CLI output:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Message {
    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),
    #[serde(rename = "system")]
    System(SystemMessage),
    #[serde(rename = "result")]
    Result(ResultMessage),
    #[serde(rename = "stream_event")]
    StreamEvent(StreamEvent),
    #[serde(rename = "user")]
    User(UserMessage),
    #[serde(rename = "control_cancel_request")]
    ControlCancelRequest(serde_json::Value),
}
```

### 8.2.2 ContentBlock Types

Content blocks represent different types of content within messages:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text(TextBlock),           // {"type": "text", "text": "..."}
    Thinking(ThinkingBlock),   // Extended thinking
    ToolUse(ToolUseBlock),     // Tool invocation
    ToolResult(ToolResultBlock), // Tool output
    Image(ImageBlock),         // Image content
}
```

**TextBlock**:
```rust
pub struct TextBlock {
    pub text: String,
}
```

**ThinkingBlock** (Extended Thinking):
```rust
pub struct ThinkingBlock {
    pub thinking: String,   // Thinking content
    pub signature: String,  // Signature for verification
}
```

**ToolUseBlock**:
```rust
pub struct ToolUseBlock {
    pub id: String,                    // Unique tool use ID
    pub name: String,                  // Tool name (e.g., "Bash")
    pub input: serde_json::Value,      // Tool input parameters
}
```

**ToolResultBlock**:
```rust
pub struct ToolResultBlock {
    pub tool_use_id: String,           // Corresponds to ToolUseBlock.id
    pub content: Option<ToolResultContent>,
    pub is_error: Option<bool>,
}
```

**ImageBlock**:
```rust
pub struct ImageBlock {
    pub source: ImageSource,  // Base64 or URL
}
```

### 8.2.3 Image Source Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    Base64 {
        media_type: String,  // MIME type: image/jpeg, image/png, image/gif, image/webp
        data: String,        // Base64-encoded data (without data URI prefix)
    },
    Url {
        url: String,         // Publicly accessible URL
    },
}
```

**Supported MIME Types**: `image/jpeg`, `image/png`, `image/gif`, `image/webp`

**Size Limit**: 15MB base64 data (results in ~20MB decoded)

### 8.2.4 UserContentBlock

Content blocks for user input:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserContentBlock {
    Text { text: String },
    Image { source: ImageSource },
}
```

**Helper Methods**:
```rust
// Create text block
let text = UserContentBlock::text("Hello world");

// Create image from base64 (with validation)
let image = UserContentBlock::image_base64("image/png", "iVBORw0KGgo=")?;

// Create image from URL
let image = UserContentBlock::image_url("https://example.com/image.png");

// Validate content blocks
UserContentBlock::validate_content(&blocks)?;  // Ensures non-empty
```

### 8.2.5 AssistantMessage

```rust
pub struct AssistantMessage {
    pub message: AssistantMessageInner,
    pub parent_tool_use_id: Option<String>,
    pub session_id: Option<String>,
    pub uuid: Option<String>,
}

pub struct AssistantMessageInner {
    pub content: Vec<ContentBlock>,
    pub model: Option<String>,
    pub id: Option<String>,
    pub stop_reason: Option<String>,
    pub usage: Option<serde_json::Value>,
    pub error: Option<AssistantMessageError>,
}
```

### 8.2.6 SystemMessage

```rust
pub struct SystemMessage {
    pub subtype: String,                    // e.g., "session_start"
    pub cwd: Option<String>,                // Current working directory
    pub session_id: Option<String>,
    pub tools: Option<Vec<String>>,         // Available tools
    pub mcp_servers: Option<Vec<serde_json::Value>>,
    pub model: Option<String>,
    pub permission_mode: Option<String>,
    pub uuid: Option<String>,
    #[serde(flatten)]
    pub data: serde_json::Value,           // Additional data
}
```

### 8.2.7 ResultMessage

```rust
pub struct ResultMessage {
    pub subtype: String,                    // e.g., "query_complete"
    pub duration_ms: u64,                   // Total duration
    pub duration_api_ms: u64,               // API duration
    pub is_error: bool,
    pub num_turns: u32,
    pub session_id: String,
    pub total_cost_usd: Option<f64>,
    pub usage: Option<serde_json::Value>,
    pub result: Option<String>,
    pub structured_output: Option<serde_json::Value>,  // When output_format specified
}
```

### 8.2.8 AssistantMessageError

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssistantMessageError {
    AuthenticationFailed,
    BillingError,
    RateLimit,
    InvalidRequest,
    ServerError,
    Unknown,
}
```

## 8.3 Configuration Types

### 8.3.1 ClaudeAgentOptions

The main configuration struct with 45+ fields using TypedBuilder:

```rust
#[derive(Clone, TypedBuilder)]
#[builder(doc)]
pub struct ClaudeAgentOptions {
    // Tools and Permissions
    pub tools: Option<Tools>,
    pub allowed_tools: Vec<String>,
    pub disallowed_tools: Vec<String>,
    pub permission_mode: Option<PermissionMode>,

    // Model Selection
    pub model: Option<String>,
    pub fallback_model: Option<String>,

    // Prompts and Sessions
    pub system_prompt: Option<SystemPrompt>,
    pub continue_conversation: bool,
    pub resume: Option<String>,

    // Execution Limits
    pub max_turns: Option<u32>,
    pub max_budget_usd: Option<f64>,
    pub max_thinking_tokens: Option<u32>,

    // MCP and Hooks
    pub mcp_servers: McpServers,
    pub hooks: Option<HashMap<HookEvent, Vec<HookMatcher>>>,

    // Environment
    pub cwd: Option<PathBuf>,
    pub env: HashMap<String, String>,
    pub add_dirs: Vec<PathBuf>,

    // CLI Configuration
    pub cli_path: Option<PathBuf>,
    pub auto_install_cli: bool,
    pub cli_install_callback: Option<Arc<dyn Fn(InstallProgress) + Send + Sync>>,

    // Callbacks
    pub can_use_tool: Option<CanUseToolCallback>,
    pub stderr_callback: Option<Arc<dyn Fn(String) + Send + Sync>>,

    // Sandbox
    pub sandbox: Option<SandboxSettings>,

    // Skills
    pub auto_discover_skills: bool,
    pub project_skills_dir: Option<PathBuf>,
    pub user_skills_dir: Option<PathBuf>,

    // Output
    pub output_format: Option<serde_json::Value>,
    pub include_partial_messages: bool,

    // Checkpointing
    pub enable_file_checkpointing: bool,

    // Plugins
    pub plugins: Vec<SdkPluginConfig>,

    // Advanced
    pub betas: Vec<SdkBeta>,
    pub setting_sources: Option<Vec<SettingSource>>,
    pub agents: Option<HashMap<String, AgentDefinition>>,
    pub max_buffer_size: Option<usize>,
    pub extra_args: HashMap<String, Option<String>>,
    pub user: Option<String>,
    pub fork_session: bool,
    pub permission_prompt_tool_name: Option<String>,
    pub settings: Option<String>,
}
```

**Example Usage**:
```rust
let options = ClaudeAgentOptions::builder()
    .model("claude-sonnet-4-20250514".to_string())
    .permission_mode(PermissionMode::BypassPermissions)
    .max_turns(10)
    .build();
```

### 8.3.2 SystemPrompt

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    Text(String),
    Preset(SystemPromptPreset),
}

pub struct SystemPromptPreset {
    #[serde(rename = "type")]
    pub type_: String,           // Always "preset"
    pub preset: String,          // e.g., "claude_code"
    pub append: Option<String>,  // Text to append
}
```

**Helper Methods**:
```rust
// From text
let prompt = SystemPrompt::from("You are a helpful assistant.");

// Preset
let preset = SystemPromptPreset::new("claude_code");
let preset = SystemPromptPreset::with_append("claude_code", "Additional instructions");
```

### 8.3.3 PermissionMode

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    #[serde(rename = "default")]
    Default,
    AcceptEdits,
    #[serde(rename = "plan")]
    Plan,
    BypassPermissions,
}
```

### 8.3.4 Tools Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tools {
    List(Vec<String>),
    Preset(ToolsPreset),
}

pub struct ToolsPreset {
    #[serde(rename = "type")]
    pub type_: String,
    pub preset: String,
}

// Helper
let preset = ToolsPreset::claude_code();
```

### 8.3.5 AgentDefinition

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct AgentDefinition {
    #[builder(setter(into))]
    pub description: String,
    #[builder(setter(into))]
    pub prompt: String,
    pub tools: Option<Vec<String>>,
    pub model: Option<AgentModel>,
}

pub enum AgentModel {
    Sonnet,
    Opus,
    Haiku,
    Inherit,
}
```

### 8.3.6 SandboxSettings

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
pub struct SandboxSettings {
    pub enabled: Option<bool>,                          // Default: false
    pub auto_allow_bash_if_sandboxed: Option<bool>,     // Default: true
    pub excluded_commands: Option<Vec<String>>,         // Commands outside sandbox
    pub allow_unsandboxed_commands: Option<bool>,       // Default: true
    pub network: Option<SandboxNetworkConfig>,
    pub ignore_violations: Option<SandboxIgnoreViolations>,
    pub enable_weaker_nested_sandbox: Option<bool>,    // Docker, Linux only
}

pub struct SandboxNetworkConfig {
    pub allow_unix_sockets: Option<Vec<String>>,
    pub allow_all_unix_sockets: Option<bool>,
    pub allow_local_binding: Option<bool>,              // macOS only
    pub http_proxy_port: Option<u16>,
    pub socks_proxy_port: Option<u16>,
}

pub struct SandboxIgnoreViolations {
    pub file: Option<Vec<String>>,
    pub network: Option<Vec<String>>,
}
```

### 8.3.7 SettingSource

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SettingSource {
    User,     // ~/.claude/settings.json
    Project,  // .claude/settings.json (team-shared)
    Local,    // .claude/settings.local.json (highest priority, git-ignored)
}
```

### 8.3.8 SdkBeta

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SdkBeta {
    #[serde(rename = "context-1m-2025-08-07")]
    Context1M,  // Extended context window (1M tokens)
}
```

## 8.4 MCP Types

### 8.4.1 McpServers

```rust
#[derive(Clone, Default)]
pub enum McpServers {
    #[default]
    Empty,
    Dict(HashMap<String, McpServerConfig>),
    Path(PathBuf),
}
```

### 8.4.2 McpServerConfig

```rust
pub enum McpServerConfig {
    Stdio(McpStdioServerConfig),
    Sse(McpSseServerConfig),
    Http(McpHttpServerConfig),
    Sdk(McpSdkServerConfig),
}
```

### 8.4.3 Server Configurations

**Stdio** (Process-based):
```rust
pub struct McpStdioServerConfig {
    pub command: String,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
}
```

**SSE** (Server-Sent Events):
```rust
pub struct McpSseServerConfig {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
}
```

**HTTP**:
```rust
pub struct McpHttpServerConfig {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
}
```

**SDK** (In-process):
```rust
pub struct McpSdkServerConfig {
    pub name: String,
    pub instance: Arc<dyn SdkMcpServer>,
}
```

### 8.4.4 SdkMcpServer Trait

```rust
#[async_trait]
pub trait SdkMcpServer: Send + Sync {
    async fn handle_message(&self, message: serde_json::Value) -> Result<serde_json::Value>;
}
```

### 8.4.5 ToolHandler Trait

```rust
pub trait ToolHandler: Send + Sync {
    fn handle(&self, args: serde_json::Value) -> BoxFuture<'static, Result<ToolResult>>;
}
```

### 8.4.6 ToolResult

```rust
pub struct ToolResult {
    pub content: Vec<ToolResultContent>,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolResultContent {
    Text { text: String },
    Image { data: String, mime_type: String },
}
```

### 8.4.7 SdkMcpTool

```rust
pub struct SdkMcpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub handler: Arc<dyn ToolHandler>,
}
```

### 8.4.8 Helper Function and Macro

```rust
// Create in-process MCP server
pub fn create_sdk_mcp_server(
    name: impl Into<String>,
    version: impl Into<String>,
    tools: Vec<SdkMcpTool>,
) -> McpSdkServerConfig;

// Macro for creating tools
tool!("tool_name", "description", json_schema, |args| async {
    // Handler implementation
    Ok(ToolResult { content: vec![...], is_error: false })
});
```

## 8.5 Hook Types

### 8.5.1 HookEvent

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEvent {
    PreToolUse,        // Before tool execution
    PostToolUse,       // After tool execution
    UserPromptSubmit,  // When user submits prompt
    Stop,              // When execution stops
    SubagentStop,      // When subagent stops
    PreCompact,        // Before conversation compaction
}
```

### 8.5.2 HookMatcher

```rust
#[derive(Clone, TypedBuilder)]
pub struct HookMatcher {
    pub matcher: Option<String>,     // Tool name pattern (e.g., "Bash")
    pub hooks: Vec<HookCallback>,    // Callbacks to invoke
    pub timeout: Option<f64>,        // Timeout in seconds (default: 60)
}
```

### 8.5.3 HookInput Variants

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "hook_event_name", rename_all = "PascalCase")]
pub enum HookInput {
    PreToolUse(PreToolUseHookInput),
    PostToolUse(PostToolUseHookInput),
    UserPromptSubmit(UserPromptSubmitHookInput),
    Stop(StopHookInput),
    SubagentStop(SubagentStopHookInput),
    PreCompact(PreCompactHookInput),
}
```

**PreToolUseHookInput**:
```rust
pub struct PreToolUseHookInput {
    pub session_id: String,
    pub transcript_path: String,
    pub cwd: String,
    pub permission_mode: Option<String>,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
}
```

**PostToolUseHookInput**:
```rust
pub struct PostToolUseHookInput {
    pub session_id: String,
    pub transcript_path: String,
    pub cwd: String,
    pub permission_mode: Option<String>,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_response: serde_json::Value,
}
```

### 8.5.4 Hook Output Types

**HookJsonOutput** (Async or Sync):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HookJsonOutput {
    Async(AsyncHookJsonOutput),
    Sync(SyncHookJsonOutput),
}
```

**AsyncHookJsonOutput**:
```rust
pub struct AsyncHookJsonOutput {
    #[serde(rename = "async")]
    pub async_: bool,              // Always true for async
    pub async_timeout: Option<u64>, // Timeout in milliseconds
}
```

**SyncHookJsonOutput**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct SyncHookJsonOutput {
    #[serde(rename = "continue")]
    pub continue_: Option<bool>,
    #[serde(rename = "suppressOutput")]
    pub suppress_output: Option<bool>,
    #[serde(rename = "stopReason")]
    pub stop_reason: Option<String>,
    pub decision: Option<String>,
    #[serde(rename = "systemMessage")]
    pub system_message: Option<String>,
    pub reason: Option<String>,
    #[serde(rename = "hookSpecificOutput")]
    pub hook_specific_output: Option<HookSpecificOutput>,
}
```

### 8.5.5 HookSpecificOutput

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "hookEventName")]
pub enum HookSpecificOutput {
    PreToolUse(PreToolUseHookSpecificOutput),
    PostToolUse(PostToolUseHookSpecificOutput),
    UserPromptSubmit(UserPromptSubmitHookSpecificOutput),
}

pub struct PreToolUseHookSpecificOutput {
    #[serde(rename = "permissionDecision")]
    pub permission_decision: Option<String>,
    #[serde(rename = "permissionDecisionReason")]
    pub permission_decision_reason: Option<String>,
    #[serde(rename = "updatedInput")]
    pub updated_input: Option<serde_json::Value>,
}

pub struct PostToolUseHookSpecificOutput {
    #[serde(rename = "additionalContext")]
    pub additional_context: Option<String>,
}
```

### 8.5.6 Hooks Builder

```rust
let mut hooks = Hooks::new();

// Add hooks for all tools
hooks.add_pre_tool_use(my_hook);
hooks.add_post_tool_use(my_hook);

// Add hooks for specific tools
hooks.add_pre_tool_use_with_matcher("Bash", my_hook);
hooks.add_post_tool_use_with_matcher("Write", my_hook);

// Add other event hooks
hooks.add_user_prompt_submit(my_hook);
hooks.add_stop(my_hook);
hooks.add_subagent_stop(my_hook);
hooks.add_pre_compact(my_hook);

// Build for use with ClaudeAgentOptions
let hooks_map = hooks.build();
```

## 8.6 Permission Types

### 8.6.1 PermissionResult

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "behavior", rename_all = "lowercase")]
pub enum PermissionResult {
    Allow(PermissionResultAllow),
    Deny(PermissionResultDeny),
}
```

### 8.6.2 PermissionResultAllow

```rust
pub struct PermissionResultAllow {
    #[serde(rename = "updatedInput")]
    pub updated_input: Option<serde_json::Value>,
    #[serde(rename = "updatedPermissions")]
    pub updated_permissions: Option<Vec<PermissionUpdate>>,
}
```

### 8.6.3 PermissionResultDeny

```rust
pub struct PermissionResultDeny {
    pub message: String,
    pub interrupt: bool,
}
```

### 8.6.4 PermissionUpdate

```rust
pub struct PermissionUpdate {
    #[serde(rename = "type")]
    pub type_: PermissionUpdateType,
    pub rules: Option<Vec<PermissionRuleValue>>,
    pub behavior: Option<PermissionBehavior>,
    pub mode: Option<PermissionMode>,
    pub directories: Option<Vec<String>>,
    pub destination: Option<PermissionUpdateDestination>,
}
```

### 8.6.5 Permission Enums

**PermissionUpdateType**:
```rust
pub enum PermissionUpdateType {
    AddRules,
    ReplaceRules,
    RemoveRules,
    SetMode,
    AddDirectories,
    RemoveDirectories,
}
```

**PermissionBehavior**:
```rust
pub enum PermissionBehavior {
    Allow,
    Deny,
    Ask,
}
```

**PermissionUpdateDestination**:
```rust
pub enum PermissionUpdateDestination {
    UserSettings,
    ProjectSettings,
    LocalSettings,
    Session,
}
```

### 8.6.6 PermissionRuleValue

```rust
pub struct PermissionRuleValue {
    #[serde(rename = "toolName")]
    pub tool_name: String,
    #[serde(rename = "ruleContent")]
    pub rule_content: Option<String>,
}
```

### 8.6.7 Callback Types

```rust
pub type CanUseToolCallback = Arc<
    dyn Fn(String, serde_json::Value, ToolPermissionContext) -> BoxFuture<'static, PermissionResult>
        + Send
        + Sync,
>;

pub struct ToolPermissionContext {
    pub signal: Option<()>,
    pub suggestions: Vec<PermissionUpdate>,
}
```

## 8.7 Plugin Types

### 8.7.1 SdkPluginConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum SdkPluginConfig {
    Local {
        path: PathBuf,
    },
}
```

**Helper Methods**:
```rust
// Create local plugin config
let plugin = SdkPluginConfig::local("./my-plugin");
let plugin = SdkPluginConfig::local("~/.claude/plugins/my-plugin");

// Get path
if let Some(path) = plugin.path() {
    println!("Plugin path: {:?}", path);
}
```

## 8.8 V2 API Types

### 8.8.1 SessionOptions

Simplified configuration for V2 API:

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

**Example**:
```rust
let options = SessionOptions::builder()
    .model("claude-sonnet-4-20250514".to_string())
    .max_turns(10)
    .permission_mode(PermissionMode::BypassPermissions)
    .build();
```

### 8.8.2 PromptResult

```rust
pub struct PromptResult {
    pub content: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model: Option<String>,
}
```

**Methods**:
```rust
let total = result.total_tokens();
let cost = result.estimated_cost_usd();  // Approximate pricing
```

### 8.8.3 V2 Message

```rust
pub enum Message {
    User { content: String },
    Assistant { content: String },
    ToolResult { tool_name: String, result: String },
}
```

**Methods**:
```rust
let text = msg.as_text();      // Option<&str>
let is_user = msg.is_user();   // bool
let is_assistant = msg.is_assistant();  // bool
let is_tool = msg.is_tool_result();     // bool
```

## 8.9 Error Types

### 8.9.1 ClaudeError

Main error enum with 14 variants:

```rust
#[derive(Debug, Error)]
pub enum ClaudeError {
    #[error("CLI connection error: {0}")]
    Connection(#[from] ConnectionError),

    #[error("Process error: {0}")]
    Process(#[from] ProcessError),

    #[error("JSON decode error: {0}")]
    JsonDecode(#[from] JsonDecodeError),

    #[error("Message parse error: {0}")]
    MessageParse(#[from] MessageParseError),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Control protocol error: {0}")]
    ControlProtocol(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("CLI not found: {0}")]
    CliNotFound(#[from] CliNotFoundError),

    #[error("Image validation error: {0}")]
    ImageValidation(#[from] ImageValidationError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}
```

### 8.9.2 Specialized Errors

**CliNotFoundError**:
```rust
pub struct CliNotFoundError {
    pub message: String,
    pub cli_path: Option<PathBuf>,
}
```

**ConnectionError**:
```rust
pub struct ConnectionError {
    pub message: String,
}
```

**ProcessError**:
```rust
pub struct ProcessError {
    pub message: String,
    pub exit_code: Option<i32>,
    pub stderr: Option<String>,
}
```

**JsonDecodeError**:
```rust
pub struct JsonDecodeError {
    pub message: String,
    pub line: String,  // The line that failed
}
```

**MessageParseError**:
```rust
pub struct MessageParseError {
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

**ImageValidationError**:
```rust
pub struct ImageValidationError {
    pub message: String,
}
```

### 8.9.3 Result Type Alias

```rust
pub type Result<T> = std::result::Result<T, ClaudeError>;
```

## 8.10 Type Conversion Patterns

### 8.10.1 From Traits

**String to UserContentBlock**:
```rust
let block: UserContentBlock = "Hello".into();
let block: UserContentBlock = String::from("Hello").into();
```

**SessionOptions to ClaudeAgentOptions**:
```rust
let options = SessionOptions::builder().build();
let full_options: ClaudeAgentOptions = options.into();
```

**V2 Message from Full Message**:
```rust
let full_msg: types::messages::Message = /* ... */;
let v2_msg: v2::Message = full_msg.into();
```

### 8.10.2 Builder Patterns

Most configuration types use TypedBuilder:
```rust
ClaudeAgentOptions::builder()
    .field1(value1)
    .field2(value2)
    .build()

HookMatcher::builder()
    .matcher(Some("Bash".to_string()))
    .hooks(vec![callback])
    .build()

SyncHookJsonOutput::builder()
    .continue_(Some(true))
    .build()
```

## 8.11 Test Coverage

| Module | Tests | Focus Areas |
|--------|-------|-------------|
| messages.rs | 26 | Serialization, validation |
| hooks.rs | 35 | Events, builder, execution |
| permissions.rs | 7 | Serialization |
| plugin.rs | 7 | Roundtrip, path |
| v2/types.rs | 9 | Builder, conversion |
| **Total** | **84** | |

## 8.12 Design Patterns

### 8.12.1 TypedBuilder

Provides compile-time safety for optional fields:
```rust
#[derive(TypedBuilder)]
pub struct MyConfig {
    #[builder(default, setter(strip_option))]
    pub optional: Option<String>,
}
```

### 8.12.2 Serde Attributes

Common patterns:
```rust
#[serde(skip_serializing_if = "Option::is_none")]  // Omit None values
#[serde(rename = "camelCase")]                      // Rename fields
#[serde(tag = "type", rename_all = "snake_case")]   // Tagged enums
#[serde(flatten)]                                   // Flatten fields
```

### 8.12.3 thiserror

Consistent error handling:
```rust
#[derive(Debug, Error)]
#[error("Error message: {field}")]
pub struct MyError {
    pub field: String,
}
```

## 8.13 API Reference

### Re-exports from types/mod.rs

```rust
pub use config::*;
pub use hooks::*;
pub use mcp::*;
pub use messages::*;
pub use permissions::*;
pub use plugin::*;
```

### Common Imports

```rust
use claude_agent_sdk::types::{
    // Messages
    Message, ContentBlock, TextBlock, ToolUseBlock,
    UserContentBlock, ImageSource,

    // Config
    ClaudeAgentOptions, SystemPrompt, PermissionMode,

    // MCP
    McpServers, McpServerConfig, ToolResult,

    // Hooks
    HookEvent, HookInput, HookJsonOutput, Hooks,

    // Permissions
    PermissionResult, PermissionBehavior,

    // Plugins
    SdkPluginConfig,
};

use claude_agent_sdk::errors::{ClaudeError, Result};

use claude_agent_sdk::v2::{
    SessionOptions, PromptResult, Message as V2Message,
};
```
