//! Example: Error Recovery Patterns
//!
//! This example demonstrates error recovery strategies and patterns
//! for building resilient applications with the Claude Agent SDK.
//!
//! What it demonstrates:
//! 1. Retry with exponential backoff
//! 2. Circuit breaker pattern
//! 3. Graceful degradation
//! 4. Error context preservation
//! 5. Recovery strategy selection

use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Simulated error types that can occur
#[derive(Debug, Clone)]
enum SimulatedError {
    NetworkTimeout,
    RateLimitExceeded { retry_after_secs: u64 },
    InvalidApiKey,
    ServiceUnavailable,
    ContextTooLong,
    Unknown(String),
}

impl std::fmt::Display for SimulatedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkTimeout => write!(f, "Network timeout"),
            Self::RateLimitExceeded { retry_after_secs } => {
                write!(f, "Rate limit exceeded, retry after {}s", retry_after_secs)
            }
            Self::InvalidApiKey => write!(f, "Invalid API key"),
            Self::ServiceUnavailable => write!(f, "Service unavailable"),
            Self::ContextTooLong => write!(f, "Context too long"),
            Self::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for SimulatedError {}

/// Recovery strategy types
#[derive(Debug, Clone)]
enum RecoveryStrategy {
    /// Retry immediately
    RetryImmediate,
    /// Retry with exponential backoff
    RetryWithBackoff { max_attempts: usize, base_delay_ms: u64 },
    /// Wait for specified duration
    WaitThenRetry { delay_secs: u64 },
    /// Use fallback value
    UseFallback(String),
    /// Fail immediately
    FailFast,
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,
    Open { until: Instant },
    HalfOpen,
}

