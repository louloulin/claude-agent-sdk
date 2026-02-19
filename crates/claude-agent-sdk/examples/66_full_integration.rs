//! Example: Full Integration - Combining Multiple SDK Features
//!
//! This example demonstrates how to combine multiple SDK features together
//! to build a complete application with Claude.
//!
//! What it demonstrates:
//! 1. Session management patterns
//! 2. Multi-modal input building
//! 3. Metrics collection and tracking
//! 4. Command handling within Claude interactions
//! 5. Subagent delegation patterns
//! 6. Streaming response handling
//! 7. Cost tracking

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::Mutex;

use claude_agent_sdk::observability::{MetricsCollector, MetricKind};

// ============================================================================
// Feature 1: Application State
// ============================================================================

/// Application state that combines multiple SDK features
struct AppState {
    /// Session counter
    session_count: AtomicU64,
    /// Total tokens used
    total_tokens: AtomicU64,
    /// Total cost in cents
    total_cost_cents: AtomicU64,
    /// Active conversations
    conversations: Arc<Mutex<HashMap<String, Conversation>>>,
    /// Metrics collector
    metrics: MetricsCollector,
}

/// A single conversation
#[derive(Debug, Clone)]
struct Conversation {
    id: String,
    created_at: Instant,
    message_count: u64,
    tokens_used: u64,
}

impl AppState {
    fn new() -> Self {
        Self {
            session_count: AtomicU64::new(0),
            total_tokens: AtomicU64::new(0),
            total_cost_cents: AtomicU64::new(0),
            conversations: Arc::new(Mutex::new(HashMap::new())),
            metrics: MetricsCollector::new(),
        }
    }

    fn record_session(&self, tokens: u64, cost_usd: f64) {
        self.session_count.fetch_add(1, Ordering::SeqCst);
        self.total_tokens.fetch_add(tokens, Ordering::SeqCst);
        self.total_cost_cents.fetch_add((cost_usd * 100.0) as u64, Ordering::SeqCst);

        // Record metrics (using empty labels slice with explicit type)
        let empty_labels: &[(&str, &str)] = &[];
        self.metrics.record("session_tokens", MetricKind::Counter, tokens as f64, empty_labels);
        self.metrics.record("session_cost", MetricKind::Counter, cost_usd, empty_labels);
    }

    fn get_stats(&self) -> AppStats {
        AppStats {
            total_sessions: self.session_count.load(Ordering::SeqCst),
            total_tokens: self.total_tokens.load(Ordering::SeqCst),
            total_cost_usd: self.total_cost_cents.load(Ordering::SeqCst) as f64 / 100.0,
        }
    }
}

#[derive(Debug)]
struct AppStats {
    total_sessions: u64,
    total_tokens: u64,
    total_cost_usd: f64,
}

// ============================================================================
// Feature 2: Session Manager
// ============================================================================

/// Session manager that handles conversation persistence
struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, SessionInfo>>>,
}

#[derive(Debug, Clone)]
struct SessionInfo {
    id: String,
    model: String,
    created_at: Instant,
    last_activity: Instant,
    message_count: u64,
    total_tokens: u64,
}

impl SessionManager {
    fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn create_session(&self, model: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let session = SessionInfo {
            id: id.clone(),
            model: model.to_string(),
            created_at: Instant::now(),
            last_activity: Instant::now(),
            message_count: 0,
            total_tokens: 0,
        };

        self.sessions.lock().await.insert(id.clone(), session);
        println!("[SessionManager] Created session: {}", &id[..8]);

        id
    }

    async fn update_session(&self, id: &str, tokens: u64) {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(id) {
            session.last_activity = Instant::now();
            session.message_count += 1;
            session.total_tokens += tokens;
        }
    }

    async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.lock().await;
        sessions.values().cloned().collect()
    }

    async fn get_session(&self, id: &str) -> Option<SessionInfo> {
        let sessions = self.sessions.lock().await;
        sessions.get(id).cloned()
    }
}

// ============================================================================
// Feature 3: Command Registry
// ============================================================================

/// Simple command registry for slash commands
struct CommandRegistry {
    commands: HashMap<String, CommandHandler>,
}

type CommandHandler = Arc<dyn Fn(Vec<String>) -> Result<String> + Send + Sync>;

