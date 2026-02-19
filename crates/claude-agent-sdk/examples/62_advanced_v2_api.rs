//! Example: Advanced V2 API Features
//!
//! This example demonstrates advanced features of the V2 API including
//! custom session handlers, streaming, error handling, and middleware.
//!
//! What it demonstrates:
//! 1. Custom session configuration with builders
//! 2. Session lifecycle management
//! 3. Streaming with V2 API
//! 4. Error handling patterns
//! 5. Middleware and hooks integration

use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Simulated V2 API types for demonstration
#[derive(Debug, Clone)]
struct Session {
    id: String,
    model: String,
    max_turns: Option<u32>,
    created_at: Instant,
    message_count: u32,
}

/// Session configuration
#[derive(Debug, Clone)]
struct SessionConfig {
    model: String,
    max_turns: Option<u32>,
    system_prompt: Option<String>,
    temperature: Option<f32>,
    timeout_secs: Option<u64>,
    permission_mode: PermissionMode,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4".to_string(),
            max_turns: None,
            system_prompt: None,
            temperature: None,
            timeout_secs: None,
            permission_mode: PermissionMode::Auto,
        }
    }
}

/// Permission mode
#[derive(Debug, Clone, Copy)]
enum PermissionMode {
    Auto,
    Bypass,
    Interactive,
}

/// Message types
#[derive(Debug, Clone)]
enum Message {
    User { content: String },
    Assistant { content: String },
    System { content: String },
}

/// Response result
#[derive(Debug, Clone)]
struct Response {
    content: String,
    tokens_used: u32,
    model: String,
    finish_reason: String,
}

/// Middleware trait for session processing
trait Middleware: Send + Sync {
    fn before_request(&self, messages: &[Message]) -> Vec<Message>;
    fn after_response(&self, response: &Response) -> Response;
}

/// Logging middleware
struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn before_request(&self, messages: &[Message]) -> Vec<Message> {
        println!("  [Middleware] Processing {} messages", messages.len());
        messages.to_vec()
    }

    fn after_response(&self, response: &Response) -> Response {
        println!("  [Middleware] Response received: {} tokens", response.tokens_used);
        response.clone()
    }
}

