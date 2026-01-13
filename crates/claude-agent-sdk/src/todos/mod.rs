//! Todo List management for Claude Agent SDK
//!
//! This module provides functionality for managing todo lists within the SDK,
//! allowing agents and users to track tasks and their completion status.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Todo status
///
/// Represents the completion status of a todo item.
///
/// # Variants
///
/// * `Pending` - Todo item is not yet started
/// * `InProgress` - Todo item is currently being worked on
/// * `Completed` - Todo item has been completed
///
/// # Example
///
/// ```
/// use claude_agent_sdk::todos::TodoStatus;
///
/// let status = TodoStatus::Pending;
/// assert_eq!(status, TodoStatus::Pending);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TodoStatus {
    /// Todo item is not yet started
    Pending,

    /// Todo item is currently being worked on
    InProgress,

    /// Todo item has been completed
    Completed,
}

impl TodoStatus {
    /// Check if the status is a completed state
    ///
    /// # Returns
    ///
    /// `true` if status is `Completed`, `false` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoStatus;
    /// assert!(!TodoStatus::Pending.is_completed());
    /// assert!(TodoStatus::Completed.is_completed());
    /// ```
    pub fn is_completed(&self) -> bool {
        matches!(self, TodoStatus::Completed)
    }

    /// Check if the status is an active state
    ///
    /// # Returns
    ///
    /// `true` if status is `Pending` or `InProgress`, `false` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoStatus;
    /// assert!(TodoStatus::Pending.is_active());
    /// assert!(TodoStatus::InProgress.is_active());
    /// assert!(!TodoStatus::Completed.is_active());
    /// ```
    pub fn is_active(&self) -> bool {
        matches!(self, TodoStatus::Pending | TodoStatus::InProgress)
    }
}

/// A todo item in a todo list
///
/// Represents a single task with content and status.
///
/// # Example
///
/// ```
/// use claude_agent_sdk::todos::TodoItem;
/// use std::str::FromStr;
///
/// let item = TodoItem {
///     id: "123".to_string(),
///     content: "Write documentation".to_string(),
///     status: claude_agent_sdk::todos::TodoStatus::Pending,
///     created_at: chrono::Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    /// Unique identifier for the todo item
    pub id: String,

    /// Content/description of the todo item
    pub content: String,

    /// Current status of the todo item
    pub status: TodoStatus,

    /// Timestamp when the todo item was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl TodoItem {
    /// Create a new todo item
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the todo item
    /// * `content` - Content/description of the todo item
    ///
    /// # Returns
    ///
    /// A new TodoItem with Pending status
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoItem;
    /// let item = TodoItem::new("123", "Write docs");
    /// assert_eq!(item.status, claude_agent_sdk::todos::TodoStatus::Pending);
    /// ```
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            status: TodoStatus::Pending,
            created_at: chrono::Utc::now(),
        }
    }

    /// Mark the todo item as completed
    ///
    /// # Returns
    ///
    /// A modified TodoItem with Completed status
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoItem;
    /// let mut item = TodoItem::new("123", "Write docs");
    /// item.complete();
    /// assert!(item.status.is_completed());
    /// ```
    pub fn complete(&mut self) {
        self.status = TodoStatus::Completed;
    }

    /// Mark the todo item as in progress
    ///
    /// # Returns
    ///
    /// A modified TodoItem with InProgress status
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoItem;
    /// let mut item = TodoItem::new("123", "Write docs");
    /// item.start();
    /// assert_eq!(item.status, claude_agent_sdk::todos::TodoStatus::InProgress);
    /// ```
    pub fn start(&mut self) {
        self.status = TodoStatus::InProgress;
    }

    /// Reset the todo item to pending
    ///
    /// # Returns
    ///
    /// A modified TodoItem with Pending status
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoItem;
    /// let mut item = TodoItem::new("123", "Write docs");
    /// item.complete();
    /// item.reset();
    /// assert_eq!(item.status, claude_agent_sdk::todos::TodoStatus::Pending);
    /// ```
    pub fn reset(&mut self) {
        self.status = TodoStatus::Pending;
    }
}