impl CommandRegistry {
    fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };

        // Register default commands
        registry.register("help", |args| {
            if args.is_empty() {
                Ok("Available commands: /help, /stats, /model, /clear, /export".to_string())
            } else {
                Ok(format!("Help for: {}", args.join(" ")))
            }
        });

        registry.register("stats", |_args| {
            Ok("Stats command - shows usage statistics".to_string())
        });

        registry.register("model", |args| {
            if args.is_empty() {
                Ok("Current model: default".to_string())
            } else {
                Ok(format!("Model set to: {}", args[0]))
            }
        });

        registry.register("clear", |_args| {
            Ok("Conversation cleared".to_string())
        });

        registry.register("export", |args| {
            let format = args.first().map(|s| s.as_str()).unwrap_or("json");
            Ok(format!("Exporting conversation as {}...", format))
        });

        registry
    }

    fn register<F: Fn(Vec<String>) -> Result<String> + Send + Sync + 'static>(
        &mut self,
        name: &str,
        handler: F,
    ) {
        self.commands.insert(name.to_string(), Arc::new(handler));
    }

    fn execute(&self, command: &str, args: Vec<String>) -> Result<String> {
        if let Some(handler) = self.commands.get(command) {
            handler(args)
        } else {
            Err(anyhow::anyhow!("Unknown command: /{}", command))
        }
    }

    fn parse_and_execute(&self, input: &str) -> Option<Result<String>> {
        if !input.starts_with('/') {
            return None;
        }

        let parts: Vec<&str> = input[1..].split_whitespace().collect();
        if parts.is_empty() {
            return Some(Err(anyhow::anyhow!("Empty command")));
        }

        let command = parts[0];
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        Some(self.execute(command, args))
    }
}

// ============================================================================
// Feature 4: Subagent Dispatcher
// ============================================================================

/// Subagent type for specialized tasks
#[derive(Debug, Clone)]
enum SubagentType {
    CodeReviewer,
    SecurityAnalyzer,
    DocumentationWriter,
    TestGenerator,
    DataAnalyst,
}

/// Subagent dispatcher for delegating tasks
struct SubagentDispatcher;

impl SubagentDispatcher {
    fn new() -> Self {
        Self
    }

    fn dispatch(&self, task_type: SubagentType, input: &str) -> SubagentTask {
        println!("[SubagentDispatcher] Dispatching to {:?} subagent", task_type);

        SubagentTask {
            task_type,
            input: input.to_string(),
            created_at: Instant::now(),
        }
    }

    fn recommend_subagent(&self, input: &str) -> Option<SubagentType> {
        let input_lower = input.to_lowercase();

        if input_lower.contains("review") || input_lower.contains("code quality") {
            Some(SubagentType::CodeReviewer)
        } else if input_lower.contains("security") || input_lower.contains("vulnerability") {
            Some(SubagentType::SecurityAnalyzer)
        } else if input_lower.contains("document") || input_lower.contains("readme") {
            Some(SubagentType::DocumentationWriter)
        } else if input_lower.contains("test") || input_lower.contains("coverage") {
            Some(SubagentType::TestGenerator)
        } else if input_lower.contains("analyze") || input_lower.contains("data") {
            Some(SubagentType::DataAnalyst)
        } else {
            None
        }
    }
}

struct SubagentTask {
    task_type: SubagentType,
    input: String,
    created_at: Instant,
}

// ============================================================================
// Feature 5: Multi-modal Content Builder
// ============================================================================

/// Builder for multi-modal content
struct MultiModalBuilder {
    parts: Vec<ContentPart>,
}

#[derive(Debug, Clone)]
enum ContentPart {
    Text(String),
    Image { data: String, media_type: String },
    ImageUrl(String),
}

impl MultiModalBuilder {
    fn new() -> Self {
        Self { parts: Vec::new() }
    }

    fn text(mut self, text: impl Into<String>) -> Self {
        self.parts.push(ContentPart::Text(text.into()));
        self
    }

    fn image_base64(mut self, media_type: &str, data: &str) -> Self {
        self.parts.push(ContentPart::Image {
            data: data.to_string(),
            media_type: media_type.to_string(),
        });
        self
    }

    fn image_url(mut self, url: &str) -> Self {
        self.parts.push(ContentPart::ImageUrl(url.to_string()));
        self
    }

    fn build(self) -> Vec<ContentPart> {
        self.parts
    }

    fn to_summary(&self) -> String {
        let text_count = self.parts.iter().filter(|p| matches!(p, ContentPart::Text(_))).count();
        let image_count = self.parts.iter().filter(|p| matches!(p, ContentPart::Image { .. } | ContentPart::ImageUrl(_))).count();

        format!("{} text parts, {} images", text_count, image_count)
    }
}

// ============================================================================
// Feature 6: Streaming Response Handler
// ============================================================================

/// Handler for streaming responses with callbacks
struct StreamingHandler {
    on_text: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    on_complete: Option<Arc<dyn Fn() + Send + Sync>>,
    on_error: Option<Arc<dyn Fn(&anyhow::Error) + Send + Sync>>,
    text_buffer: String,
}

impl StreamingHandler {
    fn new() -> Self {
        Self {
            on_text: None,
            on_complete: None,
            on_error: None,
            text_buffer: String::new(),
        }
    }

