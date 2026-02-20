# 第1章：入门指南与核心API

本章介绍 Claude Agent SDK for Rust，涵盖 SDK 概述、架构设计、快速入门示例以及单次查询 API。

## 概述

Claude Agent SDK for Rust 提供了对 Claude Code CLI 的编程访问能力，支持完整的双向流式通信，并与官方 Python SDK 保持 100% 功能对等。

### 核心特性

| 特性 | 说明 |
|------|------|
| **简洁查询 API** | 使用 `query()` 和 `query_stream()` 进行单次查询 |
| **双向流式通信** | 通过 `ClaudeClient` 实现实时流式通信 |
| **动态控制** | 执行过程中可中断、修改权限、切换模型 |
| **钩子系统** | 6 种钩子类型，可拦截和控制 Claude 的行为 |
| **自定义工具** | 使用便捷的 `tool!` 宏创建进程内 MCP 服务器 |
| **插件系统** | 加载自定义插件以扩展 Claude 的能力 |
| **权限管理** | 对工具执行进行细粒度控制 |
| **成本控制** | 生产环境下的预算限制和备用模型 |
| **扩展思考** | 配置复杂推理的最大思考令牌数 |
| **会话管理** | 恢复、分叉和管理对话会话 |
| **多模态输入** | 使用 base64 或 URL 发送图像与文本 |

## 架构设计

SDK 采用分层架构设计：

```
┌─────────────────────────────────────────────────────────────┐
│                     应用层 (Application Layer)               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │   V2 API     │  │   Query API  │  │  ClaudeClient    │   │
│  │  (简化接口)   │  │  (单次查询)   │  │  (双向流式)       │   │
│  └──────────────┘  └──────────────┘  └──────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    功能层 (Feature Layer)                    │
│  ┌──────────┐ ┌───────────┐ ┌───────────┐ ┌─────────────┐   │
│  │  Skills  │ │  Agents   │ │Subagents  │ │  Commands   │   │
│  │  技能系统 │ │  代理     │ │ 子代理    │ │   命令      │   │
│  └──────────┘ └───────────┘ └───────────┘ └─────────────┘   │
│  ┌──────────┐ ┌───────────┐ ┌───────────┐ ┌─────────────┐   │
│  │   MCP    │ │   Todos   │ │Orchestration│Observability│  │
│  │ 工具协议  │ │  待办事项  │ │  编排系统  │ │  可观测性   │   │
│  └──────────┘ └───────────┘ └───────────┘ └─────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    核心层 (Core Layer)                       │
│  ┌──────────────────┐  ┌──────────────────────────────┐     │
│  │  Internal Client │  │    Transport (Subprocess)    │     │
│  │    内部客户端     │  │      传输层 (子进程)          │     │
│  └──────────────────┘  └──────────────────────────────┘     │
│  ┌──────────────────┐  ┌──────────────────────────────┐     │
│  │  Types & Config  │  │      Error Handling          │     │
│  │  类型与配置       │  │        错误处理              │     │
│  └──────────────────┘  └──────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

### 模块结构

```
crates/claude-agent-sdk/src/
├── lib.rs          # 核心 SDK 入口点，重导出
├── client.rs       # ClaudeClient 实现
├── query.rs        # 单次查询 API
├── errors.rs       # 错误类型
├── version.rs      # 版本信息
├── v2/             # V2 API（简化版，TypeScript 风格）
├── types/          # 核心类型定义
│   ├── config.rs   # 配置选项
│   ├── messages.rs # 消息类型
│   ├── hooks.rs    # 钩子系统
│   ├── mcp.rs      # MCP 类型
│   ├── permissions.rs
│   └── plugin.rs
├── skills/         # 技能系统
├── orchestration/  # 代理编排
├── subagents/      # 子代理支持
├── internal/       # 内部实现
│   ├── client.rs   # 内部客户端
│   ├── transport/  # 子进程传输
│   └── ...
├── observability/  # 日志和指标
├── commands/       # 斜杠命令
├── mcp/            # MCP 集成
├── partnership/    # 合作功能
└── todos/          # 待办事项支持
```

## 安装

添加到你的 `Cargo.toml`：

```toml
[dependencies]
claude-agent-sdk = "0.1"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

