//! Example: Todo List Management
//!
//! This example demonstrates the todo list management functionality
//! for tracking tasks and their completion status.
//!
//! What it demonstrates:
//! 1. Creating todo lists with TodoList::new()
//! 2. Adding items with add()
//! 3. Managing item status: start(), complete(), reset()
//! 4. Filtering and counting items by status
//! 5. Calculating completion percentage

use claude_agent_sdk::todos::{TodoError, TodoItem, TodoList, TodoStatus};

fn main() -> anyhow::Result<()> {
    println!("=== Todo List Management Examples ===\n");

    basic_todo_list_example()?;
    status_management_example()?;
    filtering_and_statistics_example()?;
    error_handling_example()?;

    Ok(())
}

/// Demonstrates basic todo list creation and item management
fn basic_todo_list_example() -> anyhow::Result<()> {
    println!("=== Basic Todo List Example ===\n");

    // Create a new todo list
    let mut todo_list = TodoList::new("Project Tasks");
    println!("Created todo list: '{}' (ID: {})", todo_list.name, todo_list.id);

    // Add items to the list
    let task1 = todo_list.add("Design the architecture");
    println!("Added task: '{}' (ID: {})", task1.content, task1.id);

    let task2 = todo_list.add("Implement core features");
    println!("Added task: '{}' (ID: {})", task2.content, task2.id);

    let task3 = todo_list.add("Write unit tests");
    println!("Added task: '{}' (ID: {})", task3.content, task3.id);

    let task4 = todo_list.add("Create documentation");
    println!("Added task: '{}' (ID: {})", task4.content, task4.id);

    println!("\nTotal tasks: {}", todo_list.len());
    println!();

    Ok(())
}

/// Demonstrates status management for todo items
fn status_management_example() -> anyhow::Result<()> {
    println!("=== Status Management Example ===\n");

    let mut todo_list = TodoList::new("Sprint Tasks");

    // Add some tasks
    let id1 = todo_list.add("Task A: Setup environment").id.clone();
    let id2 = todo_list.add("Task B: Write code").id.clone();
    let id3 = todo_list.add("Task C: Code review").id.clone();
    let id4 = todo_list.add("Task D: Deploy").id.clone();

    println!("Initial state:");
    print_todo_status(&todo_list);

    // Start working on Task A
    todo_list.start(&id1)?;
    println!("\nAfter starting Task A:");
    print_todo_status(&todo_list);

    // Complete Task A
    todo_list.complete(&id1)?;
    println!("\nAfter completing Task A:");
    print_todo_status(&todo_list);

    // Start and complete Task B
    todo_list.start(&id2)?;
    todo_list.complete(&id2)?;
    println!("\nAfter completing Task B:");
    print_todo_status(&todo_list);

    // Reset Task A (needs rework)
    todo_list.reset(&id1)?;
    println!("\nAfter resetting Task A:");
    print_todo_status(&todo_list);

    println!();
    Ok(())
}

