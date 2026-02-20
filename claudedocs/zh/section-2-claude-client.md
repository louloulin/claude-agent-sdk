# 第2章：ClaudeClient 深入解析

本章深入探讨 `ClaudeClient`，这是与 Claude Code CLI 进行双向流通信的核心组件。

## 概述

`ClaudeClient` 提供与 Python `ClaudeSDKClient` 相同的功能，支持实时双向通信、流式响应和动态控制 Claude 会话。

### 核心结构

```rust
pub struct ClaudeClient {
    options: ClaudeAgentOptions,
    query: Option<Arc<Mutex<QueryFull>>>,
    connected: bool,
}
```

### 关键能力

| 功能 | 描述 |
|------|------|
| **双向流** | 与 Claude 实时双向通信 |
| **会话管理** | 多个独立的对话上下文 |
| **动态控制** | 中断操作、更改权限、执行中切换模型 |
| **钩子支持** | 注册 PreToolUse、PostToolUse 等钩子 |
| **多模态输入** | 在流模式下发送图像和文本 |
| **文件检查点** | 将跟踪的文件回滚到之前的状态 |

## 连接生命周期

### 创建客户端

提供两种构造方法：

```rust,no_run
use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions};

// 简单创建（延迟验证）
let client = ClaudeClient::new(ClaudeAgentOptions::default());

// 早期验证（立即捕获配置错误）
let client = ClaudeClient::try_new(ClaudeAgentOptions::default())?;
```

**何时使用 `try_new()`：**
- 在异步上下文之前验证配置
- 早期检测无效的工作目录
- 在 `connect()` 之前发现缺失的 CLI

### 连接流程

```
┌─────────────────┐
│   ClaudeClient  │
│    new/try_new  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    connect()    │  ◄── 启动 CLI 子进程
└────────┬────────┘
         │        ├── 创建 SubprocessTransport
         │        ├── 初始化 QueryFull
         │        ├── 启动后台读取器
         │        └── 发送初始化请求
         ▼
┌─────────────────┐
│    Connected    │  ◄── 准备就绪，可接收查询
│     State       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   disconnect()  │  ◄── 优雅关闭
└─────────────────┘
```

### 完整连接示例

```rust,no_run
use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());

    // 建立连接
    client.connect().await?;

    // 使用客户端...

    // 断开连接
    client.disconnect().await?;

    Ok(())
}
```

### Drop 警告

如果 `ClaudeClient` 在未调用 `disconnect()` 的情况下被 drop，将打印警告：

```
Warning: ClaudeClient dropped without calling disconnect(). Resources may not be cleaned up properly.
```

**最佳实践：** 在客户端超出作用域之前始终调用 `disconnect()`。

## 查询方法

### 2.1 `query()` - 简单文本查询

```rust
pub async fn query(&mut self, prompt: impl Into<String>) -> Result<()>
```

使用默认会话（"default"）发送文本提示。

**示例：**
```rust,no_run
client.query("法国的首都是什么？").await?;
```

### 2.2 `query_with_session()` - 指定会话查询

```rust
pub async fn query_with_session(
    &mut self,
    prompt: impl Into<String>,
    session_id: impl Into<String>,
) -> Result<()>
```

向特定对话上下文发送查询。不同会话维护独立的对话历史。

**示例：**
```rust,no_run
// 两个独立的对话
client.query_with_session("关于 Python", "python-session").await?;
client.query_with_session("关于 Rust", "rust-session").await?;
```

### 2.3 `query_with_content()` - 多模态查询

```rust
pub async fn query_with_content(
    &mut self,
    content: impl Into<Vec<UserContentBlock>>,
) -> Result<()>
```

发送包含文本和图像的结构化内容，用于视觉相关任务。

**示例：**
```rust,no_run
use claude_agent_sdk::UserContentBlock;

let base64_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";

client.query_with_content(vec![
    UserContentBlock::text("这张图片里有什么？"),
    UserContentBlock::image_base64("image/png", base64_data)?,
]).await?;
```

### 2.4 `query_with_content_and_session()` - 完全控制

```rust
pub async fn query_with_content_and_session(
    &mut self,
    content: impl Into<Vec<UserContentBlock>>,
    session_id: impl Into<String>,
) -> Result<()>
```

结合多模态输入与会话管理。

**示例：**
```rust,no_run
use claude_agent_sdk::UserContentBlock;

client.query_with_content_and_session(
    vec![
        UserContentBlock::text("分析这个图表"),
        UserContentBlock::image_url("https://example.com/chart.png"),
    ],
    "analysis-session",
).await?;
```