/// A todo list containing multiple todo items
///
/// # Example
///
/// ```
/// use claude_agent_sdk::todos::TodoList;
///
/// let mut list = TodoList::new("Project Tasks");
/// list.add("Task 1");
/// list.add("Task 2");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoList {
    /// Unique identifier for the todo list
    pub id: String,

    /// Name of the todo list
    pub name: String,

    /// Todo items in the list
    pub items: Vec<TodoItem>,
}

impl TodoList {
    /// Create a new todo list
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the todo list
    ///
    /// # Returns
    ///
    /// A new TodoList with a unique ID
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoList;
    /// let list = TodoList::new("My Tasks");
    /// assert!(!list.id.is_empty());
    /// assert_eq!(list.name, "My Tasks");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            items: Vec::new(),
        }
    }

    /// Add a new todo item to the list
    ///
    /// # Arguments
    ///
    /// * `content` - Content/description for the new todo item
    ///
    /// # Returns
    ///
    /// Reference to the newly added todo item
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoList;
    /// # let mut list = TodoList::new("My Tasks");
    /// let item = list.add("Write documentation");
    /// assert_eq!(item.content, "Write documentation");
    /// ```
    pub fn add(&mut self, content: impl Into<String>) -> &TodoItem {
        let item = TodoItem::new(uuid::Uuid::new_v4().to_string(), content);
        self.items.push(item);
        self.items.last().unwrap()
    }

    /// Complete a todo item by ID
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the todo item to complete
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, Err(TodoError) if item not found
    ///
    /// # Errors
    ///
    /// Returns `TodoError::NotFound` if the item doesn't exist
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoList;
    /// # let mut list = TodoList::new("My Tasks");
    /// # let item = list.add("Write docs");
    /// // Complete the item
    /// # let result = list.complete(&item.id);
    /// # assert!(result.is_ok());
    /// ```
    pub fn complete(&mut self, id: &str) -> Result<(), TodoError> {
        let item = self
            .items
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| TodoError::NotFound(id.to_string()))?;

        item.complete();
        Ok(())
    }

    /// Start a todo item by ID
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the todo item to start
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, Err(TodoError) if item not found
    ///
    /// # Errors
    ///
    /// Returns `TodoError::NotFound` if the item doesn't exist
    pub fn start(&mut self, id: &str) -> Result<(), TodoError> {
        let item = self
            .items
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| TodoError::NotFound(id.to_string()))?;

        item.start();
        Ok(())
    }

    /// Reset a todo item to pending by ID
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the todo item to reset
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, Err(TodoError) if item not found
    ///
    /// # Errors
    ///
    /// Returns `TodoError::NotFound` if the item doesn't exist
    pub fn reset(&mut self, id: &str) -> Result<(), TodoError> {
        let item = self
            .items
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| TodoError::NotFound(id.to_string()))?;

        item.reset();
        Ok(())
    }

    /// Remove a todo item by ID
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the todo item to remove
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, Err(TodoError) if item not found
    ///
    /// # Errors
    ///
    /// Returns `TodoError::NotFound` if the item doesn't exist
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoList;
    /// # let mut list = TodoList::new("My Tasks");
    /// # let item = list.add("Write docs");
    /// // Remove the item
    /// # let result = list.remove(&item.id);
    /// # assert!(result.is_ok());
    /// # assert_eq!(list.items.len(), 0);
    /// ```
    pub fn remove(&mut self, id: &str) -> Result<(), TodoError> {
        let index = self
            .items
            .iter()
            .position(|item| item.id == id)
            .ok_or_else(|| TodoError::NotFound(id.to_string()))?;

        self.items.remove(index);
        Ok(())
    }

    /// Get a todo item by ID
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the todo item to retrieve
    ///
    /// # Returns
    ///
    /// Some(item) if found, None otherwise
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoList;
    /// # let mut list = TodoList::new("My Tasks");
    /// # let item = list.add("Write docs");
    /// let found = list.get(&item.id);
    /// assert!(found.is_some());
    /// ```
    pub fn get(&self, id: &str) -> Option<&TodoItem> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Get all todo items with a specific status
    ///
    /// # Arguments
    ///
    /// * `status` - Status to filter by
    ///
    /// # Returns
    ///
    /// Vector of todo items with the specified status
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::{TodoList, TodoStatus};
    /// # let mut list = TodoList::new("My Tasks");
    /// # list.add("Task 1");
    /// # list.add("Task 2");
    /// # let pending = list.filter_by_status(TodoStatus::Pending);
    /// # assert_eq!(pending.len(), 2);
    /// ```
    pub fn filter_by_status(&self, status: TodoStatus) -> Vec<&TodoItem> {
        self.items
            .iter()
            .filter(|item| item.status == status)
            .collect()
    }

    /// Get the count of items by status
    ///
    /// # Returns
    ///
    /// HashMap mapping status to count
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoList;
    /// # let mut list = TodoList::new("My Tasks");
    /// # list.add("Task 1");
    /// # list.add("Task 2");
    /// let counts = list.count_by_status();
    /// # assert_eq!(counts.get(&claude_agent_sdk::todos::TodoStatus::Pending).copied(), Some(2));
    /// ```
    pub fn count_by_status(&self) -> HashMap<TodoStatus, usize> {
        let mut counts = HashMap::new();
        for item in &self.items {
            *counts.entry(item.status).or_insert(0) += 1;
        }
        counts
    }

    /// Get the total number of todo items
    ///
    /// # Returns
    ///
    /// Total count of todo items
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoList;
    /// # let mut list = TodoList::new("My Tasks");
    /// # assert_eq!(list.len(), 0);
    /// # list.add("Task 1");
    /// # assert_eq!(list.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the todo list is empty
    ///
    /// # Returns
    ///
    /// `true` if there are no items, `false` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoList;
    /// # let list = TodoList::new("My Tasks");
    /// # assert!(list.is_empty());
    /// # list.add("Task 1");
    /// # assert!(!list.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of completed items
    ///
    /// # Returns
    ///
    /// Count of completed todo items
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoList;
    /// # let mut list = TodoList::new("My Tasks");
    /// # list.add("Task 1");
    /// # list.complete(&list.items[0].id);
    /// # assert_eq!(list.completed_count(), 1);
    /// ```
    pub fn completed_count(&self) -> usize {
        self.items.iter().filter(|item| item.status.is_completed()).count()
    }

    /// Calculate completion percentage
    ///
    /// # Returns
    ///
    /// Percentage of completed items (0-100), or 0 if empty
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::todos::TodoList;
    /// # let mut list = TodoList::new("My Tasks");
    /// # list.add("Task 1");
    /// # list.add("Task 2");
    /// # list.complete(&list.items[0].id);
    /// # assert_eq!(list.completion_percentage(), 50);
    /// ```
    pub fn completion_percentage(&self) -> f64 {
        if self.is_empty() {
            return 0.0;
        }
        (self.completed_count() as f64 / self.len() as f64) * 100.0
    }
}

