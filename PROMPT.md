# Claude Agent SDK 商业化计划 - 扩展包架构版 (v2.0)

## 概述

基于对 Claude Agent SDK 商业化案例的深入研究，本文档制定了 Rust SDK 的商业化路径计划。采用**扩展包架构 (Extension Crates)** 设计，保持核心 crate 精简，按需加载扩展功能。

**本版本重点**: 聚焦于除 pool 之外的丰富功能模块，将其提取为独立扩展包，实现最小改造和最大代码复用。

### 当前状态 (2026-02-21)

**核心 crate 模块分析:**

| 模块 | 文件数 | 行数估计 | 提取建议 |
|------|--------|----------|----------|
| `skills/` | 17 | ~3000 | → `claude-agent-sdk-skills` |
| `orchestration/` | 7 | ~1500 | → `claude-agent-sdk-orchestration` |
| `observability/` | 3 | ~800 | → `claude-agent-sdk-observability` |
| `subagents/` | 2 | ~350 | → `claude-agent-sdk-agents` |
| `mcp/` | 2 | ~400 | → `claude-agent-sdk-mcp-tasks` |
| `commands/` | 1 | ~470 | → `claude-agent-sdk-commands` |
| `todos/` | 1 | ~770 | → `claude-agent-sdk-todos` |
| `partnership/` | 4 | ~600 | → `claude-agent-sdk-partnership` |
| `v2/` | 2 | ~270 | 保留核心 (简化 API) |
| `types/` | 6 | ~2000 | 部分保留，部分导出 |
| `internal/` | 8 | ~2000 | 保留核心 (transport, client) |

---

## 一、扩展包架构设计

### 1.1 架构原则

| 原则 | 说明 | 收益 |
|------|------|------|
| **核心精简** | `claude-agent-sdk` 只包含基础 API + types | 快速编译、小二进制 |
| **按需加载** | 用户只引入需要的扩展包 | 减少依赖树 |
| **特性标志** | 通过 feature flags 控制功能 | 编译时优化 |
| **版本独立** | 各扩展包可独立版本迭代 | 灵活升级 |
| **最小改造** | 复制优先，保持向后兼容 | 低风险迁移 |

### 1.2 Crate 结构

```
crates/
├── claude-agent-sdk/                 # 核心 crate (必需)
│   ├── 核心 API: query(), prompt(), ClaudeClient
│   ├── Transport 层: SubprocessTransport
│   ├── 错误处理: ClaudeError, ClaudeResult
│   ├── 基础配置: ClaudeAgentOptions
│   ├── 基础类型: Message, Content, Tool, Hooks, Permissions
│   └── V2 简化 API: prompt(), create_session()
│
├── claude-agent-sdk-skills/          # 技能系统扩展 (可选) ⭐重点
│   ├── Skill trait, SkillRegistry
│   ├── SKILL.md 解析 (skill_md.rs)
│   ├── 技能发现 (discovery.rs)
│   ├── 安全沙箱 (sandbox.rs)
│   ├── 技能审计 (auditor.rs)
│   ├── 热重载 (hot_reload.rs)
│   ├── 渐进式加载 (progressive_disclosure.rs)
│   ├── VS Code 导出 (vscode.rs)
│   └── API 客户端 (api.rs)
│
├── claude-agent-sdk-orchestration/   # 多Agent编排扩展 (可选) ⭐重点
│   ├── Agent trait, AgentRegistry
│   ├── Orchestrator trait
│   ├── SequentialOrchestrator
│   ├── ParallelOrchestrator
│   ├── ExecutionContext, ExecutionTrace
│   └── 错误处理 (OrchestrationError)
│
├── claude-agent-sdk-observability/   # 可观测性扩展 (可选)
│   ├── Logger, LogObserver, LogLevel
│   ├── MetricsCollector, Histogram
│   ├── TracingConfig, tracing_setup
│   └── 请求 ID 生成
│
├── claude-agent-sdk-agents/          # 子代理系统扩展 (可选)
│   ├── Subagent, SubagentConfig
│   ├── SubagentExecutor
│   ├── DelegationStrategy
│   └── SubagentError
│
├── claude-agent-sdk-mcp-tasks/       # MCP Tasks 扩展 (可选)
│   ├── TaskManager
│   ├── TaskRequest, TaskResult
│   ├── TaskState, TaskStatus
│   └── TaskHandle, TaskProgress
│
├── claude-agent-sdk-commands/        # 斜杠命令扩展 (可选)
│   ├── SlashCommand
│   ├── CommandRegistry
│   ├── CommandHandler
│   └── CommandError
│
├── claude-agent-sdk-todos/           # Todo管理扩展 (可选)
│   ├── TodoList
│   ├── TodoItem, TodoStatus
│   └── TodoError
│
├── claude-agent-sdk-partnership/     # AI投资合伙扩展 (可选)
│   ├── PartnershipBuilder
│   ├── Partner, Portfolio
│   ├── PartnershipAgreement
│   └── InvestmentStrategy
│
└── claude-agent-sdk-batch/           # 批量操作扩展 (新增)
    ├── BatchExecutor
    ├── batch_query()
    └── BatchResult<T>
```

