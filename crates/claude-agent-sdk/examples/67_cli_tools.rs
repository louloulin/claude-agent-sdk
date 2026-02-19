//! Example: CLI Tools - Building Interactive CLI Applications
//!
//! This example demonstrates how to build interactive CLI tools
//! using the Claude Agent SDK with rich terminal features.
//!
//! What it demonstrates:
//! 1. Interactive REPL (Read-Eval-Print Loop)
//! 2. Command parsing and handling
//! 3. Progress indicators
//! 4. Rich output formatting
//! 5. Session management in CLI context
//! 6. Configuration management
//! 7. Error handling and recovery

use anyhow::Result;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::Mutex;

// ============================================================================
// CLI Configuration
// ============================================================================

/// CLI configuration
#[derive(Debug, Clone)]
struct CliConfig {
    model: String,
    max_turns: u32,
    temperature: f64,
    system_prompt: Option<String>,
    output_format: OutputFormat,
    verbose: bool,
}

#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Text,
    Markdown,
    Json,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4".to_string(),
            max_turns: 10,
            temperature: 0.7,
            system_prompt: None,
            output_format: OutputFormat::Text,
            verbose: false,
        }
    }
}

impl CliConfig {
    fn load() -> Self {
        // In a real app, this would load from file/env
        Self::default()
    }
}

// ============================================================================
// CLI Session
// ============================================================================

/// CLI session state
struct CliSession {
    id: String,
    config: CliConfig,
    message_count: AtomicU64,
    total_tokens: AtomicU64,
    start_time: Instant,
    running: AtomicBool,
    history: Arc<Mutex<Vec<HistoryEntry>>>,
}

#[derive(Debug, Clone)]
struct HistoryEntry {
    timestamp: Instant,
    input: String,
    output: String,
    tokens: u64,
}

impl CliSession {
    fn new(config: CliConfig) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
            config,
            message_count: AtomicU64::new(0),
            total_tokens: AtomicU64::new(0),
            start_time: Instant::now(),
            running: AtomicBool::new(true),
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    async fn add_history(&self, input: String, output: String, tokens: u64) {
        self.message_count.fetch_add(1, Ordering::SeqCst);
        self.total_tokens.fetch_add(tokens, Ordering::SeqCst);

        self.history.lock().await.push(HistoryEntry {
            timestamp: Instant::now(),
            input,
            output,
            tokens,
        });
    }

    fn stats(&self) -> SessionStats {
        SessionStats {
            messages: self.message_count.load(Ordering::SeqCst),
            tokens: self.total_tokens.load(Ordering::SeqCst),
            duration: self.start_time.elapsed(),
        }
    }
}

#[derive(Debug)]
struct SessionStats {
    messages: u64,
    tokens: u64,
    duration: std::time::Duration,
}

// ============================================================================
// CLI Commands
// ============================================================================

/// CLI command handler
struct CliCommands {
    commands: HashMap<String, CommandInfo>,
}

#[derive(Clone)]
struct CommandInfo {
    name: String,
    description: String,
    usage: String,
    handler: fn(&mut CliSession, &[String]) -> Result<String>,
}

