# 第 5 节：MCP 集成

本节介绍 MCP (Model Context Protocol) 2025-11-25 实现，重点关注异步 Tasks 原语，用于"立即调用，稍后获取"的工作流程。

## 概述

MCP 模块提供最新 MCP 协议特性的实现，包括异步 Tasks、OAuth 改进和扩展。Tasks 原语支持异步请求处理，客户端可以立即收到响应，并可以稍后轮询获取结果。

```rust
use claude_agent_sdk::mcp::tasks::{TaskManager, TaskRequest, TaskHandle};
use serde_json::json;
```

## 核心类型

### TaskRequest

用于创建异步任务的请求结构。

```rust
let request = TaskRequest {
    method: "tools/call".to_string(),       // JSON-RPC 方法名
    params: json!({"name": "my_tool"}),     // 请求参数
    task_hint: Some(TaskHint {              // 可选：执行提示
        estimated_duration_secs: Some(30),
        supports_progress: true,
        cancellable: true,
    }),
    priority: Some(TaskPriority::High),     // 可选：调度优先级
};
```

### TaskHint

提供任务执行行为的提示。

| 字段 | 类型 | 描述 |
|------|------|------|
| `estimated_duration_secs` | `Option<u64>` | 预期执行时间 |
| `supports_progress` | `bool` | 是否发送进度更新 |
| `cancellable` | `bool` | 任务是否可以取消 |

### TaskPriority

任务调度的优先级级别。

```rust
pub enum TaskPriority {
    Low,
    Normal,    // 默认
    High,
    Urgent,
}
```

### TaskState

任务生命周期状态及转换。

```
                    ┌─────────────┐
                    │   Queued    │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
            ┌──────►│   Working   │◄──────┐
            │       └──────┬──────┘       │
            │              │              │
     ┌──────┴──────┐       │       ┌──────┴──────┐
     │InputRequired│       │       │  Cancelled  │
     └──────┬──────┘       │       └─────────────┘
            │              │
            └──────┬───────┴──────┐
                   │              │
            ┌──────▼──────┐ ┌─────▼─────┐
            │  Completed  │ │   Failed  │
            └─────────────┘ └───────────┘
```

**状态属性：**

| 状态 | 活跃 | 终态 | 描述 |
|------|------|------|------|
| `Queued` | 是 | 否 | 等待开始 |
| `Working` | 是 | 否 | 进行中 |
| `InputRequired` | 是 | 否 | 需要用户输入 |
| `Completed` | 否 | 是 | 成功完成 |
| `Failed` | 否 | 是 | 失败出错 |
| `Cancelled` | 否 | 是 | 已被取消 |

```rust
let state = TaskState::Working;
assert!(state.is_active());
assert!(!state.is_terminal());
```

### TaskProgress

带有可选消息的进度跟踪。

```rust
let progress = TaskProgress::new(0.5)
    .with_message("正在处理批次 5/10");

assert_eq!(progress.value, 0.5);
assert_eq!(progress.message, Some("正在处理批次 5/10".to_string()));
```

### TaskStatus

任务的完整状态信息。

```rust
pub struct TaskStatus {
    pub id: TaskId,
    pub state: TaskState,
    pub progress: Option<TaskProgress>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
```

### TaskResult

已完成任务的结果。

```rust
pub struct TaskResult {
    pub id: TaskId,
    pub data: serde_json::Value,    // 结果数据
    pub completed_at: DateTime<Utc>,
}
```

### TaskHandle

创建任务时的即时响应。

```rust
pub struct TaskHandle {
    pub id: TaskId,
    pub uri: TaskUri,               // 轮询 URI
    pub status: TaskStatus,         // 初始状态 (Queued)
}
```

## TaskManager API

`TaskManager` 管理异步任务的完整生命周期。

### 创建 TaskManager

```rust
// 默认配置
let manager = TaskManager::new();

// 自定义任务资源的基础 URI
let manager = TaskManager::with_base_uri("mcp://my-tasks");
```

### 创建任务

```rust
let request = TaskRequest {
    method: "tools/call".to_string(),
    params: json!({"name": "analyze", "arguments": {"file": "data.csv"}}),
    task_hint: Some(TaskHint {
        estimated_duration_secs: Some(60),
        supports_progress: true,
        ..Default::default()
    }),
    ..Default::default()
};

let handle = manager.create_task(request).await?;
println!("任务 ID: {}", handle.id);
println!("轮询 URI: {}", handle.uri);
```

### 轮询状态

```rust
loop {
    let status = manager.get_task_status(&handle.id).await?;

    if let Some(progress) = &status.progress {
        println!("进度: {:.0}% - {:?}",
            progress.value * 100.0,
            progress.message
        );
    }

    if status.is_terminal() {
        break;
    }

    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

### 获取结果

```rust
// 仅适用于已完成的任务
match manager.get_task_result(&handle.id).await {
    Ok(result) => {
        println!("结果: {:?}", result.data);
        println!("完成时间: {:?}", result.completed_at);
    }
    Err(ClaudeError::InvalidInput(msg)) => {
        println!("任务未完成: {}", msg);
    }
    Err(e) => return Err(e),
}
```

### 状态转换

执行任务的工作器调用这些方法：

```rust
// 标记为已开始
manager.mark_working(&task_id).await?;