/// Demonstrates filtering and statistics
fn filtering_and_statistics_example() -> anyhow::Result<()> {
    println!("=== Filtering and Statistics Example ===\n");

    let mut todo_list = TodoList::new("Development Tasks");

    // Add 10 tasks
    for i in 1..=10 {
        todo_list.add(&format!("Task {}", i));
    }

    // Complete some tasks using direct status update to avoid borrow issues
    let ids: Vec<String> = todo_list.items.iter().map(|i| i.id.clone()).collect();
    for (i, id) in ids.iter().enumerate() {
        if i < 3 {
            todo_list.complete(id)?;
        } else if i < 5 {
            todo_list.start(id)?;
        }
    }

    // Get counts by status
    let counts = todo_list.count_by_status();
    println!("Tasks by status:");
    println!("  Pending: {}", counts.get(&TodoStatus::Pending).unwrap_or(&0));
    println!(
        "  In Progress: {}",
        counts.get(&TodoStatus::InProgress).unwrap_or(&0)
    );
    println!(
        "  Completed: {}",
        counts.get(&TodoStatus::Completed).unwrap_or(&0)
    );

    // Calculate completion percentage
    println!(
        "\nCompletion: {:.1}% ({}/{} tasks)",
        todo_list.completion_percentage(),
        todo_list.completed_count(),
        todo_list.len()
    );

    // Filter by status
    println!("\nPending tasks:");
    for item in todo_list.filter_by_status(TodoStatus::Pending) {
        println!("  - {}", item.content);
    }

    println!("\nIn progress tasks:");
    for item in todo_list.filter_by_status(TodoStatus::InProgress) {
        println!("  - {}", item.content);
    }

    println!("\nCompleted tasks:");
    for item in todo_list.filter_by_status(TodoStatus::Completed) {
        println!("  - {}", item.content);
    }

    // Get a specific item
    if let Some(first_item) = todo_list.items.first() {
        let found = todo_list.get(&first_item.id);
        if let Some(item) = found {
            println!("\nRetrieved item by ID: '{}'", item.content);
        }
    }

    // Remove a task - clone the data we need before mutating
    let pending_to_remove = todo_list
        .filter_by_status(TodoStatus::Pending)
        .first()
        .map(|item| (item.id.clone(), item.content.clone()));

    if let Some((id, content)) = pending_to_remove {
        todo_list.remove(&id)?;
        println!("\nRemoved task: '{}'", content);
        println!("Remaining tasks: {}", todo_list.len());
    }

    println!();
    Ok(())
}

/// Demonstrates error handling
fn error_handling_example() -> anyhow::Result<()> {
    println!("=== Error Handling Example ===\n");

    let mut todo_list = TodoList::new("Test Tasks");
    todo_list.add("Only task");

    // Try to complete a non-existent task
    match todo_list.complete("non-existent-id") {
        Err(TodoError::NotFound(id)) => {
            println!("Expected error: Task '{}' not found", id);
        }
        Err(e) => {
            println!("Unexpected error: {}", e);
        }
        Ok(_) => {
            println!("Unexpected success");
        }
    }

    // Try to start a non-existent task
    match todo_list.start("invalid-id") {
        Err(TodoError::NotFound(id)) => {
            println!("Expected error: Task '{}' not found", id);
        }
        Err(e) => {
            println!("Unexpected error: {}", e);
        }
        Ok(_) => {
            println!("Unexpected success");
        }
    }

    // Try to remove a non-existent task
    match todo_list.remove("missing-id") {
        Err(TodoError::NotFound(id)) => {
            println!("Expected error: Task '{}' not found", id);
        }
        Err(e) => {
            println!("Unexpected error: {}", e);
        }
        Ok(_) => {
            println!("Unexpected success");
        }
    }

    // Test TodoStatus methods
    println!("\nTodoStatus methods:");
    println!(
        "Pending.is_completed(): {}",
        TodoStatus::Pending.is_completed()
    );
    println!("Pending.is_active(): {}", TodoStatus::Pending.is_active());
    println!(
        "InProgress.is_completed(): {}",
        TodoStatus::InProgress.is_completed()
    );
    println!(
        "InProgress.is_active(): {}",
        TodoStatus::InProgress.is_active()
    );
    println!(
        "Completed.is_completed(): {}",
        TodoStatus::Completed.is_completed()
    );
    println!("Completed.is_active(): {}", TodoStatus::Completed.is_active());

    println!();
    Ok(())
}

/// Helper function to print todo status
fn print_todo_status(todo_list: &TodoList) {
    for item in &todo_list.items {
        let status_icon = match item.status {
            TodoStatus::Pending => "⏳",
            TodoStatus::InProgress => "🔄",
            TodoStatus::Completed => "✅",
        };
        println!("  {} {} - {}", status_icon, item.content, item.status_display());
    }
}

/// Helper trait for status display
trait StatusDisplay {
    fn status_display(&self) -> &'static str;
}

impl StatusDisplay for TodoItem {
    fn status_display(&self) -> &'static str {
        match self.status {
            TodoStatus::Pending => "Pending",
            TodoStatus::InProgress => "In Progress",
            TodoStatus::Completed => "Completed",
        }
    }
}
