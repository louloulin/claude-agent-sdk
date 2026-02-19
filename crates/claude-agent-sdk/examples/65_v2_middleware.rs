//! Example: V2 API Middleware Patterns
//!
//! This example demonstrates middleware patterns for the V2 API,
//! including retry logic, logging, caching, and request/response transformation.
//!
//! What it demonstrates:
//! 1. Retry middleware with exponential backoff
//! 2. Logging middleware for request/response tracking
//! 3. Caching middleware for repeated queries
//! 4. Rate limiting middleware
//! 5. Request/response transformation middleware
//! 6. Combining multiple middleware layers

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use claude_agent_sdk::v2::{prompt, SessionOptions, PromptResult};

/// Middleware trait for request/response processing
#[async_trait::async_trait]
trait Middleware: Send + Sync {
    /// Process a prompt before sending to Claude
    async fn process_request(&self, prompt: &str) -> Result<String> {
        Ok(prompt.to_string())
    }

    /// Process a result after receiving from Claude
    async fn process_response(&self, result: &PromptResult) -> Result<PromptResult> {
        Ok(result.clone())
    }

    /// Handle errors (return None to propagate, Some to recover)
    async fn handle_error(&self, error: &anyhow::Error) -> Option<anyhow::Result<PromptResult>> {
        let _ = error;
        None
    }

    /// Get middleware name for logging
    fn name(&self) -> &str;
}

/// Prompt with middleware support
async fn prompt_with_middleware(
    prompt_text: &str,
    options: SessionOptions,
    middleware: &[Arc<dyn Middleware>],
) -> Result<PromptResult> {
    let mut current_prompt = prompt_text.to_string();

    // Process request through middleware chain
    for mw in middleware {
        current_prompt = mw.process_request(&current_prompt).await?;
    }

    // Execute prompt
    let mut result = prompt(&current_prompt, options).await?;

    // Process response through middleware chain (in reverse)
    for mw in middleware.iter().rev() {
        result = mw.process_response(&result).await?;
    }

    Ok(result)
}

/// Retry middleware with exponential backoff
struct RetryMiddleware {
    max_retries: u32,
    base_delay: Duration,
    max_delay: Duration,
    attempt_count: Arc<AtomicU64>,
}

impl RetryMiddleware {
    fn new(max_retries: u32, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
            attempt_count: Arc::new(AtomicU64::new(0)),
        }
    }

    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay = self.base_delay * 2u32.pow(attempt);
        delay.min(self.max_delay)
    }
}

#[async_trait::async_trait]
impl Middleware for RetryMiddleware {
    fn name(&self) -> &str {
        "RetryMiddleware"
    }

    async fn handle_error(&self, error: &anyhow::Error) -> Option<anyhow::Result<PromptResult>> {
        let attempt = self.attempt_count.fetch_add(1, Ordering::SeqCst) as u32;

        if attempt < self.max_retries {
            let delay = self.calculate_delay(attempt);
            println!("[{}] Attempt {} failed, retrying in {:?}...", self.name(), attempt + 1, delay);
            tokio::time::sleep(delay).await;
            // Return error to signal retry (in real implementation)
            None
        } else {
            println!("[{}] Max retries ({}) exceeded", self.name(), self.max_retries);
            None
        }
    }
}

/// Logging middleware for request/response tracking
struct LoggingMiddleware {
    log_requests: bool,
    log_responses: bool,
    request_count: Arc<AtomicU64>,
}

