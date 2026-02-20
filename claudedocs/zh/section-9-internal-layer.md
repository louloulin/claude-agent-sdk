# 第9章：内部层

本章介绍 Claude Agent SDK for Rust 的内部实现层，该层负责与 Claude Code CLI 的底层通信。

## 9.1 概述

内部层为 SDK-CLI 通信提供核心基础设施：

```
internal/
├── mod.rs            # 模块导出
├── client.rs         # 简单客户端包装器
├── message_parser.rs # JSON到Message的解析
├── query_full.rs     # 完整双向控制协议
├── cli_installer.rs  # 自动安装系统
└── transport/
    ├── mod.rs        # 传输模块导出
    ├── trait_def.rs  # Transport trait 定义
    └── subprocess.rs # 子进程传输实现
```

**核心组件**：
- **传输层**：CLI 通信的抽象接口和子进程实现
- **消息解析器**：将 JSON 输出转换为类型化的 Message 结构
- **QueryFull**：支持钩子和 MCP 的双向控制协议
- **CLI 安装器**：自动下载和安装 CLI

## 9.2 Transport Trait

### 9.2.1 Transport 定义

`Transport` trait 定义了 CLI 通信的接口：

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    /// 连接传输层
    async fn connect(&mut self) -> Result<()>;

    /// 向传输层写入原始数据
    async fn write(&mut self, data: &str) -> Result<()>;

    /// 以 JSON 值流的形式读取消息
    fn read_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>>;

    /// 关闭传输层
    async fn close(&mut self) -> Result<()>;

    /// 检查传输层是否就绪
    fn is_ready(&self) -> bool;

    /// 结束输入流（关闭 stdin）
    async fn end_input(&mut self) -> Result<()>;
}
```

### 9.2.2 Transport 生命周期

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

### 9.3.1 结构

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

### 9.3.2 QueryPrompt 类型

```rust
pub enum QueryPrompt {
    /// 文本提示（一次性模式）
    Text(String),
    /// 结构化内容块（支持图片和文本）
    Content(Vec<UserContentBlock>),
    /// 流式模式（无初始提示）
    Streaming,
}
```

**转换 Trait**：
```rust
// 从字符串
let prompt: QueryPrompt = "Hello".into();
let prompt: QueryPrompt = String::from("Hello").into();

// 从内容块
let blocks = vec![UserContentBlock::text("Hello")];
let prompt: QueryPrompt = blocks.into();
```

### 9.3.3 CLI 发现策略

传输层使用 5 层 CLI 发现策略：

| 优先级 | 策略 | 说明 |
|--------|------|------|
| 1 | 执行 `claude --version` | 最可靠，使用 shell PATH |
| 2 | `which claude` (Unix) | 显式 PATH 查找 |
| 3 | `where claude` (Windows) | Windows 等效命令 |
| 4 | 常用路径 | `/usr/local/bin`、`~/.local/bin` 等 |
| 5 | `CLAUDE_CLI_PATH` 环境变量 | 手动覆盖 |

**常用安装路径**：
```rust
// Unix 类系统
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

### 9.3.4 命令构建

传输层从 `ClaudeAgentOptions` 构建 CLI 参数：

```rust
fn build_command(&self) -> Vec<String>
```

**生成的参数**：

| 选项 | CLI 参数 |
|------|----------|
| `output_format` | `--output-format stream-json --verbose` |
| `system_prompt` | `--system-prompt "text"` 或 `--append-system-prompt` |
| `tools` | `--tools tool1,tool2` 或 `--tools default` |
| `permission_mode` | `--permission-mode bypassPermissions` |
| `allowed_tools` | `--allowedTools Bash,Read` |
| `model` | `--model claude-sonnet-4-20250514` |
| `max_turns` | `--max-turns 10` |
| `resume` | `--resume session-id` |
| `continue_conversation` | `--continue` |
| `max_budget_usd` | `--max-budget-usd 1.0` |
| `betas` | `--betas context-1m-2025-08-07` |

**环境变量**：
```rust
CLAUDE_CODE_ENTRYPOINT=rust-sdk
CLAUDE_AGENT_SDK_VERSION=0.1.0
CLAUDE_CODE_ENABLE_SDK_FILE_CHECKPOINTING=true  // 如果启用
```

### 9.3.5 版本检查

```rust
async fn check_claude_version(&self) -> Result<()>
```

- 将 CLI 版本与 `MIN_CLI_VERSION` 比较
- 如果设置 `CLAUDE_SKIP_VERSION_CHECK=1` 则跳过检查
- 如果版本低于最低要求则记录警告

### 9.3.6 消息流

```rust
fn read_messages(
    &mut self,
) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>>
```

**实现细节**：
1. 使用 `async_stream::stream!` 进行惰性求值
2. 从 stdout 逐行读取
3. 根据 `max_buffer_size`（默认：10MB）跟踪缓冲区大小
4. 将每行解析为 JSON
5. 产出 `Result<serde_json::Value>`

## 9.4 CLI 自动安装器

### 9.4.1 InstallProgress 事件