/// Errors that can occur in todo operations
///
/// # Variants
///
/// * `NotFound` - Todo item not found
/// * `InvalidInput` - Invalid input provided
///
/// # Example
///
/// ```
/// use claude_agent_sdk::todos::TodoError;
///
/// let error = TodoError::NotFound("123".to_string());
/// assert_eq!(format!("{}", error), "Todo item not found: 123");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TodoError {
    /// Todo item not found
    NotFound(String),

    /// Invalid input provided
    InvalidInput(String),
}

impl std::fmt::Display for TodoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoError::NotFound(id) => write!(f, "Todo item not found: {}", id),
            TodoError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for TodoError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_status() {
        let status = TodoStatus::Pending;
        assert!(!status.is_completed());
        assert!(status.is_active());

        let status = TodoStatus::InProgress;
        assert!(!status.is_completed());
        assert!(status.is_active());

        let status = TodoStatus::Completed;
        assert!(status.is_completed());
        assert!(!status.is_active());
    }

    #[test]
    fn test_todo_item_creation() {
        let item = TodoItem::new("123", "Test task");
        assert_eq!(item.id, "123");
        assert_eq!(item.content, "Test task");
        assert_eq!(item.status, TodoStatus::Pending);
    }

    #[test]
    fn test_todo_item_complete() {
        let mut item = TodoItem::new("123", "Test task");
        item.complete();
        assert!(item.status.is_completed());
    }

    #[test]
    fn test_todo_item_start() {
        let mut item = TodoItem::new("123", "Test task");
        item.start();
        assert_eq!(item.status, TodoStatus::InProgress);
    }

    #[test]
    fn test_todo_item_reset() {
        let mut item = TodoItem::new("123", "Test task");
        item.complete();
        item.reset();
        assert_eq!(item.status, TodoStatus::Pending);
    }

    #[test]
    fn test_todo_list_creation() {
        let list = TodoList::new("My Tasks");
        assert_eq!(list.name, "My Tasks");
        assert!(!list.id.is_empty());
        assert!(list.is_empty());
    }

    #[test]
    fn test_todo_list_add() {
        let mut list = TodoList::new("My Tasks");
        let item = list.add("Task 1");
        assert_eq!(item.content, "Task 1");
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_todo_list_complete() {
        let mut list = TodoList::new("My Tasks");
        list.add("Task 1");
        let id = list.items[0].id.clone();

        let result = list.complete(&id);
        assert!(result.is_ok());
        assert!(list.items[0].status.is_completed());
    }

    #[test]
    fn test_todo_list_complete_not_found() {
        let mut list = TodoList::new("My Tasks");
        let result = list.complete("nonexistent");
        assert!(matches!(result, Err(TodoError::NotFound(_))));
    }

    #[test]
    fn test_todo_list_remove() {
        let mut list = TodoList::new("My Tasks");
        list.add("Task 1");
        let id = list.items[0].id.clone();

        let result = list.remove(&id);
        assert!(result.is_ok());
        assert!(list.is_empty());
    }

    #[test]
    fn test_todo_list_get() {
        let mut list = TodoList::new("My Tasks");
        list.add("Task 1");
        let id = list.items[0].id.clone();

        let item = list.get(&id);
        assert!(item.is_some());
        assert_eq!(item.unwrap().content, "Task 1");

        let not_found = list.get("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_todo_list_filter_by_status() {
        let mut list = TodoList::new("My Tasks");
        list.add("Task 1");
        list.add("Task 2");
        let id = list.items[0].id.clone();
        list.complete(&id).unwrap();

        let pending = list.filter_by_status(TodoStatus::Pending);
        assert_eq!(pending.len(), 1);

        let completed = list.filter_by_status(TodoStatus::Completed);
        assert_eq!(completed.len(), 1);
    }

    #[test]
    fn test_todo_list_count_by_status() {
        let mut list = TodoList::new("My Tasks");
        list.add("Task 1");
        list.add("Task 2");
        list.add("Task 3");
        let id = list.items[0].id.clone();
        list.complete(&id).unwrap();

        let counts = list.count_by_status();
        assert_eq!(*counts.get(&TodoStatus::Pending).unwrap_or(&0), 2);
        assert_eq!(*counts.get(&TodoStatus::Completed).unwrap_or(&0), 1);
    }

    #[test]
    fn test_todo_list_completed_count() {
        let mut list = TodoList::new("My Tasks");
        list.add("Task 1");
        list.add("Task 2");
        assert_eq!(list.completed_count(), 0);

        let id = list.items[0].id.clone();
        list.complete(&id).unwrap();
        assert_eq!(list.completed_count(), 1);
    }

    #[test]
    fn test_todo_list_completion_percentage() {
        let mut list = TodoList::new("My Tasks");
        assert_eq!(list.completion_percentage(), 0.0);

        list.add("Task 1");
        list.add("Task 2");
        let id = list.items[0].id.clone();
        list.complete(&id).unwrap();
        assert_eq!(list.completion_percentage(), 50.0);
    }

    #[test]
    fn test_todo_error_display() {
        let error = TodoError::NotFound("123".to_string());
        assert_eq!(format!("{}", error), "Todo item not found: 123");

        let error = TodoError::InvalidInput("test".to_string());
        assert_eq!(format!("{}", error), "Invalid input: test");
    }

    #[test]
    fn test_todo_list_start() {
        let mut list = TodoList::new("My Tasks");
        list.add("Task 1");
        let id = list.items[0].id.clone();

        let result = list.start(&id);
        assert!(result.is_ok());
        assert_eq!(list.items[0].status, TodoStatus::InProgress);
    }

    #[test]
    fn test_todo_list_reset() {
        let mut list = TodoList::new("My Tasks");
        list.add("Task 1");
        let id = list.items[0].id.clone();

        list.complete(&id).unwrap();
        assert!(list.items[0].status.is_completed());

        list.reset(&id).unwrap();
        assert_eq!(list.items[0].status, TodoStatus::Pending);
    }
}