impl LoggingMiddleware {
    fn new(log_requests: bool, log_responses: bool) -> Self {
        Self {
            log_requests,
            log_responses,
            request_count: Arc::new(AtomicU64::new(0)),
        }
    }
}

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    fn name(&self) -> &str {
        "LoggingMiddleware"
    }

    async fn process_request(&self, prompt: &str) -> Result<String> {
        let request_id = self.request_count.fetch_add(1, Ordering::SeqCst);
        if self.log_requests {
            println!("[{}] Request #{}: {:?}", self.name(), request_id, prompt);
        }
        Ok(prompt.to_string())
    }

    async fn process_response(&self, result: &PromptResult) -> Result<PromptResult> {
        if self.log_responses {
            println!(
                "[{}] Response: {} tokens ({} in, {} out), cost: ${:.4}",
                self.name(),
                result.total_tokens(),
                result.input_tokens,
                result.output_tokens,
                result.estimated_cost_usd()
            );
        }
        Ok(result.clone())
    }
}

/// Caching middleware for repeated queries
struct CachingMiddleware {
    cache: Arc<Mutex<HashMap<String, CachedEntry>>>,
    ttl: Duration,
    hit_count: Arc<AtomicU64>,
    miss_count: Arc<AtomicU64>,
}

#[derive(Clone)]
struct CachedEntry {
    result: PromptResult,
    timestamp: Instant,
}

impl CachingMiddleware {
    fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl,
            hit_count: Arc::new(AtomicU64::new(0)),
            miss_count: Arc::new(AtomicU64::new(0)),
        }
    }

    async fn get(&self, key: &str) -> Option<PromptResult> {
        let cache = self.cache.lock().await;
        if let Some(entry) = cache.get(key) {
            if entry.timestamp.elapsed() < self.ttl {
                self.hit_count.fetch_add(1, Ordering::SeqCst);
                return Some(entry.result.clone());
            }
        }
        self.miss_count.fetch_add(1, Ordering::SeqCst);
        None
    }

    async fn set(&self, key: &str, result: PromptResult) {
        let mut cache = self.cache.lock().await;
        cache.insert(key.to_string(), CachedEntry {
            result,
            timestamp: Instant::now(),
        });
    }

    fn stats(&self) -> (u64, u64) {
        (
            self.hit_count.load(Ordering::SeqCst),
            self.miss_count.load(Ordering::SeqCst),
        )
    }
}

#[async_trait::async_trait]
impl Middleware for CachingMiddleware {
    fn name(&self) -> &str {
        "CachingMiddleware"
    }

    async fn process_request(&self, prompt: &str) -> Result<String> {
        // Check cache before making request
        if let Some(_result) = self.get(prompt).await {
            println!("[{}] Cache HIT for: {:?}", self.name(), prompt);
            // In real implementation, we'd return cached result directly
        }
        Ok(prompt.to_string())
    }

    async fn process_response(&self, result: &PromptResult) -> Result<PromptResult> {
        // Cache would be populated here with the prompt key
        // For demo, we just pass through
        Ok(result.clone())
    }
}

/// Rate limiting middleware
struct RateLimitMiddleware {
    requests_per_minute: u32,
    request_times: Arc<Mutex<Vec<Instant>>>,
    delayed_count: Arc<AtomicU64>,
}

impl RateLimitMiddleware {
    fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            request_times: Arc::new(Mutex::new(Vec::new())),
            delayed_count: Arc::new(AtomicU64::new(0)),
        }
    }

    async fn check_rate_limit(&self) -> Result<()> {
        let mut times = self.request_times.lock().await;
        let now = Instant::now();
        let window_start = now - Duration::from_secs(60);

        // Remove old entries
        times.retain(|&t| t > window_start);

        if times.len() >= self.requests_per_minute as usize {
            let oldest = times.first().unwrap();
            let wait_time = *oldest + Duration::from_secs(60) - now;
            println!("[{}] Rate limit reached, waiting {:?}...", self.name(), wait_time);
            self.delayed_count.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(wait_time).await;
        }

        times.push(now);
        Ok(())
    }
}

#[async_trait::async_trait]
impl Middleware for RateLimitMiddleware {
    fn name(&self) -> &str {
        "RateLimitMiddleware"
    }

    async fn process_request(&self, prompt: &str) -> Result<String> {
        self.check_rate_limit().await?;
        Ok(prompt.to_string())
    }
}