/// Rate limiting middleware
struct RateLimitMiddleware {
    min_interval_ms: u64,
    last_request: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl RateLimitMiddleware {
    fn new(min_interval_ms: u64) -> Self {
        Self {
            min_interval_ms,
            last_request: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
}

impl Middleware for RateLimitMiddleware {
    fn before_request(&self, messages: &[Message]) -> Vec<Message> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let last = self.last_request.load(std::sync::atomic::Ordering::SeqCst);
        let elapsed = now.saturating_sub(last);

        if elapsed < self.min_interval_ms {
            let wait = self.min_interval_ms - elapsed;
            println!("  [RateLimit] Waiting {}ms", wait);
            // In real async code: tokio::time::sleep(Duration::from_millis(wait)).await;
        }

        self.last_request.store(now, std::sync::atomic::Ordering::SeqCst);
        messages.to_vec()
    }

    fn after_response(&self, response: &Response) -> Response {
        response.clone()
    }
}

/// Session manager for advanced lifecycle management
struct SessionManager {
    sessions: Vec<Session>,
    max_sessions: usize,
}

impl SessionManager {
    fn new(max_sessions: usize) -> Self {
        Self {
            sessions: Vec::new(),
            max_sessions,
        }
    }

    fn create_session(&mut self, config: SessionConfig) -> Result<Session> {
        if self.sessions.len() >= self.max_sessions {
            // Remove oldest inactive session
            self.sessions.retain(|s| s.message_count > 0);
            if self.sessions.len() >= self.max_sessions {
                return Err(anyhow::anyhow!("Maximum sessions reached"));
            }
        }

        let session = Session {
            id: format!("sess_{}", uuid::Uuid::new_v4()),
            model: config.model,
            max_turns: config.max_turns,
            created_at: Instant::now(),
            message_count: 0,
        };

        self.sessions.push(session.clone());
        Ok(session)
    }

    fn get_session(&self, id: &str) -> Option<&Session> {
        self.sessions.iter().find(|s| s.id == id)
    }

    fn list_sessions(&self) -> &[Session] {
        &self.sessions
    }
}

fn main() -> Result<()> {
    println!("=== Advanced V2 API Features Examples ===\n");

    custom_session_config_example()?;
    session_lifecycle_example()?;
    streaming_v2_example()?;
    error_handling_patterns_example()?;
    middleware_example()?;

    Ok(())
}

/// Demonstrates custom session configuration with builders
fn custom_session_config_example() -> Result<()> {
    println!("=== Custom Session Configuration ===\n");

    // Basic configuration
    let basic_config = SessionConfig::default();
    println!("Basic config:");
    println!("  Model: {}", basic_config.model);
    println!("  Max turns: {:?}", basic_config.max_turns);
    println!();

    // Builder pattern for custom configuration
    let custom_config = SessionConfig {
        model: "claude-opus-4".to_string(),
        max_turns: Some(10),
        system_prompt: Some("You are a helpful coding assistant.".to_string()),
        temperature: Some(0.7),
        timeout_secs: Some(60),
        permission_mode: PermissionMode::Bypass,
    };

    println!("Custom config:");
    println!("  Model: {}", custom_config.model);
    println!("  Max turns: {:?}", custom_config.max_turns);
    println!("  System prompt: {:?}", custom_config.system_prompt);
    println!("  Temperature: {:?}", custom_config.temperature);
    println!("  Timeout: {:?}s", custom_config.timeout_secs);
    println!("  Permission mode: {:?}", custom_config.permission_mode);
    println!();

    // Configuration presets
    println!("Configuration presets:");
    let presets = vec![
        ("Coding assistant", SessionConfig {
            model: "claude-sonnet-4".to_string(),
            system_prompt: Some("Expert programmer".to_string()),
            temperature: Some(0.3),
            ..Default::default()
        }),
        ("Creative writer", SessionConfig {
            model: "claude-sonnet-4".to_string(),
            system_prompt: Some("Creative writer".to_string()),
            temperature: Some(0.9),
            ..Default::default()
        }),
        ("Quick answers", SessionConfig {
            model: "claude-sonnet-4".to_string(),
            max_turns: Some(1),
            temperature: Some(0.0),
            ..Default::default()
        }),
    ];

    for (name, config) in &presets {
        println!("  '{}': model={}, temp={:?}",
            name, config.model, config.temperature);
    }
    println!();

    Ok(())
}

/// Demonstrates session lifecycle management
fn session_lifecycle_example() -> Result<()> {
    println!("=== Session Lifecycle Management ===\n");

    let mut manager = SessionManager::new(5);

    println!("Creating sessions:");

    // Create multiple sessions
    let configs = vec![
        ("chat", SessionConfig::default()),
        ("analysis", SessionConfig {
            model: "claude-sonnet-4".to_string(),
            max_turns: Some(5),
            ..Default::default()
        }),
        ("coding", SessionConfig {
            system_prompt: Some("Code assistant".to_string()),
            ..Default::default()
        }),
    ];

    for (name, config) in &configs {
        let session = manager.create_session(config.clone())?;
        println!("  Created '{}' session: {} (model: {})", name, session.id, session.model);
    }

    println!("\nActive sessions: {}", manager.list_sessions().len());

    // Session reuse pattern
    println!("\nSession reuse pattern:");
    println!("  1. Create session with specific configuration");
    println!("  2. Store session ID for reuse");
    println!("  3. Resume session for related queries");
    println!("  4. Close session when done");

    println!("\nLifecycle best practices:");
    println!("  - Reuse sessions for related conversations");
    println!("  - Set appropriate max_turns limits");
    println!("  - Clean up inactive sessions");
    println!("  - Monitor session age and usage");
    println!();

    Ok(())
}

/// Demonstrates streaming with V2 API
fn streaming_v2_example() -> Result<()> {
    println!("=== Streaming with V2 API ===\n");

    println!("Streaming response simulation:");
    println!();

    // Simulate streaming chunks
    let chunks = vec![
        "The",
        " quick",
        " brown",
        " fox",
        " jumps",
        " over",
        " the",
        " lazy",
        " dog.",
    ];

    println!("  Streaming: ");
    let start = Instant::now();
    for (i, chunk) in chunks.iter().enumerate() {
        print!("{}", chunk);
        // Simulate network delay
        std::thread::sleep(Duration::from_millis(50));
    }
    println!();
    println!("  Stream completed in {:?}", start.elapsed());
    println!();

    // Streaming vs collecting comparison
    println!("Streaming vs Collecting:");
    println!("  Collecting:");
    println!("    - Wait for complete response");
    println!("    - Simpler error handling");
    println!("    - Use when you need complete output");
    println!();
    println!("  Streaming:");
    println!("    - Start processing immediately");
    println!("    - Better UX for long responses");
    println!("    - Lower perceived latency");
    println!("    - Handle partial results on error");
    println!();

    // Stream processing patterns
    println!("Stream processing patterns:");
    println!("  1. Real-time display: Print chunks as received");
    println!("  2. Token counting: Track tokens as streamed");
    println!("  3. Early termination: Stop on specific content");
    println!("  4. Parallel processing: Process chunks concurrently");
    println!();

    Ok(())
}

/// Demonstrates error handling patterns
fn error_handling_patterns_example() -> Result<()> {
    println!("=== Error Handling Patterns ===\n");

    // Error types in V2 API
    #[derive(Debug)]
    enum V2Error {
        Network(String),
        RateLimit { retry_after: u64 },
        InvalidInput(String),
        SessionExpired(String),
        ModelUnavailable(String),
    }

    let errors = vec![
        V2Error::Network("Connection timeout".to_string()),
        V2Error::RateLimit { retry_after: 60 },
        V2Error::InvalidInput("Context too long".to_string()),
        V2Error::SessionExpired("Session not found".to_string()),
        V2Error::ModelUnavailable("Model overloaded".to_string()),
    ];

    println!("Error types and handling:");
    for error in &errors {
        let (error_type, recovery) = match error {
            V2Error::Network(msg) => ("Network", format!("Retry with backoff: {}", msg)),
            V2Error::RateLimit { retry_after } => ("RateLimit", format!("Wait {}s then retry", retry_after)),
            V2Error::InvalidInput(msg) => ("InvalidInput", format!("Fix input: {}", msg)),
            V2Error::SessionExpired(msg) => ("SessionExpired", format!("Create new session: {}", msg)),
            V2Error::ModelUnavailable(msg) => ("ModelUnavailable", format!("Use fallback model: {}", msg)),
        };
        println!("  {}: {}", error_type, recovery);
    }
    println!();

    // Error recovery flow
    println!("Error recovery flow:");
    println!("  1. Identify error type");
    println!("  2. Check if recoverable");
    println!("  3. Apply appropriate recovery strategy");
    println!("  4. Log error for debugging");
    println!("  5. Notify user if needed");
    println!();

    // Graceful degradation example
    println!("Graceful degradation example:");
    println!("  try: Primary model (claude-opus-4)");
    println!("  fallback: claude-sonnet-4");
    println!("  fallback: claude-haiku");
    println!("  last resort: cached response");
    println!();

    Ok(())
}

/// Demonstrates middleware integration
fn middleware_example() -> Result<()> {
    println!("=== Middleware Integration ===\n");

    let middlewares: Vec<Box<dyn Middleware>> = vec![
        Box::new(LoggingMiddleware),
        Box::new(RateLimitMiddleware::new(100)), // 100ms min interval
    ];

    let messages = vec![
        Message::User { content: "Hello".to_string() },
        Message::Assistant { content: "Hi there!".to_string() },
        Message::User { content: "How are you?".to_string() },
    ];

    println!("Processing request through middleware chain:");
    println!();

    // Before request
    let mut processed_messages = messages.clone();
    for (i, middleware) in middlewares.iter().enumerate() {
        println!("Middleware {} (before):", i + 1);
        processed_messages = middleware.before_request(&processed_messages);
    }
    println!();

    // Simulate response
    let response = Response {
        content: "I'm doing well, thank you!".to_string(),
        tokens_used: 42,
        model: "claude-sonnet-4".to_string(),
        finish_reason: "stop".to_string(),
    };

    println!("Processing response through middleware chain:");
    println!();

    // After response
    let mut processed_response = response.clone();
    for (i, middleware) in middlewares.iter().enumerate() {
        println!("Middleware {} (after):", i + 1);
        processed_response = middleware.after_response(&processed_response);
    }
    println!();

    println!("Final response: {}", processed_response.content);
    println!();

    // Middleware use cases
    println!("Common middleware use cases:");
    println!("  1. Logging: Track all requests/responses");
    println!("  2. Rate limiting: Prevent API abuse");
    println!("  3. Caching: Store frequent responses");
    println!("  4. Validation: Check input/output format");
    println!("  5. Transformation: Modify messages");
    println!("  6. Metrics: Track performance");
    println!("  7. Authentication: Verify credentials");
    println!();

    Ok(())
}

/// Helper to create a session builder (simulated)
#[allow(dead_code)]
struct SessionBuilder {
    config: SessionConfig,
}

#[allow(dead_code)]
impl SessionBuilder {
    fn new() -> Self {
        Self {
            config: SessionConfig::default(),
        }
    }

    fn model(mut self, model: &str) -> Self {
        self.config.model = model.to_string();
        self
    }

    fn max_turns(mut self, turns: u32) -> Self {
        self.config.max_turns = Some(turns);
        self
    }

    fn system_prompt(mut self, prompt: &str) -> Self {
        self.config.system_prompt = Some(prompt.to_string());
        self
    }

    fn temperature(mut self, temp: f32) -> Self {
        self.config.temperature = Some(temp);
        self
    }

    fn build(self) -> SessionConfig {
        self.config
    }
}