### 1.3 依赖关系图

```
                    ┌─────────────────────────┐
                    │   claude-agent-sdk      │
                    │      (核心 crate)        │
                    │  - query/prompt API     │
                    │  - Transport layer      │
                    │  - Error types          │
                    │  - V2 simplified API    │
                    └─────────────────────────┘
                              │
   ┌──────────┬───────────┬───┴───┬──────────┬──────────┬──────────┐
   │          │           │       │          │          │          │
   ▼          ▼           ▼       ▼          ▼          ▼          ▼
┌──────┐ ┌──────────┐ ┌──────┐ ┌──────┐ ┌────────┐ ┌────────┐ ┌────────┐
│skills│ │orchestr. │ │observ│ │agents│ │mcp-    │ │commands│ │ todos  │
│技能   │ │编排      │ │可观测│ │子代理│ │tasks   │ │命令    │ │任务    │
└──────┘ └──────────┘ └──────┘ └──────┘ └────────┘ └────────┘ └────────┘
   │          │                                          │
   ▼          ▼                                          ▼
┌──────┐ ┌──────────┐                              ┌────────┐
│sandbox│ │patterns  │                              │partners│
│沙箱   │ │模式      │                              │合伙    │
└──────┘ └──────────┘                              └────────┘
```

### 1.4 Cargo.toml 配置示例

```toml
# workspace Cargo.toml
[workspace]
members = [
    "crates/claude-agent-sdk",
    "crates/claude-agent-sdk-skills",
    "crates/claude-agent-sdk-orchestration",
    "crates/claude-agent-sdk-observability",
    "crates/claude-agent-sdk-agents",
    "crates/claude-agent-sdk-mcp-tasks",
    "crates/claude-agent-sdk-commands",
    "crates/claude-agent-sdk-todos",
    "crates/claude-agent-sdk-partnership",
    "crates/claude-agent-sdk-batch",
]
resolver = "2"
```

```toml
# 用户项目 Cargo.toml
[dependencies]
# 核心 (必需)
claude-agent-sdk = "0.2"

# 按需添加扩展
claude-agent-sdk-skills = "0.1"          # 需要技能系统
claude-agent-sdk-orchestration = "0.1"   # 需要多Agent编排
claude-agent-sdk-observability = "0.1"   # 需要可观测性
claude-agent-sdk-agents = "0.1"          # 需要子代理
claude-agent-sdk-commands = "0.1"        # 需要命令系统
claude-agent-sdk-todos = "0.1"           # 需要任务管理
```

---

## 二、扩展包详细设计

### 2.1 核心 Crate: `claude-agent-sdk`

**职责**: 提供最小可用的 SDK 核心功能

**功能清单**:
```rust
// 保留在核心 crate
pub mod client;        // ClaudeClient
pub mod query;         // query(), query_stream()
pub mod errors;        // ClaudeError, ErrorCategory
pub mod types;         // 基础类型
pub mod v2;            // 简化 API (prompt, create_session)
mod internal;          // transport, cli_installer
```