## 快速入门

### 前置条件

1. 安装 Claude Code CLI（或启用自动安装功能）
2. 设置你的 Anthropic API 密钥：`ANTHROPIC_API_KEY=your_key`

### 简单查询

与 Claude 交互的最简单方式：

```rust,no_run
use claude_agent_sdk::{query, Message, ContentBlock};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 单次查询，收集所有消息
    let messages = query("2 + 2 等于多少？", None).await?;

    for message in messages {
        if let Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
    }

    Ok(())
}
```

### 流式查询

用于高效处理大响应的内存优化方式：

```rust,no_run
use claude_agent_sdk::{query_stream, Message, ContentBlock};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 流式查询，内存高效处理
    let mut stream = query_stream("解释 Rust 的所有权机制", None).await?;

    while let Some(result) = stream.next().await {
        let message = result?;
        if let Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
    }

    Ok(())
}
```

### 双向客户端

用于交互式、多轮对话：

```rust,no_run
use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions, Message, PermissionMode};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .max_turns(5)
        .build();

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    // 发送查询
    client.query("什么是 Rust？").await?;

    // 接收响应
    {
        let mut stream = client.receive_response();
        while let Some(result) = stream.next().await {
            match result? {
                Message::Assistant(msg) => {
                    println!("收到助手消息");
                }
                Message::Result(_) => break,
                _ => {}
            }
        }
    } // 流在此处被丢弃

    client.disconnect().await?;
    Ok(())
}
```

## Query API 参考

### 1.1 `query()` - 收集式查询

```rust
pub async fn query(
    prompt: impl Into<String>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Vec<Message>>
```

发送单次查询并在返回前将所有消息收集到内存中。

**适用场景：**
- 需要一次性获取所有消息进行分析
- 响应大小可控
- 优先考虑简单性而非内存效率

**参数：**
| 参数 | 类型 | 说明 |
|------|------|------|
| `prompt` | `impl Into<String>` | 提示文本 |
| `options` | `Option<ClaudeAgentOptions>` | 可选配置 |

**返回值：** `Result<Vec<Message>>` - 对话中的所有消息

**示例：**
```rust,no_run
use claude_agent_sdk::{query, Message, ContentBlock};

let messages = query("2 + 2 等于多少？", None).await?;

for message in messages {
    match message {
        Message::Assistant(msg) => {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
        Message::Result(result) => {
            println!("费用: ${:.4}", result.total_cost_usd.unwrap_or(0.0));
        }
        _ => {}
    }
}
```

### 1.2 `query_stream()` - 流式查询

```rust
pub async fn query_stream(
    prompt: impl Into<String>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>>
```

返回一个流，在消息从 Claude 到达时逐个产生。

**适用场景：**
- 处理大型响应
- 需要实时显示消息
- 内存效率重要

**性能对比：**
| 函数 | 内存使用 | 处理方式 |
|------|----------|----------|
| `query()` | O(n) - 存储所有消息 | 等待完成 |
| `query_stream()` | O(1) 每条消息 | 实时 |

**示例：**
```rust,no_run
use claude_agent_sdk::{query_stream, Message, ContentBlock};
use futures::stream::StreamExt;

let mut stream = query_stream("2 + 2 等于多少？", None).await?;

while let Some(result) = stream.next().await {
    match result? {
        Message::Assistant(msg) => {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
        _ => {}
    }
}
```

### 1.3 `query_with_content()` - 多模态查询

```rust
pub async fn query_with_content(
    content: impl Into<Vec<UserContentBlock>>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Vec<Message>>
```

