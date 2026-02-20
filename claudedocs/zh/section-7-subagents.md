# 第 7 章: 子代理系统 (Subagents System)

## 7.1 概述

子代理模块提供了一个用于创建和管理具有特定能力和指令的专业化 Claude 实例的系统。这使得不同子代理可以处理不同任务的多代理架构成为可能。

**模块位置**: `crates/claude-agent-sdk/src/subagents/`

**核心组件**:
- **Subagent**: 专业化代理实例的配置
- **SubagentExecutor**: 执行子代理的运行时管理器
- **DelegationStrategy**: 任务委派的控制流
- **SubagentConfig**: 多子代理配置容器

## 7.2 核心类型

### 7.2.1 Subagent

`Subagent` 表示具有特定能力的专业化 Claude 实例。

```rust
pub struct Subagent {
    pub name: String,              // 唯一标识符
    pub description: String,       // 用途描述
    pub instructions: String,      // 具体行为指令
    pub allowed_tools: Vec<String>, // 工具白名单
    pub max_turns: Option<u32>,    // 轮次限制 (None = 无限制)
    pub model: Option<String>,     // 模型覆盖 (None = 默认)
}
```

**示例**:
```rust
use claude_agent_sdk::subagents::Subagent;

let code_reviewer = Subagent {
    name: "code-reviewer".to_string(),
    description: "Expert code reviewer".to_string(),
    instructions: "Review code for bugs and best practices".to_string(),
    allowed_tools: vec!["Read".to_string(), "Grep".to_string()],
    max_turns: Some(5),
    model: Some("claude-sonnet-4".to_string()),
};
```

### 7.2.2 DelegationStrategy

控制任务如何委派给子代理:

```rust
pub enum DelegationStrategy {
    Auto,      // Claude 自动决定何时委派
    Manual,    // 需要显式调用 SubagentTool
    ToolCall,  // 通过工具调用委派
}
```

| 策略 | 描述 | 使用场景 |
|------|------|----------|
| `Auto` | Claude 自主决定 | 通用委派 |
| `Manual` | 需要显式控制 | 确定性工作流 |
| `ToolCall` | 工具介导的委派 | 与工具系统集成 |

### 7.2.3 SubagentCall

表示单个子代理执行请求:

```rust
pub struct SubagentCall {
    pub subagent_name: String,
    pub input: String,
    pub output: Option<String>,
}
```

**方法**:
- `new(name, input)` - 创建新调用
- `is_executed()` - 检查是否存在输出

**示例**:
```rust
use claude_agent_sdk::subagents::SubagentCall;

let call = SubagentCall::new("code-reviewer", "Review src/main.rs");
assert!(!call.is_executed());
```

### 7.2.4 SubagentOutput

子代理执行的结果:

```rust
pub struct SubagentOutput {
    pub subagent_name: String,
    pub messages: Vec<serde_json::Value>,
}
```

> **注意**: 消息被序列化为 `serde_json::Value` 以保持灵活性。

### 7.2.5 SubagentError

子代理操作的错误类型:

```rust
pub enum SubagentError {
    NotFound(String),       // 未找到子代理
    AlreadyExists(String),  // 注册时名称重复
    ExecutionFailed(String), // 查询执行失败
    InvalidInput(String),   // 提供了无效输入
}
```

## 7.3 SubagentConfig

用于管理多个子代理的配置容器:

```rust
pub struct SubagentConfig {
    pub subagents: Vec<Subagent>,
    pub delegation_strategy: DelegationStrategy,
}
```

**方法**:
- `new(strategy)` - 创建空配置
- `add_subagent(subagent)` - 将子代理添加到列表
- `get_subagent(name)` - 按名称线性搜索 (O(n))
- `to_map()` - 转换为 HashMap 以实现 O(1) 查找

**示例**:
```rust
use claude_agent_sdk::subagents::{SubagentConfig, Subagent, DelegationStrategy};

let mut config = SubagentConfig::new(DelegationStrategy::Auto);

config.add_subagent(Subagent {
    name: "reviewer".to_string(),
    description: "Code reviewer".to_string(),
    instructions: "Review code for quality".to_string(),
    allowed_tools: vec!["Read".to_string()],
    max_turns: Some(5),
    model: None,
});

// 对于频繁查找，转换为 map
let map = config.to_map(); // O(1) 查找
```