**保留的 types**:
- `config.rs`: ClaudeAgentOptions, ClaudeAgentOptionsBuilder
- `messages.rs`: Message, ContentBlock, UserContentBlock
- `hooks.rs`: Hook, HookType, HookPayload
- `permissions.rs`: PermissionMode, PermissionStatus
- `plugin.rs`: SdkPluginConfig
- `mcp.rs`: 基础 MCP 类型 (ToolHandler, ToolResult)

**Feature Flags**:
```toml
[features]
default = ["json"]
json = ["serde", "serde_json"]
zero-copy = []          # 零拷贝解析 (内部 feature)
tracing = ["dep:tracing"]  # 基础 tracing
```

### 2.2 扩展 Crate: `claude-agent-sdk-skills` ⭐重点

**职责**: 完整的 Claude Code Skills 系统

**源文件** (17个文件，~3000行):
- `skill_md.rs`: SKILL.md 解析，YAML frontmatter
- `discovery.rs`: 技能发现，目录扫描
- `trait_impl.rs`: Skill trait, SkillBox
- `sandbox.rs`: 安全沙箱执行
- `auditor.rs`: 技能安全审计
- `hot_reload.rs`: 热重载支持
- `progressive_disclosure.rs`: 渐进式加载
- `vscode.rs`: VS Code 导出
- `api.rs`: Skills API 客户端
- `types.rs`: SkillMetadata, SkillPackage
- `dependency.rs`: 依赖解析
- `version.rs`: 版本兼容性
- `tags.rs`: 标签查询
- `tool_restriction.rs`: 工具限制
- `performance.rs`: 性能优化 (LRU, 索引)

**API 设计**:
```rust
use claude_agent_sdk_skills::{
    Skill, SkillRegistry, SkillPackage, SkillAuditor,
    discover_skill_md_from_dir, ProgressiveSkillLoader,
    SandboxExecutor, HotReloadManager, export_to_vscode,
};

// 发现技能
let packages = SkillRegistry::discover_skill_md_from_dir(".claude/skills")?;

// 审计技能
let auditor = SkillAuditor::new(AuditConfig::strict());
let report = auditor.audit(&package)?;

// 沙箱执行
let executor = SandboxExecutor::new(SandboxConfig::default());
let result = executor.execute(&skill, &input).await?;

// VS Code 导出
export_to_vscode(&package, VsCodeExportConfig::default())?;
```

**依赖**:
```toml
[dependencies]
claude-agent-sdk = { path = "../claude-agent-sdk", version = "0.2" }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
regex = "1"
walkdir = "2"
tokio = { version = "1", features = ["process", "io-util"] }
```

### 2.3 扩展 Crate: `claude-agent-sdk-orchestration` ⭐重点

**职责**: 多Agent编排框架

**源文件** (7个文件，~1500行):
- `agent.rs`: Agent trait, AgentInput, AgentOutput
- `orchestrator.rs`: Orchestrator trait, OrchestratorInput, OrchestratorOutput
- `patterns/sequential.rs`: SequentialOrchestrator
- `patterns/parallel.rs`: ParallelOrchestrator
- `context.rs`: ExecutionContext, ExecutionTrace, ExecutionConfig
- `registry.rs`: AgentRegistry, AgentMetadata, AgentFilter
- `errors.rs`: OrchestrationError

**API 设计**:
```rust
use claude_agent_sdk_orchestration::{
    Agent, AgentInput, AgentOutput, AgentRegistry,
    SequentialOrchestrator, ParallelOrchestrator,
    Orchestrator, ExecutionContext,
};
use async_trait::async_trait;

// 定义 Agent
struct Researcher;
#[async_trait]
impl Agent for Researcher {
    fn name(&self) -> &str { "Researcher" }
    fn description(&self) -> &str { "Researches topics" }
    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // ...
    }
}

// 注册并编排
let mut registry = AgentRegistry::new();
registry.register(Box::new(Researcher))?;

let orchestrator = SequentialOrchestrator::new(registry);
let output = orchestrator.orchestrate(input).await?;
```