impl CliCommands {
    fn new() -> Self {
        let mut cmds = Self {
            commands: HashMap::new(),
        };

        // Register built-in commands
        cmds.register(CommandInfo {
            name: "help".to_string(),
            description: "Show available commands".to_string(),
            usage: "/help [command]".to_string(),
            handler: |_session, args| {
                if args.is_empty() {
                    Ok("Available commands:\n  /help - Show this help\n  /stats - Show session statistics\n  /model <name> - Change model\n  /config - Show configuration\n  /history - Show command history\n  /export [format] - Export session\n  /clear - Clear screen\n  /quit - Exit CLI".to_string())
                } else {
                    Ok(format!("Help for: {}", args.join(" ")))
                }
            },
        });

        cmds.register(CommandInfo {
            name: "stats".to_string(),
            description: "Show session statistics".to_string(),
            usage: "/stats".to_string(),
            handler: |session, _args| {
                let stats = session.stats();
                Ok(format!(
                    "Session Statistics:\n  Messages: {}\n  Tokens: {}\n  Duration: {:?}",
                    stats.messages, stats.tokens, stats.duration
                ))
            },
        });

        cmds.register(CommandInfo {
            name: "model".to_string(),
            description: "Change the model".to_string(),
            usage: "/model <model-name>".to_string(),
            handler: |session, args| {
                if args.is_empty() {
                    Ok(format!("Current model: {}", session.config.model))
                } else {
                    session.config.model = args[0].clone();
                    Ok(format!("Model changed to: {}", args[0]))
                }
            },
        });

        cmds.register(CommandInfo {
            name: "config".to_string(),
            description: "Show current configuration".to_string(),
            usage: "/config".to_string(),
            handler: |session, _args| {
                Ok(format!(
                    "Configuration:\n  Model: {}\n  Max turns: {}\n  Temperature: {}\n  Output format: {:?}\n  Verbose: {}",
                    session.config.model,
                    session.config.max_turns,
                    session.config.temperature,
                    session.config.output_format,
                    session.config.verbose
                ))
            },
        });

        cmds.register(CommandInfo {
            name: "history".to_string(),
            description: "Show command history".to_string(),
            usage: "/history [n]".to_string(),
            handler: |_session, _args| {
                // In real implementation, would show actual history
                Ok("Command history (last 10):\n  [1] hello\n  [2] what is rust?\n  [3] /stats".to_string())
            },
        });

        cmds.register(CommandInfo {
            name: "export".to_string(),
            description: "Export session data".to_string(),
            usage: "/export [json|markdown]".to_string(),
            handler: |_session, args| {
                let format = args.first().map(|s| s.as_str()).unwrap_or("json");
                Ok(format!("Exporting session as {}...", format))
            },
        });

        cmds.register(CommandInfo {
            name: "clear".to_string(),
            description: "Clear the screen".to_string(),
            usage: "/clear".to_string(),
            handler: |_session, _args| {
                print!("\x1B[2J\x1B[1;1H"); // ANSI clear screen
                io::stdout().flush()?;
                Ok(String::new())
            },
        });

        cmds.register(CommandInfo {
            name: "quit".to_string(),
            description: "Exit the CLI".to_string(),
            usage: "/quit".to_string(),
            handler: |session, _args| {
                session.stop();
                Ok("Goodbye!".to_string())
            },
        });

        cmds
    }

    fn register(&mut self, info: CommandInfo) {
        self.commands.insert(info.name.clone(), info);
    }

    fn execute(&self, session: &mut CliSession, input: &str) -> Option<Result<String>> {
        let input = input.trim();

        if !input.starts_with('/') {
            return None;
        }

        let parts: Vec<&str> = input[1..].split_whitespace().collect();
        if parts.is_empty() {
            return Some(Err(anyhow::anyhow!("Empty command")));
        }

        let command = parts[0];
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        if let Some(info) = self.commands.get(command) {
            Some((info.handler)(session, &args))
        } else {
            Some(Err(anyhow::anyhow!("Unknown command: /{}", command)))
        }
    }

    fn list_commands(&self) -> Vec<&CommandInfo> {
        self.commands.values().collect()
    }
}

// ============================================================================
// Output Formatter
// ============================================================================

/// Format output based on configuration
struct OutputFormatter {
    format: OutputFormat,
}