```rust
pub enum InstallProgress {
    /// 检查安装状态
    Checking(String),
    /// 下载进度
    Downloading {
        current: u64,
        total: Option<u64>,
    },
    /// 安装进行中
    Installing(String),
    /// 安装完成
    Done(PathBuf),
    /// 安装失败
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

**方法**：
```rust
// 创建安装器
let installer = CliInstaller::new(true);

// 带进度回调
let installer = CliInstaller::new(true)
    .with_progress_callback(Arc::new(|event| {
        match event {
            InstallProgress::Downloading { current, total } => {
                println!("Downloaded: {} bytes", current);
            }
            // ...
        }
    }));

// 如果需要则安装
let cli_path = installer.install_if_needed().await?;
```

### 9.4.3 安装策略

```
┌───────────────────────┐
│  检查现有 CLI         │
└──────────┬────────────┘
           │ 未找到
           ▼
┌───────────────────────┐
│   尝试 npm 安装       │
│  (npm install -g      │
│   @anthropic-ai/      │
│   claude-code)        │
└──────────┬────────────┘
           │ 失败
           ▼
┌───────────────────────┐
│  尝试直接下载         │
│  (GitHub Releases)    │
└──────────┬────────────┘
           │ 失败
           ▼
┌───────────────────────┐
│     返回错误          │
└───────────────────────┘
```

### 9.4.4 平台检测

```rust
fn detect_platform() -> (&'static str, &'static str)

