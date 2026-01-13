//! Advanced concurrency patterns for parallel query execution.
//!
//! Demonstrates:
//! - Parallel query execution with Tokio
//! - Concurrent rate limiting
//! - Batch processing with controlled concurrency
//! - Fan-out/fan-in patterns

use claude_agent_sdk::{ClaudeError, ContentBlock, Message, query};
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Execute multiple queries in parallel with concurrency control
async fn parallel_queries(
    prompts: Vec<String>,
    max_concurrency: usize,
) -> Vec<(String, Result<Vec<Message>, ClaudeError>)> {
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let start_time = Instant::now();

    let results = stream::iter(prompts.into_iter())
        .map(|prompt| {
            let semaphore = semaphore.clone();
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                let prompt_start = Instant::now();

                println!("  → Starting: {}", prompt);

                let result = query(&prompt, None).await;

                let elapsed = prompt_start.elapsed();
                match &result {
                    Ok(_) => println!("  ✓ Completed in {:.2}s: {}", elapsed.as_secs_f64(), prompt),
                    Err(e) => eprintln!(
                        "  ✗ Failed in {:.2}s: {} ({})",
                        elapsed.as_secs_f64(),
                        prompt,
                        e
                    ),
                }

                (prompt, result)
            }
        })
        .buffer_unordered(max_concurrency)
        .collect::<Vec<(String, Result<Vec<Message>, ClaudeError>)>>()
        .await;

    let total_elapsed = start_time.elapsed();
    println!(
        "\n📊 Total time: {:.2}s ({} queries, {} concurrent)",
        total_elapsed.as_secs_f64(),
        results.len(),
        max_concurrency
    );

    results
}

/// Batch processing with controlled concurrency
async fn batch_process<T, F>(
    items: Vec<T>,
    batch_size: usize,
    processor: F,
) -> Vec<(T, anyhow::Result<String>)>
where
    T: Send + 'static + Clone + std::fmt::Display,
    F: Fn(T) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send>>
        + Send
        + Sync
        + 'static,
{
    let processor = Arc::new(processor);

    stream::iter(items)
        .map(|item| {
            let processor = processor.clone();
            async move {
                let result = processor(item.clone()).await;
                (item, result)
            }
        })
        .buffer_unordered(batch_size)
        .collect()
        .await
}

/// Rate-limited query execution
struct RateLimiter {
    semaphore: Arc<Semaphore>,
    permits_per_second: usize,
}

impl RateLimiter {
    fn new(requests_per_second: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(requests_per_second)),
            permits_per_second: requests_per_second,
        }
    }

    async fn acquire(&self) {
        let _permit = self.semaphore.acquire().await.unwrap();

        // Throttle to maintain rate limit
        let interval = Duration::from_secs(1) / self.permits_per_second as u32;
        tokio::time::sleep(interval).await;
    }
}

