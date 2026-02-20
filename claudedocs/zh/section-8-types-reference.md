# 第 8 章：类型参考

本章提供 Claude Agent SDK for Rust 中所有类型的完整文档。

## 8.1 概述

SDK 的类型系统分为多个模块：

```
types/
├── mod.rs        # 重导出 6 个子模块
├── messages.rs   # 消息类型（774 行，26 个测试）
├── config.rs     # 使用 TypedBuilder 的配置类型
├── mcp.rs        # MCP 服务器类型和 trait
├── hooks.rs      # 钩子系统（1030 行，35 个测试）
├── permissions.rs # 权限类型（263 行，7 个测试）
└── plugin.rs     # 插件配置（157 行，7 个测试）

v2/
└── types.rs      # V2 简化类型（454 行，9 个测试）

errors.rs         # 使用 thiserror 的错误类型
```

## 8.2 消息类型

### 8.2.1 Message 枚举

包含 CLI 输出所有变体的主消息类型：

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

### 8.2.2 ContentBlock 类型

内容块表示消息中不同类型的内容：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text(TextBlock),           // {"type": "text", "text": "..."}
    Thinking(ThinkingBlock),   // 扩展思考
    ToolUse(ToolUseBlock),     // 工具调用
    ToolResult(ToolResultBlock), // 工具输出
    Image(ImageBlock),         // 图片内容
}
```

**TextBlock**：
```rust
pub struct TextBlock {
    pub text: String,
}
```

**ThinkingBlock**（扩展思考）：
```rust
pub struct ThinkingBlock {
    pub thinking: String,   // 思考内容
    pub signature: String,  // 用于验证的签名
}
```

**ToolUseBlock**：
```rust
pub struct ToolUseBlock {
    pub id: String,                    // 唯一工具使用 ID
    pub name: String,                  // 工具名称（如 "Bash"）
    pub input: serde_json::Value,      // 工具输入参数
}
```

**ToolResultBlock**：
```rust
pub struct ToolResultBlock {
    pub tool_use_id: String,           // 对应 ToolUseBlock.id
    pub content: Option<ToolResultContent>,
    pub is_error: Option<bool>,
}
```

**ImageBlock**：
```rust
pub struct ImageBlock {
    pub source: ImageSource,  // Base64 或 URL
}
```

### 8.2.3 ImageSource 类型

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    Base64 {
        media_type: String,  // MIME 类型：image/jpeg、image/png、image/gif、image/webp
        data: String,        // Base64 编码数据（不带 data URI 前缀）
    },
    Url {
        url: String,         // 公开可访问的 URL
    },
}
```

**支持的 MIME 类型**：`image/jpeg`、`image/png`、`image/gif`、`image/webp`

**大小限制**：15MB base64 数据（解码后约 20MB）

### 8.2.4 UserContentBlock

用于用户输入的内容块：

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserContentBlock {
    Text { text: String },
    Image { source: ImageSource },
}
```

**辅助方法**：
```rust
// 创建文本块
let text = UserContentBlock::text("Hello world");

// 从 base64 创建图片（带验证）
let image = UserContentBlock::image_base64("image/png", "iVBORw0KGgo=")?;

// 从 URL 创建图片
let image = UserContentBlock::image_url("https://example.com/image.png");