**依赖**:
```toml
[dependencies]
claude-agent-sdk = { path = "../claude-agent-sdk", version = "0.2" }
async-trait = "0.1"
tokio = { version = "1", features = ["rt", "sync", "time"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
```

### 2.4 扩展 Crate: `claude-agent-sdk-observability`

**职责**: 可观测性 - 日志、指标、追踪

**源文件** (3个文件，~800行):
- `logger.rs`: Logger, LogObserver, ConsoleLogObserver, LogLevel, LogFormat
- `metrics.rs`: MetricsCollector, Histogram, TimerGuard, MetricStorage
- `tracing_setup.rs`: TracingConfig, init_tracing, generate_request_id

**API 设计**:
```rust
use claude_agent_sdk_observability::{
    Logger, MetricsCollector, TracingConfig,
    init_tracing, LogLevel, Histogram,
};

// 初始化
init_tracing(TracingConfig::production());

// 日志
let logger = Logger::new("MyAgent");
logger.info("Starting", &[("task_id", "123")]);

// 指标
let metrics = MetricsCollector::new();
let _timer = metrics.start_timer("execution", &[("agent", "researcher")]);
```

**依赖**:
```toml
[dependencies]
claude-agent-sdk = { path = "../claude-agent-sdk", version = "0.2" }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
parking_lot = "0.12"
```

### 2.5 扩展 Crate: `claude-agent-sdk-agents`

**职责**: 子代理系统

**源文件** (2个文件，~350行):
- `mod.rs`: SubagentExecutor, Subagent, DelegationStrategy
- `types.rs`: SubagentConfig, SubagentOutput, SubagentError

**API 设计**:
```rust
use claude_agent_sdk_agents::{
    Subagent, SubagentExecutor, DelegationStrategy,
};

let mut executor = SubagentExecutor::new(DelegationStrategy::Auto);

executor.register(Subagent {
    name: "code-reviewer".to_string(),
    description: "Expert code reviewer".to_string(),
    instructions: "Review code for bugs".to_string(),
    allowed_tools: vec!["Read".to_string(), "Grep".to_string()],
    max_turns: Some(5),
    model: Some("claude-sonnet-4".to_string()),
})?;

let output = executor.execute("code-reviewer", "Review this file").await?;
```

### 2.6 扩展 Crate: `claude-agent-sdk-mcp-tasks`

**职责**: MCP 协议异步 Tasks 原语

**源文件** (2个文件，~400行):
- `mod.rs`: TaskManager, TaskHandle
- `tasks.rs`: TaskRequest, TaskResult, TaskState, TaskStatus, TaskProgress

**API 设计**:
```rust
use claude_agent_sdk_mcp_tasks::{
    TaskManager, TaskRequest, TaskHandle, TaskStatus,
};

let manager = TaskManager::new();

let request = TaskRequest {
    method: "tools/call".to_string(),
    params: json!({"name": "my_tool", "arguments": {}}),
    ..Default::default()
};

let task = manager.create_task(request).await?;
println!("Task created: {}", task.id);

// Later: fetch result
let result = manager.get_result(&task.id).await?;
```

### 2.7 扩展 Crate: `claude-agent-sdk-commands`

**职责**: 斜杠命令系统

**源文件** (1个文件，~470行):
- `mod.rs`: SlashCommand, CommandRegistry, CommandHandler, CommandError

**API 设计**:
```rust
use claude_agent_sdk_commands::{
    SlashCommand, CommandRegistry, CommandHandler,
};

let mut registry = CommandRegistry::new();

registry.register(SlashCommand::new(
    "deploy",
    "Deploy the application",
    Arc::new(|_name, args| {
        Box::pin(async move {
            // Deploy logic
            Ok("Deployed successfully".to_string())
        })
    }),
))?;

let result = registry.execute("deploy", vec!["--env".to_string(), "prod".to_string()]).await?;
```

### 2.8 扩展 Crate: `claude-agent-sdk-todos`

**职责**: Todo 列表管理

**源文件** (1个文件，~770行):
- `mod.rs`: TodoList, TodoItem, TodoStatus, TodoError

