//! MCP 2025-11-25 Async Tasks Example
//!
//! This example demonstrates the Tasks primitive from the MCP 2025-11-25 spec,
//! enabling "call-now, fetch-later" asynchronous workflows.
//!
//! Run with:
//! ```sh
//! cargo run --example 42_mcp_async_tasks
//! ```

use claude_agent_sdk::mcp::tasks::{
    TaskHint, TaskManager, TaskPriority, TaskProgress, TaskRequest, TaskState,
};
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MCP 2025-11-25 Async Tasks Demo\n");

    // Scenario 1: Basic Task Creation
    println!("📋 Scenario 1: Basic Task Creation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    basic_task_creation().await?;
    println!();

    // Scenario 2: Task with Progress Tracking
    println!("📊 Scenario 2: Task with Progress Tracking");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    task_with_progress().await?;
    println!();

    // Scenario 3: Task Cancellation
    println!("❌ Scenario 3: Task Cancellation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    task_cancellation().await?;
    println!();

    // Scenario 4: Task Priorities
    println!("🎯 Scenario 4: Task Priorities");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    task_priorities().await?;
    println!();

    // Scenario 5: Error Handling
    println!("💥 Scenario 5: Error Handling");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    error_handling().await?;
    println!();

    // Scenario 6: List and Cleanup
    println!("🧹 Scenario 6: List and Cleanup");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    list_and_cleanup().await?;
    println!();

    println!("✅ All scenarios completed successfully!");

    Ok(())
}

/// Scenario 1: Basic task creation and polling
async fn basic_task_creation() -> Result<(), Box<dyn std::error::Error>> {
    let manager = TaskManager::new();

    // Create a simple task
    let request = TaskRequest {
        method: "tools/call".to_string(),
        params: json!({
            "name": "calculate",
            "arguments": {"operation": "2 + 2"}
        }),
        ..Default::default()
    };

    let task = manager.create_task(request).await?;
    println!("  ✨ Task created: {}", task.id);
    println!("  📍 Task URI: {}", task.uri);
    println!("  🎭 Initial state: {:?}", task.status.state);

    // Simulate task completion
    manager.mark_working(&task.id).await?;
    manager
        .mark_completed(&task.id, json!({"result": 4}))
        .await?;

    // Get final status
    let status = manager.get_task_status(&task.id).await?;
    println!("  ✅ Final state: {:?}", status.state);
    println!(
        "  📦 Result: {:?}",
        manager.get_task_result(&task.id).await?
    );

    Ok(())
}

/// Scenario 2: Task with progress tracking
async fn task_with_progress() -> Result<(), Box<dyn std::error::Error>> {
    let manager = TaskManager::new();

    // Create a task with progress support
    let request = TaskRequest {
        method: "tools/call".to_string(),
        params: json!({"name": "long_running_task"}),
        task_hint: Some(TaskHint {
            estimated_duration_secs: Some(10),
            supports_progress: true,
            cancellable: true,
        }),
        ..Default::default()
    };

    let task = manager.create_task(request).await?;
    println!("  ✨ Task created: {}", task.id);
    println!("  ⏱️  Estimated duration: 10 seconds");

    // Simulate progress updates
    manager.mark_working(&task.id).await?;

    for i in 0..=10 {
        let progress = (i as f64) / 10.0;
        manager
            .update_progress(
                &task.id,
                TaskProgress::new(progress).with_message(format!("Progress: {}%", i * 10)),
            )
            .await?;

        let status = manager.get_task_status(&task.id).await?;
        if let Some(prog) = &status.progress {
            println!(
                "  📊 Progress: {:.0}% - {}",
                prog.value * 100.0,
                prog.message.as_ref().unwrap()
            );
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Complete the task
    manager
        .mark_completed(&task.id, json!({"status": "completed"}))
        .await?;

    println!("  ✅ Task completed!");
    println!(
        "  📦 Result: {:?}",
        manager.get_task_result(&task.id).await?
    );

    Ok(())
}

/// Scenario 3: Task cancellation
async fn task_cancellation() -> Result<(), Box<dyn std::error::Error>> {
    let manager = TaskManager::new();

    // Create a cancellable task
    let request = TaskRequest {
        method: "tools/call".to_string(),
        params: json!({"name": "cancellable_task"}),
        task_hint: Some(TaskHint {
            cancellable: true,
            ..Default::default()
        }),
        ..Default::default()
    };

    let task = manager.create_task(request).await?;
    println!("  ✨ Task created: {}", task.id);
    println!("  ✅ Task is cancellable: true");

    // Start working on the task
    manager.mark_working(&task.id).await?;
    println!("  🔨 Task started working...");

    // Cancel the task
    manager.cancel_task(&task.id).await?;
    println!("  ❌ Task cancelled!");

    let status = manager.get_task_status(&task.id).await?;
    println!("  🎭 Final state: {:?}", status.state);
    assert_eq!(status.state, TaskState::Cancelled);

    Ok(())
}

/// Scenario 4: Task priorities
async fn task_priorities() -> Result<(), Box<dyn std::error::Error>> {
    let manager = TaskManager::new();

    println!("  📊 Creating tasks with different priorities...");

    // Create tasks with different priorities
    let priorities = vec![
        TaskPriority::Low,
        TaskPriority::Normal,
        TaskPriority::High,
        TaskPriority::Urgent,
    ];

    for priority in &priorities {
        let request = TaskRequest {
            method: "tools/call".to_string(),
            params: json!({"name": "priority_task"}),
            priority: Some(*priority),
            ..Default::default()
        };

        let task = manager.create_task(request).await?;
        println!("  🎯 Task created with priority: {:?}", priority);
        println!("     Task ID: {}", task.id);
    }

    // List all tasks
    let tasks = manager.list_tasks().await?;
    println!("  📋 Total tasks: {}", tasks.len());

    Ok(())
}

/// Scenario 5: Error handling
async fn error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let manager = TaskManager::new();

    // Create a task
    let request = TaskRequest {
        method: "tools/call".to_string(),
        params: json!({"name": "failing_task"}),
        ..Default::default()
    };

    let task = manager.create_task(request).await?;
    println!("  ✨ Task created: {}", task.id);

    // Mark task as failed
    manager.mark_working(&task.id).await?;
    manager
        .mark_failed(&task.id, "Something went wrong!")
        .await?;

    // Check status
    let status = manager.get_task_status(&task.id).await?;
    println!("  🎭 State: {:?}", status.state);
    println!("  💥 Error: {}", status.error.unwrap());

    // Try to get result from failed task
    let result = manager.get_task_result(&task.id).await;
    println!("  ⚠️  Get result error: {:?}", result.is_err());

    Ok(())
}

/// Scenario 6: List and cleanup
async fn list_and_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let manager = TaskManager::new();

    println!("  📊 Creating multiple tasks...");

    // Create several tasks
    for i in 0..=5 {
        let request = TaskRequest {
            method: "tools/call".to_string(),
            params: json!({"name": format!("task_{}", i)}),
            ..Default::default()
        };

        let task = manager.create_task(request).await?;

        // Complete half of them
        if i % 2 == 0 {
            manager.mark_completed(&task.id, json!({"task": i})).await?;
        }
    }

    // List all tasks
    let tasks = manager.list_tasks().await?;
    println!("  📋 Total tasks: {}", tasks.len());

    let active = tasks.iter().filter(|t| t.is_active()).count();
    let terminal = tasks.iter().filter(|t| t.is_terminal()).count();
    println!("  🔄 Active tasks: {}", active);
    println!("  ✅ Terminal tasks: {}", terminal);

    // Cleanup old tasks (older than 0 seconds)
    let cleaned = manager
        .cleanup_old_tasks(chrono::Duration::seconds(0))
        .await?;
    println!("  🧹 Cleaned up {} completed tasks", cleaned);

    let remaining = manager.list_tasks().await?;
    println!("  📋 Remaining tasks: {}", remaining.len());

    Ok(())
}
