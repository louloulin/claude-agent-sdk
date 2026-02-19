//! Example: Slash Commands System
//!
//! This example demonstrates the slash command registration and execution system
//! for creating custom commands with async handlers.
//!
//! What it demonstrates:
//! 1. Creating commands with SlashCommand::new()
//! 2. Registering commands in CommandRegistry
//! 3. Executing commands with arguments
//! 4. Error handling for invalid commands
//! 5. Listing and managing registered commands

use claude_agent_sdk::commands::{CommandError, CommandRegistry, SlashCommand};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Slash Commands System Examples ===\n");

    basic_commands_example().await?;
    command_with_arguments_example().await?;
    error_handling_example().await?;
    registry_management_example().await?;

    Ok(())
}

/// Demonstrates basic command creation and execution
async fn basic_commands_example() -> anyhow::Result<()> {
    println!("=== Basic Commands Example ===\n");

    // Create a new registry
    let mut registry = CommandRegistry::new();
    println!("Created empty registry: {} commands", registry.len());

    // Create a simple "greet" command
    let greet_cmd = SlashCommand::new(
        "greet",
        "Greet the user",
        Arc::new(|_name, args| {
            Box::pin(async move {
                let name = args.first().map(|s| s.as_str()).unwrap_or("World");
                Ok(format!("Hello, {}!", name))
            })
        }),
    );

    registry.register(greet_cmd)?;
    println!("Registered 'greet' command");

    // Create an "echo" command
    let echo_cmd = SlashCommand::new(
        "echo",
        "Echo back the provided arguments",
        Arc::new(|_name, args| {
            Box::pin(async move { Ok(args.join(" ")) })
        }),
    );

    registry.register(echo_cmd)?;
    println!("Registered 'echo' command");

    // Create a "time" command
    let time_cmd = SlashCommand::new(
        "time",
        "Get current time",
        Arc::new(|_name, _args| {
            Box::pin(async move {
                let now = chrono::Utc::now();
                Ok(format!("Current time: {}", now.format("%Y-%m-%d %H:%M:%S UTC")))
            })
        }),
    );

    registry.register(time_cmd)?;
    println!("Registered 'time' command");

    // Execute commands
    println!("\n--- Executing Commands ---");

    let result = registry.execute("greet", vec!["Claude".to_string()]).await?;
    println!("$ /greet Claude");
    println!("> {}", result);

    let result = registry.execute("echo", vec!["Hello".to_string(), "World".to_string()]).await?;
    println!("$ /echo Hello World");
    println!("> {}", result);

    let result = registry.execute("time", vec![]).await?;
    println!("$ /time");
    println!("> {}", result);

    println!();
    Ok(())
}

/// Demonstrates commands that process arguments
async fn command_with_arguments_example() -> anyhow::Result<()> {
    println!("=== Commands with Arguments Example ===\n");

    let mut registry = CommandRegistry::new();

    // Math command: sum numbers
    let sum_cmd = SlashCommand::new(
        "sum",
        "Calculate the sum of numbers",
        Arc::new(|_name, args| {
            Box::pin(async move {
                let numbers: Vec<i64> = args.iter().filter_map(|s| s.parse().ok()).collect();
                let total: i64 = numbers.iter().sum();
                Ok(format!(
                    "Sum of {:?}: {}",
                    numbers, total
                ))
            })
        }),
    );
    registry.register(sum_cmd)?;

    // Stats command: calculate statistics
    let stats_cmd = SlashCommand::new(
        "stats",
        "Calculate statistics for numbers",
        Arc::new(|_name, args| {
            Box::pin(async move {
                let numbers: Vec<f64> = args.iter().filter_map(|s| s.parse().ok()).collect();
                if numbers.is_empty() {
                    return Ok("No valid numbers provided".to_string());
                }

                let count = numbers.len();
                let sum: f64 = numbers.iter().sum();
                let avg = sum / count as f64;
                let min = numbers.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = numbers.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

                Ok(format!(
                    "Statistics: count={}, sum={:.2}, avg={:.2}, min={:.2}, max={:.2}",
                    count, sum, avg, min, max
                ))
            })
        }),
    );
    registry.register(stats_cmd)?;

    // Repeat command: repeat text N times
    let repeat_cmd = SlashCommand::new(
        "repeat",
        "Repeat text N times (usage: /repeat <count> <text>)",
        Arc::new(|_name, args| {
            Box::pin(async move {
                if args.len() < 2 {
                    return Err(CommandError::ExecutionFailed(
                        "Usage: /repeat <count> <text>".to_string(),
                    ));
                }

                let count: usize = args[0].parse().map_err(|_| {
                    CommandError::ExecutionFailed("Invalid count".to_string())
                })?;

                let text = args[1..].join(" ");
                let result = vec![text.as_str(); count].join("\n");
                Ok(result)
            })
        }),
    );
    registry.register(repeat_cmd)?;

    // Execute commands with arguments
    println!("--- Executing Commands with Arguments ---");

    let result = registry.execute("sum", vec!["10".to_string(), "20".to_string(), "30".to_string()]).await?;
    println!("$ /sum 10 20 30");
    println!("> {}", result);

    let result = registry.execute("stats", vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string()]).await?;
    println!("$ /stats 1 2 3 4 5");
    println!("> {}", result);

    let result = registry.execute("repeat", vec!["3".to_string(), "Hello!".to_string()]).await?;
    println!("$ /repeat 3 Hello!");
    println!("> {}", result);

    println!();
    Ok(())
}

