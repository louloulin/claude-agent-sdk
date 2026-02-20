# 第11章：示例指南

本章提供了 Claude Agent SDK for Rust 中所有 70 多个示例的全面指南，按类别组织，包含使用模式和最佳实践。

## 概述

SDK 包含大量示例，演示了每个功能和常见使用模式。所有示例位于 `crates/claude-agent-sdk/examples/`，可以通过以下方式运行：

```bash
cargo run --example <example_name>
```

## 示例分类

| 类别 | 示例编号 | 描述 |
|----------|----------|-------------|
| 入门指南 | 01, 20, 56 | 基本使用模式 |
| 客户端与流式传输 | 06, 07, 14, 20, 54 | ClaudeClient 和流式处理 |
| 钩子与权限 | 04, 05, 15 | 钩子系统和权限管理 |
| 配置 | 13, 17-19, 21-22, 46 | 选项和设置 |
| 会话管理 | 16 | 多轮对话 |
| 多模态 | 23, 61 | 图像和混合内容 |
| 技能系统 | 30-42, 50, 55 | 技能系统功能 |
| MCP 与任务 | 08, 42 | MCP 集成 |
| 错误处理 | 43, 46, 53 | 错误处理模式 |
| 并发 | 44, 47, 48 | 并行处理 |
| 测试与生产 | 49-50 | 最佳实践 |
| V2 API | 56, 62, 65 | 简化 API |
| 编排系统 | 51, 52, 59, 60 | 多代理系统 |

---

## 1. 入门指南

### 示例 01: Hello World

最简单的示例，演示基本查询和文件创建。

```bash
cargo run --example 01_hello_world
```

**功能说明:**
1. 让 Claude 编写一个 Python hello world 脚本
2. 将其保存到 `./fixtures/hello.py`
3. 运行脚本验证其工作正常

```rust
use claude_agent_sdk::{ClaudeAgentOptions, ContentBlock, Message, query};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions::builder()
        .allowed_tools(vec!["Write".to_string()])
        .permission_mode(claude_agent_sdk::PermissionMode::AcceptEdits)
        .max_turns(5)
        .build();

    let messages = query(
        "Write a simple Python hello world script to ./fixtures/hello.py",
        Some(options),
    ).await?;

    for message in &messages {
        match message {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                    }
                }
            },
            Message::Result(result) => {
                println!("Duration: {}ms, Cost: ${:.4}",
                    result.duration_ms,
                    result.total_cost_usd.unwrap_or(0.0));
            },
            _ => {},
        }
    }

    Ok(())
}
```

### 示例 20: 流式查询 API

`query()` 的内存高效流式替代方案。

```bash
cargo run --example 20_query_stream
```

**与 `query()` 的关键区别:**

| 函数 | 内存使用 | 处理方式 |
|----------|-------------|------------|
| `query()` | O(n) - 存储所有消息 | 等待完成 |
| `query_stream()` | 每条消息 O(1) | 实时处理 |

```rust
use claude_agent_sdk::{query_stream, Message, ContentBlock};
use futures::stream::StreamExt;

let mut stream = query_stream("Explain Rust ownership", None).await?;

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
```

### 示例 56: V2 API - 简化接口

TypeScript 风格的简化 API，用于快速交互。

```bash
cargo run --example 56_v2_api
```

```rust
use claude_agent_sdk::v2::{prompt, create_session, SessionOptions};

// 一次性提示
let result = prompt("What is 2 + 2?", Default::default()).await?;
println!("Answer: {}", result.content);
println!("Tokens: {}", result.total_tokens());

// 基于会话的对话
let mut session = create_session(Default::default()).await?;
session.send("My favorite color is blue").await?;
let messages = session.receive().await?;
session.close().await?;
```

---

## 2. ClaudeClient 与双向流

### 示例 06: 双向客户端

具有上下文保持的多轮对话。

```bash
cargo run --example 06_bidirectional_client
```

```rust
use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions, ContentBlock, Message};
use futures::StreamExt;

let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
client.connect().await?;

// 第一次查询
client.query("What is your name?").await?;
let mut stream = client.receive_response();
while let Some(message) = stream.next().await {
    // 处理响应...
}
drop(stream); // 在下一次查询前释放借用

// 第二次查询 - Claude 记得上下文!
client.query("Can you remember what I just asked?").await?;
// ... 处理响应

client.disconnect().await?;
```