// 验证内容块
UserContentBlock::validate_content(&blocks)?;  // 确保非空
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
    pub subtype: String,                    // 如 "session_start"
    pub cwd: Option<String>,                // 当前工作目录
    pub session_id: Option<String>,
    pub tools: Option<Vec<String>>,         // 可用工具
    pub mcp_servers: Option<Vec<serde_json::Value>>,
    pub model: Option<String>,
    pub permission_mode: Option<String>,
    pub uuid: Option<String>,
    #[serde(flatten)]
    pub data: serde_json::Value,           // 附加数据
}
```

### 8.2.7 ResultMessage

```rust
pub struct ResultMessage {
    pub subtype: String,                    // 如 "query_complete"
    pub duration_ms: u64,                   // 总耗时
    pub duration_api_ms: u64,               // API 耗时
    pub is_error: bool,
    pub num_turns: u32,
    pub session_id: String,
    pub total_cost_usd: Option<f64>,
    pub usage: Option<serde_json::Value>,
    pub result: Option<String>,
    pub structured_output: Option<serde_json::Value>,  // 指定 output_format 时
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

## 8.3 配置类型

### 8.3.1 ClaudeAgentOptions

使用 TypedBuilder 的主配置结构体，包含 45+ 个字段：

```rust
#[derive(Clone, TypedBuilder)]
#[builder(doc)]
pub struct ClaudeAgentOptions {
    // 工具和权限
    pub tools: Option<Tools>,
    pub allowed_tools: Vec<String>,
    pub disallowed_tools: Vec<String>,
    pub permission_mode: Option<PermissionMode>,

    // 模型选择
    pub model: Option<String>,
    pub fallback_model: Option<String>,

    // 提示和会话
    pub system_prompt: Option<SystemPrompt>,
    pub continue_conversation: bool,
    pub resume: Option<String>,

    // 执行限制
    pub max_turns: Option<u32>,
    pub max_budget_usd: Option<f64>,
    pub max_thinking_tokens: Option<u32>,

    // MCP 和钩子
    pub mcp_servers: McpServers,
    pub hooks: Option<HashMap<HookEvent, Vec<HookMatcher>>>,

    // 环境
    pub cwd: Option<PathBuf>,
    pub env: HashMap<String, String>,
    pub add_dirs: Vec<PathBuf>,

    // CLI 配置
    pub cli_path: Option<PathBuf>,
    pub auto_install_cli: bool,
    pub cli_install_callback: Option<Arc<dyn Fn(InstallProgress) + Send + Sync>>,

    // 回调
    pub can_use_tool: Option<CanUseToolCallback>,
    pub stderr_callback: Option<Arc<dyn Fn(String) + Send + Sync>>,

    // 沙箱
    pub sandbox: Option<SandboxSettings>,

    // 技能
    pub auto_discover_skills: bool,
    pub project_skills_dir: Option<PathBuf>,
    pub user_skills_dir: Option<PathBuf>,

    // 输出
    pub output_format: Option<serde_json::Value>,
    pub include_partial_messages: bool,

    // 检查点
    pub enable_file_checkpointing: bool,

    // 插件
    pub plugins: Vec<SdkPluginConfig>,

    // 高级选项
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

**使用示例**：
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
    pub type_: String,           // 总是 "preset"
    pub preset: String,          // 如 "claude_code"
    pub append: Option<String>,  // 要追加的文本
}
```

**辅助方法**：
```rust
// 从文本创建
let prompt = SystemPrompt::from("You are a helpful assistant.");

// 预设
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

### 8.3.4 Tools 配置

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

// 辅助方法
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
    pub enabled: Option<bool>,                          // 默认：false
    pub auto_allow_bash_if_sandboxed: Option<bool>,     // 默认：true
    pub excluded_commands: Option<Vec<String>>,         // 沙箱外的命令
    pub allow_unsandboxed_commands: Option<bool>,       // 默认：true
    pub network: Option<SandboxNetworkConfig>,
    pub ignore_violations: Option<SandboxIgnoreViolations>,
    pub enable_weaker_nested_sandbox: Option<bool>,    // 仅 Docker、Linux
}

pub struct SandboxNetworkConfig {
    pub allow_unix_sockets: Option<Vec<String>>,
    pub allow_all_unix_sockets: Option<bool>,
    pub allow_local_binding: Option<bool>,              // 仅 macOS
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
    Project,  // .claude/settings.json（团队共享）
    Local,    // .claude/settings.local.json（最高优先级，git 忽略）
}
```

### 8.3.8 SdkBeta

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SdkBeta {
    #[serde(rename = "context-1m-2025-08-07")]
    Context1M,  // 扩展上下文窗口（1M tokens）
}
```

## 8.4 MCP 类型

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

### 8.4.3 服务器配置

**Stdio**（基于进程）：
```rust
pub struct McpStdioServerConfig {
    pub command: String,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
}
```

**SSE**（服务器推送事件）：
```rust
pub struct McpSseServerConfig {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
}
```

**HTTP**：
```rust
pub struct McpHttpServerConfig {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
}
```

**SDK**（进程内）：
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

### 8.4.8 辅助函数和宏

```rust
// 创建进程内 MCP 服务器
pub fn create_sdk_mcp_server(
    name: impl Into<String>,
    version: impl Into<String>,
    tools: Vec<SdkMcpTool>,
) -> McpSdkServerConfig;

// 用于创建工具的宏
tool!("tool_name", "description", json_schema, |args| async {
    // 处理器实现
    Ok(ToolResult { content: vec![...], is_error: false })
});
```

## 8.5 Hook 类型

### 8.5.1 HookEvent

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEvent {
    PreToolUse,        // 工具执行前
    PostToolUse,       // 工具执行后
    UserPromptSubmit,  // 用户提交提示时
    Stop,              // 执行停止时
    SubagentStop,      // 子代理停止时
    PreCompact,        // 对话压缩前
}
```

### 8.5.2 HookMatcher

```rust
#[derive(Clone, TypedBuilder)]
pub struct HookMatcher {
    pub matcher: Option<String>,     // 工具名称模式（如 "Bash"）
    pub hooks: Vec<HookCallback>,    // 要调用的回调
    pub timeout: Option<f64>,        // 超时时间（秒）（默认：60）
}
```

### 8.5.3 HookInput 变体

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

**PreToolUseHookInput**：
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

**PostToolUseHookInput**：
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

### 8.5.4 Hook 输出类型

**HookJsonOutput**（异步或同步）：
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HookJsonOutput {
    Async(AsyncHookJsonOutput),
    Sync(SyncHookJsonOutput),
}
```

**AsyncHookJsonOutput**：
```rust
pub struct AsyncHookJsonOutput {
    #[serde(rename = "async")]
    pub async_: bool,              // 异步时总是 true
    pub async_timeout: Option<u64>, // 超时时间（毫秒）
}
```

**SyncHookJsonOutput**：
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

### 8.5.6 Hooks 构建器

```rust
let mut hooks = Hooks::new();

// 为所有工具添加钩子
hooks.add_pre_tool_use(my_hook);
hooks.add_post_tool_use(my_hook);

// 为特定工具添加钩子
hooks.add_pre_tool_use_with_matcher("Bash", my_hook);
hooks.add_post_tool_use_with_matcher("Write", my_hook);

// 添加其他事件钩子
hooks.add_user_prompt_submit(my_hook);
hooks.add_stop(my_hook);
hooks.add_subagent_stop(my_hook);
hooks.add_pre_compact(my_hook);

// 构建用于 ClaudeAgentOptions
let hooks_map = hooks.build();
```

## 8.6 权限类型

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

### 8.6.5 权限枚举

**PermissionUpdateType**：
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

**PermissionBehavior**：
```rust
pub enum PermissionBehavior {
    Allow,
    Deny,
    Ask,
}
```

**PermissionUpdateDestination**：
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

### 8.6.7 回调类型

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

## 8.7 插件类型

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

**辅助方法**：
```rust
// 创建本地插件配置
let plugin = SdkPluginConfig::local("./my-plugin");
let plugin = SdkPluginConfig::local("~/.claude/plugins/my-plugin");

// 获取路径
if let Some(path) = plugin.path() {
    println!("Plugin path: {:?}", path);
}
```

## 8.8 V2 API 类型

### 8.8.1 SessionOptions

V2 API 的简化配置：

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

**示例**：
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

**方法**：
```rust
let total = result.total_tokens();
let cost = result.estimated_cost_usd();  // 大致定价
```

### 8.8.3 V2 Message

```rust
pub enum Message {
    User { content: String },
    Assistant { content: String },
    ToolResult { tool_name: String, result: String },
}
```

**方法**：
```rust
let text = msg.as_text();      // Option<&str>
let is_user = msg.is_user();   // bool
let is_assistant = msg.is_assistant();  // bool
let is_tool = msg.is_tool_result();     // bool
```

## 8.9 错误类型

### 8.9.1 ClaudeError

包含 14 个变体的主错误枚举：

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

### 8.9.2 专用错误

**CliNotFoundError**：
```rust
pub struct CliNotFoundError {
    pub message: String,
    pub cli_path: Option<PathBuf>,
}
```

**ConnectionError**：
```rust
pub struct ConnectionError {
    pub message: String,
}
```

**ProcessError**：
```rust
pub struct ProcessError {
    pub message: String,
    pub exit_code: Option<i32>,
    pub stderr: Option<String>,
}
```

**JsonDecodeError**：
```rust
pub struct JsonDecodeError {
    pub message: String,
    pub line: String,  // 失败的行
}
```

**MessageParseError**：
```rust
pub struct MessageParseError {
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

**ImageValidationError**：
```rust
pub struct ImageValidationError {
    pub message: String,
}
```

### 8.9.3 Result 类型别名

```rust
pub type Result<T> = std::result::Result<T, ClaudeError>;
```

## 8.10 类型转换模式

### 8.10.1 From Traits

**字符串转 UserContentBlock**：
```rust
let block: UserContentBlock = "Hello".into();
let block: UserContentBlock = String::from("Hello").into();
```

**SessionOptions 转 ClaudeAgentOptions**：
```rust
let options = SessionOptions::builder().build();
let full_options: ClaudeAgentOptions = options.into();
```

**V2 Message 从完整 Message 转换**：
```rust
let full_msg: types::messages::Message = /* ... */;
let v2_msg: v2::Message = full_msg.into();
```

### 8.10.2 Builder 模式

大多数配置类型使用 TypedBuilder：
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

## 8.11 测试覆盖

| 模块 | 测试数 | 关注领域 |
|--------|-------|-------------|
| messages.rs | 26 | 序列化、验证 |
| hooks.rs | 35 | 事件、构建器、执行 |
| permissions.rs | 7 | 序列化 |
| plugin.rs | 7 | 往返转换、路径 |
| v2/types.rs | 9 | 构建器、转换 |
| **总计** | **84** | |

## 8.12 设计模式

### 8.12.1 TypedBuilder

为可选字段提供编译时安全性：
```rust
#[derive(TypedBuilder)]
pub struct MyConfig {
    #[builder(default, setter(strip_option))]
    pub optional: Option<String>,
}
```

### 8.12.2 Serde 属性

常见模式：
```rust
#[serde(skip_serializing_if = "Option::is_none")]  // 省略 None 值
#[serde(rename = "camelCase")]                      // 重命名字段
#[serde(tag = "type", rename_all = "snake_case")]   // 标签枚举
#[serde(flatten)]                                   // 展平字段
```

### 8.12.3 thiserror

一致的错误处理：
```rust
#[derive(Debug, Error)]
#[error("Error message: {field}")]
pub struct MyError {
    pub field: String,
}
```

## 8.13 API 参考

### types/mod.rs 的重导出

```rust
pub use config::*;
pub use hooks::*;
pub use mcp::*;
pub use messages::*;
pub use permissions::*;
pub use plugin::*;
```

### 常用导入

```rust
use claude_agent_sdk::types::{
    // 消息
    Message, ContentBlock, TextBlock, ToolUseBlock,
    UserContentBlock, ImageSource,

    // 配置
    ClaudeAgentOptions, SystemPrompt, PermissionMode,

    // MCP
    McpServers, McpServerConfig, ToolResult,

    // 钩子
    HookEvent, HookInput, HookJsonOutput, Hooks,

    // 权限
    PermissionResult, PermissionBehavior,

    // 插件
    SdkPluginConfig,
};

use claude_agent_sdk::errors::{ClaudeError, Result};

use claude_agent_sdk::v2::{
    SessionOptions, PromptResult, Message as V2Message,
};
```