发送包含文本和图像的混合内容。

**支持的图像格式：**
- JPEG (`image/jpeg`)
- PNG (`image/png`)
- GIF (`image/gif`)
- WebP (`image/webp`)

**大小限制：**
- 最大 base64 数据大小：15MB

**图像示例：**
```rust,no_run
use claude_agent_sdk::{query_with_content, UserContentBlock, Message, ContentBlock};

// 加载并编码你的图像（此示例使用 1x1 红色像素）
let base64_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";

let messages = query_with_content(vec![
    UserContentBlock::text("这张图片是什么颜色？"),
    UserContentBlock::image_base64("image/png", base64_data)?,
], None).await?;

for message in messages {
    if let Message::Assistant(msg) = message {
        for block in &msg.message.content {
            if let ContentBlock::Text(text) = block {
                println!("Claude: {}", text.text);
            }
        }
    }
}
```

**图像 URL 示例：**
```rust,no_run
use claude_agent_sdk::{query_with_content, UserContentBlock};

let messages = query_with_content(vec![
    UserContentBlock::text("描述这个架构图"),
    UserContentBlock::image_url("https://example.com/diagram.png"),
], None).await?;
```

### 1.4 `query_stream_with_content()` - 流式多模态

```rust
pub async fn query_stream_with_content(
    content: impl Into<Vec<UserContentBlock>>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>>
```

结合流式处理优势与多模态输入支持。

**示例：**
```rust,no_run
use claude_agent_sdk::{query_stream_with_content, UserContentBlock, Message, ContentBlock};
use futures::stream::StreamExt;

let png_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

let mut stream = query_stream_with_content(vec![
    UserContentBlock::image_base64("image/png", png_base64)?,
    UserContentBlock::text("这张图片里有什么？"),
], None).await?;

while let Some(result) = stream.next().await {
    let message = result?;
    if let Message::Assistant(msg) = message {
        for block in &msg.message.content {
            if let ContentBlock::Text(text) = block {
                print!("{}", text.text);
            }
        }
    }
}
```

## 配置

### ClaudeAgentOptions

使用构建器模式的主配置结构：

```rust,no_run
use claude_agent_sdk::{ClaudeAgentOptions, PermissionMode, SdkPluginConfig};

let options = ClaudeAgentOptions::builder()
    // 模型选择
    .model("claude-opus-4")
    .fallback_model("claude-sonnet-4")

    // 成本控制
    .max_budget_usd(10.0)
    .max_thinking_tokens(2000)
    .max_turns(10)

    // 权限
    .permission_mode(PermissionMode::Default)

    // 插件
    .plugins(vec![SdkPluginConfig::local("./my-plugin")])

    .build();
```

### 主要配置选项

| 选项 | 类型 | 说明 |
|------|------|------|
| `model` | `Option<String>` | 主模型（如 "claude-sonnet-4"、"claude-opus-4"） |
| `fallback_model` | `Option<String>` | 主模型失败时的备用模型 |
| `max_budget_usd` | `Option<f64>` | 预算限制（美元） |
| `max_thinking_tokens` | `Option<u32>` | 扩展思考的最大令牌数 |
| `max_turns` | `Option<u32>` | 最大对话轮次 |
| `permission_mode` | `Option<PermissionMode>` | 权限处理模式 |
| `system_prompt` | `Option<SystemPrompt>` | 自定义系统提示 |
| `tools` | `Option<Tools>` | 工具配置 |
| `allowed_tools` | `Vec<String>` | 允许的工具名称列表 |
| `mcp_servers` | `McpServers` | MCP 服务器配置 |
| `cwd` | `Option<PathBuf>` | 工作目录 |
| `resume` | `Option<String>` | 要恢复的会话 ID |
| `auto_install_cli` | `bool` | 缺失时自动安装 CLI |

### 权限模式