/// Demonstrates error handling
async fn error_handling_example() -> anyhow::Result<()> {
    println!("=== Error Handling Example ===\n");

    let mut registry = CommandRegistry::new();

    // Register a command that always fails
    let fail_cmd = SlashCommand::new(
        "fail",
        "A command that always fails",
        Arc::new(|_name, _args| {
            Box::pin(async move {
                Err(CommandError::ExecutionFailed(
                    "This command always fails intentionally".to_string(),
                ))
            })
        }),
    );
    registry.register(fail_cmd)?;

    // Try to execute non-existent command
    match registry.execute("nonexistent", vec![]).await {
        Err(CommandError::NotFound(name)) => {
            println!("✓ Expected error: Command '{}' not found", name);
        }
        Err(e) => println!("Unexpected error: {}", e),
        Ok(_) => println!("Unexpected success"),
    }

    // Try to execute failing command
    match registry.execute("fail", vec![]).await {
        Err(CommandError::ExecutionFailed(msg)) => {
            println!("✓ Expected execution error: {}", msg);
        }
        Err(e) => println!("Unexpected error: {}", e),
        Ok(_) => println!("Unexpected success"),
    }

    // Try to register duplicate command
    let dupe_cmd = SlashCommand::new(
        "fail",
        "Duplicate command",
        Arc::new(|_name, _args| Box::pin(async { Ok(String::new()) })),
    );

    match registry.register(dupe_cmd) {
        Err(CommandError::AlreadyRegistered(name)) => {
            println!("✓ Expected error: Command '{}' already registered", name);
        }
        Err(e) => println!("Unexpected error: {}", e),
        Ok(_) => println!("Unexpected success"),
    }

    // Try to register invalid command names
    let invalid_names = vec!["", "has space", "123starts", "test-cmd", "test_cmd"];

    for name in invalid_names {
        let cmd = SlashCommand::new(
            name,
            "Test",
            Arc::new(|_name, _args| Box::pin(async { Ok(String::new()) })),
        );

        let result = registry.register(cmd);
        if result.is_err() {
            println!("✓ '{}' correctly rejected as invalid name", if name.is_empty() { "<empty>" } else { name });
        } else {
            println!("✓ '{}' accepted as valid name", name);
        }
    }

    println!();
    Ok(())
}

/// Demonstrates registry management
async fn registry_management_example() -> anyhow::Result<()> {
    println!("=== Registry Management Example ===\n");

    let mut registry = CommandRegistry::new();

    // Register multiple commands
    let commands = vec![
        ("help", "Show available commands"),
        ("status", "Show system status"),
        ("version", "Show version info"),
        ("config", "Show configuration"),
        ("debug", "Toggle debug mode"),
    ];

    for (name, desc) in commands {
        let cmd = SlashCommand::new(
            name,
            desc,
            Arc::new(move |_name, _args| {
                let desc = desc.to_string();
                Box::pin(async move { Ok(desc.to_string()) })
            }),
        );
        registry.register(cmd)?;
    }

    println!("Registered {} commands", registry.len());
    println!("Available commands: {:?}", registry.list_names());

    // Check if commands exist
    println!("\nCommand existence check:");
    println!("  'help' exists: {}", registry.exists("help"));
    println!("  'unknown' exists: {}", registry.exists("unknown"));

    // Get command details
    if let Some(cmd) = registry.get("status") {
        println!("\nCommand 'status':");
        println!("  Name: {}", cmd.name);
        println!("  Description: {}", cmd.description);
    }

    // Unregister a command
    registry.unregister("debug")?;
    println!("\nAfter unregistering 'debug':");
    println!("  Commands: {:?}", registry.list_names());

    // Clear all commands
    registry.clear();
    println!("\nAfter clearing:");
    println!("  Commands: {:?}", registry.list_names());
    println!("  Is empty: {}", registry.is_empty());

    println!();
    Ok(())
}
