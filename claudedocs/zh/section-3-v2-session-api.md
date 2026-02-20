# 第3章：V2 会话 API

本章介绍 V2 API，它提供了一种简化、类似 TypeScript 风格的接口来与 Claude 交互。

## 概述

与 V1 相比，V2 API 提供了更人性化的接口：
- **一次性提示**：简单的 `prompt()` 函数用于单次查询
- **会话式 API**：`create_session()` 和 `resume_session()` 用于多轮对话
- **简化的选项**：`SessionOptions` 只包含常用参数
- **TypeScript 风格命名**：使用 `prompt`、`send`、`receive` 而不是 `query`、`query_with_prompt`

## 模块结构

```
v2/
├── mod.rs      # 一次性 prompt() 函数 + 模块导出 (264 行, 3 个测试)
├── session.rs  # 会话式 API (322 行, 1 个测试)
└── types.rs    # 简化的类型定义 (454 行, 9 个测试)
```

**总计**：1,040 行，13 个测试

## 快速开始

```rust,no_run
use claude_agent_sdk::v2::{prompt, create_session};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 一次性提示
    let result = prompt("What is 2 + 2?", Default::default()).await?;
    println!("Answer: {}", result.content);

    // 会话式对话
    let mut session = create_session(Default::default()).await?;
    session.send("Hello, Claude!").await?;

    for message in session.receive().await? {
        println!("Message: {:?}", message);
    }

    Ok(())
}
```

## V1 与 V2 对比

### 一次性查询

```rust,ignore
// V1
let messages = query("Question", None).await?;
for msg in messages {
    if let Message::Assistant(assist_msg) = msg {
        // 处理...
    }
}

// V2
let result = prompt("Question", Default::default()).await?;
// result.content 包含回答文本
```

### 会话式

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

## API 参考

### 3.1 一次性提示

```rust
pub async fn prompt(
    prompt: impl Into<String>,
    options: SessionOptions,
) -> Result<PromptResult>
```

向 Claude 发送单个提示并返回完整响应。

**参数**：
- `prompt` - 提示文本（接受 `&str`、`String` 等）
- `options` - 会话配置选项

**返回**：`PromptResult` 包含：
- `content: String` - 响应文本
- `input_tokens: u64` - 输入 token 数量
- `output_tokens: u64` - 输出 token 数量
- `model: Option<String>` - 使用的模型（如果可用）

**示例**：
```rust,no_run
use claude_agent_sdk::v2::prompt;

let result = prompt("What is 2 + 2?", Default::default()).await?;
println!("Response: {}", result.content);
println!("Tokens: {}", result.total_tokens());
println!("Est. Cost: ${:.4}", result.estimated_cost_usd());
```

### 3.2 会话管理

#### create_session

```rust
pub async fn create_session(options: SessionOptions) -> Result<Session>
```

创建带有自动生成 UUID 的新会话并连接到 Claude。

**示例**：
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

根据 ID 恢复现有会话。注意：完整的会话持久化尚未实现；当前会使用提供的 ID 创建新会话。

**示例**：
```rust,no_run
use claude_agent_sdk::v2::{resume_session, SessionOptions};

let session = resume_session("existing-session-id", SessionOptions::default()).await?;
```

### 3.3 Session 结构体

```rust
pub struct Session {
    pub id: String,
    pub options: SessionOptions,
    // 内部 client...
}
```

**方法**：

| 方法 | 返回类型 | 描述 |
|------|----------|------|
| `send(&mut self, message)` | `Result<()>` | 向 Claude 发送消息 |
| `receive(&self)` | `Result<Vec<V2Message>>` | 接收 Claude 的响应 |
| `model(&self)` | `Option<String>` | 获取使用的模型 |
| `is_connected(&self)` | `bool` | 检查连接状态 |
| `close(self)` | `Result<()>` | 关闭并释放资源 |