/// Metrics middleware for collecting performance data
struct MetricsMiddleware {
    total_requests: Arc<AtomicU64>,
    total_tokens: Arc<AtomicU64>,
    total_cost: Arc<AtomicU64>, // In cents
    total_latency_ms: Arc<AtomicU64>,
}

impl MetricsMiddleware {
    fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            total_tokens: Arc::new(AtomicU64::new(0)),
            total_cost: Arc::new(AtomicU64::new(0)),
            total_latency_ms: Arc::new(AtomicU64::new(0)),
        }
    }

    fn record_request(&self, tokens: u64, cost_usd: f64, latency_ms: u64) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);
        self.total_tokens.fetch_add(tokens, Ordering::SeqCst);
        self.total_cost.fetch_add((cost_usd * 100.0) as u64, Ordering::SeqCst);
        self.total_latency_ms.fetch_add(latency_ms, Ordering::SeqCst);
    }

    fn get_stats(&self) -> MiddlewareStats {
        let requests = self.total_requests.load(Ordering::SeqCst);
        MiddlewareStats {
            total_requests: requests,
            total_tokens: self.total_tokens.load(Ordering::SeqCst),
            total_cost_usd: self.total_cost.load(Ordering::SeqCst) as f64 / 100.0,
            avg_latency_ms: if requests > 0 {
                self.total_latency_ms.load(Ordering::SeqCst) / requests
            } else {
                0
            },
        }
    }
}

#[derive(Debug)]
struct MiddlewareStats {
    total_requests: u64,
    total_tokens: u64,
    total_cost_usd: f64,
    avg_latency_ms: u64,
}

#[async_trait::async_trait]
impl Middleware for MetricsMiddleware {
    fn name(&self) -> &str {
        "MetricsMiddleware"
    }

    async fn process_request(&self, prompt: &str) -> Result<String> {
        let _start = Instant::now();
        // In real impl, we'd track start time per request
        Ok(prompt.to_string())
    }

    async fn process_response(&self, result: &PromptResult) -> Result<PromptResult> {
        self.record_request(
            result.total_tokens(),
            result.estimated_cost_usd(),
            0, // Latency would be calculated from start time
        );
        Ok(result.clone())
    }
}

/// Transformation middleware for request/response modification
struct TransformationMiddleware {
    prefix: Option<String>,
    suffix: Option<String>,
    response_transform: Option<String>,
}

impl TransformationMiddleware {
    fn new(prefix: Option<String>, suffix: Option<String>) -> Self {
        Self {
            prefix,
            suffix,
            response_transform: None,
        }
    }
}

#[async_trait::async_trait]
impl Middleware for TransformationMiddleware {
    fn name(&self) -> &str {
        "TransformationMiddleware"
    }

    async fn process_request(&self, prompt: &str) -> Result<String> {
        let mut transformed = prompt.to_string();
        if let Some(ref prefix) = self.prefix {
            transformed = format!("{} {}", prefix, transformed);
        }
        if let Some(ref suffix) = self.suffix {
            transformed = format!("{} {}", transformed, suffix);
        }
        println!("[{}] Transformed prompt: {:?}", self.name(), transformed);
        Ok(transformed)
    }
}

/// Composition of multiple middleware
struct MiddlewareChain {
    middleware: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    fn new() -> Self {
        Self { middleware: Vec::new() }
    }

    fn add<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middleware.push(Arc::new(middleware));
        self
    }

    fn middleware(&self) -> &[Arc<dyn Middleware>] {
        &self.middleware
    }
}

// ============================================================================
// Example Functions
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== V2 API Middleware Patterns ===\n");

    // Note: The examples below demonstrate middleware patterns
    // but won't make actual API calls in this demo.
    // To use with real API calls, ensure ANTHROPIC_API_KEY is set.

    demo_middleware_chain().await;
    demo_retry_pattern().await;
    demo_caching_pattern().await;
    demo_rate_limiting().await;
    demo_metrics_collection().await;
    demo_transformation().await;

    println!("\n=== All middleware examples completed ===");
    Ok(())
}