## 响应流

### 2.5 `receive_messages()` - 持续流

```rust
pub fn receive_messages(&self) -> Pin<Box<dyn Stream<Item = Result<Message>> + Send + '_>>
```

返回一个无限期 yield 所有消息的流，直到连接关闭。

**使用场景：**
- 处理多轮对话
- 处理系统事件
- 长时间运行的会话

**示例：**
```rust,no_run
use futures::StreamExt;

let mut stream = client.receive_messages();
while let Some(message) = stream.next().await {
    match message? {
        Message::Assistant(msg) => println!("助手: {:?}", msg),
        Message::System(sys) => println!("系统: {:?}", sys),
        Message::Result(result) => println!("完成: ${:?}", result.total_cost_usd),
        _ => {}
    }
}
```

### 2.6 `receive_response()` - 单轮流

```rust
pub fn receive_response(&self) -> Pin<Box<dyn Stream<Item = Result<Message>> + Send + '_>>
```

返回一个流，yield 消息直到收到 `ResultMessage`，表示一轮对话完成。

**使用场景：**
- 逐个处理查询
- 等待完整响应
- 每轮成本追踪

**示例：**
```rust,no_run
use futures::StreamExt;

client.query("你好 Claude！").await?;

let mut stream = client.receive_response();
while let Some(message) = stream.next().await {
    match message? {
        Message::Assistant(msg) => {
            println!("收到助手响应");
        }
        Message::Result(result) => {
            println!("本轮完成！成本: ${:.4}", result.total_cost_usd.unwrap_or(0.0));
            break;
        }
        _ => {}
    }
}
```

### 重要：流所有权

流会不可变地借用客户端。在调用查询方法之前必须 drop 流：

```rust,no_run
// 正确：流在下一个查询之前超出作用域
{
    let mut stream = client.receive_response();
    while let Some(msg) = stream.next().await {
        // 处理消息
    }
} // 流在此处被 drop

client.query("下一个问题").await?; // 现在可以了
```

## 动态控制

### 2.7 `interrupt()` - 停止当前操作

```rust
pub async fn interrupt(&self) -> Result<()>
```

发送中断信号立即停止当前 Claude 操作。

**使用场景：**
- 用户取消请求
- 超时
- 需要停止长时间运行的操作

**示例：**
```rust,no_run
use tokio::time::{timeout, Duration};

// 30秒后中断
match timeout(Duration::from_secs(30), async {
    client.query("复杂分析...").await?;
    // 处理响应...
    Ok::<_, claude_agent_sdk::ClaudeError>(())
})
.await
{
    Ok(result) => result?,
    Err(_) => {
        println!("超时，正在中断...");
        client.interrupt().await?;
    }
}
```

### 2.8 `set_permission_mode()` - 更改权限

```rust
pub async fn set_permission_mode(&self, mode: PermissionMode) -> Result<()>
```

在活动会话期间动态更改权限模式。

**权限模式：**
| 模式 | 行为 |
|------|------|
| `Default` | 标准权限提示 |
| `AcceptEdits` | 自动接受文件编辑 |
| `Plan` | 计划模式 |
| `BypassPermissions` | 无提示（谨慎使用） |

**示例：**
```rust,no_run
use claude_agent_sdk::PermissionMode;

// 以默认权限开始
client.query("读取我的文件").await?;

// 切换到自动接受编辑
client.set_permission_mode(PermissionMode::AcceptEdits).await?;
client.query("重构所有文件").await?;

// 返回安全模式
client.set_permission_mode(PermissionMode::Default).await?;
```

### 2.9 `set_model()` - 切换 AI 模型

```rust
pub async fn set_model(&self, model: Option<&str>) -> Result<()>
```

在会话中切换 AI 模型。传递 `None` 使用默认模型。

**示例：**
```rust,no_run
// 对简单任务使用快速模型
client.set_model(Some("claude-sonnet-4")).await?;
client.query("快速摘要").await?;

// 对复杂分析切换到强大模型
client.set_model(Some("claude-opus-4")).await?;
client.query("深度分析这个代码库").await?;

// 重置为默认
client.set_model(None).await?;
```

### 2.10 `rewind_files()` - 文件检查点

```rust
pub async fn rewind_files(&self, user_message_id: &str) -> Result<()>
```

将跟踪的文件回滚到特定用户消息检查点时的状态。