// 返回 (platform, arch)：
// - ("darwin", "x64")    - macOS Intel
// - ("darwin", "arm64")  - macOS Apple Silicon
// - ("linux", "x64")     - Linux x86_64
// - ("linux", "arm64")   - Linux ARM64
// - ("windows", "x64")   - Windows x86_64
```

### 9.4.5 安装目录

| 平台 | 安装路径 |
|------|----------|
| Unix | `~/.local/bin/claude` |
| Windows | `%USERPROFILE%\AppData\Local\Programs\Claude\claude.exe` |

## 9.5 MessageParser

### 9.5.1 结构

```rust
pub struct MessageParser;
```

### 9.5.2 解析

```rust
impl MessageParser {
    /// 将 JSON 值解析为 Message
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

**使用示例**：
```rust
let json: serde_json::Value = /* ... */;
let message = MessageParser::parse(json)?;
```

## 9.6 InternalClient

### 9.6.1 结构

```rust
pub struct InternalClient {
    transport: SubprocessTransport,
}
```

### 9.6.2 简单执行流程

```rust
impl InternalClient {
    /// 创建新客户端
    pub fn new(prompt: QueryPrompt, options: ClaudeAgentOptions) -> Result<Self>

    /// 连接并获取消息
    pub async fn execute(mut self) -> Result<Vec<Message>>
}
```

**执行流程**：
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

### 9.7.1 结构

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

### 9.7.2 控制协议

**请求格式**（SDK → CLI）：
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

**响应格式**（CLI → SDK）：
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

### 9.7.3 初始化

```rust
pub async fn initialize(
    &self,
    hooks: Option<HashMap<String, Vec<HookMatcher>>>,
) -> Result<serde_json::Value>
```

**钩子注册**：
1. 分配唯一回调 ID（`hook_0`、`hook_1`...）
2. 将回调存储在 `hook_callbacks` 映射中
3. 向 CLI 发送匹配器配置
4. 返回初始化结果

### 9.7.4 后台消息处理

```rust
pub async fn start(&self) -> Result<()>
```

**生成后台任务**：
```
┌─────────────────────────────────────────────────┐
│              后台任务                            │
├─────────────────────────────────────────────────┤
│  loop {                                          │
│    match message.type {                          │
│      "control_response" => 处理响应              │
│      "control_request"  => 处理传入请求          │
│      _                 => 转发到 message_tx      │
│    }                                             │
│  }                                               │
└─────────────────────────────────────────────────┘
```

### 9.7.5 控制请求处理器

**钩子回调**（`subtype: "hook_callback"`）：
```rust
// 1. 通过 ID 查找回调
// 2. 从请求解析 HookInput
// 3. 执行回调
// 4. 返回 HookJsonOutput
```

**MCP 消息**（`subtype: "mcp_message"`）：
```rust
// 1. 通过名称查找 SDK MCP 服务器
// 2. 调用 server.handle_message()
// 3. 返回响应
```

### 9.7.6 控制方法

**中断**：
```rust
pub async fn interrupt(&self) -> Result<()>
```

**更改权限模式**：
```rust
pub async fn set_permission_mode(
    &self,
    mode: PermissionMode,
) -> Result<()>
```

**更改模型**：
```rust
pub async fn set_model(&self, model: Option<&str>) -> Result<()>
```

**回滚文件**：
```rust
/// 将跟踪的文件回滚到特定用户消息时的状态。
///
/// 需要：
/// - `enable_file_checkpointing=true`
/// - `extra_args={"replay-user-messages": None}`
pub async fn rewind_files(&self, user_message_id: &str) -> Result<()>
```

**获取服务器信息**：
```rust
pub async fn get_initialization_result(&self) -> Option<serde_json::Value>
```

## 9.8 消息流

### 9.8.1 一次性查询流程

```
用户代码
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

### 9.8.2 双向控制流程

```
用户代码
    │
    ▼
QueryFull::initialize(hooks)
    │
    ▼
QueryFull::start()  ──────────────────┐
    │                                 │
    │                          ┌──────▼──────┐
    │                          │ 后台任务    │
    │                          │ 循环        │
    │                          └──────┬──────┘
    │                                 │
    ▼                                 │
message_rx.recv() ◀───────────────────┘
    │
    ▼
处理消息

控制操作：
    interrupt() ──────▶ send_control_request()
    set_model() ──────▶ send_control_request()
    rewind_files() ───▶ send_control_request()
```

## 9.9 错误处理

### 9.9.1 错误类型

| 错误类型 | 来源 | 示例 |
|----------|------|------|
| `CliNotFound` | CLI 不在 PATH 中 | "Claude Code CLI not found" |
| `ConnectionError` | 连接失败 | "Failed to get stdin" |
| `ProcessError` | CLI 退出失败 | "Non-zero exit status" |
| `JsonDecodeError` | 解析失败 | "Failed to parse JSON" |
| `MessageParseError` | 消息类型错误 | "Failed to parse message" |
| `Transport` | I/O 错误 | "Failed to write to stdin" |
| `ControlProtocol` | 协议错误 | "Missing subtype" |

### 9.9.2 恢复模式

**CLI 未找到**：
```rust
// 选项 1：手动安装提示
// 选项 2：启用自动安装
let options = ClaudeAgentOptions::builder()
    .auto_install_cli(true)
    .cli_install_callback(Arc::new(|event| {
        // 处理进度
    }))
    .build();
```

**版本不匹配**：
```rust
// 跳过版本检查（不推荐）
std::env::set_var("CLAUDE_SKIP_VERSION_CHECK", "1");
```

**缓冲区溢出**：
```rust
// 增加缓冲区大小
let options = ClaudeAgentOptions::builder()
    .max_buffer_size(50 * 1024 * 1024) // 50MB
    .build();
```

## 9.10 测试覆盖

| 模块 | 测试数 | 重点 |
|------|--------|------|
| cli_installer.rs | 2 | 平台检测、安装目录 |
| message_parser.rs | 0 | （由 types 模块测试覆盖） |
| client.rs | 0 | （简单包装器，通过集成测试） |
| query_full.rs | 0 | （复杂异步，通过集成测试） |
| transport/subprocess.rs | 0 | （基于进程，通过集成测试） |

**注意**：内部层主要通过集成测试和示例代码进行测试。

## 9.11 API 参考

### internal/mod.rs 重导出

```rust
pub mod cli_installer;
pub mod client;
pub mod message_parser;
pub mod query_full;
pub mod transport;
```

### transport/mod.rs 重导出

```rust
pub use subprocess::SubprocessTransport;
pub use trait_def::Transport;
```

### 常用导入

```rust
use claude_agent_sdk::internal::{
    client::InternalClient,
    message_parser::MessageParser,
    query_full::QueryFull,
    cli_installer::{CliInstaller, InstallProgress},
    transport::{Transport, SubprocessTransport, QueryPrompt},
};
```

## 9.12 设计模式

### 9.12.1 异步流模式

```rust
fn read_messages(&mut self) -> Pin<Box<dyn Stream<Item = Result<Value>> + Send + '_>> {
    Box::pin(async_stream::stream! {
        // 产出值
        yield Ok(json);
    })
}
```

### 9.12.2 后台任务模式

```rust
tokio::spawn(async move {
    while let Some(result) = stream.next().await {
        // 在后台处理
    }
});
```

### 9.12.3 直接 Stdin 访问模式

```rust
// 绕过传输锁以实现并发写入
if let Some(ref stdin) = self.stdin {
    let mut guard = stdin.lock().await;
    if let Some(ref mut stream) = *guard {
        stream.write_all(data.as_bytes()).await?;
    }
}
```

## 9.13 性能考虑

### 9.13.1 进程启动开销

- 每次查询启动新的 CLI 进程（约 50-100ms）
- 对于高频查询，考虑连接池
- V2 Session API 保持持久连接

### 9.13.2 缓冲区管理

- 默认 10MB 缓冲区大小
- 根据预期消息大小配置
- 溢出会导致流终止

### 9.13.3 锁竞争

- 使用 `Arc<Mutex<>>` 管理共享状态
- 后台任务持有传输锁
- 直接 stdin 访问绕过锁

## 9.14 安全考虑

| 关注点 | 缓解措施 |
|--------|----------|
| 路径注入 | 启动前验证 CLI 路径 |
| 命令注入 | 无 shell 解释 |
| 进程隔离 | 独立进程，受控环境 |
| Stdin/stdout 访问 | Mutex 保护的流 |
| 自动安装 | 仅从官方源下载 |