**API 设计**:
```rust
use claude_agent_sdk_todos::{TodoList, TodoItem, TodoStatus};

let mut list = TodoList::new("Project Tasks");
list.add("Implement feature A");
list.add("Write tests");

let id = list.items[0].id.clone();
list.start(&id)?;
// ... do work ...
list.complete(&id)?;

println!("Progress: {}%", list.completion_percentage());
```

### 2.9 扩展 Crate: `claude-agent-sdk-partnership`

**职责**: AI 投资合伙模型

**源文件** (4个文件，~600行):
- `types.rs`: Partner, RiskProfile, InvestmentGoals, InvestmentStrategy
- `agreement.rs`: PartnershipAgreement, ConcentrationLimits, RedemptionPolicy
- `portfolio.rs`: Portfolio, Position, PerformanceHistory, PerformanceMetrics
- `builder.rs`: PartnershipBuilder

**API 设计**:
```rust
use claude_agent_sdk_partnership::{
    PartnershipBuilder, Partner, InvestmentStrategy, RiskProfile,
};

let partners = vec![
    Partner {
        id: "partner-1".to_string(),
        name: "Alice".to_string(),
        capital_contribution: 100_000.0,
        profit_share: 0.8,
        voting_rights: true,
        risk_profile: RiskProfile::Moderate,
        investment_goals: InvestmentGoals {
            target_return: 0.15,
            time_horizon: Duration::days(365 * 5),
            liquidity_needs: LiquidityNeeds::Low,
        },
    },
];

let partnership = PartnershipBuilder::create_partnership(
    "AI Investment Partnership".to_string(),
    partners,
    InvestmentStrategy::ValueInvesting,
).await?;
```

### 2.10 扩展 Crate: `claude-agent-sdk-batch` (新增)

**职责**: 批量操作 API

**API 设计**:
```rust
use claude_agent_sdk_batch::{BatchExecutor, BatchResult};

let executor = BatchExecutor::new(client);

let queries = vec![
    "分析这段代码的安全性",
    "生成单元测试",
    "添加文档注释",
];

let results: BatchResult<Vec<Analysis>> = executor
    .batch_query(&queries)
    .parallelism(3)
    .retry(2)
    .await?;

for (query, result) in results {
    println!("{}: {:?}", query, result);
}
```

---

## 三、实现计划 (最小改造版)

### 设计原则: 最小改造 + 最大复用

| 原则 | 实践 | 示例 |
|------|------|------|
| **复制优先** | 先复制代码到新 crate，而非直接移动 | skills/ 复制到 skills crate |
| **渐进迁移** | 核心 crate 保持向后兼容，使用 feature flags | `features = ["skills"]` |
| **重新导出** | 核心 crate 重新导出扩展类型 | `pub use skills::*` |
| **独立测试** | 每个扩展包独立测试套件 | `cargo test -p claude-agent-sdk-skills` |
| **文档同步** | 更新所有受影响的文档和示例 | 更新 example 使用新 crate |

### Phase 0: 准备工作 (1天)

**目标**: 创建扩展包骨架结构，验证编译

**任务**:
```
[ ] 创建 crates/claude-agent-sdk-skills/ 目录和 Cargo.toml
[ ] 创建 crates/claude-agent-sdk-orchestration/ 目录和 Cargo.toml
[ ] 创建 crates/claude-agent-sdk-observability/ 目录和 Cargo.toml
[ ] 创建 crates/claude-agent-sdk-agents/ 目录和 Cargo.toml
[ ] 创建 crates/claude-agent-sdk-mcp-tasks/ 目录和 Cargo.toml
[ ] 创建 crates/claude-agent-sdk-commands/ 目录和 Cargo.toml
[ ] 创建 crates/claude-agent-sdk-todos/ 目录和 Cargo.toml
[ ] 创建 crates/claude-agent-sdk-partnership/ 目录和 Cargo.toml
[ ] 创建 crates/claude-agent-sdk-batch/ 目录和 Cargo.toml
[ ] 更新 workspace Cargo.toml 添加新 members
[ ] 验证空 crate 编译通过
```