/// Fan-out: Distribute work to multiple workers
async fn fan_out_pattern(
    prompts: Vec<String>,
    num_workers: usize,
) -> Vec<(String, Result<String, anyhow::Error>)> {
    println!(
        "🚀 Fan-out: Distributing {} queries to {} workers\n",
        prompts.len(),
        num_workers
    );

    let start_time = Instant::now();
    let prompts_per_worker = prompts.len().div_ceil(num_workers);

    // Create work channel (use bounded channel with sufficient capacity)
    let (tx, rx) = tokio::sync::mpsc::channel(num_workers * 10);
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    // Spawn workers
    for worker_id in 0..num_workers {
        let rx = rx.clone();
        tokio::spawn(async move {
            let mut processed = 0;
            loop {
                let prompt = {
                    let mut rx = rx.lock().await;
                    rx.recv().await
                };
                let prompt = match prompt {
                    Some(p) => p,
                    None => break, // Channel closed
                };
                println!("  [Worker {}] Processing: {}", worker_id + 1, prompt);

                match query(&prompt, None).await {
                    Ok(messages) => {
                        if let Some(Message::Assistant(msg)) = messages.first() {
                            for block in &msg.message.content {
                                if let ContentBlock::Text(text) = block {
                                    println!(
                                        "  [Worker {}] ✓ Result: {}",
                                        worker_id + 1,
                                        text.text.chars().take(50).collect::<String>()
                                    );
                                }
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("  [Worker {}] ✗ Error: {}", worker_id + 1, e);
                    },
                }

                processed += 1;
                if processed >= prompts_per_worker {
                    break;
                }
            }
        });
    }

    // Distribute work
    for prompt in prompts {
        if let Err(e) = tx.send(prompt).await {
            eprintln!("Send error: {}", e);
            return Vec::new();
        }
    }

    let elapsed = start_time.elapsed();
    println!("\n✅ Fan-out completed in {:.2}s\n", elapsed.as_secs_f64());

    // Simplified result (in real implementation, use channels to collect results)
    Vec::new()
}

/// Fan-in: Aggregate results from multiple sources
async fn fan_in_pattern(prompts: Vec<String>) -> std::collections::HashMap<String, Vec<String>> {
    println!(
        "🎯 Fan-in: Aggregating results from {} queries\n",
        prompts.len()
    );

    let mut results = std::collections::HashMap::new();

    for prompt in prompts {
        match query(&prompt, None).await {
            Ok(messages) => {
                let mut responses = Vec::new();
                for msg in messages {
                    if let Message::Assistant(assistant) = msg {
                        for block in assistant.message.content {
                            if let ContentBlock::Text(text) = block {
                                responses.push(text.text.clone());
                            }
                        }
                    }
                }
                results.insert(prompt, responses);
            },
            Err(e) => {
                eprintln!("  ✗ Failed: {} ({})", prompt, e);
                results.insert(prompt, vec![format!("Error: {}", e)]);
            },
        }
    }

    println!("✅ Aggregated {} results\n", results.len());
    results
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("⚡ Advanced Concurrency Patterns\n");

    // Example 1: Parallel queries with controlled concurrency
    println!("📡 Example 1: Parallel Queries (max 3 concurrent)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let prompts = vec![
        "What is 2 + 2? Answer in one word.".to_string(),
        "What is 3 + 3? Answer in one word.".to_string(),
        "What is 4 + 4? Answer in one word.".to_string(),
        "What is 5 + 5? Answer in one word.".to_string(),
        "What is 6 + 6? Answer in one word.".to_string(),
        "What is 7 + 7? Answer in one word.".to_string(),
    ];

    let results = parallel_queries(prompts, 3).await;

    println!("\n📊 Results:");
    for (prompt, result) in results {
        match result {
            Ok(messages) => {
                if let Some(Message::Assistant(msg)) = messages.first() {
                    for block in &msg.message.content {
                        if let ContentBlock::Text(text) = block {
                            println!("  {}: {}", prompt, text.text);
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("  {}: Error - {}", prompt, e);
            },
        }
    }

    println!("\n");

    // Example 2: Batch processing
    println!("📦 Example 2: Batch Processing");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let items = vec!["apple", "banana", "cherry", "date", "elderberry"];
    let results = batch_process(items, 2, |item| {
        Box::pin(async move {
            let prompt = format!("Describe {} in one sentence", item);
            let messages = query(&prompt, None).await?;

            for msg in messages {
                if let Message::Assistant(assistant) = msg {
                    for block in assistant.message.content {
                        if let ContentBlock::Text(text) = block {
                            return Ok(text.text.clone());
                        }
                    }
                }
            }

            Ok("No response".to_string())
        })
    })
    .await;

    println!("Results:");
    for (item, result) in results {
        match result {
            Ok(description) => println!("  {}: {}", item, description),
            Err(e) => eprintln!("  {}: Error - {}", item, e),
        }
    }

    println!("\n");

    // Example 3: Rate-limited queries
    println!("⏱️  Example 3: Rate-Limited Queries (2 req/sec)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let rate_limiter = RateLimiter::new(2);
    let start_time = Instant::now();

    for i in 1..=4 {
        rate_limiter.acquire().await;
        let prompt = format!("What is {} + {}? One word only.", i, i);
        println!("  Query {}: {}", i, prompt);

        match query(&prompt, None).await {
            Ok(messages) => {
                if let Some(Message::Assistant(msg)) = messages.first() {
                    for block in &msg.message.content {
                        if let ContentBlock::Text(text) = block {
                            println!("  Result: {}\n", text.text);
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("  Error: {}\n", e);
            },
        }
    }

    let elapsed = start_time.elapsed();
    println!(
        "Total time: {:.2}s (should be ~2s for 4 queries at 2 req/sec)\n",
        elapsed.as_secs_f64()
    );

    // Example 4: Fan-out pattern
    println!("🚀 Example 4: Fan-Out Pattern");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let prompts = vec![
        "What is Rust? One sentence.".to_string(),
        "What is Go? One sentence.".to_string(),
        "What is Python? One sentence.".to_string(),
        "What is JavaScript? One sentence.".to_string(),
    ];

    let _fan_out_results = fan_out_pattern(prompts, 2).await;

    // Example 5: Fan-in pattern
    println!("🎯 Example 5: Fan-In Pattern");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let prompts = vec![
        "What is 1 + 1? One number.".to_string(),
        "What is 2 + 2? One number.".to_string(),
        "What is 3 + 3? One number.".to_string(),
    ];

    let aggregated = fan_in_pattern(prompts).await;

    println!("Aggregated Results:");
    for (prompt, responses) in aggregated {
        if let Some(response) = responses.first() {
            println!("  {}: {}", prompt, response);
        }
    }

    println!();

    // Summary
    println!("{}", "=".repeat(50));
    println!("✅ Concurrency Patterns Examples Completed");
    println!("{}", "=".repeat(50));
    println!("\nKey Patterns:");
    println!("  📡 Parallel queries with semaphore-based concurrency control");
    println!("  📦 Batch processing with buffer_unordered");
    println!("  ⏱️  Rate limiting to prevent API overload");
    println!("  🚀 Fan-out: Distribute work across workers");
    println!("  🎯 Fan-in: Aggregate results from multiple sources");
    println!("\nPerformance Tips:");
    println!("  • Use buffer_unordered for CPU-bound tasks");
    println!("  • Use Semaphore for I/O-bound tasks");
    println!("  • Respect rate limits to avoid throttling");
    println!("  • Batch operations to reduce overhead");

    Ok(())
}
