# 第6章：Agent 编排

Agent 编排模块提供了一个全面的框架，用于协调多个 AI Agent 协作完成复杂任务。它支持多种编排模式，包括顺序执行和并行执行。

## 6.1 概述

### 模块架构

```
orchestration/
├── 核心类型 (agent.rs, orchestrator.rs)
├── 上下文管理 (context.rs)
├── 错误处理 (errors.rs)
├── Agent 注册表 (registry.rs)
└── 模式
    ├── 顺序执行 (patterns/sequential.rs)
    └── 并行执行 (patterns/parallel.rs)
```

### 功能标志

| 功能 | 描述 |
|------|------|
| Default | 启用所有编排功能 |
| `tracing` | 启用执行追踪支持 |

### 主要特性

- **灵活的编排模式**：支持顺序和并行执行模式
- **类型安全的 Agent 接口**：使用 Rust 的 trait 系统实现强类型 Agent 定义
- **异步优先设计**：完全支持 async/await 和 Tokio
- **执行追踪**：提供全面的执行跟踪，用于调试和监控
- **错误恢复**：内置指数退避的重试逻辑
- **可扩展**：易于添加自定义 Agent 和编排器

---

## 6.2 核心类型

### AgentInput

Agent 的输入类型：

```rust
use claude_agent_sdk::orchestration::AgentInput;

let input = AgentInput::new("Research quantum computing")
    .with_context(serde_json::json!({"depth": "advanced"}))
    .with_metadata("priority", "high");
```

**字段：**
| 字段 | 类型 | 描述 |
|------|------|------|
| `content` | String | Agent 的主要内容/提示词 |
| `context` | serde_json::Value | 附加上下文数据 |
| `metadata` | HashMap<String, String> | 元数据键值对 |

### AgentOutput

Agent 的输出类型：

```rust
use claude_agent_sdk::orchestration::AgentOutput;

let output = AgentOutput::new("Research complete: 5 articles found")
    .with_confidence(0.95)
    .with_data(serde_json::json!({"articles": 5}))
    .with_metadata("duration_ms", "1500");

// 检查输出是否表示成功
if output.is_successful() {
    println!("Agent succeeded with confidence: {}", output.confidence);
}
```

**字段：**
| 字段 | 类型 | 描述 |
|------|------|------|
| `content` | String | 主要响应内容 |
| `data` | serde_json::Value | 附加结构化数据 |
| `confidence` | f64 | 置信度分数 (0.0 - 1.0) |
| `metadata` | HashMap<String, String> | 元数据键值对 |

### OrchestratorInput / OrchestratorOutput

编排器的输入和输出类型：

```rust
use claude_agent_sdk::orchestration::{OrchestratorInput, OrchestratorOutput};

// 创建编排器输入
let input = OrchestratorInput::new("Analyze market trends")
    .with_context(serde_json::json!({"region": "US"}))
    .with_metadata("request_id", "12345");

// 检查输出
if output.is_successful() {
    println!("Final result: {}", output.result);
    println!("Agents executed: {}", output.agent_outputs.len());
}
```

---

## 6.3 Agent Trait

`Agent` trait 定义了参与编排的接口：

```rust
use claude_agent_sdk::orchestration::{Agent, AgentInput, AgentOutput, Result};
use async_trait::async_trait;

#[derive(Debug)]
struct ResearchAgent;

#[async_trait]
impl Agent for ResearchAgent {
    fn name(&self) -> &str {
        "ResearchAgent"
    }

    fn description(&self) -> &str {
        "Researches topics and gathers information"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 执行研究逻辑
        let research_result = format!("Found 10 articles about: {}", input.content);

        Ok(AgentOutput::new(research_result)
            .with_confidence(0.9)
            .with_data(serde_json::json!({"sources": 10})))
    }
}
```

### SimpleAgent

用于从闭包创建 Agent 的便捷包装器：

```rust
use claude_agent_sdk::orchestration::agent::SimpleAgent;

let agent = SimpleAgent::new(
    "QuickAgent",
    "A simple agent for quick tasks",
    |input| {
        Ok(AgentOutput::new(format!("Processed: {}", input.content)))
    }
);
```

---

## 6.4 编排模式

### 顺序编排

Agent 按顺序执行，每个 Agent 的输出成为下一个 Agent 的输入：

```text
Input → Agent A → Agent B → Agent C → Output
```

**适用场景：**
- 数据处理管道
- 多步骤推理
- 内容生成和精炼