// 执行期间更新进度
manager.update_progress(&task_id, TaskProgress::new(0.3)
    .with_message("步骤 1/3 已完成")).await?;

// 标记为已完成
manager.mark_completed(&task_id, json!({"output": "success"})).await?;

// 或标记为失败
manager.mark_failed(&task_id, "数据库连接失败").await?;

// 或标记为需要输入
manager.mark_input_required(&task_id).await?;
```

### 取消任务

```rust
// 取消任务（仅当 task_hint 中 cancellable: true 时）
match manager.cancel_task(&task_id).await {
    Ok(()) => println!("任务已取消"),
    Err(ClaudeError::InvalidInput(msg)) => {
        println!("无法取消: {}", msg);  // 不可取消或已是终态
    }
    Err(e) => return Err(e),
}
```

### 列表和清理

```rust
// 列出所有任务
let all_tasks = manager.list_tasks().await?;
for status in all_tasks {
    println!("{}: {:?}", status.id, status.state);
}

// 清理 1 小时前完成的任务
let cleaned = manager.cleanup_old_tasks(chrono::Duration::hours(1)).await?;
println!("已移除 {} 个旧任务", cleaned);
```

## 设计模式

### 即时响应模式

任务立即返回句柄，允许非阻塞工作流：

```rust
// 客户端创建任务 - 立即返回
let handle = manager.create_task(request).await?;

// 客户端可以做其他工作
do_other_work().await;

// 准备好后轮询
let status = manager.get_task_status(&handle.id).await?;
```

### 进度模式

对于长时间运行的操作，提供进度更新：

```rust
// 工作器端
for (i, item) in items.iter().enumerate() {
    process(item)?;

    let progress = (i + 1) as f64 / items.len() as f64;
    manager.update_progress(&task_id, TaskProgress::new(progress)
        .with_message(format!("已处理 {}/{}", i + 1, items.len()))).await?;
}

manager.mark_completed(&task_id, result).await?;
```

### 取消模式

支持用户发起的取消：

```rust
// 客户端创建可取消任务
let request = TaskRequest {
    method: "long_operation".to_string(),
    params: json!({}),
    task_hint: Some(TaskHint {
        cancellable: true,
        ..Default::default()
    }),
    ..Default::default()
};

// 工作器检查取消状态
if manager.get_task_status(&task_id).await?.state == TaskState::Cancelled {
    cleanup();
    return Ok(());
}
```

## 完整示例

一个完整的异步任务工作流：

```rust
use claude_agent_sdk::mcp::tasks::{
    TaskManager, TaskRequest, TaskHint, TaskProgress, TaskPriority
};
use serde_json::json;
use std::time::Duration;

async fn task_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let manager = TaskManager::new();

    // 1. 创建任务
    let request = TaskRequest {
        method: "tools/call".to_string(),
        params: json!({
            "name": "analyze_data",
            "arguments": {"file": "large_dataset.csv"}
        }),
        task_hint: Some(TaskHint {
            estimated_duration_secs: Some(120),
            supports_progress: true,
            cancellable: true,
        }),
        priority: Some(TaskPriority::High),
    };

    let handle = manager.create_task(request).await?;
    println!("任务已创建: {}", handle.id);

    // 2. 轮询等待完成
    loop {
        let status = manager.get_task_status(&handle.id).await?;

        if let Some(progress) = &status.progress {
            println!("[{:.0}%] {:?}",
                progress.value * 100.0,
                progress.message.as_deref().unwrap_or("")
            );
        }

        if status.is_terminal() {
            break;
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // 3. 获取最终结果
    let result = manager.get_task_result(&handle.id).await?;
    println!("结果: {:?}", result.data);

    Ok(())
}
```

## 错误处理

```rust
use claude_agent_sdk::errors::ClaudeError;

match manager.get_task_result(&task_id).await {
    Ok(result) => process(result),
    Err(ClaudeError::NotFound(msg)) => {
        // 任务不存在
        eprintln!("任务未找到: {}", msg);
    }
    Err(ClaudeError::InvalidInput(msg)) => {
        // 任务尚未完成
        eprintln!("任务仍在运行: {}", msg);
    }
    Err(e) => return Err(e),
}
```

## 线程安全

`TaskManager` 实现了 `Clone`，内部使用 `Arc<RwLock<>>`，可以安全地跨线程共享：

```rust
let manager = TaskManager::new();

// 为每个线程克隆
let manager1 = manager.clone();
let manager2 = manager.clone();

// 两者都可以安全访问任务
tokio::spawn(async move {
    manager1.create_task(request1).await
});

tokio::spawn(async move {
    manager2.list_tasks().await
});
```

## 相关章节

- [第 1 节：入门指南](section-1-getting-started.md) - SDK 概述
- [第 8 节：类型参考](section-8-types-reference.md) - 核心类型定义
- [第 9 节：内部层](section-9-internal-layer.md) - 实现细节