**目录结构**:
```
crates/
├── claude-agent-sdk/              # 核心 crate (已存在)
│
├── claude-agent-sdk-skills/       # 新建
│   ├── Cargo.toml
│   └── src/lib.rs                 # 初始为空
│
├── claude-agent-sdk-orchestration/  # 新建
│   ├── Cargo.toml
│   └── src/lib.rs
│
├── ... (其他新建 crate)
```

### Phase 1: 提取 Skills 系统 (3天) ⭐优先

**目标**: 将 `skills/` 模块移动到独立 crate

**策略 - 最小改造**:
```rust
// 步骤 1: 复制而非移动
// crates/claude-agent-sdk-skills/src/lib.rs
pub use crate::{
    api::*,
    auditor::*,
    dependency::*,
    discovery::*,
    error::*,
    hot_reload::*,
    performance::*,
    progressive_disclosure::*,
    sandbox::*,
    skill_md::*,
    tags::*,
    tool_restriction::*,
    trait_impl::*,
    types::*,
    version::*,
    vscode::*,
};

mod api;
mod auditor;
// ... 所有模块

// 步骤 2: 核心 crate 可选依赖
// crates/claude-agent-sdk/Cargo.toml
[features]
default = []
skills = ["dep:claude-agent-sdk-skills"]

[dependencies]
claude-agent-sdk-skills = { path = "../claude-agent-sdk-skills", optional = true }

// 步骤 3: 核心 crate 重新导出 (向后兼容)
// crates/claude-agent-sdk/src/lib.rs
#[cfg(feature = "skills")]
pub use claude_agent_sdk_skills::{
    Skill, SkillError, SkillInput, SkillOutput, SkillPackage, SkillRegistry, SkillResources,
    // ... 所有公开类型
};
```

**任务**:
```
[ ] 复制 skills/ 目录所有文件到 skills crate
[ ] 调整模块中的导入路径 (crate:: -> claude_agent_sdk::)
[ ] 配置 skills crate 的 Cargo.toml 依赖
[ ] 在核心 crate 添加可选依赖和 feature flag
[ ] 更新核心 crate 的 re-exports
[ ] 验证 cargo test --features skills 通过
[ ] 验证 cargo test (不带 features) 通过
[ ] 更新使用 skills 的示例
```

**验收标准**:
- [ ] `claude-agent-sdk-skills` 独立编译通过
- [ ] 核心 crate 带 `skills` feature 编译通过
- [ ] 核心 crate 不带 `skills` feature 编译通过
- [ ] 所有测试通过
- [ ] 文档更新

### Phase 2: 提取 Orchestration 系统 (2天) ⭐优先

**目标**: 将 `orchestration/` 模块移动到独立 crate

**策略**:
```rust
// crates/claude-agent-sdk-orchestration/src/lib.rs
pub mod agent;
pub mod context;
pub mod errors;
pub mod orchestrator;
pub mod patterns;
pub mod registry;

pub use agent::{Agent, AgentInput, AgentOutput};
pub use context::{ExecutionConfig, ExecutionContext, ExecutionTrace};
pub use errors::{OrchestrationError, Result};
pub use orchestrator::{Orchestrator, OrchestratorInput, OrchestratorOutput};
pub use registry::{AgentFilter, AgentMetadata, AgentRegistry, AgentRegistryBuilder};
pub use patterns::{ParallelOrchestrator, SequentialOrchestrator};
```

**任务**:
```
[ ] 复制 orchestration/ 目录所有文件
[ ] 调整导入路径
[ ] 配置 Cargo.toml
[ ] 添加 feature flag 到核心 crate
[ ] 更新 re-exports
[ ] 验证编译和测试
```

### Phase 3: 提取 Observability (1天)

**目标**: 将 `observability/` 模块移动到独立 crate

**策略**:
```rust
// crates/claude-agent-sdk-observability/src/lib.rs
pub mod logger;
pub mod metrics;
pub mod tracing_setup;

pub use logger::{ConsoleLogObserver, GlobalLogger, LogEntry, LogFormat, LogLevel, LogObserver, Logger};
pub use metrics::{Histogram, HistogramBuckets, LabeledMetric, MetricKind, MetricStorage, MetricsCollector, TimerGuard};
pub use tracing_setup::{generate_request_id, generate_span_id, init_default, init_tracing, is_initialized, log_counter, log_gauge, log_timing, OutputFormat, TracingConfig};
```