### 示例 07: 动态控制

Claude 行为的运行时控制。

```bash
cargo run --example 07_dynamic_control
```

**动态控制方法:**
- `interrupt()` - 停止当前操作
- `set_permission_mode(mode)` - 动态更改权限
- `set_model(model)` - 在会话中切换 AI 模型

```rust
// 更改权限模式
client.set_permission_mode(PermissionMode::AcceptEdits).await?;

// 切换模型
client.set_model(Some("claude-sonnet-4-20250514")).await?;

// 中断长时间运行的任务
client.interrupt().await?;
```

---

## 3. 钩子系统

### 示例 05: PreToolUse 钩子

在工具执行前阻止或允许执行。

```bash
cargo run --example 05_hooks_pretooluse
```

```rust
use claude_agent_sdk::{Hooks, HookInput, HookJsonOutput, SyncHookJsonOutput,
    HookSpecificOutput, PreToolUseHookSpecificOutput};

async fn block_dangerous_bash(
    input: HookInput,
    _tool_use_id: Option<String>,
    _context: HookContext,
) -> HookJsonOutput {
    match input {
        HookInput::PreToolUse(pre_tool) if pre_tool.tool_name == "Bash" => {
            let command = pre_tool.tool_input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if command.contains("rm -rf") {
                return HookJsonOutput::Sync(SyncHookJsonOutput {
                    hook_specific_output: Some(HookSpecificOutput::PreToolUse(
                        PreToolUseHookSpecificOutput {
                            permission_decision: Some("deny".to_string()),
                            permission_decision_reason: Some(
                                "Dangerous command blocked".to_string()
                            ),
                            ..Default::default()
                        }
                    )),
                    ..Default::default()
                });
            }
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        },
        _ => HookJsonOutput::Sync(SyncHookJsonOutput::default()),
    }
}

// 注册钩子
let mut hooks = Hooks::new();
hooks.add_pre_tool_use_with_matcher("Bash", block_dangerous_bash);

let options = ClaudeAgentOptions::builder()
    .hooks(hooks.build())
    .build();
```

### 示例 15: 综合钩子

所有钩子类型的详细示例。

```bash
cargo run --example 15_hooks_comprehensive all
cargo run --example 15_hooks_comprehensive PreToolUse
cargo run --example 15_hooks_comprehensive PostToolUse
cargo run --example 15_hooks_comprehensive UserPromptSubmit
```

**钩子类型:**

| 钩子 | 调用时机 | 用途 |
|------|-------------|----------|
| PreToolUse | 工具执行前 | 阻止/允许工具 |
| PostToolUse | 工具执行后 | 审查/修改输出 |
| UserPromptSubmit | 用户发送提示时 | 添加上下文 |

---

## 4. 配置

### 示例 17: 备用模型

自动故障转移到备用模型。

```bash
cargo run --example 17_fallback_model
```

```rust
let options = ClaudeAgentOptions::builder()
    .model("claude-opus-4-5")
    .fallback_model("claude-sonnet-4-5")
    .build();
```

### 示例 21: 自定义插件

加载自定义插件以扩展 Claude 的能力。

```bash
cargo run --example 21_custom_plugins
```

```rust
use claude_agent_sdk::SdkPluginConfig;

let options = ClaudeAgentOptions::builder()
    .plugins(vec![
        SdkPluginConfig::local("./plugins/database-tools"),
        SdkPluginConfig::local("~/.claude/plugins/company-tools"),
    ])
    .build();
```

### 示例 18 & 19: 预算和思考令牌

成本控制和扩展思考。

```bash
cargo run --example 18_max_budget_usd
cargo run --example 19_max_thinking_tokens
```

```rust
let options = ClaudeAgentOptions::builder()
    .max_budget_usd(10.0)           // 限制花费
    .max_thinking_tokens(2000)      // 扩展思考
    .max_turns(10)                  // 轮次限制
    .build();
```

---

## 5. 会话管理

### 示例 16: 会话管理

具有独立上下文的多轮对话。

