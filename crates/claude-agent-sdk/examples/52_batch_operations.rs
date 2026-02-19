//! Example: Batch Operations
//!
//! This example demonstrates batch processing patterns for efficiently
//! handling multiple operations with progress tracking and error handling.
//!
//! What it demonstrates:
//! 1. Sequential vs parallel batch processing
//! 2. Chunked batch processing for large datasets
//! 3. Progress tracking with callbacks
//! 4. Error aggregation and partial success handling
//! 5. Batch result aggregation and statistics

use anyhow::Result;
use claude_agent_sdk::{query, Message, ClaudeAgentOptions, ClaudeClient, PermissionMode};
use futures::stream::{StreamExt, TryStreamExt};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Represents a batch item to process
#[derive(Debug, Clone)]
struct BatchItem {
    id: usize,
    prompt: String,
    category: String,
}

/// Represents the result of processing a batch item
#[derive(Debug)]
struct BatchResult {
    item_id: usize,
    success: bool,
    response: Option<String>,
    error: Option<String>,
    duration_ms: u64,
}

/// Progress callback type
type ProgressCallback = Arc<dyn Fn(usize, usize, &str) + Send + Sync>;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Batch Operations Examples ===\n");

    sequential_batch_example().await?;
    parallel_batch_example().await?;
    chunked_batch_example().await?;
    progress_tracking_example().await?;
    error_aggregation_example()?;

    Ok(())
}

/// Demonstrates sequential batch processing
async fn sequential_batch_example() -> Result<()> {
    println!("=== Sequential Batch Processing ===\n");

    let items = create_sample_items(5);
    let mut results = Vec::new();
    let start = Instant::now();

    for item in &items {
        let item_start = Instant::now();
        println!("Processing item {}: {}", item.id, item.prompt);

        // In a real scenario, you would call the API here
        // For this example, we simulate the result
        let result = BatchResult {
            item_id: item.id,
            success: true,
            response: Some(format!("Response for: {}", item.prompt)),
            error: None,
            duration_ms: item_start.elapsed().as_millis() as u64,
        };
        results.push(result);
    }

    let total_time = start.elapsed();
    print_batch_summary(&results, total_time);

    println!();
    Ok(())
}

/// Demonstrates parallel batch processing with controlled concurrency
async fn parallel_batch_example() -> Result<()> {
    println!("=== Parallel Batch Processing ===\n");

    let items = create_sample_items(10);
    let max_concurrent = 3;
    let start = Instant::now();

    println!("Processing {} items with max {} concurrent", items.len(), max_concurrent);

    let results: Vec<BatchResult> = futures::stream::iter(items.clone())
        .map(|item| async move {
            let item_start = Instant::now();
            println!("  Starting item {}: {}", item.id, item.prompt);

            // Simulate processing
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            BatchResult {
                item_id: item.id,
                success: true,
                response: Some(format!("Processed: {}", item.category)),
                error: None,
                duration_ms: item_start.elapsed().as_millis() as u64,
            }
        })
        .buffer_unordered(max_concurrent)
        .collect()
        .await;

    let total_time = start.elapsed();
    print_batch_summary(&results, total_time);

    println!();
    Ok(())
}

/// Demonstrates chunked batch processing for large datasets
async fn chunked_batch_example() -> Result<()> {
    println!("=== Chunked Batch Processing ===\n");

    let items = create_sample_items(20);
    let chunk_size = 5;
    let chunks: Vec<Vec<BatchItem>> = items.chunks(chunk_size).map(|c| c.to_vec()).collect();

    println!("Processing {} items in {} chunks of size {}", items.len(), chunks.len(), chunk_size);

    let start = Instant::now();
    let mut all_results = Vec::new();

    for (chunk_idx, chunk) in chunks.iter().enumerate() {
        println!("\n  Processing chunk {}/{}", chunk_idx + 1, chunks.len());

        let chunk_results: Vec<BatchResult> = futures::stream::iter(chunk.clone())
            .map(|item| async move {
                let item_start = Instant::now();
                tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

                BatchResult {
                    item_id: item.id,
                    success: true,
                    response: Some(format!("Chunk processed: {}", item.prompt)),
                    error: None,
                    duration_ms: item_start.elapsed().as_millis() as u64,
                }
            })
            .buffer_unordered(chunk_size)
            .collect()
            .await;

        let successful = chunk_results.iter().filter(|r| r.success).count();
        println!("    Chunk {} completed: {}/{} successful", chunk_idx + 1, successful, chunk_results.len());

        all_results.extend(chunk_results);
    }

    let total_time = start.elapsed();
    print_batch_summary(&all_results, total_time);

    println!();
    Ok(())
}