### Phase 4: 提取其他模块 (2天)

**目标**: 移动 agents, mcp-tasks, commands, todos, partnership

每个模块独立任务:
```
[ ] 复制 subagents/ → claude-agent-sdk-agents
[ ] 复制 mcp/      → claude-agent-sdk-mcp-tasks
[ ] 复制 commands/ → claude-agent-sdk-commands
[ ] 复制 todos/    → claude-agent-sdk-todos
[ ] 复制 partnership/ → claude-agent-sdk-partnership
```

### Phase 5: 创建 Batch 扩展 (2天)

**目标**: 新建 batch 扩展 crate

**实现**:
```rust
// crates/claude-agent-sdk-batch/src/lib.rs
use claude_agent_sdk::{ClaudeClient, ClaudeError, Message};
use futures::{stream, StreamExt};

pub struct BatchExecutor {
    client: ClaudeClient,
    parallelism: usize,
    retry_count: u32,
}

impl BatchExecutor {
    pub fn new(client: ClaudeClient) -> Self { ... }

    pub async fn batch_query(
        &self,
        queries: &[&str],
    ) -> Vec<Result<Vec<Message>, ClaudeError>> {
        stream::iter(queries)
            .map(|q| self.execute_with_retry(q))
            .buffer_unordered(self.parallelism)
            .collect()
            .await
    }
}
```

### Phase 6: 清理和文档 (1天)

**任务**:
```
[ ] 移除核心 crate 中已迁移模块的源文件
[ ] 更新 README.md 说明新的 crate 结构
[ ] 更新所有示例使用新的 crate
[ ] 添加迁移指南
[ ] 运行完整测试套件
[ ] Clippy 检查
```

---

## 四、迁移指南

### 从核心 crate 迁移到扩展包

**场景 1: 使用 Skills 系统**
```toml
# 之前 (所有功能在核心 crate)
[dependencies]
claude-agent-sdk = { version = "0.2", features = ["skills"] }

# 之后 (推荐: 使用独立 crate)
[dependencies]
claude-agent-sdk = "0.2"
claude-agent-sdk-skills = "0.1"

# 之后 (兼容: 继续使用 feature flag)
[dependencies]
claude-agent-sdk = { version = "0.2", features = ["skills"] }
```

**场景 2: 使用 Orchestration**
```rust
// 之前
use claude_agent_sdk::orchestration::{Agent, SequentialOrchestrator};

// 之后 (独立 crate)
use claude_agent_sdk_orchestration::{Agent, SequentialOrchestrator};
```

**场景 3: 使用 Observability**
```rust
// 之前
use claude_agent_sdk::observability::{Logger, MetricsCollector};

// 之后 (独立 crate)
use claude_agent_sdk_observability::{Logger, MetricsCollector};
```

### API 兼容性承诺

| 版本 | 兼容性策略 |
|------|-----------|
| **0.x** | 可能破坏性变更，feature flags 保持稳定 |
| **1.0+** | 语义化版本，核心 API 稳定 |

### 废弃计划

| 功能 | 废弃版本 | 移除版本 | 替代方案 |
|------|----------|----------|----------|
| 核心 crate 内 skills 模块 | 0.3 | 1.0 | `claude-agent-sdk-skills` crate |
| 核心 crate 内 orchestration 模块 | 0.3 | 1.0 | `claude-agent-sdk-orchestration` crate |
| 核心 crate 内 observability 模块 | 0.3 | 1.0 | `claude-agent-sdk-observability` crate |

---

## 五、代码复用示例

### 核心 crate 到扩展包的代码迁移

**示例: skills/mod.rs 迁移**