```bash
cargo run --example 16_session_management
```

```rust
// 为不同上下文使用独立会话
client.query_with_session("Math question", "math-session").await?;
client.query_with_session("Programming question", "coding-session").await?;

// 分叉会话以获得全新开始
let options = ClaudeAgentOptions::builder()
    .fork_session(true)
    .build();

// 使用便捷方法创建新会话
client.new_session("new-id", "Starting fresh").await?;
```

---

## 6. 多模态输入

### 示例 23: 图像输入

与文本一起发送图像。

```bash
cargo run --example 23_image_input
```

```rust
use claude_agent_sdk::{query_with_content, UserContentBlock};

// Base64 编码的图像
let content = vec![
    UserContentBlock::text("What color is this image?"),
    UserContentBlock::image_base64("image/png", base64_data)?,
];

let messages = query_with_content(content, None).await?;

// 图像 URL
let content = vec![
    UserContentBlock::text("Describe this diagram"),
    UserContentBlock::image_url("https://example.com/diagram.png"),
];
```

**支持的格式:** JPEG, PNG, GIF, WebP

---

## 7. 技能系统

### 示例 30: 基本技能

创建模块化、可重用的 AI 能力。

```bash
cargo run --example 30_agent_skills
```

```rust
use claude_agent_sdk::skills::*;
use async_trait::async_trait;

struct FibonacciSkill;

#[async_trait]
impl Skill for FibonacciSkill {
    fn name(&self) -> String { "fibonacci".to_string() }
    fn description(&self) -> String { "Calculates Fibonacci numbers".to_string() }

    async fn execute(&self, _input: SkillInput) -> SkillResult {
        let result = fibonacci(10);
        Ok(SkillOutput::ok(serde_json::json!({"result": result})))
    }
}

// 注册并执行
let mut registry = SkillRegistry::new();
registry.register(Box::new(FibonacciSkill))?;
```

### 技能示例索引

| 示例 | 主题 |
|---------|-------|
| 30 | 基本技能创建 |
| 31 | 技能持久化 |
| 32 | 技能发现 |
| 33 | 资源处理 |
| 34 | 依赖管理 |
| 35 | 版本管理 |
| 36 | 标签系统 |
| 37 | YAML 配置 |
| 38 | 热重载 |
| 39 | 沙箱模式 |
| 40 | 性能优化 |
| 41 | VS Code 集成 |
| 42 | SKILL.md 文件 |

---

## 8. MCP 集成

### 示例 42: 异步任务 (MCP 2025-11-25)

立即调用、稍后获取的异步工作流。

```bash
cargo run --example 42_mcp_async_tasks
```

```rust
use claude_agent_sdk::mcp::tasks::{
    TaskManager, TaskRequest, TaskHint, TaskPriority, TaskProgress,
};

let manager = TaskManager::new();

// 使用提示创建任务
let request = TaskRequest {
    method: "tools/call".to_string(),
    params: json!({"name": "long_running_task"}),
    task_hint: Some(TaskHint {
        estimated_duration_secs: Some(10),
        supports_progress: true,
        cancellable: true,
    }),
    priority: Some(TaskPriority::High),
};

let task = manager.create_task(request).await?;

// 更新进度
manager.mark_working(&task.id).await?;
manager.update_progress(&task.id,
    TaskProgress::new(0.5).with_message("50% complete")
).await?;

// 完成
manager.mark_completed(&task.id, json!({"result": "done"})).await?;
```

---

## 9. 错误处理

### 示例 43: 错误处理模式

全面的错误处理策略。

```bash
cargo run --example 43_error_handling
```

```rust
use claude_agent_sdk::{query, ClaudeError};

match query("Hello", None).await {
    Ok(messages) => { /* 处理 */ },
    Err(ClaudeError::CliNotFound(e)) => {
        eprintln!("CLI not found: {:?}", e.cli_path);
    },
    Err(ClaudeError::Connection(e)) => {
        eprintln!("Connection failed: {}", e.message);
    },
    Err(ClaudeError::Process(e)) => {
        eprintln!("Process failed: {:?}", e.exit_code);
    },
    Err(e) => eprintln!("Error: {}", e),
}
```

### 带退避的重试

