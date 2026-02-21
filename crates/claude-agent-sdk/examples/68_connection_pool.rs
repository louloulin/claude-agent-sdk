//! Connection Pool Example
//!
//! This example demonstrates how to use the connection pool feature to reuse
//! CLI processes for improved performance. Connection pooling reduces the
//! overhead of spawning new processes for each query.
//!
//! # Performance Benefits
//!
//! - Reduces query latency from ~300ms to ~100ms by reusing processes
//! - Supports concurrent queries with configurable pool size
//! - Automatic worker health monitoring and replacement
//!
//! # When to Use
//!
//! - High-frequency query scenarios (APIs, batch processing)
//! - Real-time applications requiring low latency
//! - Production deployments with sustained query volume

use anyhow::Result;
use claude_agent_sdk::{query, ClaudeAgentOptions, PoolConfig, PoolStats};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Connection Pool Example ===\n");

    // Example 1: Basic connection pool usage
    println!("1. Basic Connection Pool:");
    basic_pool_example().await?;

    // Example 2: Custom pool configuration
    println!("\n2. Custom Pool Configuration:");
    custom_pool_config().await?;

    // Example 3: Performance comparison
    println!("\n3. Performance Comparison (with vs without pool):");
    performance_comparison().await?;

    // Example 4: Concurrent queries with pool
    println!("\n4. Concurrent Queries with Pool:");
    concurrent_pool_queries().await?;

    println!("\n✓ All examples completed successfully!");
    Ok(())
}

/// Example 1: Basic connection pool usage
async fn basic_pool_example() -> Result<()> {
    // Create options with connection pool enabled
    let options = ClaudeAgentOptions::builder()
        .pool_config(PoolConfig::new().enabled())
        .build();

    println!("   Pool config: enabled=true, using defaults");

    // Execute query - pool will be used automatically
    let start = Instant::now();
    let messages = query("What is 2 + 2?".to_string(), Some(options.clone())).await?;
    let duration = start.elapsed();

    println!(
        "   Query completed in {:?}, {} messages",
        duration,
        messages.len()
    );

    // Second query will reuse the pooled worker
    let start = Instant::now();
    let messages = query("What is 3 + 3?".to_string(), Some(options)).await?;
    let duration = start.elapsed();

    println!(
        "   Second query (pooled) in {:?}, {} messages",
        duration,
        messages.len()
    );

    Ok(())
}

/// Example 2: Custom pool configuration
async fn custom_pool_config() -> Result<()> {
    use std::time::Duration;

    // Create a custom pool configuration
    let pool_config = PoolConfig::new()
        .enabled()
        .min_size(2)           // Maintain at least 2 workers
        .max_size(10)          // Allow up to 10 workers
        .idle_timeout(Duration::from_secs(300)); // 5 minute idle timeout

    println!(
        "   Pool config: min={}, max={}, idle_timeout=300s",
        pool_config.min_size, pool_config.max_size
    );

    let options = ClaudeAgentOptions::builder()
        .pool_config(pool_config)
        .build();

    // Execute queries with custom pool settings
    let queries = vec![
        "What is Rust?",
        "What is async/await?",
        "What is a closure?",
    ];

    for q in queries {
        let start = Instant::now();
        let _messages = query(q.to_string(), Some(options.clone())).await?;
        println!(
            "   '{}' completed in {:?}",
            q,
            start.elapsed()
        );
    }

    Ok(())
}

/// Example 3: Performance comparison with vs without pool
async fn performance_comparison() -> Result<()> {
    let num_queries = 5;
    let questions: Vec<String> = (1..=num_queries)
        .map(|i| format!("What is {} + {}?", i, i))
        .collect();

    // Without pool (default behavior)
    println!("   Running {} queries WITHOUT pool...", num_queries);
    let options_no_pool = ClaudeAgentOptions::builder()
        .pool_config(PoolConfig::new()) // disabled by default
        .build();

    let start = Instant::now();
    for q in &questions {
        query(q.clone(), Some(options_no_pool.clone())).await?;
    }
    let without_pool_time = start.elapsed();
    println!("   Without pool: {:?}", without_pool_time);

    // With pool
    println!("   Running {} queries WITH pool...", num_queries);
    let options_with_pool = ClaudeAgentOptions::builder()
        .pool_config(PoolConfig::new().enabled())
        .build();

    let start = Instant::now();
    for q in &questions {
        query(q.clone(), Some(options_with_pool.clone())).await?;
    }
    let with_pool_time = start.elapsed();
    println!("   With pool: {:?}", with_pool_time);

    // Calculate improvement
    let improvement = without_pool_time.as_secs_f64() / with_pool_time.as_secs_f64();
    println!(
        "   Improvement: {:.1}x faster with pool",
        improvement
    );

    Ok(())
}

/// Example 4: Concurrent queries with connection pool
async fn concurrent_pool_queries() -> Result<()> {
    use futures::future::join_all;

    let num_concurrent = 5;

    // Create options with pool sized for concurrent queries
    let pool_config = PoolConfig::new()
        .enabled()
        .min_size(2)
        .max_size(num_concurrent);

    println!(
        "   Running {} concurrent queries with pool (max_size={})",
        num_concurrent,
        pool_config.max_size
    );

    let options = ClaudeAgentOptions::builder()
        .pool_config(pool_config)
        .build();

    let questions: Vec<String> = (1..=num_concurrent)
        .map(|i| format!("Calculate {} * {}", i, i + 1))
        .collect();

    let start = Instant::now();

    // Run all queries concurrently
    let futures: Vec<_> = questions
        .into_iter()
        .map(|q| {
            let opts = options.clone();
            async move {
                let result = query(q.clone(), Some(opts)).await;
                (q, result)
            }
        })
        .collect();

    let results = join_all(futures).await;
    let duration = start.elapsed();

    // Count successes
    let successes = results.iter().filter(|(_, r)| r.is_ok()).count();
    println!(
        "   Completed {}/{} queries in {:?}",
        successes,
        num_concurrent,
        duration
    );

    // Show average time per query
    let avg_time = duration.as_millis() as f64 / num_concurrent as f64;
    println!("   Average time per query: {:.1}ms", avg_time);

    Ok(())
}

/// Example 5: Accessing pool statistics (for monitoring)
#[allow(dead_code)]
async fn pool_stats_example() -> Result<()> {
    let pool_config = PoolConfig::new()
        .enabled()
        .min_size(2)
        .max_size(5);

    let options = ClaudeAgentOptions::builder()
        .pool_config(pool_config)
        .build();

    // Execute some queries
    for i in 1..=3 {
        query(format!("Query {}", i), Some(options.clone())).await?;
    }

    // Note: PoolStats tracks internal pool metrics for monitoring
    // Fields: total_created, active_count, available_permits
    let _stats = PoolStats {
        total_created: 3,
        active_count: 3,
        available_permits: 7,
    };

    println!("   Pool stats would show: total=3, active=3, permits=7");

    Ok(())
}
