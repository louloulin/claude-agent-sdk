# Claude Agent SDK Rust - Architecture Documentation

**本文档详细说明 Claude Agent SDK Rust 的系统架构、设计决策和实现细节。**

---

## 📋 目录

1. [概述](#概述)
2. [架构设计原则](#架构设计原则)
3. [核心组件](#核心组件)
4. [模块架构](#模块架构)
5. [数据流](#数据流)
6. [并发模型](#并发模型)
7. [错误处理](#错误处理)
8. [扩展机制](#扩展机制)
9. [性能考虑](#性能考虑)
10. [设计决策](#设计决策)

---

## 概述

Claude Agent SDK Rust 是一个高性能、类型安全的 Rust SDK，用于与 Anthropic Claude AI 进行交互。

### 核心目标

1. **性能**: 原生性能，零成本抽象
2. **安全性**: 内存安全，类型安全
3. **可靠性**: 完善的错误处理
4. **可扩展性**: 插件系统、Hooks、MCP
5. **易用性**: 简洁的 API，丰富的示例

### 技术栈

- **语言**: Rust 2024 Edition
- **异步运行时**: Tokio 1.48
- **序列化**: Serde 1.0
- **流处理**: Futures 0.3
- **错误处理**: thiserror 2.0, anyhow 1.0

---

## 架构设计原则

### 1. 关注点分离 (Separation of Concerns)

每个模块有明确的职责：

```
src/
├── client.rs          # 客户端 API
├── query.rs           # 简单查询函数
├── internal/          # 内部实现
│   ├── client.rs      # 内部客户端
│   └── transport/     # 传输层
├── types/             # 类型定义
│   ├── config.rs      # 配置
│   ├── hooks.rs       # Hooks
│   ├── permissions.rs # 权限
│   └── messages.rs    # 消息
├── mcp/               # MCP 协议
│   └── tasks.rs       # 异步任务
└── skills/            # Agent Skills
```

### 2. 零成本抽象 (Zero-Cost Abstractions)

使用 Rust 的零成本抽象特性：

```rust
// 编译时优化，运行时无开销
pub async fn query(prompt: &str, options: Option<ClaudeAgentOptions>) -> Result<Vec<Message>> {
    // 直接调用，无虚函数开销
    InternalClient::new(QueryPrompt::Text(prompt.into()), options.unwrap_or_default())?
        .execute()
        .await
}
```

### 3. 类型安全 (Type Safety)

利用 Rust 类型系统防止错误：

```rust
// 编译时保证类型安全
pub enum QueryPrompt {
    Text(String),
    Content(Vec<UserContentBlock>),
    Streaming,
}

// 不可能错误地使用错误的类型
let prompt = QueryPrompt::Text("hello".to_string());
```

### 4. 异步优先 (Async-First)

全异步设计，高并发支持：

```rust
pub async fn query_stream(
    prompt: impl Into<String>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>>
```

---

## 核心组件

### 3.1 Client Layer

**职责**: 提供高级 API，管理会话状态

```rust
pub struct ClaudeClient {
    transport: SubprocessTransport,
    options: ClaudeAgentOptions,
    connected: bool,
}
```

**关键方法**:
- `new()`: 创建客户端
- `connect()`: 连接到 Claude CLI
- `query()`: 发送查询
- `execute()`: 执行命令
- `disconnect()`: 断开连接

### 3.2 Transport Layer

**职责**: 处理与 Claude CLI 的通信

```rust
pub struct SubprocessTransport {
    cli_path: PathBuf,
    cwd: Option<PathBuf>,
    options: ClaudeAgentOptions,
    prompt: QueryPrompt,
    process: Option<Child>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stdout: Arc<Mutex<Option<BufReader<ChildStdout>>>>,
}
```

**关键特性**:
- 启动和管理 Claude CLI 子进程
- 标准输入/输出的异步读写
- JSON 消息流解析
- 连接状态管理

### 3.3 Message Parser

**职责**: 解析 Claude CLI 的 JSON 输出

```rust
pub struct MessageParser;

impl MessageParser {
    pub fn parse(json: serde_json::Value) -> Result<Message> {
        match json["type"].as_str() {
            Some("assistant") => { /* ... */ }
            Some("result") => { /* ... */ }
            Some("error") => { /* ... */ }
            _ => Err(/* ... */),
        }
    }
}
```

**支持的消息类型**:
- `Assistant`: Claude 的响应
- `Result`: 查询结果
- `Error`: 错误信息
- `System`: 系统消息

---

## 模块架构

### 4.1 类型系统 (Types)

```
types/
├── config.rs          # ClaudeAgentOptions
├── hooks.rs           # Hooks 事件系统
├── permissions.rs     # 权限管理
└── messages.rs        # 消息类型定义
```

#### ClaudeAgentOptions

```rust
pub struct ClaudeAgentOptions {
    // 模型配置
    pub model: Option<String>,
    pub fallback_model: Option<String>,

    // 响应控制
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,

    // 会话控制
    pub max_turns: Option<u32>,
    pub thinking: Option<bool>,
    pub max_thinking_tokens: Option<u32>,

    // 权限管理
    pub permission_mode: Option<PermissionMode>,
    pub allowed_tools: Option<Vec<String>>,

    // 扩展功能
    pub hooks: Option<Hooks>,
    pub plugins: Vec<SdkPluginConfig>,
    pub budget: Option<BudgetOptions>,

    // 工作目录
    pub cwd: Option<PathBuf>,
}
```

### 4.2 Hooks 系统

```
types/hooks/
├── mod.rs
├── events.rs          # Hook 事件定义
├── matcher.rs         # Hook 匹配器
└── handlers.rs        # Hook 处理器
```

**Hook 事件类型**:

```rust
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    ToolError,
    PreCompact,
    Stop,
    Init,
}
```

**Hooks 架构**:

```
User Request
    ↓
Hook System
    ↓
PreToolUse Hook → Tool Execution → PostToolUse Hook
    ↓
Response
```

### 4.3 MCP (Model Context Protocol)

```
mcp/
├── mod.rs
├── client.rs          # MCP 客户端
├── server.rs          # MCP 服务器
└── tasks.rs           # 异步任务协议
```

**MCP 异步任务** (2025-11-25):

```rust
pub struct Task {
    pub id: String,
    pub status: TaskStatus,
    pub result: Option<serde_json::Value>,
}

pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}
```

### 4.4 Agent Skills 系统

```
skills/
├── mod.rs
├── trait_impl.rs      # Skill trait 定义
├── types.rs           # 技能类型
├── registry.rs        # 技能注册表
├── dependency.rs      # 依赖管理
├── version.rs         # 版本管理
├── tags.rs            # 标签系统
├── hot_reload.rs      # 热重载
├── sandbox.rs         # 沙箱执行
├── performance.rs     # 性能优化
├── vscode.rs          # VSCode 集成
└── error.rs           # 错误类型
```

**Skills 架构**:

```
Skill Definition
    ↓
Skill Registry
    ↓
Dependency Resolution
    ↓
Version Compatibility Check
    ↓
Skill Execution (with optional Sandbox)
```

---

## 数据流

### 5.1 简单查询流程

```
User Code
    ↓
query("prompt", None)
    ↓
QueryPrompt::Text("prompt")
    ↓
InternalClient::new()
    ↓
SubprocessTransport::new()
    ↓
SubprocessTransport::connect()
    ↓
启动 Claude CLI 子进程
    ↓
发送 JSON 到 stdin
    ↓
从 stdout 读取 JSON 流
    ↓
MessageParser::parse()
    ↓
返回 Vec<Message>
```

### 5.2 流式查询流程

```
User Code
    ↓
query_stream("prompt", None)
    ↓
创建异步流
    ↓
SubprocessTransport::read_messages()
    ↓
Stream<Item = Result<Message>>
    ↓
用户逐条消费消息
```

### 5.3 双向通信流程

```
User Code
    ↓
ClaudeClient::new()
    ↓
ClaudeClient::query()
    ↓
发送到 stdin
    ↓
ClaudeClient::receive_response()
    ↓
从 stdout 读取流
    ↓
用户可以发送 interrupt
```

---

## 并发模型

### 6.1 Tokio 运行时

使用 Tokio 异步运行时：

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Tokio 多线程运行时
    let messages = query("prompt", None).await?;
    Ok(())
}
```

**运行时配置**:
- **工作线程**: 默认 CPU 核心数
- **阻塞线程**: 默认 512 个
- **调度策略**: 工作窃取 (work-stealing)

### 6.2 流处理

使用 `futures` 和 `async-stream`:

```rust
use futures::stream::{Stream, StreamExt};
use async_stream::stream;

pub async fn query_stream(...) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>> {
    let stream = stream! {
        let mut message_stream = transport.read_messages();
        while let Some(json_result) = message_stream.next().await {
            match json_result {
                Ok(json) => {
                    match MessageParser::parse(json) {
                        Ok(message) => yield Ok(message),
                        Err(e) => yield Err(e),
                    }
                }
                Err(e) => yield Err(e),
            }
        }
    };

    Ok(Box::pin(stream))
}
```

### 6.3 并发控制

使用 `Semaphore` 控制并发：

```rust
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));

async fn controlled_query(prompt: &str) -> Result<Vec<Message>> {
    let _permit = semaphore.acquire().await?;
    query(prompt, None).await
}
```

---

## 错误处理

### 7.1 错误类型层次

```
std::error::Error (trait)
    ↓
thiserror::Error (derive macro)
    ↓
ClaudeError (enum)
    ├── CliNotFoundError
    ├── ConnectionError
    ├── JsonDecodeError
    ├── ProcessError
    ├── InvalidConfig
    └── ToolError
```

### 7.2 错误传播

使用 `?` 运算符和 `anyhow`:

```rust
use anyhow::{Context, Result};

pub async fn process_query(prompt: &str) -> Result<Vec<Message>> {
    let messages = query(prompt, None)
        .await
        .context("Failed to execute query")?;

    Ok(messages)
}
```

### 7.3 错误恢复

实现优雅降级：

```rust
pub async fn query_with_fallback(prompt: &str) -> Result<Vec<Message>> {
    // 尝试主要查询
    match query(prompt, None).await {
        Ok(messages) => Ok(messages),
        Err(e) => {
            // 降级到简化查询
            warn!("Primary query failed, trying fallback: {}", e);

            let simple_options = ClaudeAgentOptions {
                max_tokens: Some(1000),
                ..Default::default()
            };

            query(prompt, Some(simple_options)).await
        }
    }
}
```

---

## 扩展机制

### 8.1 插件系统

**插件配置**:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SdkPluginConfig {
    Local {
        path: PathBuf,
    },
}
```

**插件加载流程**:

```
Plugin Configuration
    ↓
Validate Plugin Path
    ↓
Load plugin.json
    ↓
Pass to Claude CLI
    ↓
Initialize Plugin
```

### 8.2 自定义工具 (MCP)

使用 `tool!` 宏定义工具：

```rust
tool!(MyCustomTool {
    description: "A custom tool",
    parameters: {
        input: String,
    },
    execute: |input| async move {
        // 工具逻辑
        Ok(serde_json::json!({ "result": "success" }).to_string())
    }
});
```

### 8.3 Hooks 扩展点

**可用的 Hook**:

```rust
pub struct Hooks {
    pre_tool_use: Vec<HookHandler>,
    post_tool_use: Vec<HookHandler>,
    tool_error: Vec<HookHandler>,
    pre_compact: Vec<HookHandler>,
    stop: Vec<HookHandler>,
    init: Vec<HookHandler>,
}
```

---

## 性能考虑

### 9.1 内存管理

**零拷贝设计**:

```rust
// 避免不必要的克隆
pub fn parse_message(json: &serde_json::Value) -> Result<Message> {
    // 使用引用而非克隆
    let msg_type = json["type"].as_str().unwrap();
    // ...
}
```

**流式处理**:

```rust
// O(1) 内存每消息，而非 O(n)
let mut stream = query_stream(large_prompt, None).await?;
while let Some(result) = stream.next().await {
    // 处理完立即丢弃
}
```

### 9.2 并发性能

**Tokio 工作窃取调度**:
- 多个工作线程
- 自动负载均衡
- 高 CPU 利用率

**异步 I/O**:
- 非阻塞读写
- 高并发能力
- 资源高效利用

### 9.3 性能优化策略

1. **连接复用**: 避免重复创建客户端
2. **批处理**: 合并多个小查询
3. **缓存**: 缓存重复查询结果
4. **流式**: 大响应使用流式 API
5. **并发控制**: 使用 Semaphore 限制并发

---

## 设计决策

### 10.1 为什么使用子进程而非 HTTP API?

**决策**: 通过子进程与 Claude CLI 通信

**理由**:
- ✅ **一致性**: 与 Python SDK 保持一致
- ✅ **功能完整**: CLI 提供所有功能
- ✅ **简化维护**: 依赖 CLI 而非直接 API
- ✅ **本地执行**: 支持本地文件操作

**权衡**:
- ⚠️ **进程开销**: 启动子进程的成本
- ⚠️ **平台依赖**: 依赖 CLI 安装

**缓解措施**:
- 使用 `ClaudeClient` 复用连接
- 长时间运行的会话
- 连接池（未来可能）

### 10.2 为什么使用 JSON 而非二进制协议?

**决策**: 使用 JSON 进行通信

**理由**:
- ✅ **可读性**: 易于调试
- ✅ **灵活性**: 易于扩展
- ✅ **兼容性**: 与 Python SDK 一致
- ✅ **生态**: 丰富的 JSON 工具

**权衡**:
- ⚠️ **性能**: JSON 解析开销
- ⚠️ **大小**: JSON 比 MessagePack 大

**缓解措施**:
- 使用 `serde_json` (高性能)
- 流式解析
- 按需反序列化

### 10.3 为什么使用 Stream 而非 Iterator?

**决策**: 使用 `Stream` trait 而非 `Iterator`

**理由**:
- ✅ **异步**: Stream 支持 async
- ✅ **非阻塞**: 不会阻塞运行时
- ✅ **背压**: 支持背压控制
- ✅ **组合性**: 易于组合操作

**权衡**:
- ⚠️ **复杂性**: 稍微复杂

### 10.4 为什么使用 Arc<Mutex<T>> 而非 Rc<RefCell<T>>?

**决策**: 使用 `Arc<Mutex<T>>`

**理由**:
- ✅ **线程安全**: Mutex 支持多线程
- ✅ **异步友好**: `tokio::sync::Mutex` 可跨 await
- ✅ **性能**: 比 RwLock 在单写场景更快

### 10.5 为什么提供两种 API (query 和 query_stream)?

**决策**: 同时提供 `query()` 和 `query_stream()`

**理由**:
- ✅ **易用性**: `query()` 简单直接
- ✅ **性能**: `query_stream()` 内存高效
- ✅ **灵活性**: 用户选择合适的 API

**使用场景**:
- `query()`: 小响应、简单场景
- `query_stream()`: 大响应、实时处理

---

## 未来架构演进

### 11.1 短期计划 (Q2 2026)

- [ ] Rig 框架集成
- [ ] 性能优化实施
- [ ] 更多集成测试

### 11.2 中期计划 (Q3 2026)

- [ ] 多 Agent 编排
- [ ] 连接池
- [ ] HTTP API 支持（可选）

### 11.3 长期计划 (Q4 2026)

- [ ] 分布式执行
- [ ] GraphQL 支持
- [ ] gRPC 支持

---

## 总结

### 关键架构特性

1. **分层架构**: Client → Transport → Parser
2. **异步优先**: 全异步设计
3. **类型安全**: 利用 Rust 类型系统
4. **零成本抽象**: 编译时优化
5. **可扩展**: Hooks、插件、MCP

### 性能特征

- **延迟**: 2-5秒（取决于查询复杂度）
- **吞吐量**: 0.5+ req/s
- **内存**: O(1) 每消息（流式）
- **并发**: 支持高并发

### 可靠性保证

- **错误处理**: 完善的错误类型
- **重试机制**: 支持重试和降级
- **超时控制**: 防止无限等待
- **资源清理**: RAII + Drop

---

**文档版本**: v0.6.0
**最后更新**: 2026-01-08
**维护者**: Claude Agent SDK Rust Team
