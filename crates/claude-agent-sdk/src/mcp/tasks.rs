//! MCP 2025-11-25 Async Tasks Implementation
//!
//! This module implements the Tasks primitive from the MCP 2025-11-25 spec,
//! enabling "call-now, fetch-later" asynchronous workflows.
//!
//! # Overview
//!
//! The Tasks primitive allows any request to become asynchronous:
//! - Client calls a tool with a task hint
//! - Server returns a task handle immediately
//! - Client polls or subscribes to the task resource for progress and results
//!
//! # Task States
//!
//! Tasks move through these states:
//! - `Queued` - Task is waiting to start
//! - `Working` - Task is in progress
//! - `InputRequired` - Task needs user input
//! - `Completed` - Task finished successfully
//! - `Failed` - Task failed with an error
//! - `Cancelled` - Task was cancelled
//!
//! # Example
//!
//! ```no_run
//! use claude_agent_sdk::mcp::tasks::{TaskManager, TaskStatus, TaskRequest};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let manager = TaskManager::new();
//!
//! // Create a task
//! let request = TaskRequest {
//!     method: "tools/call".to_string(),
//!     params: json!({"name": "my_tool", "arguments": {}}),
//!     ..Default::default()
//! };
//!
//! let task = manager.create_task(request).await?;
//! println!("Task ID: {}", task.id);
//!
//! // Poll for status
//! loop {
//!     let status = manager.get_task_status(&task.id).await?;
//!     if status.is_terminal() {
//!         break;
//!     }
//!     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
//! }
//!
//! // Get result
//! let result = manager.get_task_result(&task.id).await?;
//! println!("Result: {:?}", result);
//! # Ok(())
//! # }
//! ```

use crate::errors::{ClaudeError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Task ID
pub type TaskId = String;

/// Task resource URI
pub type TaskUri = String;

/// Task request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskRequest {
    /// JSON-RPC method name
    #[serde(default)]
    pub method: String,
    /// Request parameters
    #[serde(default)]
    pub params: serde_json::Value,
    /// Task hint - indicates this might take a while
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_hint: Option<TaskHint>,
    /// Priority hint for task scheduling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<TaskPriority>,
}

/// Task hint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHint {
    /// Estimated duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration_secs: Option<u64>,
    /// Whether progress notifications will be sent
    #[serde(default)]
    pub supports_progress: bool,
    /// Whether the task can be cancelled
    #[serde(default)]
    pub cancellable: bool,
}

impl Default for TaskHint {
    fn default() -> Self {
        Self {
            estimated_duration_secs: None,
            supports_progress: false,
            cancellable: true,
        }
    }
}

/// Task priority
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

/// Task state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    Queued,
    Working,
    InputRequired,
    Completed,
    Failed,
    Cancelled,
}

impl TaskState {
    /// Check if this is a terminal state (no further transitions possible)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Check if this state represents an active task
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Queued | Self::Working | Self::InputRequired)
    }
}

/// Task progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    /// Progress value between 0.0 and 1.0
    pub value: f64,
    /// Human-readable progress message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl TaskProgress {
    /// Create new progress
    pub fn new(value: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&value),
            "Progress must be between 0.0 and 1.0"
        );
        Self {
            value,
            message: None,
        }
    }

    /// Add a message to the progress
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    /// Task ID
    pub id: TaskId,
    /// Task state
    pub state: TaskState,
    /// Current progress (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<TaskProgress>,
    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Timestamp when task was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when task was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when task completed (if terminal)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TaskStatus {
    /// Check if task is in a terminal state
    pub fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }

    /// Check if task is still active
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }
}

/// Task result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task ID
    pub id: TaskId,
    /// Result data
    pub data: serde_json::Value,
    /// Timestamp when result was produced
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Task handle - returned immediately when creating a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHandle {
    /// Task ID
    pub id: TaskId,
    /// Task resource URI for polling/subscribing
    pub uri: TaskUri,
    /// Initial status
    pub status: TaskStatus,
}