/// Demonstrates progress tracking with callbacks
async fn progress_tracking_example() -> Result<()> {
    println!("=== Progress Tracking ===\n");

    let items = create_sample_items(8);
    let total = items.len();
    let completed = Arc::new(AtomicUsize::new(0));

    // Create progress callback
    let progress_callback: ProgressCallback = Arc::new(move |done, total, current_item| {
        let percentage = (done as f64 / total as f64 * 100.0) as usize;
        println!("  Progress: {}/{} ({}%) - {}", done, total, percentage, current_item);
    });

    let start = Instant::now();

    let results: Vec<BatchResult> = futures::stream::iter(items.clone())
        .map(|item| {
            let completed = completed.clone();
            let callback = progress_callback.clone();
            async move {
                let item_start = Instant::now();

                // Report progress before processing
                let done = completed.fetch_add(1, Ordering::SeqCst);
                callback(done, total, &item.prompt);

                // Simulate processing
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

                BatchResult {
                    item_id: item.id,
                    success: true,
                    response: Some(format!("Progress tracked: {}", item.category)),
                    error: None,
                    duration_ms: item_start.elapsed().as_millis() as u64,
                }
            }
        })
        .buffer_unordered(3)
        .collect()
        .await;

    let total_time = start.elapsed();
    print_batch_summary(&results, total_time);

    println!();
    Ok(())
}

/// Demonstrates error aggregation and partial success handling
fn error_aggregation_example() -> Result<()> {
    println!("=== Error Aggregation ===\n");

    // Simulate batch results with some failures
    let results = vec![
        BatchResult {
            item_id: 1,
            success: true,
            response: Some("Success 1".to_string()),
            error: None,
            duration_ms: 100,
        },
        BatchResult {
            item_id: 2,
            success: false,
            response: None,
            error: Some("Connection timeout".to_string()),
            duration_ms: 5000,
        },
        BatchResult {
            item_id: 3,
            success: true,
            response: Some("Success 3".to_string()),
            error: None,
            duration_ms: 150,
        },
        BatchResult {
            item_id: 4,
            success: false,
            response: None,
            error: Some("Rate limit exceeded".to_string()),
            duration_ms: 50,
        },
        BatchResult {
            item_id: 5,
            success: true,
            response: Some("Success 5".to_string()),
            error: None,
            duration_ms: 200,
        },
    ];

    // Aggregate results
    let successful: Vec<_> = results.iter().filter(|r| r.success).collect();
    let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();

    println!("Batch Results Summary:");
    println!("  Total items: {}", results.len());
    println!("  Successful: {}", successful.len());
    println!("  Failed: {}", failed.len());

    if !failed.is_empty() {
        println!("\nFailed Items:");
        for result in &failed {
            println!("  Item {}: {}", result.item_id, result.error.as_ref().unwrap());
        }

        // Error categorization
        let mut error_types = std::collections::HashMap::new();
        for result in &failed {
            let error_msg = result.error.as_ref().unwrap();
            let error_type = if error_msg.contains("timeout") {
                "Timeout"
            } else if error_msg.contains("rate limit") {
                "Rate Limit"
            } else {
                "Other"
            };
            *error_types.entry(error_type).or_insert(0) += 1;
        }

        println!("\nError Types:");
        for (error_type, count) in &error_types {
            println!("  {}: {}", error_type, count);
        }
    }

    // Success rate
    let success_rate = successful.len() as f64 / results.len() as f64 * 100.0;
    println!("\nSuccess Rate: {:.1}%", success_rate);

    // Partial success handling strategy
    println!("\nPartial Success Strategies:");
    println!("  1. Retry failed items with exponential backoff");
    println!("  2. Log failures and continue with successful results");
    println!("  3. Fail entire batch if success rate < threshold");
    println!("  4. Queue failed items for later retry");

    println!();
    Ok(())
}

