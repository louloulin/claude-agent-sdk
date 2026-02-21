//! Production deployment patterns and best practices.
//!
//! Demonstrates:
//! - Configuration management
//! - Error handling and logging
//! - Metrics and monitoring
//! - Graceful shutdown
//! - Health checks
//! - Deployment strategies

use claude_agent_sdk::{
    ContentBlock, Message, query, types::config::ClaudeAgentOptions,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::signal;
use tokio::time::sleep;

// ============================================================================
// Configuration Management
// ============================================================================

#[derive(Debug, Clone)]
struct ProductionConfig {
    /// Maximum concurrent requests
    #[allow(dead_code)]
    max_concurrent_requests: usize,

    /// Request timeout in seconds
    request_timeout_seconds: u64,

    /// Maximum tokens per request
    #[allow(dead_code)]
    max_tokens: u32,

    /// Enable debug logging
    debug_mode: bool,

    /// Health check interval in seconds
    health_check_interval_seconds: u64,

    /// Metrics collection enabled
    enable_metrics: bool,
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 10,
            request_timeout_seconds: 30,
            max_tokens: 4096,
            debug_mode: false,
            health_check_interval_seconds: 30,
            enable_metrics: true,
        }
    }
}

impl ProductionConfig {
    /// Load configuration from environment variables
    fn from_env() -> Self {
        Self {
            max_concurrent_requests: std::env::var("MAX_CONCURRENT_REQUESTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),

            request_timeout_seconds: std::env::var("REQUEST_TIMEOUT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),

            max_tokens: std::env::var("MAX_TOKENS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4096),

            debug_mode: std::env::var("DEBUG_MODE")
                .ok()
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),

            health_check_interval_seconds: std::env::var("HEALTH_CHECK_INTERVAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),

            enable_metrics: std::env::var("ENABLE_METRICS")
                .ok()
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
        }
    }

    fn to_claude_options(&self) -> ClaudeAgentOptions {
        ClaudeAgentOptions::default()
    }
}

// ============================================================================
// Metrics and Monitoring
// ============================================================================

#[derive(Debug, Clone, Default)]
struct Metrics {
    total_requests: Arc<std::sync::atomic::AtomicU64>,
    successful_requests: Arc<std::sync::atomic::AtomicU64>,
    failed_requests: Arc<std::sync::atomic::AtomicU64>,
    total_latency_ms: Arc<std::sync::atomic::AtomicU64>,
    total_tokens_used: Arc<std::sync::atomic::AtomicU64>,
}

impl Metrics {
    fn new() -> Self {
        Self::default()
    }

    fn record_request(&self, success: bool, latency_ms: u64, tokens: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
        self.total_latency_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
        self.total_tokens_used.fetch_add(tokens, Ordering::Relaxed);
    }

    fn get_stats(&self) -> MetricStats {
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ms.load(Ordering::Relaxed);
        let total_tokens = self.total_tokens_used.load(Ordering::Relaxed);

        MetricStats {
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            success_rate: if total > 0 {
                (successful as f64 / total as f64) * 100.0
            } else {
                0.0
            },
            avg_latency_ms: if total > 0 { total_latency / total } else { 0 },
            total_tokens_used: total_tokens,
        }
    }
}

#[derive(Debug)]
struct MetricStats {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    success_rate: f64,
    avg_latency_ms: u64,
    total_tokens_used: u64,
}

// ============================================================================
// Health Checks
// ============================================================================

#[derive(Clone)]
struct HealthChecker {
    is_healthy: Arc<AtomicBool>,
}

impl HealthChecker {
    fn new() -> Self {
        Self {
            is_healthy: Arc::new(AtomicBool::new(true)),
        }
    }

    async fn check(&self) -> HealthStatus {
        // Perform health checks
        let basic_check = self.check_basic_connectivity().await;
        let performance_check = self.check_performance().await;

        let is_healthy = basic_check.is_healthy && performance_check.is_healthy;
        self.is_healthy.store(is_healthy, Ordering::Relaxed);

        HealthStatus {
            is_healthy,
            checks: vec![basic_check, performance_check],
        }
    }

    async fn check_basic_connectivity(&self) -> CheckResult {
        // Simulate basic connectivity check
        CheckResult {
            name: "basic_connectivity".to_string(),
            is_healthy: true,
            message: "All systems operational".to_string(),
        }
    }

    async fn check_performance(&self) -> CheckResult {
        // Simulate performance check
        CheckResult {
            name: "performance".to_string(),
            is_healthy: true,
            message: "Response time within acceptable range".to_string(),
        }
    }

    #[allow(dead_code)]
    fn is_healthy(&self) -> bool {
        self.is_healthy.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
struct HealthStatus {
    is_healthy: bool,
    checks: Vec<CheckResult>,
}

#[derive(Debug)]
struct CheckResult {
    name: String,
    is_healthy: bool,
    #[allow(dead_code)]
    message: String,
}

// ============================================================================
// Graceful Shutdown
// ============================================================================

#[derive(Clone)]
struct ShutdownSignal {
    shutdown: Arc<AtomicBool>,
}

impl ShutdownSignal {
    fn new() -> Self {
        Self {
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    async fn wait_for_signal(&self) {
        let shutdown = self.shutdown.clone();

        // Handle Ctrl+C
        tokio::spawn(async move {
            match signal::ctrl_c().await {
                Ok(()) => {
                    println!("\n🛑 Received shutdown signal");
                    shutdown.store(true, Ordering::Relaxed);
                },
                Err(err) => {
                    eprintln!("❌ Failed to listen for shutdown signal: {}", err);
                },
            }
        });

        // Wait for signal
        while !self.is_shutdown() {
            sleep(Duration::from_millis(100)).await;
        }
    }

    fn trigger(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

// ============================================================================
// Production Service
// ============================================================================

struct ProductionService {
    config: ProductionConfig,
    metrics: Metrics,
    health_checker: HealthChecker,
    shutdown: ShutdownSignal,
}

impl ProductionService {
    fn new(config: ProductionConfig) -> Self {
        Self {
            config,
            metrics: Metrics::new(),
            health_checker: HealthChecker::new(),
            shutdown: ShutdownSignal::new(),
        }
    }

    async fn start(&self) -> anyhow::Result<()> {
        println!("🚀 Starting Production Service");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Configuration: {:?}", self.config);
        println!("\n");

        // Start background tasks
        let health_checker = self.health_checker.clone();
        let metrics = self.metrics.clone();
        let config = self.config.clone();
        let shutdown_health = self.shutdown.clone();
        let _shutdown_metrics = self.shutdown.clone();

        // Health check task
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_secs(config.health_check_interval_seconds));

            loop {
                interval.tick().await;

                if shutdown_health.is_shutdown() {
                    println!("🛑 Health checker shutting down");
                    break;
                }

                let status = health_checker.check().await;
                println!(
                    "🏥 Health Check: {}",
                    if status.is_healthy {
                        "✅ Healthy"
                    } else {
                        "❌ Unhealthy"
                    }
                );

                for check in &status.checks {
                    println!(
                        "  {}: {}",
                        check.name,
                        if check.is_healthy { "✅" } else { "❌" }
                    );
                }
            }
        });

        // Metrics reporting task
        let shutdown_metrics = self.shutdown.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                if shutdown_metrics.is_shutdown() {
                    println!("🛑 Metrics reporter shutting down");
                    break;
                }

                if config.enable_metrics {
                    let stats = metrics.get_stats();
                    println!("📊 Metrics Report:");
                    println!("  Total requests: {}", stats.total_requests);
                    println!("  Success rate: {:.1}%", stats.success_rate);
                    println!("  Avg latency: {}ms", stats.avg_latency_ms);
                    println!("  Total tokens: {}", stats.total_tokens_used);
                }
            }
        });

        println!("✅ Service started successfully\n");

        Ok(())
    }

    async fn handle_request(&self, prompt: &str) -> anyhow::Result<String> {
        let start = Instant::now();

        let result = tokio::time::timeout(
            Duration::from_secs(self.config.request_timeout_seconds),
            query(prompt, Some(self.config.to_claude_options())),
        )
        .await;

        let latency_ms = start.elapsed().as_millis() as u64;
        let (success, response) = match result {
            Ok(Ok(messages)) => {
                let text = messages
                    .iter()
                    .find_map(|m| {
                        if let Message::Assistant(msg) = m {
                            Some(
                                msg.message
                                .content
                                .iter()
                                .filter_map(|b| {
                                    if let ContentBlock::Text(t) = b {
                                        Some(t.text.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(" ")
                            )
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(String::new);

                (true, text)
            },
            Ok(Err(e)) => (false, format!("Error: {}", e)),
            Err(_) => (false, "Error: Request timeout".to_string()),
        };

        self.metrics.record_request(success, latency_ms, 0);

        if self.config.debug_mode {
            println!("🔍 Request: {}", prompt);
            println!("⏱️  Latency: {}ms", latency_ms);
            println!("✅ Success: {}", success);
        }

        Ok(response)
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        println!("\n🛑 Initiating graceful shutdown...");

        self.shutdown.trigger();

        // Give time for tasks to complete
        sleep(Duration::from_secs(2)).await;

        println!("✅ Shutdown complete");

        Ok(())
    }
}

// ============================================================================
// Deployment Examples
// ============================================================================

async fn example_deployment() -> anyhow::Result<()> {
    println!("🚀 Production Deployment Example\n");

    // Load configuration
    let config = ProductionConfig::from_env();

    // Initialize service
    let service = ProductionService::new(config);

    // Start service
    service.start().await?;

    // Simulate some requests
    println!("📨 Processing sample requests...\n");

    let requests = vec![
        "What is 2 + 2? Answer with just the number.",
        "What is the capital of France? One word.",
        "Explain Rust in one sentence.",
    ];

    for (i, prompt) in requests.iter().enumerate() {
        println!("Request {}:", i + 1);
        let response = service.handle_request(prompt).await?;
        println!("Response: {}\n", response);
        sleep(Duration::from_millis(500)).await;
    }

    // Show final metrics
    let stats = service.metrics.get_stats();
    println!("📊 Final Metrics:");
    println!("  Total requests: {}", stats.total_requests);
    println!("  Successful: {}", stats.successful_requests);
    println!("  Failed: {}", stats.failed_requests);
    println!("  Success rate: {:.1}%", stats.success_rate);
    println!("  Avg latency: {}ms", stats.avg_latency_ms);

    // Graceful shutdown
    service.shutdown().await?;

    Ok(())
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🌐 Production Deployment Patterns");
    println!("{}", "=".repeat(50));
    println!("\nThis example demonstrates:");
    println!("  ⚙️  Configuration management");
    println!("  📊 Metrics and monitoring");
    println!("  🏥 Health checks");
    println!("  🛑 Graceful shutdown");
    println!("  🔐 Error handling and timeouts");
    println!("\n{}\n", "=".repeat(50));

    example_deployment().await?;

    println!("\n{}", "=".repeat(50));
    println!("✅ Production Deployment Example Completed");
    println!("{}", "=".repeat(50));

    println!("\n🎯 Key Production Considerations:");
    println!("  ⚙️  Use environment variables for configuration");
    println!("  📊 Collect metrics for monitoring and debugging");
    println!("  🏥 Implement health checks for load balancers");
    println!("  🛑 Handle graceful shutdown to avoid dropped requests");
    println!("  ⏱️  Set appropriate timeouts for all operations");
    println!("  🔄 Implement retry logic with backoff");
    println!("  📝 Use structured logging for debugging");
    println!("  🔐 Secure secrets and API keys");
    println!("  🚀 Plan for deployment strategies (blue-green, canary)");

    Ok(())
}