```rust
use claude_agent_sdk::orchestration::{
    SequentialOrchestrator, Orchestrator, OrchestratorInput, Agent
};

// 创建顺序编排器
let orchestrator = SequentialOrchestrator::new()
    .with_max_retries(3);

// 定义管道 Agent
let agents: Vec<Box<dyn Agent>> = vec![
    create_researcher(),
    create_writer(),
    create_editor(),
];

// 执行管道
let input = OrchestratorInput::new("Climate Change");
let output = orchestrator.orchestrate(agents, input).await?;

println!("Final result: {}", output.result);
// 每个 Agent 的输出传递给下一个：
// Researcher: "Found 5 articles about Climate Change"
//   → Writer: "Article draft based on Found 5 articles..."
//     → Editor: "Final version: Article draft..."
```

### 并行编排

多个 Agent 并发执行，它们的输出被聚合：

```text
        → Agent A ─┐
Input ─┼→ Agent B ─┼→ Aggregator → Output
        → Agent C ─┘
```

**适用场景：**
- 多角度分析
- 并行任务处理
- 性能优化

```rust
use claude_agent_sdk::orchestration::{
    ParallelOrchestrator, Orchestrator, OrchestratorInput, Agent
};

// 创建带并发控制的并行编排器
let orchestrator = ParallelOrchestrator::new()
    .with_max_retries(2)
    .with_parallel_limit(5);  // 最多 5 个并发 Agent

// 定义并行 Agent
let agents: Vec<Box<dyn Agent>> = vec![
    create_analyzer("technical"),
    create_analyzer("business"),
    create_analyzer("security"),
];

// 并行执行
let input = OrchestratorInput::new("New Architecture Proposal");
let output = orchestrator.orchestrate(agents, input).await?;

// 结果被聚合
println!("Aggregated results:\n{}", output.result);
// Output:
// Parallel execution results:
// 1. Technical Analysis: ...
// 2. Business Analysis: ...
// 3. Security Analysis: ...
```

---

## 6.5 执行上下文

### ExecutionConfig

编排执行的配置：

```rust
use claude_agent_sdk::orchestration::context::ExecutionConfig;
use std::time::Duration;

let config = ExecutionConfig::new()
    .with_timeout(Duration::from_secs(120))   // 最长 2 分钟
    .with_max_retries(5)                      // 每个 Agent 最多 5 次重试
    .with_parallel_limit(20)                  // 最多 20 个并行 Agent
    .with_logging(true)                       // 启用日志
    .with_tracing(true);                      // 启用追踪
```

**默认值：**
| 设置 | 默认值 |
|------|--------|
| timeout | 300 秒 (5 分钟) |
| max_retries | 3 |
| parallel_limit | 10 |
| enable_logging | true |
| enable_tracing | true |

### ExecutionContext

管理编排状态：

```rust
use claude_agent_sdk::orchestration::context::ExecutionContext;

let ctx = ExecutionContext::new(config);

// 存储状态
ctx.set_state("key", serde_json::json!("value")).await;

// 获取状态
if let Some(value) = ctx.get_state("key").await {
    println!("State: {}", value);
}

// 获取执行追踪
let trace = ctx.get_trace().await;
println!("Executions: {}", trace.agent_executions.len());
```

### ExecutionTrace

记录执行历史：

```rust
use claude_agent_sdk::orchestration::context::ExecutionTrace;

let trace = output.execution_trace;

println!("Started: {}", trace.start_time);
println!("Duration: {:?}", trace.duration());

// 访问各个 Agent 的执行记录
for exec in &trace.agent_executions {
    println!(
        "Agent: {} - Success: {} - Duration: {:?}ms",
        exec.agent_name,
        exec.success,
        exec.duration_ms
    );
}
```

### AgentExecution

单个 Agent 执行的记录：

```rust
use claude_agent_sdk::orchestration::context::AgentExecution;

// 执行记录会自动创建
// 可以通过 ExecutionTrace.agent_executions 访问

for exec in &output.execution_trace.agent_executions {
    println!("Agent: {}", exec.agent_name);
    println!("  Input: {}", exec.input.content);
    if let Some(ref output) = exec.output {
        println!("  Output: {}", output.content);
    }
    if let Some(ref error) = exec.error {
        println!("  Error: {}", error);
    }
}
```

---

## 6.6 Agent 注册表

注册表为 Agent 定义提供集中管理：

### AgentMetadata

Agent 定义的丰富元数据：