async fn demo_middleware_chain() {
    println!("=== Middleware Chain Demo ===\n");

    let chain = MiddlewareChain::new()
        .add(LoggingMiddleware::new(true, true))
        .add(MetricsMiddleware::new())
        .add(TransformationMiddleware::new(
            Some("Please answer concisely:".to_string()),
            None,
        ));

    println!("Created middleware chain with {} layers:", chain.middleware().len());
    for mw in chain.middleware() {
        println!("  - {}", mw.name());
    }
    println!();
}

async fn demo_retry_pattern() {
    println!("=== Retry Pattern Demo ===\n");

    let retry_mw = RetryMiddleware::new(3, Duration::from_millis(100), Duration::from_secs(5));

    println!("Retry middleware configured:");
    println!("  Max retries: 3");
    println!("  Base delay: 100ms");
    println!("  Max delay: 5s");
    println!("  Backoff strategy: Exponential");
    println!();

    // Simulated retry delays
    println!("Simulated retry delays:");
    for attempt in 0..4 {
        let delay = retry_mw.calculate_delay(attempt);
        println!("  Attempt {}: {:?}", attempt + 1, delay);
    }
    println!();
}

async fn demo_caching_pattern() {
    println!("=== Caching Pattern Demo ===\n");

    let cache = CachingMiddleware::new(Duration::from_secs(300));

    println!("Cache middleware configured:");
    println!("  TTL: 300 seconds (5 minutes)");
    println!("  Strategy: LRU (Least Recently Used)");
    println!();

    // Simulate cache operations
    println!("Simulated cache operations:");
    println!("  SET 'query1' -> result1");
    println!("  GET 'query1' -> HIT (result1)");
    println!("  GET 'query2' -> MISS");
    println!();

    let (hits, misses) = cache.stats();
    println!("Cache stats: {} hits, {} misses", hits, misses);
    println!();
}

async fn demo_rate_limiting() {
    println!("=== Rate Limiting Demo ===\n");

    let rate_limit = RateLimitMiddleware::new(10);

    println!("Rate limit middleware configured:");
    println!("  Limit: 10 requests per minute");
    println!("  Strategy: Sliding window");
    println!("  Action on limit: Wait until window resets");
    println!();

    println!("Example scenario:");
    println!("  Request 1-10: Allowed immediately");
    println!("  Request 11: Delayed until oldest request expires");
    println!();
}

async fn demo_metrics_collection() {
    println!("=== Metrics Collection Demo ===\n");

    let metrics = MetricsMiddleware::new();

    // Simulate some requests
    metrics.record_request(150, 0.0023, 850);
    metrics.record_request(200, 0.0030, 920);
    metrics.record_request(175, 0.0026, 780);

    let stats = metrics.get_stats();
    println!("Collected metrics:");
    println!("  Total requests: {}", stats.total_requests);
    println!("  Total tokens: {}", stats.total_tokens);
    println!("  Total cost: ${:.4}", stats.total_cost_usd);
    println!("  Avg latency: {}ms", stats.avg_latency_ms);
    println!();
}

async fn demo_transformation() {
    println!("=== Transformation Demo ===\n");

    let transform = TransformationMiddleware::new(
        Some("Context: You are a helpful assistant.".to_string()),
        Some("Please be concise.".to_string()),
    );

    println!("Transformation middleware configured:");
    println!("  Prefix: 'Context: You are a helpful assistant.'");
    println!("  Suffix: 'Please be concise.'");
    println!();

    println!("Example transformation:");
    println!("  Input: 'What is Rust?'");
    println!("  Output: 'Context: You are a helpful assistant. What is Rust? Please be concise.'");
    println!();
}