// ============== Helper Functions ==============

/// Creates sample batch items for demonstration
fn create_sample_items(count: usize) -> Vec<BatchItem> {
    let categories = vec!["general", "code", "math", "creative", "analysis"];

    (1..=count)
        .map(|id| BatchItem {
            id,
            prompt: format!("Question {} about {}", id, categories[id % categories.len()]),
            category: categories[id % categories.len()].to_string(),
        })
        .collect()
}

/// Prints a summary of batch results
fn print_batch_summary(results: &[BatchResult], total_time: std::time::Duration) {
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    let total_duration_ms: u64 = results.iter().map(|r| r.duration_ms).sum();
    let avg_duration_ms = if !results.is_empty() {
        total_duration_ms as f64 / results.len() as f64
    } else {
        0.0
    };

    println!("\n  Batch Summary:");
    println!("    Total items: {}", results.len());
    println!("    Successful: {}", successful);
    println!("    Failed: {}", failed);
    println!("    Total time: {:?}", total_time);
    println!("    Avg item time: {:.1}ms", avg_duration_ms);
    println!("    Throughput: {:.1} items/sec", results.len() as f64 / total_time.as_secs_f64());
}

/// Batch processor with retry logic (for real usage)
#[allow(dead_code)]
async fn batch_with_retry(
    items: Vec<BatchItem>,
    max_retries: usize,
) -> Result<Vec<BatchResult>> {
    let mut results = Vec::new();

    for item in items {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts <= max_retries {
            attempts += 1;

            // Simulate API call
            let success = attempts > 1; // Simulate success on retry

            if success {
                results.push(BatchResult {
                    item_id: item.id,
                    success: true,
                    response: Some("Success after retry".to_string()),
                    error: None,
                    duration_ms: 100 * attempts as u64,
                });
                break;
            } else {
                last_error = Some("Temporary failure".to_string());

                // Exponential backoff
                let delay = std::time::Duration::from_millis(100 * 2u64.pow(attempts as u32 - 1));
                tokio::time::sleep(delay).await;
            }
        }

        if attempts > max_retries {
            results.push(BatchResult {
                item_id: item.id,
                success: false,
                response: None,
                error: last_error,
                duration_ms: 100 * max_retries as u64,
            });
        }
    }

    Ok(results)
}

/// Batch processor with timeout per item (for real usage)
#[allow(dead_code)]
async fn batch_with_timeout(
    items: Vec<BatchItem>,
    timeout_secs: u64,
) -> Result<Vec<BatchResult>> {
    let timeout = tokio::time::Duration::from_secs(timeout_secs);

    let results: Vec<BatchResult> = futures::stream::iter(items)
        .map(|item| async move {
            let item_start = Instant::now();

            match tokio::time::timeout(timeout, async {
                // Simulate processing
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                Ok::<_, anyhow::Error>(format!("Response for {}", item.prompt))
            })
            .await
            {
                Ok(Ok(response)) => BatchResult {
                    item_id: item.id,
                    success: true,
                    response: Some(response),
                    error: None,
                    duration_ms: item_start.elapsed().as_millis() as u64,
                },
                Ok(Err(e)) => BatchResult {
                    item_id: item.id,
                    success: false,
                    response: None,
                    error: Some(e.to_string()),
                    duration_ms: item_start.elapsed().as_millis() as u64,
                },
                Err(_) => BatchResult {
                    item_id: item.id,
                    success: false,
                    response: None,
                    error: Some(format!("Timeout after {}s", timeout_secs)),
                    duration_ms: item_start.elapsed().as_millis() as u64,
                },
            }
        })
        .buffer_unordered(3)
        .collect()
        .await;

    Ok(results)
}