```rust
use claude_agent_sdk::orchestration::registry::AgentMetadata;

let metadata = AgentMetadata::new(
    "research-agent",
    "Academic Researcher",
    "Expert in academic research",
    "research"
)
.with_tool("web-search")
.with_tool("read-file")
.with_skill("citation")
.with_tag("academic")
.with_version("2.0.0")
.with_max_retries(5)
.with_timeout(120);
```

**元数据字段：**
| 字段 | 类型 | 描述 |
|------|------|------|
| `id` | String | 唯一标识符 |
| `name` | String | 人类可读的名称 |
| `description` | String | Agent 的功能描述 |
| `category` | String | 领域 (例如 "research", "analysis") |
| `version` | String | 语义化版本 |
| `tools` | Vec<String> | 此 Agent 可使用的工具 |
| `skills` | Vec<String> | 此 Agent 拥有的技能 |
| `tags` | Vec<String> | 用于筛选的标签 |
| `max_retries` | usize | 最大重试次数 |
| `timeout_secs` | u64 | 超时时间（秒） |
| `enabled` | bool | Agent 是否启用 |

### AgentRegistry

Agent 管理的集中注册表：

```rust
use claude_agent_sdk::orchestration::registry::{AgentRegistry, AgentMetadata};

let registry = AgentRegistry::new();

// 注册 Agent
let agent = SimpleAgent::new("researcher", "Academic researcher", |input| {
    Ok(AgentOutput::new(format!("Researched: {}", input.content)))
});

let metadata = AgentMetadata::new(
    "researcher",
    "researcher",  // 必须与 agent.name() 匹配
    "Academic researcher",
    "research"
);

registry.register(Box::new(agent), metadata).await?;

// 检查注册
assert!(registry.contains("researcher").await);

// 通过 ID 执行 Agent
let output = registry.execute_agent(
    "researcher",
    AgentInput::new("Quantum computing")
).await?;
```

### AgentFilter

用于搜索 Agent 的筛选条件：

```rust
use claude_agent_sdk::orchestration::registry::AgentFilter;

// 按条件查找 Agent
let filter = AgentFilter::new()
    .with_category("research")
    .with_tool("web-search")
    .with_tag("academic")
    .enabled_only();

let matching = registry.find(&filter).await;

for metadata in matching {
    println!("Found: {} - {}", metadata.name, metadata.description);
}
```

### AgentRegistryBuilder

用于创建注册表的 Builder 模式：

```rust
use claude_agent_sdk::orchestration::registry::AgentRegistryBuilder;

let registry = AgentRegistryBuilder::new()
    .with_agent(
        Box::new(SimpleAgent::new("agent1", "First", |input| {
            Ok(AgentOutput::new(format!("1: {}", input.content)))
        })),
        AgentMetadata::new("agent1", "agent1", "First agent", "test")
    ).await?
    .with_agent(
        Box::new(SimpleAgent::new("agent2", "Second", |input| {
            Ok(AgentOutput::new(format!("2: {}", input.content)))
        })),
        AgentMetadata::new("agent2", "agent2", "Second agent", "test")
    ).await?
    .build();
```

---

## 6.7 错误处理

### OrchestrationError

编排操作的错误类型：

```rust
use claude_agent_sdk::orchestration::errors::OrchestrationError;

match orchestrator.orchestrate(agents, input).await {
    Ok(output) => println!("Success: {}", output.result),
    Err(OrchestrationError::AgentFailed(name, reason)) => {
        eprintln!("Agent {} failed: {}", name, reason);
    }
    Err(OrchestrationError::Timeout(msg)) => {
        eprintln!("Operation timed out: {}", msg);
    }
    Err(OrchestrationError::InvalidConfig(msg)) => {
        eprintln!("Configuration error: {}", msg);
    }
    Err(OrchestrationError::PartialSuccess(count)) => {
        eprintln!("{} agents failed", count);
    }
    Err(e) => eprintln!("Other error: {}", e),
}

// 检查错误是否可重试
if error.is_retryable() {
    // 重试操作
}
```

**错误变体：**
| 变体 | 描述 | 可重试 |
|------|------|--------|
| `AgentFailed` | Agent 执行失败 | 是 |
| `AgentError` | 通用 Agent 错误 | 否 |
| `OrchestratorFailed` | 编排器失败 | 否 |
| `Timeout` | 操作超时 | 是 |
| `InvalidConfig` | 配置无效 | 否 |
| `Cancelled` | 执行被取消 | 否 |
| `PartialSuccess` | 部分 Agent 失败 | 否 |