**要求：**
- 在选项中启用 `enable_file_checkpointing(true)`
- 添加 `extra_args={"replay-user-messages": None}` 以接收带有 UUID 的 `UserMessage`

**示例：**
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

// 进行更改并捕获检查点
client.query("创建一个新模块").await?;
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

// 进行更多更改
client.query("添加更多功能").await?;

// 稍后，撤销更改回到检查点
if let Some(id) = checkpoint_id {
    client.rewind_files(&id).await?;
}
```

## 会话管理

### 2.11 `new_session()` - 启动新会话

```rust
pub async fn new_session(
    &mut self,
    session_id: impl Into<String>,
    prompt: impl Into<String>,
) -> Result<()>
```

启动新对话上下文的便捷方法。

**示例：**
```rust,no_run
// 第一个对话
client.query("关于 Python").await?;

// 开始全新的对话
client.new_session("new-topic", "关于 Rust").await?;
```

### 2.12 `get_server_info()` - 服务器能力

```rust
pub async fn get_server_info(&self) -> Option<serde_json::Value>
```

返回来自 Claude Code CLI 的初始化信息，包括可用命令和输出样式。

**示例：**
```rust,no_run
if let Some(info) = client.get_server_info().await {
    if let Some(commands) = info.get("commands").and_then(|c| c.as_array()) {
        println!("可用命令数: {}", commands.len());
    }
    if let Some(style) = info.get("output_style") {
        println!("输出样式: {:?}", style);
    }
}
```

## 内部架构

### 通信流程

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

### 锁管理

客户端仔细管理锁以避免死锁：

1. **直接 stdin 访问**：单独存储 `stdin` 以绕过传输锁
2. **Arc<Mutex>** 模式：允许并发读/写操作
3. **Guard 丢弃**：在获取新锁之前显式 drop guards

```rust
// 内部使用的锁获取模式
let query_guard = query.lock().await;
let stdin = query_guard.stdin.clone();
drop(query_guard);  // 在下一个锁之前释放

if let Some(stdin_arc) = stdin {
    let mut stdin_guard = stdin_arc.lock().await;
    // 使用 stdin...
}
```

## 与 Python SDK 对比

| 功能 | Python (`ClaudeSDKClient`) | Rust (`ClaudeClient`) |
|------|---------------------------|----------------------|
| 连接 | `async with client:` | `client.connect().await` |
| 断开 | `__aexit__` | `client.disconnect().await` |
| 查询 | `client.query(prompt)` | `client.query(prompt).await` |
| 流 | `async for msg in client:` | `client.receive_response().next().await` |
| 中断 | `client.interrupt()` | `client.interrupt().await` |
| 设置模型 | `client.set_model(model)` | `client.set_model(model).await` |
| 设置权限 | `client.set_permission_mode(mode)` | `client.set_permission_mode(mode).await` |
| 回滚文件 | `client.rewind_files(id)` | `client.rewind_files(id).await` |

## 最佳实践

### 1. 始终断开连接

```rust,no_run
// 使用 RAII 模式包装
struct ManagedClient {
    client: ClaudeClient,
}

impl Drop for ManagedClient {
    fn drop(&mut self) {
        // 注意：不能在 Drop 中 await，使用 tokio::task::block_in_place
        // 或设计代码显式调用 disconnect()
    }
}
```

### 2. 处理流生命周期

```rust,no_run
// 处理响应，然后 drop 流
{
    let mut stream = client.receive_response();
    while let Some(msg) = stream.next().await {
        // 处理消息
    }
} // 流被 drop，客户端再次可用

// 现在可以安全发送下一个查询
client.query("下一个问题").await?;
```

### 3. 使用会话 ID 分离上下文

```rust,no_run
// 隔离不同任务
client.query_with_session("项目 A 问题", "project-a").await?;
client.query_with_session("项目 B 问题", "project-b").await?;
```

### 4. 为破坏性操作启用检查点

```rust,no_run
let options = ClaudeAgentOptions::builder()
    .enable_file_checkpointing(true)
    .extra_args(HashMap::from([("replay-user-messages".to_string(), None)]))
    .build();
```

## 相关章节

- **第1章**：入门与核心 API - SDK 概述和一次性查询
- **第3章**：V2 Session API - 简化的 TypeScript 风格接口
- **第5章**：MCP 集成 - 自定义工具和 MCP 服务器
- **第9章**：内部层 - Transport 和 QueryFull 实现