impl OutputFormatter {
    fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    fn format_response(&self, response: &str, metadata: Option<&ResponseMetadata>) -> String {
        match self.format {
            OutputFormat::Text => {
                let mut output = response.to_string();
                if let Some(meta) = metadata {
                    output.push_str(&format!("\n\n---\nTokens: {} | Time: {:?}",
                        meta.tokens, meta.duration));
                }
                output
            }
            OutputFormat::Markdown => {
                let mut output = format!("## Response\n\n{}", response);
                if let Some(meta) = metadata {
                    output.push_str(&format!("\n\n---\n> Tokens: {} | Time: {:?}",
                        meta.tokens, meta.duration));
                }
                output
            }
            OutputFormat::Json => {
                let meta_json = metadata.map(|m| format!(
                    r#","metadata":{{"tokens":{},"duration_ms":{}}}"#,
                    m.tokens,
                    m.duration.as_millis()
                )).unwrap_or_default();
                format!(r#"{{"response":"{}"{}"#,
                    response.replace('"', "\\\"").replace('\n', "\\n"),
                    meta_json
                ) + "}"
            }
        }
    }
}

#[derive(Debug)]
struct ResponseMetadata {
    tokens: u64,
    duration: std::time::Duration,
}

// ============================================================================
// Progress Indicator
// ============================================================================

/// Simple progress indicator for CLI
struct ProgressIndicator {
    message: String,
    start: Instant,
}

impl ProgressIndicator {
    fn new(message: &str) -> Self {
        print!("\x1B[90m{}...\x1B[0m ", message);
        io::stdout().flush().ok();
        Self {
            message: message.to_string(),
            start: Instant::now(),
        }
    }

    fn complete(&self, result: &str) {
        println!("\x1B[90m{} in {:?}\x1B[0m", result, self.start.elapsed());
    }
}

impl Drop for ProgressIndicator {
    fn drop(&mut self) {
        // Clear the progress line if not completed
        print!("\r\x1B[K");
        io::stdout().flush().ok();
    }
}

// ============================================================================
// REPL Engine
// ============================================================================

/// Interactive REPL engine
struct ReplEngine {
    session: CliSession,
    commands: CliCommands,
    formatter: OutputFormatter,
}

impl ReplEngine {
    fn new(config: CliConfig) -> Self {
        let output_format = config.output_format;
        Self {
            session: CliSession::new(config),
            commands: CliCommands::new(),
            formatter: OutputFormatter::new(output_format),
        }
    }

    async fn process_input(&mut self, input: &str) -> Result<String> {
        let input = input.trim();

        // Skip empty input
        if input.is_empty() {
            return Ok(String::new());
        }

        // Handle commands
        if let Some(result) = self.commands.execute(&mut self.session, input) {
            return result;
        }

        // Process as query
        let progress = ProgressIndicator::new("Thinking");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Simulate processing

        // In real implementation, would call Claude API here
        let response = format!("Echo: {}", input);
        let tokens = input.len() as u64 / 4 + response.len() as u64 / 4;

        progress.complete("Done");

        // Record in history
        self.session.add_history(input.to_string(), response.clone(), tokens).await;

        // Format output
        let metadata = ResponseMetadata {
            tokens,
            duration: std::time::Duration::from_millis(100),
        };

        Ok(self.formatter.format_response(&response, Some(&metadata)))
    }

    fn print_welcome(&self) {
        println!("\x1B[1;36m╔════════════════════════════════════════════════════╗\x1B[0m");
        println!("\x1B[1;36m║     Claude Agent SDK CLI v1.0                     ║\x1B[0m");
        println!("\x1B[1;36m╚════════════════════════════════════════════════════╝\x1B[0m");
        println!();
        println!("  Model: \x1B[33m{}\x1B[0m", self.session.config.model);
        println!("  Session: \x1B[90m{}\x1B[0m", self.session.id);
        println!();
        println!("  Type \x1B[36m/help\x1B[0m for commands, \x1B[36m/quit\x1B[0m to exit");
        println!();
    }

    fn print_prompt(&self) {
        print!("\x1B[32m❯\x1B[0m ");
        io::stdout().flush().ok();
    }
}

// ============================================================================
// Input Handler
// ============================================================================

/// Handle user input with line editing support
fn read_line() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

