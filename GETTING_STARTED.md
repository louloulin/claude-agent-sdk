# Getting Started with Claude Agent SDK Rust

**欢迎使用 Claude Agent SDK Rust！**

本教程将引导您快速上手使用 Rust SDK 与 Claude AI 进行交互。无论您是构建 AI 应用、自动化工具，还是探索 AI 的可能性，这个教程都将帮助您快速入门。

---

## 📋 目录

1. [前置要求](#前置要求)
2. [安装](#安装)
3. [快速开始](#快速开始)
4. [基础概念](#基础概念)
5. [常见使用场景](#常见使用场景)
6. [下一步](#下一步)
7. [故障排除](#故障排除)

---

## 前置要求

在开始之前，请确保您的系统已安装以下软件：

### 必需项

- **Rust**: 1.85 或更高版本
  ```bash
  rustc --version
  ```

- **Claude CLI**: Claude Code CLI 工具
  ```bash
  claude --version
  ```

### 可选项

- **Cargo**: Rust 包管理器（通常随 Rust 一起安装）
- **Git**: 版本控制（用于克隆示例代码）

---

## 安装

### 1. 创建新项目

使用 Cargo 创建一个新的 Rust 项目：

```bash
cargo new my-claude-app
cd my-claude-app
```

### 2. 添加依赖

在 `Cargo.toml` 文件中添加 Claude Agent SDK 依赖：

```toml
[dependencies]
claude-agent-sdk-rs = "0.6"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"  # 可选，用于错误处理
```

### 3. 安装依赖

运行以下命令下载并构建依赖：

```bash
cargo build
```

---

## 快速开始

### 您的第一个 Claude 查询

让我们从最简单的示例开始 - 向 Claude 提问并获取回答。

编辑 `src/main.rs`：

```rust
use claude_agent_sdk_rs::query;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 向 Claude 提问
    let messages = query("What is 2 + 2?", None).await?;

    // 处理响应
    for message in messages {
        if let claude_agent_sdk_rs::Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let claude_agent_sdk_rs::ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
    }

    Ok(())
}
```

运行程序：

```bash
cargo run
```

**预期输出**：
```
Claude: 2 + 2 equals 4.
```

🎉 **恭喜！** 您已经成功完成了第一个 Claude 查询！

---

## 基础概念

### 1. Simple Query API

`query()` 函数是最简单的交互方式，适合一次性查询：

```rust
use claude_agent_sdk_rs::query;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let messages = query("Explain quantum computing in simple terms", None).await?;

    // 处理响应...
    Ok(())
}
```

### 2. 流式响应

对于长响应，使用 `query_stream()` 可以实时获取消息：

```rust
use claude_agent_sdk_rs::query_stream;
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut stream = query_stream("List 10 programming tips", None).await?;

    while let Some(result) = stream.next().await {
        match result? {
            claude_agent_sdk_rs::Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let claude_agent_sdk_rs::ContentBlock::Text(text) = block {
                        print!("{}", text.text);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
```

### 3. 双向通信

使用 `ClaudeClient` 进行多轮对话：

```rust
use claude_agent_sdk_rs::ClaudeClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ClaudeClient::new(
        vec!["Remember the number 42".to_string()],
        None,
    )?;

    // 第一轮对话
    let _messages = client.execute().await?;

    // 后续对话
    let messages = client.query("What number did I mention?", None).await?;

    // 处理响应...
    Ok(())
}
```

---

## 常见使用场景

### 场景 1: 代码生成

让 Claude 帮您生成代码：

```rust
use claude_agent_sdk_rs::query;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let prompt = r#"
Write a Rust function to calculate fibonacci numbers.
Include error handling and documentation.
"#;

    let messages = query(prompt, None).await?;

    for message in messages {
        if let claude_agent_sdk_rs::Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let claude_agent_sdk_rs::ContentBlock::Text(text) = block {
                    println!("{}", text.text);
                }
            }
        }
    }

    Ok(())
}
```

### 场景 2: 文本分析

分析文本内容：

```rust
use claude_agent_sdk_rs::query;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let text = "Rust is a systems programming language that runs blazingly fast...";

    let prompt = format!(
        "Analyze the following text and summarize its key points:\n\n{}",
        text
    );

    let messages = query(&prompt, None).await?;

    // 处理响应...
    Ok(())
}
```

### 场景 3: 带图片的多模态查询

发送图片给 Claude 分析：

```rust
use claude_agent_sdk_rs::{query_with_content, UserContentBlock};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let content = vec![
        UserContentBlock::text("What's in this image?"),
        UserContentBlock::image_url("https://example.com/diagram.png"),
    ];

    let messages = query_with_content(content, None).await?;

    // 处理响应...
    Ok(())
}
```

### 场景 4: 自定义配置

使用自定义配置控制 Claude 的行为：

```rust
use claude_agent_sdk_rs::{query, ClaudeAgentOptions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions {
        max_tokens: Some(1000),           // 限制响应长度
        temperature: Some(0.7),           // 控制创造性 (0.0-1.0)
        model: Some("claude-3-5-sonnet".to_string()),  // 使用特定模型
        ..Default::default()
    };

    let messages = query("Write a short poem about Rust", Some(options)).await?;

    // 处理响应...
    Ok(())
}
```

---

## 配置选项

### 常用配置项

```rust
use claude_agent_sdk_rs::ClaudeAgentOptions;

let options = ClaudeAgentOptions {
    // 模型选择
    model: Some("claude-3-5-sonnet-20241022".to_string()),

    // 响应控制
    max_tokens: Some(4096),
    temperature: Some(0.5),

    // 会话控制
    max_turns: Some(5),

    // 权限管理
    permission_mode: Some(PermissionMode::Default),
    allowed_tools: Some(vec!["Bash".to_string()]),

    // 工作目录
    cwd: Some("/path/to/working/directory".into()),

    ..Default::default()
};
```

### Builder 模式

使用 Builder 模式更简洁地构建配置：

```rust
use claude_agent_sdk_rs::ClaudeAgentOptions;

let options = ClaudeAgentOptions::builder()
    .model("claude-3-5-sonnet-20241022")
    .max_tokens(4096)
    .temperature(0.5)
    .max_turns(5)
    .build();
```

---

## 错误处理

### 基础错误处理

```rust
use claude_agent_sdk_rs::query;

#[tokio::main]
async fn main() {
    match query("What is 2 + 2?", None).await {
        Ok(messages) => {
            println!("Query successful!");
            // 处理消息...
        }
        Err(e) => {
            eprintln!("Query failed: {}", e);
            // 处理错误...
        }
    }
}
```

### 使用 anyhow 进行错误处理

```rust
use claude_agent_sdk_rs::query;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let messages = query("What is 2 + 2?", None).await?;

    // anyhow 自动将错误转换为 Result
    // 处理消息...

    Ok(())
}
```

### 常见错误类型

| 错误类型 | 描述 | 解决方法 |
|---------|------|----------|
| `CliNotFoundError` | Claude CLI 未安装 | 安装 Claude CLI |
| `ConnectionError` | 无法连接到 Claude CLI | 检查 CLI 是否正常运行 |
| `InvalidConfig` | 配置无效 | 检查配置参数 |
| `ProcessError` | 子进程错误 | 查看 Claude CLI 日志 |

---

## 高级功能预览

### Hooks 系统

在特定事件上执行自定义逻辑：

```rust
use claude_agent_sdk_rs::{ClaudeAgentOptions, Hooks};
use std::collections::HashMap;

let hooks = Hooks::new()
    .on_pre_tool_use(Box::new(|event| {
        println!("Tool about to be used: {:?}", event);
        Box::pin(async { Ok(()) })
    }));

let options = ClaudeAgentOptions {
    hooks: Some(hooks),
    ..Default::default()
};
```

### 自定义工具（MCP）

创建自定义工具供 Claude 使用：

```rust
use claude_agent_sdk_rs::tool;

tool!(MyCalculator {
    description: "A simple calculator",
    parameters: {
        operation: String,
        a: f64,
        b: f64
    },
    execute: |input| async move {
        let result = match input.operation.as_str() {
            "add" => input.a + input.b,
            "multiply" => input.a * input.b,
            _ => return Err(anyhow::anyhow!("Unknown operation")),
        };
        Ok(serde_json::json!({ "result": result }).to_string())
    }
});
```

### Agent Skills 系统

管理和组织可重用的 AI 技能：

```rust
use claude_agent_sdk_rs::skills::{Skill, SkillRegistry};

let skill = Skill::new("code_review")
    .description("Reviews code for best practices")
    .author("Your Name")
    .version("1.0.0");

let mut registry = SkillRegistry::new();
registry.register_skill(skill)?;
```

---

## 性能优化技巧

### 1. 使用流式处理

对于长响应，流式处理可以减少内存使用：

```rust
// ❌ 不推荐：一次性加载所有响应
let messages = query("Generate 100 items...", None).await?;

// ✅ 推荐：流式处理
let mut stream = query_stream("Generate 100 items...", None).await?;
while let Some(result) = stream.next().await {
    // 实时处理每条消息
}
```

### 2. 并发查询

使用 Tokio 并发执行多个查询：

```rust
use futures::future::join_all;

let prompts = vec
!["What is 2 + 2?", "What is 3 + 3?", "What is 4 + 4?"];

let futures = prompts.into_iter()
    .map(|p| query(p, None));

let results = join_all(futures).await?;
```

### 3. 复用客户端

复用 `ClaudeClient` 而不是创建新实例：

```rust
// ❌ 不推荐：每次查询创建新客户端
let messages1 = query("Query 1", None).await?;
let messages2 = query("Query 2", None).await?;

// ✅ 推荐：复用客户端
let client = ClaudeClient::new(vec
!["Initial context".to_string()], None)?;
let messages1 = client.query("Query 1", None).await?;
let messages2 = client.query("Query 2", None).await?;
```

---

## 下一步

### 📚 继续学习

1. **最佳实践指南**: 阅读 [BEST_PRACTICES.md](BEST_PRACTICES.md)
2. **架构文档**: 了解 [ARCHITECTURE.md](ARCHITECTURE.md)
3. **示例程序**: 查看 `examples/` 目录中的 50+ 个示例
4. **API 文档**: 运行 `cargo doc --open` 查看完整 API 文档

### 🎯 实践项目

尝试构建这些小项目来练习：

- **代码审查工具**: 使用 Claude 审查代码
- **文档生成器**: 自动生成项目文档
- **聊天机器人**: 构建对话式 AI 应用
- **数据分析助手**: 分析和解释数据

### 🌟 高级主题

- Agent Skills 系统
- MCP (Model Context Protocol) 集成
- 自定义工具和插件
- Hooks 事件系统
- 多模态输入（图片）
- WASM 浏览器部署

---

## 故障排除

### 问题: "Claude CLI not found"

**错误信息**:
```
Error: Claude CLI not found
```

**解决方案**:
1. 确认 Claude CLI 已安装: `claude --version`
2. 如果未安装，按照官方指南安装 Claude CLI
3. 确保 `claude` 命令在 PATH 中

### 问题: "Permission denied"

**错误信息**:
```
Error: Permission denied when executing Claude CLI
```

**解决方案**:
1. 检查 Claude CLI 的执行权限
2. 确保 API 密钥已正确配置
3. 检查用户权限设置

### 问题: "Connection timeout"

**错误信息**:
```
Error: Connection timeout
```

**解决方案**:
1. 检查网络连接
2. 确认 Anthropic API 服务正常运行
3. 增加超时时间配置

### 问题: 查询响应很慢

**可能原因**:
- 网络延迟
- API 服务负载高
- 查询过于复杂

**解决方案**:
1. 使用 `max_tokens` 限制响应长度
2. 简化查询内容
3. 使用流式响应提升用户体验

### 问题: 内存使用过高

**解决方案**:
```rust
// 使用流式处理而非一次性加载
let mut stream = query_stream(prompt, None).await?;
while let Some(result) = stream.next().await {
    // 逐条处理，不缓存所有消息
}
```

---

## 获取帮助

### 📖 文档资源

- **GitHub 仓库**: [https://github.com/tyrchen/claude-agent-sdk-rs](https://github.com/tyrchen/claude-agent-sdk-rs)
- **API 文档**: 运行 `cargo doc --open`
- **示例代码**: `examples/` 目录

### 💬 社区支持

- **GitHub Issues**: 报告 bug 和功能请求
- **Discussions**: 技术讨论和问答

### 🐛 报告问题

遇到 bug？请在 GitHub 上创建 issue，包含：

1. Rust 和 SDK 版本
2. 完整的错误信息
3. 最小可复现示例
4. 预期行为 vs 实际行为

---

## 总结

恭喜您完成 Getting Started 教程！您已经学会：

✅ 安装和配置 Claude Agent SDK Rust
✅ 使用基础查询 API
✅ 实现流式响应
✅ 进行多轮对话
✅ 自定义配置选项
✅ 处理错误
✅ 优化性能

**下一步**:
- 🎯 尝试构建您的第一个 AI 应用
- 📖 阅读 [BEST_PRACTICES.md](BEST_PRACTICES.md) 了解最佳实践
- 🏗️ 查看 [ARCHITECTURE.md](ARCHITECTURE.md) 理解系统架构
- 💡 探索 `examples/` 目录中的 50+ 个示例程序

**祝您使用 Claude Agent SDK Rust 构建出色的 AI 应用！** 🚀

---

**文档版本**: v0.6.0
**最后更新**: 2026-01-08
**维护者**: Claude Agent SDK Rust Team
