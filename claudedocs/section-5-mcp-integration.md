# Section 5: MCP Integration

This section covers the MCP (Model Context Protocol) 2025-11-25 implementation, focusing on the async Tasks primitive for "call-now, fetch-later" workflows.

## Overview

The MCP module provides implementations for the latest MCP protocol features, including async Tasks, OAuth improvements, and extensions. The Tasks primitive enables asynchronous request handling where clients receive an immediate response and can poll for results later.

```rust
use claude_agent_sdk::mcp::tasks::{TaskManager, TaskRequest, TaskHandle};
use serde_json::json;
```

## Core Types

### TaskRequest

A request structure for creating async tasks.

```rust
let request = TaskRequest {
    method: "tools/call".to_string(),       // JSON-RPC method name
    params: json!({"name": "my_tool"}),     // Request parameters
    task_hint: Some(TaskHint {              // Optional: hints for execution
        estimated_duration_secs: Some(30),
        supports_progress: true,
        cancellable: true,
    }),
    priority: Some(TaskPriority::High),     // Optional: scheduling priority
};
```

### TaskHint

Provides hints about task execution behavior.

| Field | Type | Description |
|-------|------|-------------|
| `estimated_duration_secs` | `Option<u64>` | Expected execution time |
| `supports_progress` | `bool` | Whether progress updates will be sent |
| `cancellable` | `bool` | Whether the task can be cancelled |

### TaskPriority

Priority levels for task scheduling.

```rust
pub enum TaskPriority {
    Low,
    Normal,    // Default
    High,
    Urgent,
}
```

### TaskState

Task lifecycle states with transitions.

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

**State Properties:**

| State | Active | Terminal | Description |
|-------|--------|----------|-------------|
| `Queued` | Yes | No | Waiting to start |
| `Working` | Yes | No | In progress |
| `InputRequired` | Yes | No | Needs user input |
| `Completed` | No | Yes | Finished successfully |
| `Failed` | No | Yes | Failed with error |
| `Cancelled` | No | Yes | Was cancelled |

```rust
let state = TaskState::Working;
assert!(state.is_active());
assert!(!state.is_terminal());
```

### TaskProgress

Progress tracking with optional messages.

```rust
let progress = TaskProgress::new(0.5)
    .with_message("Processing batch 5 of 10");

assert_eq!(progress.value, 0.5);
assert_eq!(progress.message, Some("Processing batch 5 of 10".to_string()));
```

### TaskStatus

Complete status information for a task.

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

The result of a completed task.

```rust
pub struct TaskResult {
    pub id: TaskId,
    pub data: serde_json::Value,    // Result data
    pub completed_at: DateTime<Utc>,
}
```

### TaskHandle

Immediate response when creating a task.

```rust
pub struct TaskHandle {
    pub id: TaskId,
    pub uri: TaskUri,               // URI for polling
    pub status: TaskStatus,         // Initial status (Queued)
}
```

## TaskManager API

The `TaskManager` manages the complete lifecycle of async tasks.

### Creating a TaskManager

```rust
// Default configuration
let manager = TaskManager::new();

// Custom base URI for task resources
let manager = TaskManager::with_base_uri("mcp://my-tasks");
```