**示例**：
```rust,no_run
use claude_agent_sdk::v2::{create_session, SessionOptions};

let mut session = create_session(SessionOptions::default()).await?;

// 发送消息
session.send("What is 2 + 2?").await?;

// 接收响应
let messages = session.receive().await?;
for msg in messages {
    if let Some(text) = msg.as_text() {
        println!("Claude: {}", text);
    }
}

// 关闭会话
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

**字段**：

| 字段 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `model` | `Option<String>` | `None` | 使用的模型（None = 系统默认） |
| `permission_mode` | `Option<PermissionMode>` | `None` | 工具权限处理方式 |
| `max_budget_usd` | `Option<f64>` | `None` | 预算限制（美元） |
| `max_turns` | `Option<u32>` | `None` | 最大对话轮数 |
| `max_thinking_tokens` | `Option<u32>` | `None` | 扩展思考 token 限制 |
| `system_prompt` | `Option<String>` | `None` | 自定义系统提示 |
| `include_partial_messages` | `bool` | `false` | 包含部分流消息 |

**构建器示例**：
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

**方法**：

| 方法 | 返回类型 | 描述 |
|------|----------|------|
| `total_tokens(&self)` | `u64` | 输入 + 输出 token 总和 |
| `estimated_cost_usd(&self)` | `f64` | 预估成本（输入：$3/M，输出：$15/M） |

**示例**：
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

### 3.6 V2Message（会话）

```rust
#[derive(Debug, Clone)]
pub enum V2Message {
    Assistant {
        content: String,
    },
}
```

用于会话 `receive()` 方法的简化消息类型。

**方法**：

| 方法 | 返回类型 | 描述 |
|------|----------|------|
| `as_text(&self)` | `Option<&str>` | 获取文本内容 |

### 3.7 Message（类型）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    User { content: String },
    Assistant { content: String },
    ToolResult { tool_name: String, result: String },
}
```

V2 API 的扩展消息类型，包含用户、助手和工具结果变体。

**方法**：

| 方法 | 返回类型 | 描述 |
|------|----------|------|
| `as_text(&self)` | `Option<&str>` | 获取文本（ToolResult 返回 None） |
| `is_user(&self)` | `bool` | 检查是否为用户消息 |
| `is_assistant(&self)` | `bool` | 检查是否为助手消息 |
| `is_tool_result(&self)` | `bool` | 检查是否为工具结果 |

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

控制 Claude 如何请求使用工具的权限。

| 变体 | 描述 |
|------|------|
| `Default` | 标准权限提示 |
| `AcceptEdits` | 自动接受编辑操作 |
| `Plan` | 计划模式（不执行） |
| `BypassPermissions` | 自动批准所有工具使用 |

## 实现细节

### prompt() 实现

```rust
pub async fn prompt(prompt: impl Into<String>, options: SessionOptions) -> Result<PromptResult>
```

**流程**：
1. 将 `SessionOptions` 转换为 `ClaudeAgentOptions`
2. 使用选项创建新的 `ClaudeClient`
3. 调用 `client.connect().await`
4. 调用 `client.query(&prompt_text).await`
5. 通过 `receive_response()` 流式传输消息
6. 从 `Assistant` 消息中提取文本
7. 从 `usage` JSON 字段解析 token 使用情况
8. 返回 `PromptResult`

**关键分析**：
- ✅ 简单、专注的实现
- ✅ 正确从 usage JSON 提取 token
- ⚠️ 每次调用创建新客户端（无连接池）
- ✅ 优雅处理所有消息类型（忽略非文本）

### Session 实现

**Session 结构**：
```rust
pub struct Session {
    pub id: String,
    pub options: SessionOptions,
    client: Arc<Mutex<ClaudeClient>>,
}
```

- 使用 `Arc<Mutex<...>>` 实现线程安全的客户端共享
- 通过 `uuid::Uuid::new_v4()` 生成基于 UUID 的会话 ID

**send() 方法**：
1. 验证消息不为空（返回 `InvalidInput` 错误）
2. 锁定客户端互斥锁
3. 调用 `client.query(&message_text).await`

**receive() 方法**：
1. 锁定客户端（持有锁时不等待 - 潜在问题）
2. 从 `receive_response()` 获取流
3. 迭代流直到 `Result` 消息（回合结束）
4. 从 `Assistant` 内容块中提取文本
5. 返回 `Vec<V2Message>`

**分析**：
- ⚠️ `receive()` 在流迭代期间持有互斥锁 - 可能阻塞并发发送
- ✅ 清晰的空消息验证和错误提示
- ✅ 通过 `Result` 消息进行干净的回合结束检测

### SessionOptions 转换

`From<SessionOptions> for ClaudeAgentOptions` 实现使用 8 分支 match 表达式处理所有可选字段的组合：

```rust
match (options.model, permission_mode, options.max_budget_usd) {
    (Some(model), Some(pm), Some(max_budget)) => { /* ... */ }
    (Some(model), Some(pm), None) => { /* ... */ }
    // ... 还有 6 个分支
}
```

**分析**：
- ⚠️ 冗长 - 8 个几乎相同的分支
- ⚠️ 对 `max_turns` 和 `max_thinking_tokens` 使用 `unwrap_or(0)` - 应该保留 None
- ⚠️ 当 None 时创建空的 `SystemPrompt::Text(String::new())` - 可能不正确
- 💡 可以用 `..Default::default()` 模式或自定义构建器简化

## 错误处理