// ============================================================================
// Main Demo
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== CLI Tools Example ===\n");
    println!("This example demonstrates building interactive CLI tools.\n");

    // Run demo scenarios
    demo_cli_commands().await;
    demo_session_management().await;
    demo_output_formatting().await;
    demo_progress_indicators().await;
    demo_repl_simulation().await;

    println!("\n=== All CLI examples completed ===");
    Ok(())
}

async fn demo_cli_commands() {
    println!("=== 1. CLI Commands ===\n");

    let config = CliConfig::default();
    let mut session = CliSession::new(config);
    let commands = CliCommands::new();

    println!("Available commands:");
    for cmd in commands.list_commands() {
        println!("  /{:<10} - {}", cmd.name, cmd.description);
    }

    println!("\nTesting commands:");
    let test_inputs = vec!["/help", "/stats", "/model claude-opus-4", "/config"];

    for input in test_inputs {
        if let Some(result) = commands.execute(&mut session, input) {
            match result {
                Ok(output) => println!("  {} -> {}", input, output.lines().next().unwrap_or("")),
                Err(e) => println!("  {} -> Error: {}", input, e),
            }
        }
    }
    println!();
}

async fn demo_session_management() {
    println!("=== 2. Session Management ===\n");

    let config = CliConfig::default();
    let session = CliSession::new(config);

    println!("Created session:");
    println!("  ID: {}", session.id);
    println!("  Model: {}", session.config.model);

    // Simulate some interactions
    session.add_history("What is Rust?".to_string(),
        "Rust is a systems programming language...".to_string(), 150).await;
    session.add_history("Explain ownership".to_string(),
        "Ownership is a key concept in Rust...".to_string(), 200).await;

    let stats = session.stats();
    println!("\nSession statistics:");
    println!("  Messages: {}", stats.messages);
    println!("  Tokens: {}", stats.tokens);
    println!("  Duration: {:?}", stats.duration);
    println!();
}

async fn demo_output_formatting() {
    println!("=== 3. Output Formatting ===\n");

    let metadata = ResponseMetadata {
        tokens: 150,
        duration: std::time::Duration::from_millis(850),
    };
    let response = "Rust is a systems programming language focused on safety, speed, and concurrency.";

    for format in [OutputFormat::Text, OutputFormat::Markdown, OutputFormat::Json] {
        let formatter = OutputFormatter::new(format);
        println!("{:?} output:", format);
        let output = formatter.format_response(response, Some(&metadata));
        for line in output.lines().take(5) {
            println!("  {}", line);
        }
        println!();
    }
}

async fn demo_progress_indicators() {
    println!("=== 4. Progress Indicators ===\n");

    println!("Simulating operations with progress:");

    let progress = ProgressIndicator::new("Loading model");
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    progress.complete("Model loaded");

    let progress = ProgressIndicator::new("Processing query");
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    progress.complete("Query processed");

    println!();
}

async fn demo_repl_simulation() {
    println!("=== 5. REPL Simulation ===\n");

    let config = CliConfig {
        model: "claude-sonnet-4".to_string(),
        verbose: true,
        ..Default::default()
    };

    let mut repl = ReplEngine::new(config);

    // Show welcome
    repl.print_welcome();

    println!("Simulated REPL session:");
    let inputs = vec![
        "Hello, Claude!",
        "/stats",
        "/model claude-opus-4",
        "What is async programming?",
        "/quit",
    ];

    for input in inputs {
        println!("\n\x1B[90mInput:\x1B[0m {}", input);

        if !repl.session.is_running() && input == "/quit" {
            println!("  Session ended.");
            break;
        }

        match repl.process_input(input).await {
            Ok(output) if !output.is_empty() => {
                println!("\x1B[90mOutput:\x1B[0m");
                for line in output.lines().take(5) {
                    println!("  {}", line);
                }
            }
            Ok(_) => {}
            Err(e) => println!("  Error: {}", e),
        }
    }

    println!();
}
