//! MCP (Model Context Protocol) 2025-11-25 Implementation
//!
//! This module provides implementations for the latest MCP protocol features,
//! including async Tasks, OAuth improvements, and extensions.
//!
//! # Modules
//!
//! - [`tasks`] - Async Tasks primitive for "call-now, fetch-later" workflows
//!
//! # Example
//!
//! ```no_run
//! use claude_agent_sdk_rs::mcp::tasks::{TaskManager, TaskRequest};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let manager = TaskManager::new();
//!
//! let request = TaskRequest {
//!     method: "tools/call".to_string(),
//!     params: json!({"name": "my_tool", "arguments": {}}),
//!     ..Default::default()
//! };
//!
//! let task = manager.create_task(request).await?;
//! println!("Task created: {}", task.id);
//! # Ok(())
//! # }
//! ```

pub mod tasks;

pub use tasks::{
    TaskHandle, TaskHint, TaskId, TaskManager, TaskPriority, TaskProgress, TaskRequest,
    TaskResult, TaskState, TaskStatus, TaskUri,
};