> **性能提示**: `get_subagent()` 使用 O(n) 线性搜索。对于频繁查找，请使用 `to_map()`。

## 7.4 SubagentExecutor

管理和执行子代理的运行时执行器。

### 7.4.1 创建

```rust
use claude_agent_sdk::subagents::{SubagentExecutor, DelegationStrategy};

let executor = SubagentExecutor::new(DelegationStrategy::Auto);
```

### 7.4.2 注册

```rust
let mut executor = SubagentExecutor::new(DelegationStrategy::Auto);

let subagent = Subagent {
    name: "my-agent".to_string(),
    description: "Description".to_string(),
    instructions: "Instructions".to_string(),
    allowed_tools: vec!["Read".to_string()],
    max_turns: Some(5),
    model: None,
};

executor.register(subagent)?; // 如果重复则返回 AlreadyExists 错误
```

### 7.4.3 执行

```rust
async fn run_subagent(executor: &SubagentExecutor) -> Result<(), Box<dyn std::error::Error>> {
    let output = executor.execute("my-agent", "Process this input").await?;
    println!("子代理 {} 返回了 {} 条消息",
        output.subagent_name,
        output.messages.len()
    );
    Ok(())
}
```

### 7.4.4 查询方法

```rust
// 列出所有已注册的子代理
let names: Vec<String> = executor.list_subagents();

// 检查子代理是否存在
if executor.has_subagent("my-agent") {
    println!("代理存在");
}

// 获取当前策略
let strategy: &DelegationStrategy = executor.strategy();
```

## 7.5 执行流程

`execute()` 方法遵循以下流程:

```
execute(name, input)
    │
    ├─► 按名称查找子代理 (O(1) HashMap)
    │       └─► 如果不存在则返回 NotFound 错误
    │
    ├─► 构建系统提示
    │       description + "\n\nInstructions:\n" + instructions
    │
    ├─► 构建 ClaudeAgentOptions
    │       ├─► 同时指定 model + max_turns
    │       ├─► 仅指定 model
    │       ├─► 仅指定 max_turns
    │       └─► 都不指定 (使用默认值)
    │
    ├─► 调用 crate::query::query()
    │       └─► 将错误包装为 ExecutionFailed
    │
    └─► 将消息序列化为 JSON
            └─► 返回 SubagentOutput
```

## 7.6 错误处理

| 错误 | 触发条件 | 恢复方法 |
|------|----------|----------|
| `NotFound` | 子代理名称未注册 | 先注册子代理 |
| `AlreadyExists` | 注册时名称重复 | 使用唯一名称 |
| `ExecutionFailed` | 查询 API 错误 | 检查 API 状态、凭证 |
| `InvalidInput` | 无效输入 (当前未使用) | 调用前验证输入 |

**错误处理示例**:
```rust
use claude_agent_sdk::subagents::{SubagentExecutor, SubagentError, DelegationStrategy};

async fn safe_execute(executor: &SubagentExecutor, name: &str, input: &str) {
    match executor.execute(name, input).await {
        Ok(output) => println!("成功: {} 条消息", output.messages.len()),
        Err(SubagentError::NotFound(name)) => {
            eprintln!("子代理 '{}' 未注册", name);
        }
        Err(SubagentError::ExecutionFailed(msg)) => {
            eprintln!("执行失败: {}", msg);
        }
        Err(e) => eprintln!("错误: {}", e),
    }
}
```

## 7.7 安全考虑

### 工具访问控制

子代理使用 `allowed_tools` 作为工具访问的白名单:

```rust
let restricted_agent = Subagent {
    name: "read-only".to_string(),
    description: "只读分析".to_string(),
    instructions: "分析但不修改".to_string(),
    allowed_tools: vec!["Read".to_string(), "Grep".to_string()],
    // 没有 Write, Edit, Bash 工具
    max_turns: None,
    model: None,
};
```

### 已知安全考虑

| 关注点 | 状态 | 说明 |
|--------|------|------|
| 工具访问控制 | ✅ 已保护 | `allowed_tools` 白名单 |
| 输入验证 | ⚠️ 无 | 没有输入字符串验证 |
| 名称注入 | ⚠️ 无 | 没有子代理名称清理 |
| 指令注入 | ⚠️ 无 | 用户提供的指令直接传递 |