```rust
async fn retry_with_backoff() -> Result<()> {
    let max_retries = 3;
    let mut attempt = 0;

    loop {
        attempt += 1;
        match query("Test", None).await {
            Ok(_) => return Ok(()),
            Err(_) if attempt < max_retries => {
                let backoff = Duration::from_millis(100 * 2_u64.pow(attempt));
                tokio::time::sleep(backoff).await;
            },
            Err(e) => return Err(e.into()),
        }
    }
}
```

---

## 10. 并发

### 示例 44: 并发查询

并行执行以提高性能。

```bash
cargo run --example 44_concurrent_queries
```

```rust
use futures::future::join_all;

// 并发运行查询
let futures: Vec<_> = questions.into_iter().map(|q| {
    query(q.to_string(), None)
}).collect();

let results = join_all(futures).await;
```

### 限速并发

```rust
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(3)); // 最多 3 个并发

let results: Vec<_> = futures::stream::iter(queries)
    .map(|q| {
        let semaphore = semaphore.clone();
        async move {
            let _permit = semaphore.acquire().await.unwrap();
            query(q, None).await
        }
    })
    .buffer_unordered(3)
    .collect()
    .await;
```

---

## 11. 编排系统

### 示例 51: 多代理编排

协调多个 AI 代理。

```bash
cargo run --example 51_orchestration
```

```rust
use claude_agent_sdk::orchestration::{
    SequentialOrchestrator, ParallelOrchestrator, Agent,
};

// 顺序流水线: 研究 → 编写 → 编辑
let agents: Vec<Box<dyn Agent>> = vec![
    create_researcher(),
    create_writer(),
    create_editor(),
];

let orchestrator = SequentialOrchestrator::new();
let output = orchestrator.orchestrate(agents, input).await?;

// 从多个角度并行分析
let agents: Vec<Box<dyn Agent>> = vec![
    create_critic("Technical"),
    create_critic("Business"),
    create_critic("UX"),
];

let orchestrator = ParallelOrchestrator::new()
    .with_parallel_limit(3);
let output = orchestrator.orchestrate(agents, input).await?;
```

---

## 12. 自动安装 CLI

### 自动安装示例

Claude CLI 的自动安装。

```bash
cargo run --example auto_install_cli
```

```rust
let options = ClaudeAgentOptions::builder()
    .auto_install_cli(true)
    .build();

// 如果未找到 CLI，客户端创建会触发自动安装
let mut client = ClaudeClient::try_new(options)?;
client.connect().await?;
```

---

## 示例快速参考

### 按用例

| 用例 | 推荐示例 |
|----------|---------------------|
| 首次使用 | 01, 56 |
| 一次性查询 | 01, 20, 56 |
| 多轮对话 | 06, 16 |
| 工具控制 | 02, 03, 04, 05 |
| 自定义行为 | 05, 15, 21 |
| 生产应用 | 17, 18, 43, 50 |
| 高性能 | 20, 44, 48 |
| 复杂工作流 | 51, 52 |
| 测试 | 49, 50 |

### 按复杂度

| 级别 | 示例 |
|-------|----------|
| 初级 | 01, 20, 56, auto_install_cli |
| 中级 | 06, 07, 16, 23, 43 |
| 高级 | 15, 42, 44, 51, 62 |

---

## 运行示例

### 前提条件

1. 安装 Claude Code CLI（或启用自动安装）
2. 设置 `ANTHROPIC_API_KEY` 环境变量

### 命令

```bash
# 列出所有示例
ls crates/claude-agent-sdk/examples/

# 运行特定示例
cargo run --example 01_hello_world

# 使用 release 优化运行
cargo run --release --example 44_concurrent_queries

# 运行特定场景的钩子示例
cargo run --example 15_hooks_comprehensive PreToolUse
```

---

## 相关章节

- **第1章**: 入门指南与核心 API - SDK 概述
- **第2章**: ClaudeClient 深入解析 - 客户端详情
- **第3章**: V2 Session API - 简化接口
- **第4章**: 技能系统 - 技能开发
- **第5章**: MCP 集成 - 自定义工具
- **第6章**: Agent 编排系统 - 多代理系统
- **第8章**: 类型参考 - 类型文档