```rust
// === 之前: crates/claude-agent-sdk/src/skills/mod.rs ===
pub use trait_impl::{Skill, SkillBox};
pub use discovery::{discover_from_dir, ...};
// ... 大量 re-exports

// === 之后: crates/claude-agent-sdk-skills/src/lib.rs ===
// 完整的 skills 模块代码
pub use crate::{
    api::*,
    auditor::*,
    // ... 所有模块
};

mod api;
mod auditor;
// ... 所有模块

// === 核心 crate: 保持重新导出 (向后兼容) ===
// crates/claude-agent-sdk/src/lib.rs
#[cfg(feature = "skills")]
pub use claude_agent_sdk_skills::{
    Skill, SkillError, SkillInput, SkillOutput, SkillPackage, SkillRegistry, SkillResources,
    // ... 关键类型
};

// 同时保留内部模块 (废弃警告)
#[deprecated(since = "0.3.0", note = "Use claude-agent-sdk-skills crate instead")]
#[cfg(feature = "skills")]
pub mod skills {
    pub use claude_agent_sdk_skills::*;
}
```

### 依赖关系管理

```toml
# crates/claude-agent-sdk-skills/Cargo.toml
[package]
name = "claude-agent-sdk-skills"
version = "0.1.0"
edition = "2021"

[dependencies]
claude-agent-sdk = { path = "../claude-agent-sdk", version = "0.2" }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
regex = "1"
walkdir = "2"
tokio = { version = "1", features = ["process", "io-util"] }
async-trait = "0.1"
thiserror = "1"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
tempfile = "3"
```

---

## 六、风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 导入路径变更导致编译错误 | 高 | 中 | 提供 feature flags 兼容 + 重新导出 |
| 循环依赖 | 低 | 高 | 仔细设计依赖关系，核心 crate 无反向依赖 |
| 测试覆盖不足 | 中 | 中 | 每个扩展包独立测试套件 |
| 文档过时 | 高 | 低 | 迁移时同步更新文档 |
| 版本不同步 | 中 | 中 | Workspace 统一版本管理 |

---

## 七、执行时间线

```
Day 1:    Phase 0 - 创建所有扩展包骨架
Day 2-4:  Phase 1 - 提取 Skills 系统 (最大模块)
Day 5-6:  Phase 2 - 提取 Orchestration 系统
Day 7:    Phase 3 - 提取 Observability
Day 8-9:  Phase 4 - 提取其他模块 (agents, mcp, commands, todos, partnership)
Day 10-11: Phase 5 - 创建 Batch 扩展
Day 12:   Phase 6 - 清理和文档
```

### 里程碑检查点

| 里程碑 | 预计完成 | 验收标准 |
|--------|----------|----------|
| **M1: 骨架就绪** | Day 1 | 9个空 crate 编译通过 |
| **M2: Skills 独立** | Day 4 | skills crate 独立可用，测试通过 |
| **M3: Orchestration 独立** | Day 6 | orchestration crate 独立可用 |
| **M4: 所有模块提取** | Day 9 | 7个扩展 crate 全部独立可用 |
| **M5: 完成** | Day 12 | 全部测试通过，文档更新 |

---

## 八、成功指标 (KPI)

### 技术指标
- 扩展包数量: **10个** (skills, orchestration, observability, agents, mcp-tasks, commands, todos, partnership, batch, pool)
- 测试覆盖率: **> 80%** (每个 crate)
- 编译时间: 核心 crate < 30s
- 二进制大小: 核心 crate 最小化

### 代码指标
- 核心 crate 代码量: **< 4000 行** (从 ~12000 行减少)
- Skills crate 代码量: **~3000 行**
- Orchestration crate 代码量: **~1500 行**

---

## 参考资料

### 官方文档
- [Claude Agent SDK 官方文档](https://docs.anthropic.com)
- [Claude Code Skills 规范](https://code.claude.com/docs/en/skills)

### Rust 最佳实践
- [The Cargo Book - Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Package Cargo.toml 最佳实践](https://doc.rust-lang.org/cargo/reference/manifest.html)

---

*文档版本: 2.0*
*创建日期: 2026-02-21*
*最后更新: 2026-02-21*
*架构变更: 聚焦于 skills, orchestration 等核心功能模块的提取，最小化 pool 关注*
*新增内容: 完整的模块分析、10个扩展 crate 设计、详细的迁移策略*