    fn on_text<F: Fn(&str) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_text = Some(Arc::new(callback));
        self
    }

    fn on_complete<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_complete = Some(Arc::new(callback));
        self
    }

    fn on_error<F: Fn(&anyhow::Error) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_error = Some(Arc::new(callback));
        self
    }

    fn handle_text(&mut self, text: &str) {
        self.text_buffer.push_str(text);
        if let Some(ref callback) = self.on_text {
            callback(text);
        }
    }

    fn handle_complete(&self) {
        if let Some(ref callback) = self.on_complete {
            callback();
        }
    }

    fn handle_error(&self, error: &anyhow::Error) {
        if let Some(ref callback) = self.on_error {
            callback(error);
        }
    }

    fn get_full_text(&self) -> &str {
        &self.text_buffer
    }
}

// ============================================================================
// Feature 7: Cost Tracker
// ============================================================================

/// Tracks costs across sessions
struct CostTracker {
    sessions: Arc<Mutex<HashMap<String, SessionCost>>>,
    total_cents: AtomicU64,
}

#[derive(Debug, Clone)]
struct SessionCost {
    session_id: String,
    input_tokens: u64,
    output_tokens: u64,
    cost_usd: f64,
}

impl CostTracker {
    fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            total_cents: AtomicU64::new(0),
        }
    }

    async fn record(&self, session_id: &str, input_tokens: u64, output_tokens: u64) {
        // Rough pricing: $3/M input, $15/M output
        let input_cost = (input_tokens as f64) / 1_000_000.0 * 3.0;
        let output_cost = (output_tokens as f64) / 1_000_000.0 * 15.0;
        let total_cost = input_cost + output_cost;

        let session_cost = SessionCost {
            session_id: session_id.to_string(),
            input_tokens,
            output_tokens,
            cost_usd: total_cost,
        };

        self.sessions.lock().await.insert(session_id.to_string(), session_cost);
        self.total_cents.fetch_add((total_cost * 100.0) as u64, Ordering::SeqCst);
    }

    fn get_total_cost(&self) -> f64 {
        self.total_cents.load(Ordering::SeqCst) as f64 / 100.0
    }

    async fn get_session_cost(&self, session_id: &str) -> Option<SessionCost> {
        let sessions = self.sessions.lock().await;
        sessions.get(session_id).cloned()
    }
}

// ============================================================================
// Main Demo
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Full Integration Example ===\n");
    println!("This example demonstrates combining multiple SDK features.\n");

    demo_app_state().await;
    demo_session_manager().await;
    demo_command_registry().await;
    demo_subagent_dispatcher().await;
    demo_multimodal_builder().await;
    demo_streaming_handler().await;
    demo_cost_tracker().await;
    demo_full_integration().await;

    println!("\n=== All integration examples completed ===");
    Ok(())
}

async fn demo_app_state() {
    println!("=== 1. Application State ===\n");

    let state = AppState::new();

    // Simulate some sessions
    state.record_session(150, 0.0023);
    state.record_session(200, 0.0030);
    state.record_session(175, 0.0026);

    let stats = state.get_stats();
    println!("App Statistics:");
    println!("  Total sessions: {}", stats.total_sessions);
    println!("  Total tokens: {}", stats.total_tokens);
    println!("  Total cost: ${:.4}", stats.total_cost_usd);
    println!();
}

async fn demo_session_manager() {
    println!("=== 2. Session Manager ===\n");

    let manager = SessionManager::new();

    // Create sessions
    let session1 = manager.create_session("claude-sonnet-4").await;
    let session2 = manager.create_session("claude-opus-4").await;

    println!("\nCreated sessions:");
    for session in manager.list_sessions().await {
        println!("  - {} (model: {})", &session.id[..8], session.model);
    }

    // Update a session
    manager.update_session(&session1, 150).await;

    if let Some(info) = manager.get_session(&session1).await {
        println!("\nSession {} info:", &session1[..8]);
        println!("  Messages: {}", info.message_count);
        println!("  Tokens: {}", info.total_tokens);
    }
    println!();
}

async fn demo_command_registry() {
    println!("=== 3. Command Registry ===\n");

    let registry = CommandRegistry::new();

    println!("Testing slash commands:");

    // Test commands
    let commands = vec!["/help", "/stats", "/model claude-opus-4", "/export json"];

    for cmd in commands {
        if let Some(result) = registry.parse_and_execute(cmd) {
            match result {
                Ok(output) => println!("  {} -> {}", cmd, output),
                Err(e) => println!("  {} -> Error: {}", cmd, e),
            }
        }
    }
    println!();
}