| 场景 | 错误类型 | 消息 |
|------|----------|------|
| 空消息 | `ClaudeError::InvalidInput` | "Message cannot be empty" |
| 连接失败 | `ClaudeError::ConnectionError` | 来自底层客户端 |
| 查询失败 | `ClaudeError::QueryError` | 来自底层客户端 |
| 响应解析 | `ClaudeError::ParseError` | 来自底层客户端 |

## 测试覆盖

**mod.rs**（3 个测试）：
- `test_prompt_result_structure` - 结构体创建和 `total_tokens()`
- `test_session_options_default` - 默认选项创建
- `test_session_options_builder` - 构建器模式

**session.rs**（1 个测试）：
- `test_v2_message_as_text` - V2Message 文本提取

**types.rs**（9 个测试）：
- `test_session_options_builder` - 构建器模式
- `test_permission_mode_conversion` - V2 到 V1 转换
- `test_prompt_result_total_tokens` - Token 计算
- `test_message_is_user` - 用户变体检查
- `test_message_is_assistant` - 助手变体检查
- `test_message_is_tool_result` - ToolResult 变体检查
- `test_prompt_result_cost_estimation` - 成本计算

**总计**：13 个测试

**覆盖评估**：
- ✅ 核心功能已测试
- ⚠️ 没有实际 API 调用的集成测试
- ⚠️ 没有 `prompt()` 函数行为的测试
- ⚠️ 没有会话 `send()`/`receive()` 流程的测试
- ⚠️ 没有 `resume_session()` 的测试

## 安全分析

| 关注点 | 状态 | 备注 |
|--------|------|------|
| 消息验证 | ✅ | 空消息检查 |
| 输入清理 | ⚠️ | 没有提示文本的清理 |
| 会话 ID 生成 | ✅ | 通过 `uuid` crate 的 UUID |
| 并发访问 | ✅ | Arc<Mutex<...>> 模式 |

## 性能分析

| 操作 | 复杂度 | 备注 |
|------|--------|------|
| `prompt()` | O(n) | n = 响应大小（流式） |
| `send()` | O(1) | 异步查询分发 |
| `receive()` | O(n) | n = 消息数量 |
| 会话创建 | O(1) | UUID 生成 + 连接 |

**性能考虑**：
1. **无连接池**：每次 `prompt()` 调用都创建新客户端
2. **互斥锁竞争**：`receive()` 在流迭代期间持有锁
3. **内存**：完整的消息内容存储在内存中

## 功能对等性

V2 提供与 V1 相同的功能，但 API 更简单：

| 功能 | V1 | V2 |
|------|----|----|
| 一次性查询 | `query()` | `prompt()` |
| 多轮会话 | `ClaudeClient` | `Session` |
| 流式传输 | `receive_response()` | `receive()` |
| 权限管理 | `PermissionMode` | `PermissionMode` |
| 成本控制 | `max_budget_usd` | `max_budget_usd` |
| 自定义工具 | 完全支持 | 通过选项 |
| 钩子 | 完全支持 | 通过选项 |
| 会话恢复 | 部分 | `resume_session()` |

## API 质量评估

| 方面 | 评级 | 备注 |
|------|------|------|
| 文档 | ✅ 优秀 | 全面的文档注释 |
| 示例 | ✅ 良好 | 文档内示例 + 快速开始 |
| 错误处理 | ✅ 良好 | 清晰的错误 |
| 类型安全 | ✅ 良好 | TypedBuilder + Serde |
| 易用性 | ✅ 优秀 | 非常简单的 API |
| 一致性 | ✅ 良好 | 遵循 Rust 约定 |

## 发现总结

**关键问题**：0

**重要问题**：0

**建议**：
1. **性能**：为 `prompt()` 添加连接池以避免每次调用创建新客户端
2. **并发**：考虑在 `receive()` 中的流迭代之前释放互斥锁
3. **重构**：使用构建器默认值或模式匹配简化 `From<SessionOptions>`
4. **默认值**：保留可选字段的 `None` 而不是 `unwrap_or(0)`
5. **测试**：添加会话流程的集成测试
6. **SystemPrompt**：比空字符串更好地处理 None 情况

**积极评价**：
- 非常干净、人性化的 API 设计
- 带有示例的优秀文档
- TypeScript 友好的命名约定
- TypedBuilder 提供编译时安全
- V1 和 V2 关注点良好分离
- 简化的选项减少了认知负担
- 会话创建时自动连接改善了开发体验

## 迁移指南

### 从 V1 到 V2

**一次性查询**：
```rust,ignore
// V1
let messages = query("Question", None).await?;
for msg in messages {
    if let Message::Assistant(assist_msg) = {
        // 处理...
    }
}

// V2
let result = prompt("Question", Default::default()).await?;
// result.content 包含回答文本
```

**会话式**：
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

**选项转换**：
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
