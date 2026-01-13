//! Concurrent Queries Example
//!
//! This example demonstrates how to run multiple queries concurrently
//! for improved performance and parallel processing.

use anyhow::Result;
use claude_agent_sdk::{Message, query, query_stream};
use futures::stream::{StreamExt, TryStreamExt};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Concurrent Queries Example ===\n");

    // Example 1: Sequential queries (baseline)
    println!("1. Sequential Queries:");
    let start = Instant::now();
    run_sequential_queries().await?;
    let sequential_time = start.elapsed();
    println!("   Total time: {:?}\n", sequential_time);

    // Example 2: Concurrent queries with join!
    println!("2. Concurrent Queries (join!):");
    let start = Instant::now();
    run_concurrent_queries_join().await?;
    let concurrent_time = start.elapsed();
    println!("   Total time: {:?}\n", concurrent_time);

    // Example 3: Concurrent queries with TaskPool
    println!("3. Concurrent Queries (TaskPool):");
    let start = Instant::now();
    run_concurrent_queries_taskpool().await?;
    let taskpool_time = start.elapsed();
    println!("   Total time: {:?}\n", taskpool_time);

    // Example 4: Streaming concurrent queries
    println!("4. Concurrent Streaming Queries:");
    let start = Instant::now();
    run_concurrent_streams().await?;
    let stream_time = start.elapsed();
    println!("   Total time: {:?}\n", stream_time);

    // Performance comparison
    println!("=== Performance Comparison ===");
    println!("Sequential:    {:?}", sequential_time);
    println!(
        "Concurrent:    {:?} ({:.1}x faster)",
        concurrent_time,
        sequential_time.as_secs_f64() / concurrent_time.as_secs_f64()
    );
    println!(
        "TaskPool:      {:?} ({:.1}x faster)",
        taskpool_time,
        sequential_time.as_secs_f64() / taskpool_time.as_secs_f64()
    );
    println!(
        "Streaming:     {:?} ({:.1}x faster)",
        stream_time,
        sequential_time.as_secs_f64() / stream_time.as_secs_f64()
    );

    Ok(())
}

/// Example 1: Run queries sequentially (baseline)
async fn run_sequential_queries() -> Result<()> {
    let questions = vec![
        "What is 2 + 2?",
        "What is the capital of France?",
        "Explain Rust ownership",
    ];

    for (i, question) in questions.iter().enumerate() {
        println!("   Query {}: {}", i + 1, question);
        let start = Instant::now();
        let _messages = query(question.to_string(), None).await?;
        println!("   Completed in {:?}", start.elapsed());
    }

    Ok(())
}

/// Example 2: Run queries concurrently using tokio::join!
async fn run_concurrent_queries_join() -> Result<()> {
    let questions = vec![
        "What is 2 + 2?",
        "What is the capital of France?",
        "Explain Rust ownership",
    ];

    // Create futures for each query
    let futures: Vec<_> = questions.into_iter().map(|q| {
        let q = q.to_string();
        async move {
            let start = Instant::now();
            let messages = query(q.to_string(), None).await?;
            println!("   Query completed in {:?}", start.elapsed());
            Ok::<Vec<Message>, anyhow::Error>(messages)
        }
    }).collect();

    // Use futures_util to run all futures concurrently
    use futures::future::join_all;
    let results = join_all(futures).await;
    let results: Result<Vec<_>, _> = results.into_iter().collect();
    let results = results?;
    println!("   All {} queries completed", results.len());

    Ok(())
}

/// Example 3: Run queries using TaskPool for controlled concurrency
async fn run_concurrent_queries_taskpool() -> Result<()> {
    use tokio::task::JoinSet;

    let questions = vec![
        "What is 2 + 2?",
        "What is the capital of France?",
        "Explain Rust ownership",
        "What is a closure?",
        "Explain async/await",
    ];

    let max_concurrent = 3;
    let mut join_set = JoinSet::new();
    let mut completed = 0;

    for (i, question) in questions.into_iter().enumerate() {
        // Wait if we've reached max concurrency
        while join_set.len() >= max_concurrent {
            if let Some(result) = join_set.join_next().await {
                let _ = result??;
                completed += 1;
                println!("   Completed {}/{}", completed, i + 1);
            }
        }

        println!("   Starting query {}: {}", i + 1, question);
        join_set.spawn(async move {
            let start = Instant::now();
            let messages = query(question.to_string(), None).await?;
            println!("   Query {} finished in {:?}", i + 1, start.elapsed());
            Ok::<Vec<Message>, anyhow::Error>(messages)
        });
    }

    // Wait for remaining tasks
    while let Some(result) = join_set.join_next().await {
        result??;
        completed += 1;
    }

    println!("   All {} queries completed", completed);
    Ok(())
}