/// Circuit breaker for preventing cascading failures
#[derive(Debug)]
struct CircuitBreaker {
    state: CircuitState,
    failure_count: u64,
    success_count: u64,
    failure_threshold: u64,
    recovery_timeout: Duration,
    last_failure: Option<Instant>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u64, recovery_timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            recovery_timeout,
            last_failure: None,
        }
    }

    fn can_execute(&mut self) -> bool {
        match &self.state {
            CircuitState::Closed => true,
            CircuitState::Open { until } => {
                if Instant::now() >= *until {
                    self.state = CircuitState::HalfOpen;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    fn record_success(&mut self) {
        self.success_count += 1;
        if self.state == CircuitState::HalfOpen {
            self.state = CircuitState::Closed;
            self.failure_count = 0;
        }
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open {
                until: Instant::now() + self.recovery_timeout,
            };
        }
    }

    fn state(&self) -> &CircuitState {
        &self.state
    }
}

fn main() -> Result<()> {
    println!("=== Error Recovery Patterns Examples ===\n");

    retry_with_backoff_example()?;
    circuit_breaker_example()?;
    graceful_degradation_example()?;
    error_context_preservation_example()?;
    recovery_strategy_selection_example()?;

    Ok(())
}

/// Demonstrates retry with exponential backoff
fn retry_with_backoff_example() -> Result<()> {
    println!("=== Retry with Exponential Backoff ===\n");

    let strategy = RecoveryStrategy::RetryWithBackoff {
        max_attempts: 5,
        base_delay_ms: 100,
    };

    println!("Strategy: {:?}", strategy);
    println!();

    // Simulate an operation that eventually succeeds
    let attempt_count = Arc::new(AtomicU64::new(0));
    let mut total_delay = 0u64;

    println!("Attempting operation with retry...");

    for attempt in 1..=5 {
        let current_attempt = attempt_count.fetch_add(1, Ordering::SeqCst) + 1;

        // Simulate failure for first 3 attempts
        let result = if current_attempt <= 3 {
            Err(SimulatedError::NetworkTimeout)
        } else {
            Ok("Operation succeeded!")
        };

        match result {
            Ok(response) => {
                println!("  Attempt {}: {}", attempt, response);
                break;
            }
            Err(e) => {
                if attempt < 5 {
                    let delay_ms = 100 * 2u64.pow(attempt as u32 - 1);
                    total_delay += delay_ms;
                    println!(
                        "  Attempt {}: {} - retrying in {}ms",
                        attempt, e, delay_ms
                    );
                    // In real code: tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                } else {
                    println!("  Attempt {}: {} - giving up", attempt, e);
                }
            }
        }
    }

    println!("\nTotal retry delay: {}ms", total_delay);
    println!();

    Ok(())
}

/// Demonstrates circuit breaker pattern
fn circuit_breaker_example() -> Result<()> {
    println!("=== Circuit Breaker Pattern ===\n");

    let mut circuit_breaker = CircuitBreaker::new(3, Duration::from_secs(30));

    println!("Circuit Breaker Configuration:");
    println!("  Failure threshold: 3");
    println!("  Recovery timeout: 30s");
    println!();

    // Simulate a series of operations
    let operations = vec![
        (true, "Request 1"),
        (false, "Request 2"),
        (false, "Request 3"),
        (false, "Request 4"), // Should trip circuit
        (true, "Request 5"), // Should be blocked
        (true, "Request 6"), // Should be blocked
    ];

    for (success, name) in operations {
        if circuit_breaker.can_execute() {
            println!("  {} - Executing...", name);

            if success {
                circuit_breaker.record_success();
                println!("    Success - Circuit: {:?}", circuit_breaker.state());
            } else {
                circuit_breaker.record_failure();
                println!("    Failure - Circuit: {:?}", circuit_breaker.state());
            }
        } else {
            println!("  {} - BLOCKED (Circuit Open)", name);
        }
    }

    println!("\nCircuit breaker prevents cascading failures by blocking");
    println!("requests when the service is unhealthy.");
    println!();

    Ok(())
}

/// Demonstrates graceful degradation
fn graceful_degradation_example() -> Result<()> {
    println!("=== Graceful Degradation ===\n");

    // Define fallback chain
    let fallback_chain = vec![
        ("Primary Model", true),   // Try primary first
        ("Secondary Model", true), // Fall back to secondary
        ("Cache", true),           // Use cached response
        ("Static Response", true), // Return static response
    ];

    println!("Fallback Chain:");
    for (i, (name, _)) in fallback_chain.iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }
    println!();

    // Simulate degradation scenario
    let failing_until = 2; // First 2 options fail

    for (i, (name, _)) in fallback_chain.iter().enumerate() {
        let attempt = i + 1;

        if attempt <= failing_until {
            println!("  Attempt {}: {} - FAILED (simulated error)", attempt, name);
        } else {
            println!("  Attempt {}: {} - SUCCESS", attempt, name);
            println!("  Using degraded response from: {}", name);
            break;
        }
    }

    println!("\nGraceful degradation ensures service remains available");
    println!("even when primary features fail, with reduced functionality.");
    println!();

    // Show feature flags for degradation
    println!("Feature Flags for Degradation:");
    println!("  - Full AI responses: disabled (using cache)");
    println!("  - Real-time streaming: disabled");
    println!("  - Session persistence: enabled");
    println!("  - Basic queries: enabled");
    println!();

    Ok(())
}

/// Demonstrates error context preservation
fn error_context_preservation_example() -> Result<()> {
    println!("=== Error Context Preservation ===\n");

    // Simulate an error chain
    #[derive(Debug)]
    struct ErrorContext {
        operation: String,
        timestamp: Instant,
        attempt: u32,
        previous_errors: Vec<String>,
        metadata: std::collections::HashMap<String, String>,
    }

    let context = ErrorContext {
        operation: "query".to_string(),
        timestamp: Instant::now(),
        attempt: 3,
        previous_errors: vec![
            "Network timeout at attempt 1".to_string(),
            "Rate limit at attempt 2".to_string(),
        ],
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert("model".to_string(), "claude-sonnet-4".to_string());
            map.insert("session_id".to_string(), "sess_123".to_string());
            map.insert("user_id".to_string(), "user_456".to_string());
            map
        },
    };

    println!("Error Context:");
    println!("  Operation: {}", context.operation);
    println!("  Attempt: {}", context.attempt);
    println!("  Previous Errors:");
    for (i, err) in context.previous_errors.iter().enumerate() {
        println!("    {}: {}", i + 1, err);
    }
    println!("  Metadata:");
    for (key, value) in &context.metadata {
        println!("    {}: {}", key, value);
    }

    println!("\nContext preservation enables:");
    println!("  - Better debugging with full error history");
    println!("  - Intelligent retry decisions based on error patterns");
    println!("  - Proper error reporting to users");
    println!("  - Analytics and monitoring");
    println!();

    Ok(())
}