async fn demo_subagent_dispatcher() {
    println!("=== 4. Subagent Dispatcher ===\n");

    let dispatcher = SubagentDispatcher::new();

    println!("Analyzing inputs for subagent delegation:");

    let inputs = vec![
        "Please review this code for bugs",
        "Check for security vulnerabilities in auth module",
        "Write documentation for the API",
        "Generate tests for the parser module",
        "Analyze the sales data from last quarter",
        "Just chat with me",
    ];

    for input in inputs {
        if let Some(subagent) = dispatcher.recommend_subagent(input) {
            println!("  '{:.40}...' -> {:?}", input, subagent);
        } else {
            println!("  '{:.40}...' -> No delegation needed", input);
        }
    }
    println!();
}

async fn demo_multimodal_builder() {
    println!("=== 5. Multi-modal Builder ===\n");

    let content = MultiModalBuilder::new()
        .text("Analyze this architecture diagram:")
        .image_url("https://example.com/architecture.png")
        .text("Focus on the data flow between services.")
        .build();

    println!("Built multi-modal content:");
    println!("  Parts: {} total", content.len());
    for (i, part) in content.iter().enumerate() {
        match part {
            ContentPart::Text(text) => println!("    {}] Text: {:.50}...", i + 1, text),
            ContentPart::Image { media_type, .. } => println!("    {}] Image (base64, {})", i + 1, media_type),
            ContentPart::ImageUrl(url) => println!("    {}] Image URL: {}", i + 1, url),
        }
    }
    println!();
}

async fn demo_streaming_handler() {
    println!("=== 6. Streaming Handler ===\n");

    let mut handler = StreamingHandler::new()
        .on_text(|text| {
            // In real usage, this would update UI
            let _ = text; // Suppress unused warning
        })
        .on_complete(|| {
            println!("  [Stream complete]");
        })
        .on_error(|e| {
            println!("  [Error: {}]", e);
        });

    println!("Simulating streaming response:");

    // Simulate streaming text
    let chunks = vec!["Hello", ", ", "this ", "is ", "a ", "streaming ", "response", "."];
    for chunk in chunks {
        handler.handle_text(chunk);
    }
    handler.handle_complete();

    println!("  Full text: {}", handler.get_full_text());
    println!();
}

async fn demo_cost_tracker() {
    println!("=== 7. Cost Tracker ===\n");

    let tracker = CostTracker::new();

    // Simulate sessions
    tracker.record("session-1", 1000, 500).await;
    tracker.record("session-2", 2000, 1000).await;
    tracker.record("session-3", 1500, 750).await;

    println!("Cost tracking:");
    println!("  Total cost: ${:.4}", tracker.get_total_cost());

    if let Some(session_cost) = tracker.get_session_cost("session-1").await {
        println!("  Session 1: ${:.4} ({} in, {} out)",
            session_cost.cost_usd,
            session_cost.input_tokens,
            session_cost.output_tokens
        );
    }
    println!();
}

async fn demo_full_integration() {
    println!("=== 8. Full Integration Demo ===\n");

    // Create all components
    let app_state = Arc::new(AppState::new());
    let session_manager = Arc::new(SessionManager::new());
    let command_registry = CommandRegistry::new();
    let cost_tracker = Arc::new(CostTracker::new());
    let subagent_dispatcher = SubagentDispatcher::new();

    println!("Initialized integrated application with:");
    println!("  - Application state tracking");
    println!("  - Session management");
    println!("  - Command registry (5 commands)");
    println!("  - Cost tracking");
    println!("  - Subagent dispatching (5 agent types)");

    // Simulate a conversation flow
    println!("\nSimulating conversation flow:");

    // 1. Create session
    let session_id = session_manager.create_session("claude-sonnet-4").await;
    println!("  1. Created session: {}...", &session_id[..8]);

    // 2. Handle user input (might be a command)
    let user_input = "Review my code for security issues";
    if let Some(result) = command_registry.parse_and_execute(user_input) {
        println!("  2. Executed command: {:?}", result);
    } else {
        println!("  2. Not a command, processing as query...");

        // 3. Check for subagent delegation
        if let Some(subagent) = subagent_dispatcher.recommend_subagent(user_input) {
            println!("  3. Delegating to {:?} subagent", subagent);
            let _task = subagent_dispatcher.dispatch(subagent, user_input);
        }

        // 4. Update session stats
        session_manager.update_session(&session_id, 250).await;

        // 5. Track costs
        cost_tracker.record(&session_id, 100, 150).await;

        // 6. Update app state
        app_state.record_session(250, 0.0030);
    }

    // Final stats
    println!("\nFinal statistics:");
    let stats = app_state.get_stats();
    println!("  App: {} sessions, {} tokens, ${:.4}",
        stats.total_sessions, stats.total_tokens, stats.total_cost_usd);
    println!("  Cost tracker: ${:.4} total", cost_tracker.get_total_cost());

    println!();
}