/// Example 4: Run concurrent streaming queries
async fn run_concurrent_streams() -> Result<()> {
    let questions = vec![
        "What is 2 + 2?",
        "What is the capital of France?",
        "Explain Rust ownership",
    ];

    let streams = futures::stream::iter(questions)
        .map(|q| {
            let q = q.to_string();
            async move {
                let start = Instant::now();
                let mut stream = query_stream(q, None).await?;
                let mut count = 0;

                while let Some(result) = stream.next().await {
                    result?;
                    count += 1;
                }

                println!(
                    "   Stream completed in {:?}, {} messages",
                    start.elapsed(),
                    count
                );
                Ok::<(), anyhow::Error>(())
            }
        })
        .buffer_unordered(3); // Process up to 3 streams concurrently

    streams.try_collect::<Vec<_>>().await?;
    Ok(())
}

/// Example 5: Batch processing with concurrent queries
#[allow(dead_code)]
async fn batch_processing_example() -> Result<()> {
    use std::collections::HashMap;

    let data = vec![
        ("apple", "fruit"),
        ("carrot", "vegetable"),
        ("chicken", "meat"),
        ("bread", "grain"),
        ("cheese", "dairy"),
    ];

    println!("   Processing {} items concurrently", data.len());

    let results: Vec<_> = futures::stream::iter(data)
        .map(|(item, category)| async move {
            let question = format!("What is {}? It's a {}", item, category);
            let _messages = query(question.to_string(), None).await?;
            Ok::<(String, String), anyhow::Error>((item.to_string(), category.to_string()))
        })
        .buffer_unordered(3)
        .try_collect()
        .await?;

    println!("   Processed {} items", results.len());

    // Convert to HashMap
    let mut map: HashMap<String, String> = HashMap::new();
    for (key, value) in results {
        map.insert(key, value);
    }

    Ok(())
}

/// Example 6: Concurrent queries with error isolation
#[allow(dead_code)]
async fn error_isolation_example() -> Result<()> {
    let queries = vec![
        "Valid query 1",
        "Invalid query that might fail",
        "Valid query 2",
    ];

    let total_queries = queries.len();

    let results: Vec<_> = futures::stream::iter(queries)
        .map(|q| async move {
            match query(q.to_string(), None).await {
                Ok(messages) => {
                    println!("   ✓ Query succeeded: {}", q);
                    Some(messages)
                },
                Err(e) => {
                    println!("   ✗ Query failed: {} - {}", q, e);
                    None
                },
            }
        })
        .buffer_unordered(3)
        .collect()
        .await;

    let successful = results.iter().filter(|r| r.is_some()).count();
    println!("   {}/{} queries succeeded", successful, total_queries);

    Ok(())
}

/// Example 7: Rate-limited concurrent queries
#[allow(dead_code)]
async fn rate_limited_concurrent() -> Result<()> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tokio::sync::Semaphore;
    use tokio::time::{Duration, sleep};

    let queries = vec!["Query 1", "Query 2", "Query 3", "Query 4", "Query 5"];
    let queries_count = queries.len();

    let rate_limit_ms = 500u64; // Max 2 queries per second
    let last_call = Arc::new(AtomicU64::new(0));
    let semaphore = Arc::new(Semaphore::new(2)); // Max 2 concurrent

    println!("   Processing {} queries with rate limit", queries_count);

    let results: Vec<_> = futures::stream::iter(queries)
        .map(|q| {
            let semaphore = semaphore.clone();
            let last_call = last_call.clone();
            async move {
                let _permit = semaphore.acquire().await.unwrap();

                // Rate limiting logic
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                let last = last_call.load(Ordering::SeqCst);
                if last > 0 {
                    let elapsed = now.saturating_sub(last);
                    if elapsed < rate_limit_ms {
                        sleep(Duration::from_millis(rate_limit_ms - elapsed)).await;
                    }
                }
                last_call.store(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    Ordering::SeqCst,
                );

                println!("   Executing: {}", q);
                query(q.to_string(), None).await
            }
        })
        .buffer_unordered(2) // Max 2 concurrent
        .collect()
        .await;

    let successful = results.iter().filter(|r| r.is_ok()).count();
    println!("   {}/{} queries succeeded", successful, queries_count);

    Ok(())
}

/// Example 8: Concurrent queries with timeout
#[allow(dead_code)]
async fn concurrent_with_timeout() -> Result<()> {
    use std::time::Duration;

    let queries = vec![
        ("Quick query", "What is 2 + 2?"),
        ("Longer query", "Explain quantum computing"),
        ("Medium query", "What is Rust?"),
    ];
    let queries_count = queries.len();

    let timeout = Duration::from_secs(10);

    let results: Vec<_> = futures::stream::iter(queries)
        .map(|(name, q)| async move {
            let start = Instant::now();
            match tokio::time::timeout(timeout, query(q.to_string(), None)).await {
                Ok(Ok(messages)) => {
                    println!("   ✓ {} completed in {:?}", name, start.elapsed());
                    Some((name, messages))
                },
                Ok(Err(e)) => {
                    println!("   ✗ {} failed: {}", name, e);
                    None
                },
                Err(_) => {
                    println!("   ✗ {} timed out after {:?}", name, timeout);
                    None
                },
            }
        })
        .buffer_unordered(3)
        .collect()
        .await;

    let completed = results.iter().filter(|r| r.is_some()).count();
    println!(
        "   {}/{} queries completed within timeout",
        completed, queries_count
    );

    Ok(())
}