**建议**: 在生产环境中，验证子代理名称（字母数字、长度限制）并清理用户输入。

## 7.8 性能特征

| 操作 | 复杂度 | 说明 |
|------|--------|------|
| `register()` | O(1) 均摊 | HashMap 插入 |
| `execute()` 查找 | O(1) | HashMap get |
| `list_subagents()` | O(n) | 键迭代 |
| `has_subagent()` | O(1) | HashMap contains_key |
| `SubagentConfig::get_subagent()` | O(n) | 线性搜索 ⚠️ |
| `SubagentConfig::to_map()` | O(n) | 一次性转换 |

## 7.9 API 参考

### 重导出 (mod.rs)

```rust
pub use types::{
    DelegationStrategy,
    Subagent,
    SubagentCall,
    SubagentConfig,
    SubagentError,
    SubagentOutput,
};
```

### SubagentExecutor 方法

| 方法 | 签名 | 描述 |
|------|------|------|
| `new` | `(strategy: DelegationStrategy) -> Self` | 创建执行器 |
| `register` | `(&mut self, subagent: Subagent) -> Result<(), SubagentError>` | 注册子代理 |
| `execute` | `(&self, name: &str, input: &str) -> Result<SubagentOutput, SubagentError>` | 异步执行 |
| `list_subagents` | `(&self) -> Vec<String>` | 获取所有名称 |
| `has_subagent` | `(&self, name: &str) -> bool` | 检查是否存在 |
| `strategy` | `(&self) -> &DelegationStrategy` | 获取策略 |

## 7.10 完整示例

```rust
use claude_agent_sdk::subagents::{
    Subagent, SubagentConfig, SubagentExecutor, DelegationStrategy
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建自动委派的执行器
    let mut executor = SubagentExecutor::new(DelegationStrategy::Auto);

    // 定义专业化子代理
    let code_reviewer = Subagent {
        name: "code-reviewer".to_string(),
        description: "专注于质量和最佳实践的代码审查专家".to_string(),
        instructions: r#"
            分析代码的:
            - Bug 和潜在错误
            - 安全漏洞
            - 性能问题
            - 代码风格和可维护性
        "#.to_string(),
        allowed_tools: vec!["Read".to_string(), "Grep".to_string()],
        max_turns: Some(5),
        model: Some("claude-sonnet-4".to_string()),
    };

    let doc_writer = Subagent {
        name: "doc-writer".to_string(),
        description: "技术文档专家".to_string(),
        instructions: "编写清晰、全面的文档".to_string(),
        allowed_tools: vec!["Read".to_string(), "Write".to_string()],
        max_turns: Some(3),
        model: None, // 使用默认模型
    };

    // 注册子代理
    executor.register(code_reviewer)?;
    executor.register(doc_writer)?;

    // 列出可用的子代理
    println!("可用子代理: {:?}", executor.list_subagents());

    // 执行子代理
    let output = executor.execute("code-reviewer", "Review src/lib.rs").await?;
    println!("从 {} 收到 {} 条消息", output.messages.len(), output.subagent_name);

    Ok(())
}
```

## 7.11 测试覆盖

**单元测试** (共 17 个):
- types.rs: 9 个测试，覆盖结构体创建、配置操作、错误显示
- mod.rs: 5 个测试，覆盖执行器操作和错误情况
- 所有测试通过

**覆盖缺口**:
- 没有成功 `execute()` 路径的集成测试（需要 API 凭证）
- 没有 `InvalidInput` 错误变体的测试

## 7.12 设计说明

### 设计决策

1. **HashMap 存储**: `SubagentExecutor` 使用 `HashMap<String, Subagent>` 实现 O(1) 查找
2. **Serde 支持**: 所有类型支持序列化，可用于配置文件
3. **Builder 模式**: `ClaudeAgentOptions` 根据可选字段条件构建
4. **异步优先**: `execute()` 是异步的，支持非阻塞操作

### 一致性说明

- `SubagentError` 手动实现 `Display` 和 `Error` trait
- 其他模块使用 `thiserror` crate 处理错误类型
- 建议迁移到 `thiserror` 以保持一致性（建议）