```rust
pub enum PermissionMode {
    Default,           // 标准权限提示
    AcceptEdits,       // 自动接受文件编辑
    Plan,              // 计划模式
    BypassPermissions, // 无权限提示（谨慎使用）
}
```

## 错误处理

SDK 提供了完善的错误类型：

```rust
pub enum ClaudeError {
    Connection(ConnectionError),     // CLI 连接问题
    Process(ProcessError),           // 进程执行错误
    JsonDecode(JsonDecodeError),     // JSON 解析错误
    MessageParse(MessageParseError), // 消息格式错误
    Transport(String),               // 传输层错误
    ControlProtocol(String),         // 控制协议错误
    InvalidConfig(String),           // 配置错误
    CliNotFound(CliNotFoundError),   // CLI 未安装
    ImageValidation(ImageValidationError), // 图像验证错误
    Io(std::io::Error),              // IO 错误
    Other(anyhow::Error),            // 其他错误
    NotFound(String),                // 资源未找到
    InvalidInput(String),            // 无效输入
    InternalError(String),           // 内部 SDK 错误
}
```

### 错误处理示例

```rust,no_run
use claude_agent_sdk::{query, ClaudeError};

match query("你好", None).await {
    Ok(messages) => {
        // 处理消息
    }
    Err(ClaudeError::CliNotFound(e)) => {
        eprintln!("Claude CLI 未找到: {}", e.message);
        eprintln!("检查路径: {:?}", e.cli_path);
    }
    Err(ClaudeError::Connection(e)) => {
        eprintln!("连接失败: {}", e.message);
    }
    Err(ClaudeError::Process(e)) => {
        eprintln!("进程失败，退出码: {:?}", e.exit_code);
        eprintln!("标准错误: {:?}", e.stderr);
    }
    Err(e) => {
        eprintln!("错误: {}", e);
    }
}
```

## 消息类型

### Message 枚举

```rust
pub enum Message {
    Assistant(AssistantMessage), // Claude 的响应
    System(SystemMessage),       // 系统通知
    Result(ResultMessage),       // 查询完成
    StreamEvent(StreamEvent),    // 流事件
    User(UserMessage),           // 用户消息
}
```

### ContentBlock 类型

```rust
pub enum ContentBlock {
    Text(TextBlock),           // 文本内容
    Thinking(ThinkingBlock),   // 扩展思考
    ToolUse(ToolUseBlock),     // 工具调用
    ToolResult(ToolResultBlock), // 工具结果
    Image(ImageBlock),         // 图像内容
}
```

### 处理消息

```rust,no_run
use claude_agent_sdk::{query, Message, ContentBlock};

let messages = query("2 + 2 等于多少？", None).await?;

for message in messages {
    match message {
        Message::Assistant(msg) => {
            println!("会话: {:?}", msg.session_id);
            for block in &msg.message.content {
                match block {
                    ContentBlock::Text(text) => {
                        println!("文本: {}", text.text);
                    }
                    ContentBlock::Thinking(thinking) => {
                        println!("思考: {}", thinking.thinking);
                    }
                    ContentBlock::ToolUse(tool) => {
                        println!("工具: {} ({})", tool.name, tool.id);
                    }
                    _ => {}
                }
            }
        }
        Message::Result(result) => {
            println!("完成耗时 {}ms", result.duration_ms);
            println!("费用: ${:.4}", result.total_cost_usd.unwrap_or(0.0));
            println!("轮次: {}", result.num_turns);
        }
        Message::System(sys) => {
            println!("系统: {} - {:?}", sys.subtype, sys.session_id);
        }
        _ => {}
    }
}
```

## 相关章节

- **第2章**: ClaudeClient 深入解析 - 双向流式通信与动态控制
- **第3章**: V2 Session API - 简化的 TypeScript 风格接口
- **第4章**: Skills 技能系统 - 自定义能力
- **第5章**: MCP 集成 - 自定义工具
- **第8章**: 类型参考 - 完整类型文档