/// Internal task storage
#[derive(Debug, Clone)]
struct Task {
    id: TaskId,
    request: TaskRequest,
    state: TaskState,
    progress: Option<TaskProgress>,
    result: Option<serde_json::Value>,
    error: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Task {
    fn new(request: TaskRequest) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            request,
            state: TaskState::Queued,
            progress: None,
            result: None,
            error: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    fn to_status(&self) -> TaskStatus {
        TaskStatus {
            id: self.id.clone(),
            state: self.state.clone(),
            progress: self.progress.clone(),
            error: self.error.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            completed_at: self.completed_at,
        }
    }
}

/// Task manager
///
/// Manages the lifecycle of async tasks, including creation,
/// status polling, progress updates, and result retrieval.
#[derive(Clone)]
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<TaskId, Task>>>,
    base_uri: String,
}

impl TaskManager {
    /// Create a new task manager
    pub fn new() -> Self {
        Self::with_base_uri("mcp://tasks".to_string())
    }

    /// Create a new task manager with a custom base URI
    pub fn with_base_uri(base_uri: impl Into<String>) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            base_uri: base_uri.into(),
        }
    }

    /// Create a new task
    ///
    /// Returns a task handle immediately with the task in Queued state.
    pub async fn create_task(&self, request: TaskRequest) -> Result<TaskHandle> {
        let task = Task::new(request);
        let task_id = task.id.clone();
        let uri = format!("{}/{}", self.base_uri, task_id);
        let status = task.to_status();

        // Store the task
        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.clone(), task);

        Ok(TaskHandle {
            id: task_id,
            uri,
            status,
        })
    }

    /// Get task status
    pub async fn get_task_status(&self, task_id: &TaskId) -> Result<TaskStatus> {
        let tasks = self.tasks.read().await;
        let task = tasks
            .get(task_id)
            .ok_or_else(|| ClaudeError::NotFound(format!("Task not found: {}", task_id)))?;

        Ok(task.to_status())
    }

    /// Get task result
    ///
    /// Returns an error if the task hasn't completed yet.
    pub async fn get_task_result(&self, task_id: &TaskId) -> Result<TaskResult> {
        let tasks = self.tasks.read().await;
        let task = tasks
            .get(task_id)
            .ok_or_else(|| ClaudeError::NotFound(format!("Task not found: {}", task_id)))?;

        if task.state != TaskState::Completed {
            return Err(ClaudeError::InvalidInput(format!(
                "Task is not completed. Current state: {:?}",
                task.state
            )));
        }

        let result = task.result.as_ref().ok_or_else(|| {
            ClaudeError::InternalError("Completed task has no result".to_string())
        })?;

        Ok(TaskResult {
            id: task_id.clone(),
            data: result.clone(),
            completed_at: task.completed_at.unwrap(),
        })
    }

    /// Update task progress
    ///
    /// This should be called by the worker executing the task.
    pub async fn update_progress(&self, task_id: &TaskId, progress: TaskProgress) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| ClaudeError::NotFound(format!("Task not found: {}", task_id)))?;

        if task.state.is_terminal() {
            return Err(ClaudeError::InvalidInput(
                "Cannot update progress for terminal task".to_string(),
            ));
        }

        task.progress = Some(progress);
        task.updated_at = chrono::Utc::now();

        Ok(())
    }

    /// Mark task as working
    pub async fn mark_working(&self, task_id: &TaskId) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| ClaudeError::NotFound(format!("Task not found: {}", task_id)))?;

        if task.state.is_terminal() {
            return Err(ClaudeError::InvalidInput(
                "Cannot transition terminal task".to_string(),
            ));
        }

        task.state = TaskState::Working;
        task.updated_at = chrono::Utc::now();

        Ok(())
    }

    /// Mark task as completed with result
    pub async fn mark_completed(&self, task_id: &TaskId, result: serde_json::Value) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| ClaudeError::NotFound(format!("Task not found: {}", task_id)))?;

        if task.state.is_terminal() {
            return Err(ClaudeError::InvalidInput(
                "Cannot transition terminal task".to_string(),
            ));
        }

        let now = chrono::Utc::now();
        task.state = TaskState::Completed;
        task.result = Some(result);
        task.updated_at = now;
        task.completed_at = Some(now);

        Ok(())
    }

    /// Mark task as failed
    pub async fn mark_failed(&self, task_id: &TaskId, error: impl Into<String>) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| ClaudeError::NotFound(format!("Task not found: {}", task_id)))?;

        if task.state.is_terminal() {
            return Err(ClaudeError::InvalidInput(
                "Cannot transition terminal task".to_string(),
            ));
        }

        let now = chrono::Utc::now();
        task.state = TaskState::Failed;
        task.error = Some(error.into());
        task.updated_at = now;
        task.completed_at = Some(now);

        Ok(())
    }

    /// Mark task as cancelled
    pub async fn mark_cancelled(&self, task_id: &TaskId) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| ClaudeError::NotFound(format!("Task not found: {}", task_id)))?;

        if task.state.is_terminal() {
            return Err(ClaudeError::InvalidInput(
                "Cannot transition terminal task".to_string(),
            ));
        }

        let now = chrono::Utc::now();
        task.state = TaskState::Cancelled;
        task.updated_at = now;
        task.completed_at = Some(now);

        Ok(())
    }

    /// Mark task as requiring input
    pub async fn mark_input_required(&self, task_id: &TaskId) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| ClaudeError::NotFound(format!("Task not found: {}", task_id)))?;

        if task.state.is_terminal() {
            return Err(ClaudeError::InvalidInput(
                "Cannot transition terminal task".to_string(),
            ));
        }

        task.state = TaskState::InputRequired;
        task.updated_at = chrono::Utc::now();

        Ok(())
    }

    /// List all tasks
    pub async fn list_tasks(&self) -> Result<Vec<TaskStatus>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.values().map(|t| t.to_status()).collect())
    }

    /// Cancel a task
    ///
    /// Returns an error if the task is already in a terminal state
    /// or doesn't support cancellation.
    pub async fn cancel_task(&self, task_id: &TaskId) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| ClaudeError::NotFound(format!("Task not found: {}", task_id)))?;

        if task.state.is_terminal() {
            return Err(ClaudeError::InvalidInput(format!(
                "Cannot cancel task in state: {:?}",
                task.state
            )));
        }

        // Check if task is cancellable (based on request hint)
        if let Some(hint) = &task.request.task_hint {
            if !hint.cancellable {
                return Err(ClaudeError::InvalidInput(
                    "Task is not cancellable".to_string(),
                ));
            }
        }

        let now = chrono::Utc::now();
        task.state = TaskState::Cancelled;
        task.updated_at = now;
        task.completed_at = Some(now);

        Ok(())
    }

    /// Clean up old completed tasks
    ///
    /// Removes tasks that completed before the given threshold.
    pub async fn cleanup_old_tasks(&self, older_than: chrono::Duration) -> Result<usize> {
        let mut tasks = self.tasks.write().await;
        let cutoff = chrono::Utc::now() - older_than;

        let initial_count = tasks.len();
        tasks.retain(|_, task| {
            if let Some(completed_at) = task.completed_at {
                completed_at > cutoff
            } else {
                true // Keep active tasks
            }
        });

        Ok(initial_count - tasks.len())
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_task_creation() {
        let manager = TaskManager::new();

        let request = TaskRequest {
            method: "tools/call".to_string(),
            params: json!({"name": "test"}),
            ..Default::default()
        };

        let handle = manager.create_task(request).await.unwrap();
        assert!(!handle.id.is_empty());
        assert!(!handle.uri.is_empty());
        assert_eq!(handle.status.state, TaskState::Queued);
    }

    #[tokio::test]
    async fn test_task_status() {
        let manager = TaskManager::new();

        let request = TaskRequest {
            method: "tools/call".to_string(),
            params: json!({}),
            ..Default::default()
        };

        let handle = manager.create_task(request).await.unwrap();
        let status = manager.get_task_status(&handle.id).await.unwrap();

        assert_eq!(status.id, handle.id);
        assert_eq!(status.state, TaskState::Queued);
        assert!(status.is_active());
        assert!(!status.is_terminal());
    }

    #[tokio::test]
    async fn test_task_lifecycle() {
        let manager = TaskManager::new();

        let request = TaskRequest {
            method: "tools/call".to_string(),
            params: json!({}),
            ..Default::default()
        };

        let handle = manager.create_task(request).await.unwrap();

        // Transition to Working
        manager.mark_working(&handle.id).await.unwrap();
        let status = manager.get_task_status(&handle.id).await.unwrap();
        assert_eq!(status.state, TaskState::Working);

        // Update progress
        let progress = TaskProgress::new(0.5).with_message("Half done");
        manager.update_progress(&handle.id, progress).await.unwrap();
        let status = manager.get_task_status(&handle.id).await.unwrap();
        assert_eq!(status.progress.as_ref().unwrap().value, 0.5);
        assert_eq!(
            status.progress.as_ref().unwrap().message.as_ref().unwrap(),
            "Half done"
        );

        // Complete with result
        let result = json!({"output": "success"});
        manager.mark_completed(&handle.id, result).await.unwrap();
        let status = manager.get_task_status(&handle.id).await.unwrap();
        assert_eq!(status.state, TaskState::Completed);
        assert!(status.is_terminal());
        assert!(!status.is_active());

        // Get result
        let task_result = manager.get_task_result(&handle.id).await.unwrap();
        assert_eq!(task_result.id, handle.id);
        assert_eq!(task_result.data, json!({"output": "success"}));
    }

    #[tokio::test]
    async fn test_task_failure() {
        let manager = TaskManager::new();

        let request = TaskRequest {
            method: "tools/call".to_string(),
            params: json!({}),
            ..Default::default()
        };

        let handle = manager.create_task(request).await.unwrap();

        manager
            .mark_failed(&handle.id, "Something went wrong")
            .await
            .unwrap();

        let status = manager.get_task_status(&handle.id).await.unwrap();
        assert_eq!(status.state, TaskState::Failed);
        assert!(status.is_terminal());
        assert_eq!(status.error.as_ref().unwrap(), "Something went wrong");
    }

    #[tokio::test]
    async fn test_task_cancellation() {
        let manager = TaskManager::new();

        let request = TaskRequest {
            method: "tools/call".to_string(),
            params: json!({}),
            task_hint: Some(TaskHint {
                cancellable: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        let handle = manager.create_task(request).await.unwrap();
        manager.cancel_task(&handle.id).await.unwrap();

        let status = manager.get_task_status(&handle.id).await.unwrap();
        assert_eq!(status.state, TaskState::Cancelled);
        assert!(status.is_terminal());
    }

    #[tokio::test]
    async fn test_non_cancellable_task() {
        let manager = TaskManager::new();

        let request = TaskRequest {
            method: "tools/call".to_string(),
            params: json!({}),
            task_hint: Some(TaskHint {
                cancellable: false,
                ..Default::default()
            }),
            ..Default::default()
        };

        let handle = manager.create_task(request).await.unwrap();
        let result = manager.cancel_task(&handle.id).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_terminal_state_transitions() {
        let manager = TaskManager::new();

        let request = TaskRequest {
            method: "tools/call".to_string(),
            params: json!({}),
            ..Default::default()
        };

        let handle = manager.create_task(request).await.unwrap();

        // Complete the task
        manager.mark_completed(&handle.id, json!({})).await.unwrap();

        // Try to transition from terminal state
        assert!(manager.mark_working(&handle.id).await.is_err());
        assert!(
            manager
                .update_progress(&handle.id, TaskProgress::new(0.5))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let manager = TaskManager::new();

        let request = TaskRequest {
            method: "tools/call".to_string(),
            params: json!({}),
            ..Default::default()
        };

        let _task1 = manager.create_task(request.clone()).await.unwrap();
        let _task2 = manager.create_task(request).await.unwrap();

        let tasks = manager.list_tasks().await.unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_progress_bounds() {
        // Test progress validation
        assert!(TaskProgress::new(0.0).value == 0.0);
        assert!(TaskProgress::new(0.5).value == 0.5);
        assert!(TaskProgress::new(1.0).value == 1.0);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        assert!(TaskPriority::Low < TaskPriority::Normal);
        assert!(TaskPriority::Normal < TaskPriority::High);
        assert!(TaskPriority::High < TaskPriority::Urgent);
    }

    #[tokio::test]
    async fn test_cleanup_old_tasks() {
        let manager = TaskManager::new();

        let request = TaskRequest {
            method: "tools/call".to_string(),
            params: json!({}),
            ..Default::default()
        };

        let handle = manager.create_task(request).await.unwrap();
        manager.mark_completed(&handle.id, json!({})).await.unwrap();

        // Cleanup tasks older than 1 second (should be none immediately)
        let cleaned = manager
            .cleanup_old_tasks(chrono::Duration::seconds(1))
            .await
            .unwrap();
        assert_eq!(cleaned, 0);

        // Cleanup tasks older than 0 seconds (should clean all)
        let cleaned = manager
            .cleanup_old_tasks(chrono::Duration::seconds(0))
            .await
            .unwrap();
        assert_eq!(cleaned, 1);
    }
}