---

## 6.8 完整示例

```rust
use claude_agent_sdk::orchestration::{
    Agent, AgentInput, AgentOutput,
    SequentialOrchestrator, ParallelOrchestrator,
    Orchestrator, OrchestratorInput,
    AgentRegistry, AgentMetadata,
};
use async_trait::async_trait;

// 定义自定义 Agent
struct Researcher;
struct Writer;
struct Editor;

#[async_trait]
impl Agent for Researcher {
    fn name(&self) -> &str { "Researcher" }
    fn description(&self) -> &str { "Gathers information" }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk::orchestration::Result<AgentOutput> {
        Ok(AgentOutput::new(format!("Research: {} - Found 10 sources", input.content))
            .with_confidence(0.95))
    }
}

#[async_trait]
impl Agent for Writer {
    fn name(&self) -> &str { "Writer" }
    fn description(&self) -> &str { "Creates content" }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk::orchestration::Result<AgentOutput> {
        Ok(AgentOutput::new(format!("Draft: {}", input.content))
            .with_confidence(0.90))
    }
}

#[async_trait]
impl Agent for Editor {
    fn name(&self) -> &str { "Editor" }
    fn description(&self) -> &str { "Refines content" }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk::orchestration::Result<AgentOutput> {
        Ok(AgentOutput::new(format!("Final: {}", input.content))
            .with_confidence(0.92))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 示例 1: 顺序管道
    println!("=== Sequential Pipeline ===");

    let sequential = SequentialOrchestrator::new();
    let agents: Vec<Box<dyn Agent>> = vec![
        Box::new(Researcher),
        Box::new(Writer),
        Box::new(Editor),
    ];

    let input = OrchestratorInput::new("AI Trends 2024");
    let output = sequential.orchestrate(agents, input).await?;

    println!("Success: {}", output.is_successful());
    println!("Steps: {}", output.agent_outputs.len());
    println!("Final: {}", output.result);
    println!("Duration: {:?}", output.execution_trace.duration());

    // 示例 2: 并行分析
    println!("\n=== Parallel Analysis ===");

    let parallel = ParallelOrchestrator::new()
        .with_parallel_limit(5);

    let analyzers: Vec<Box<dyn Agent>> = vec![
        Box::new(Researcher),  // 重用作为分析器
    ];

    let input = OrchestratorInput::new("Market Analysis");
    let output = parallel.orchestrate(analyzers, input).await?;

    println!("Success: {}", output.is_successful());
    println!("Aggregated: {}", output.result);

    Ok(())
}
```

---

## 6.9 API 参考

### `orchestration` 模块的重导出

```rust
// 核心类型
pub use agent::{Agent, AgentInput, AgentOutput, AgentError, SimpleAgent};
pub use orchestrator::{Orchestrator, OrchestratorInput, OrchestratorOutput, BaseOrchestrator};

// 上下文
pub use context::{ExecutionConfig, ExecutionContext, ExecutionTrace, AgentExecution};

// 错误
pub use errors::{OrchestrationError, Result};

// 注册表
pub use registry::{
    AgentRegistry, AgentRegistryBuilder,
    AgentMetadata, AgentFilter, RegistryError
};

// 模式
pub use patterns::{SequentialOrchestrator, ParallelOrchestrator};
```

---

## 6.10 最佳实践

### Agent 设计

1. **单一职责**：每个 Agent 应该只做好一件事
2. **清晰的契约**：明确定义期望的输入/输出格式
3. **置信度评分**：提供有意义的置信度分数 (0.0-1.0)
4. **错误处理**：为可重试和不可重试的失败返回适当的错误

### 模式选择

| 用例 | 推荐模式 |
|------|----------|
| 数据管道 | 顺序执行 |
| 多步骤工作流 | 顺序执行 |
| 多角度分析 | 并行执行 |
| 批量处理 | 并行执行 |
| 混合依赖 | 先顺序后并行 |

### 性能

1. **并行限制**：根据资源设置适当的 `parallel_limit`
2. **重试策略**：对不稳定操作使用 `with_max_retries()`
3. **超时设置**：为长时间运行的 Agent 设置合理的超时时间
4. **注册表**：使用 `AgentRegistry` 进行动态 Agent 管理

### 监控

1. **启用追踪**：在生产环境中始终启用追踪
2. **检查执行追踪**：审查 Agent 执行时间和成功率
3. **记录失败**：使用日志记录 Agent 失败以便调试