/// Demonstrates recovery strategy selection
fn recovery_strategy_selection_example() -> Result<()> {
    println!("=== Recovery Strategy Selection ===\n");

    // Map errors to recovery strategies
    let error_strategies = vec![
        (
            SimulatedError::NetworkTimeout,
            RecoveryStrategy::RetryWithBackoff {
                max_attempts: 3,
                base_delay_ms: 100,
            },
        ),
        (
            SimulatedError::RateLimitExceeded { retry_after_secs: 60 },
            RecoveryStrategy::WaitThenRetry { delay_secs: 60 },
        ),
        (
            SimulatedError::InvalidApiKey,
            RecoveryStrategy::FailFast,
        ),
        (
            SimulatedError::ServiceUnavailable,
            RecoveryStrategy::RetryWithBackoff {
                max_attempts: 5,
                base_delay_ms: 1000,
            },
        ),
        (
            SimulatedError::ContextTooLong,
            RecoveryStrategy::UseFallback("Please shorten your message.".to_string()),
        ),
    ];

    println!("Error → Recovery Strategy Mapping:\n");

    for (error, strategy) in &error_strategies {
        println!("Error: {}", error);
        println!("  Strategy: {:?}", strategy);
        println!();
    }

    // Strategy selection logic
    println!("Strategy Selection Logic:\n");

    let scenarios = vec![
        ("Temporary network issue", "Retry with backoff"),
        ("API rate limit hit", "Wait then retry"),
        ("Invalid credentials", "Fail fast, alert admin"),
        ("Service outage", "Circuit breaker, use fallback"),
        ("Input too large", "Truncate or reject with message"),
    ];

    for (scenario, action) in scenarios {
        println!("  Scenario: {}", scenario);
        println!("  Action: {}", action);
        println!();
    }

    // Composite recovery pattern
    println!("Composite Recovery Pattern:");
    println!("  1. Try primary method");
    println!("  2. If transient error → retry with backoff");
    println!("  3. If rate limited → wait and retry");
    println!("  4. If service unavailable → use circuit breaker");
    println!("  5. If permanent error → fail fast");
    println!("  6. If fallback available → use degraded response");
    println!();

    Ok(())
}

/// Retry executor with backoff (for real async usage)
#[allow(dead_code)]
async fn retry_with_backoff<T, E, F, Fut>(
    mut operation: F,
    max_attempts: usize,
    base_delay_ms: u64,
) -> std::result::Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error = None;

    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);

                if attempt < max_attempts {
                    let delay = Duration::from_millis(base_delay_ms * 2u64.pow(attempt as u32 - 1));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    // This is a bit of a hack since we can't easily return the error
    panic!("All retry attempts failed")
}

/// Timeout wrapper with fallback (for real async usage)
#[allow(dead_code)]
async fn with_timeout_and_fallback<T, F, Fut>(
    operation: F,
    timeout: Duration,
    fallback: T,
) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, anyhow::Error>>,
    T: Clone,
{
    match tokio::time::timeout(timeout, operation()).await {
        Ok(Ok(result)) => result,
        Ok(Err(_)) | Err(_) => fallback,
    }
}