### Creating Tasks

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
println!("Task ID: {}", handle.id);
println!("Poll URI: {}", handle.uri);
```

### Polling for Status

```rust
loop {
    let status = manager.get_task_status(&handle.id).await?;

    if let Some(progress) = &status.progress {
        println!("Progress: {:.0}% - {:?}",
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

### Getting Results

```rust
// Only works for completed tasks
match manager.get_task_result(&handle.id).await {
    Ok(result) => {
        println!("Result: {:?}", result.data);
        println!("Completed at: {:?}", result.completed_at);
    }
    Err(ClaudeError::InvalidInput(msg)) => {
        println!("Task not completed: {}", msg);
    }
    Err(e) => return Err(e),
}
```

### State Transitions

The worker executing the task calls these methods:

```rust
// Mark as started
manager.mark_working(&task_id).await?;

// Update progress during execution
manager.update_progress(&task_id, TaskProgress::new(0.3)
    .with_message("Step 1 of 3 complete")).await?;

// Mark as completed
manager.mark_completed(&task_id, json!({"output": "success"})).await?;

// Or mark as failed
manager.mark_failed(&task_id, "Database connection failed").await?;

// Or mark as needing input
manager.mark_input_required(&task_id).await?;
```

### Cancellation

```rust
// Cancel a task (only if cancellable: true in task_hint)
match manager.cancel_task(&task_id).await {
    Ok(()) => println!("Task cancelled"),
    Err(ClaudeError::InvalidInput(msg)) => {
        println!("Cannot cancel: {}", msg);  // Not cancellable or already terminal
    }
    Err(e) => return Err(e),
}
```

### Listing and Cleanup

```rust
// List all tasks
let all_tasks = manager.list_tasks().await?;
for status in all_tasks {
    println!("{}: {:?}", status.id, status.state);
}

// Clean up tasks completed more than 1 hour ago
let cleaned = manager.cleanup_old_tasks(chrono::Duration::hours(1)).await?;
println!("Removed {} old tasks", cleaned);
```

## Design Patterns

### Immediate Response Pattern

Tasks return a handle immediately, allowing non-blocking workflows:

```rust
// Client creates task - returns immediately
let handle = manager.create_task(request).await?;

// Client can do other work
do_other_work().await;

// Poll when ready
let status = manager.get_task_status(&handle.id).await?;
```

### Progress Pattern

For long-running operations, provide progress updates:

```rust
// Worker side
for (i, item) in items.iter().enumerate() {
    process(item)?;

    let progress = (i + 1) as f64 / items.len() as f64;
    manager.update_progress(&task_id, TaskProgress::new(progress)
        .with_message(format!("Processed {}/{}", i + 1, items.len()))).await?;
}

manager.mark_completed(&task_id, result).await?;
```

### Cancellation Pattern

Support user-initiated cancellation:

```rust
// Client creates cancellable task
let request = TaskRequest {
    method: "long_operation".to_string(),
    params: json!({}),
    task_hint: Some(TaskHint {
        cancellable: true,
        ..Default::default()
    }),
    ..Default::default()
};

// Worker checks for cancellation
if manager.get_task_status(&task_id).await?.state == TaskState::Cancelled {
    cleanup();
    return Ok(());
}
```

## Complete Example

A full async task workflow:

```rust
use claude_agent_sdk::mcp::tasks::{
    TaskManager, TaskRequest, TaskHint, TaskProgress, TaskPriority
};
use serde_json::json;
use std::time::Duration;

async fn task_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let manager = TaskManager::new();

    // 1. Create task
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
    println!("Task created: {}", handle.id);

    // 2. Poll for completion
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

    // 3. Get final result
    let result = manager.get_task_result(&handle.id).await?;
    println!("Result: {:?}", result.data);

    Ok(())
}
```

## Error Handling

```rust
use claude_agent_sdk::errors::ClaudeError;

match manager.get_task_result(&task_id).await {
    Ok(result) => process(result),
    Err(ClaudeError::NotFound(msg)) => {
        // Task doesn't exist
        eprintln!("Task not found: {}", msg);
    }
    Err(ClaudeError::InvalidInput(msg)) => {
        // Task not completed yet
        eprintln!("Task still running: {}", msg);
    }
    Err(e) => return Err(e),
}
```

## Thread Safety

`TaskManager` is `Clone` and uses `Arc<RwLock<>>` internally, making it safe to share across threads:

```rust
let manager = TaskManager::new();

// Clone for each thread
let manager1 = manager.clone();
let manager2 = manager.clone();

// Both can safely access tasks
tokio::spawn(async move {
    manager1.create_task(request1).await
});

tokio::spawn(async move {
    manager2.list_tasks().await
});
```

## Related Sections

- [Section 1: Getting Started](section-1-getting-started.md) - SDK overview
- [Section 8: Types Reference](section-8-types-reference.md) - Core type definitions
- [Section 9: Internal Layer](section-9-internal-layer.md) - Implementation details
