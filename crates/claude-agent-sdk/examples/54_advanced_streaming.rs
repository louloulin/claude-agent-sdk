//! Example: Advanced Streaming Patterns
//!
//! This example demonstrates advanced streaming patterns including
//! backpressure handling, stream composition, and real-time processing.
//!
//! What it demonstrates:
//! 1. Backpressure handling with buffer control
//! 2. Stream chunking and batching
//! 3. Stream composition and merging
//! 4. Real-time processing pipelines
//! 5. Stream timeout and cancellation

use anyhow::Result;
use futures::stream::{Stream, StreamExt, TryStreamExt};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Simulated message type for streaming
#[derive(Debug, Clone)]
struct StreamMessage {
    id: usize,
    content: String,
    timestamp: Instant,
}

/// Simulated stream chunk
#[derive(Debug, Clone)]
struct StreamChunk {
    messages: Vec<StreamMessage>,
    sequence: usize,
}

/// Stream processor configuration
#[derive(Debug, Clone)]
struct StreamConfig {
    buffer_size: usize,
    batch_size: usize,
    timeout_ms: u64,
    backpressure_threshold: usize,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 100,
            batch_size: 10,
            timeout_ms: 5000,
            backpressure_threshold: 80,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Advanced Streaming Patterns Examples ===\n");

    backpressure_example().await?;
    stream_chunking_example().await?;
    stream_composition_example().await?;
    real_time_pipeline_example().await?;
    stream_timeout_cancellation_example().await?;

    Ok(())
}

/// Demonstrates backpressure handling with buffer control
async fn backpressure_example() -> Result<()> {
    println!("=== Backpressure Handling ===\n");

    let config = StreamConfig::default();
    let processed = Arc::new(AtomicUsize::new(0));
    let buffer_usage = Arc::new(AtomicUsize::new(0));

    println!("Configuration:");
    println!("  Buffer size: {}", config.buffer_size);
    println!("  Backpressure threshold: {}%", config.backpressure_threshold);
    println!();

    // Simulate a fast producer and slow consumer
    println!("Simulating fast producer / slow consumer:");

    let producer_count = Arc::new(AtomicUsize::new(0));

    // Create a stream of messages
    let messages: Vec<StreamMessage> = (1..=20)
        .map(|id| StreamMessage {
            id,
            content: format!("Message {}", id),
            timestamp: Instant::now(),
        })
        .collect();

    let results: Vec<StreamMessage> = futures::stream::iter(messages)
        .map(|msg| {
            let producer_count = producer_count.clone();
            let buffer_usage = buffer_usage.clone();
            async move {
                // Simulate fast production
                let count = producer_count.fetch_add(1, Ordering::SeqCst) + 1;
                let buffer_percent = (count * 100) / 20;

                if buffer_percent > config.backpressure_threshold {
                    println!("  [BACKPRESSURE] Buffer at {}%", buffer_percent);
                    // In real code: signal producer to slow down
                }

                buffer_usage.store(count, Ordering::SeqCst);
                msg // Return message directly, no Result wrapper
            }
        })
        .buffer_unordered(5) // Limit concurrent items
        .then(|msg| {
            let processed = processed.clone();
            async move {
                // Simulate slow consumption
                tokio::time::sleep(Duration::from_millis(50)).await;
                let count = processed.fetch_add(1, Ordering::SeqCst) + 1;
                println!("  Processed: {} - Buffer: {} items", count, 20 - count);
                msg
            }
        })
        .collect()
        .await;

    println!("\nProcessed {} messages", results.len());

    println!("\nBackpressure strategies:");
    println!("  1. Buffer size limiting (buffer_unordered)");
    println!("  2. Throttle producer when threshold exceeded");
    println!("  3. Drop oldest messages if buffer full");
    println!("  4. Block producer until space available");
    println!();

    Ok(())
}

/// Demonstrates stream chunking and batching
async fn stream_chunking_example() -> Result<()> {
    println!("=== Stream Chunking and Batching ===\n");

    let batch_size = 5;

    // Create a stream of individual messages
    let messages: Vec<StreamMessage> = (1..=25)
        .map(|id| StreamMessage {
            id,
            content: format!("Content {}", id),
            timestamp: Instant::now(),
        })
        .collect();

    println!("Processing {} messages in chunks of {}", messages.len(), batch_size);

    // Chunk the stream into batches
    let mut sequence = 0;
    let chunks: Vec<StreamChunk> = futures::stream::iter(messages)
        .chunks(batch_size)
        .map(|messages| {
            sequence += 1;
            StreamChunk {
                messages,
                sequence,
            }
        })
        .collect()
        .await;

    println!("\nCreated {} chunks:", chunks.len());
    for chunk in &chunks {
        let ids: Vec<_> = chunk.messages.iter().map(|m| m.id).collect();
        println!("  Chunk {}: {:?} ({} items)", chunk.sequence, ids, chunk.messages.len());
    }

    println!("\nBenefits of chunking:");
    println!("  - Reduced API calls for batch endpoints");
    println!("  - Better throughput for high-volume streams");
    println!("  - Easier progress tracking");
    println!("  - Memory optimization for large datasets");
    println!();

    Ok(())
}

/// Demonstrates stream composition and merging
async fn stream_composition_example() -> Result<()> {
    println!("=== Stream Composition and Merging ===\n");

    // Create multiple streams
    let stream1: Pin<Box<dyn Stream<Item = Result<String>> + Send>> = Box::pin(
        futures::stream::iter(vec!["A1", "A2", "A3"])
            .map(|s| Ok(s.to_string())),
    );

    let stream2: Pin<Box<dyn Stream<Item = Result<String>> + Send>> = Box::pin(
        futures::stream::iter(vec!["B1", "B2", "B3"])
            .map(|s| Ok(s.to_string())),
    );

    let stream3: Pin<Box<dyn Stream<Item = Result<String>> + Send>> = Box::pin(
        futures::stream::iter(vec!["C1", "C2", "C3"])
            .map(|s| Ok(s.to_string())),
    );

    println!("Merging multiple streams:");

    // Merge streams (interleave items)
    let merged: Vec<String> = futures::stream::iter(vec![stream1, stream2, stream3])
        .flatten()
        .try_collect()
        .await?;

    println!("  Merged order: {:?}", merged);

    // Create another set for selective merge
    let stream_a = futures::stream::iter(vec!["X1", "X2"]);
    let stream_b = futures::stream::iter(vec!["Y1", "Y2"]);

    println!("\nSelecting from streams:");

    // Use select to take from whichever is ready first
    let mut combined = stream_a.chain(stream_b);
    let mut results = Vec::new();
    while let Some(item) = combined.next().await {
        results.push(item);
    }
    println!("  Chained order: {:?}", results);

    println!("\nStream composition patterns:");
    println!("  - merge: Interleave items from multiple streams");
    println!("  - chain: Process streams sequentially");
    println!("  - zip: Combine items from pairs of streams");
    println!("  - select: Take from whichever stream is ready");
    println!();

    Ok(())
}

/// Demonstrates real-time processing pipelines
async fn real_time_pipeline_example() -> Result<()> {
    println!("=== Real-time Processing Pipeline ===\n");

    let processed_count = Arc::new(AtomicUsize::new(0));

    println!("Pipeline stages:");
    println!("  1. Source: Generate messages");
    println!("  2. Transform: Process content");
    println!("  3. Filter: Remove invalid items");
    println!("  4. Enrich: Add metadata");
    println!("  5. Sink: Final processing");
    println!();

    // Create the pipeline
    let pipeline_results: Vec<(usize, String, Instant)> = futures::stream::iter(1..=10)
        // Stage 1: Source - generate messages
        .map(|id| {
            let content = format!("Raw message {}", id);
            (id, content, Instant::now())
        })
        // Stage 2: Transform - process content
        .map(|(id, content, ts)| {
            let transformed = content.to_uppercase();
            (id, transformed, ts)
        })
        // Stage 3: Filter - remove certain items (async)
        .filter(|(id, _, _)| {
            let id = *id;
            async move {
                // Filter out even numbers for this example
                id % 2 == 1
            }
        })
        // Stage 4: Enrich - add metadata (async)
        .then(|(id, content, ts)| {
            let processed_count = processed_count.clone();
            async move {
                let count = processed_count.fetch_add(1, Ordering::SeqCst) + 1;
                let enriched = format!("[#{}] {}", count, content);
                (id, enriched, ts)
            }
        })
        // Collect results
        .collect()
        .await;

    println!("Pipeline results:");
    for (id, content, ts) in &pipeline_results {
        println!("  ID {}: {} (age: {:?})", id, content, ts.elapsed());
    }

    println!("\nPipeline benefits:");
    println!("  - Clear separation of concerns");
    println!("  - Easy to test individual stages");
    println!("  - Composable and reusable");
    println!("  - Backpressure handling built-in");
    println!();

    Ok(())
}

/// Demonstrates stream timeout and cancellation
async fn stream_timeout_cancellation_example() -> Result<()> {
    println!("=== Stream Timeout and Cancellation ===\n");

    // Timeout per item
    println!("1. Per-item timeout:");

    let items_with_timeouts = vec![
        ("Fast item", 10),
        ("Slow item", 200),
        ("Very slow item", 500),
        ("Another fast item", 10),
    ];

    let item_timeout = Duration::from_millis(100);

    let results: Vec<_> = futures::stream::iter(items_with_timeouts)
        .map(|(name, delay_ms)| async move {
            match tokio::time::timeout(
                item_timeout,
                async move {
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    Ok::<_, anyhow::Error>(format!("Completed: {}", name))
                }
            ).await {
                Ok(Ok(result)) => (name, Some(result)),
                Ok(Err(e)) => (name, Some(format!("Error: {}", e))),
                Err(_) => (name, None), // Timeout
            }
        })
        .buffer_unordered(2)
        .collect()
        .await;

    for (name, result) in &results {
        match result {
            Some(r) => println!("  {} -> {}", name, r),
            None => println!("  {} -> TIMEOUT", name),
        }
    }

    // Stream cancellation
    println!("\n2. Stream cancellation with take:");

    let items: Vec<_> = (1..=100).collect();
    let start = Instant::now();

    let taken: Vec<_> = futures::stream::iter(items)
        .then(|i| async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            i
        })
        .take(5) // Only take first 5 items
        .collect()
        .await;

    println!("  Took {} items in {:?}", taken.len(), start.elapsed());
    println!("  Items: {:?}", taken);

    // Early termination condition
    println!("\n3. Early termination on condition:");

    let values: Vec<_> = futures::stream::iter(1..=100)
        .take_while(|&i| async move { i <= 10 }) // Stop when condition fails
        .collect()
        .await;

    println!("  Collected until condition: {:?}", values);

    println!("\nTimeout strategies:");
    println!("  - Per-item timeout: Maximum time for each item");
    println!("  - Total timeout: Maximum time for entire stream");
    println!("  - Cancellation: Stop processing on signal");
    println!("  - Take first N: Limit items processed");
    println!();

    Ok(())
}

/// Creates a throttled stream (for real usage)
#[allow(dead_code)]
fn throttled_stream<T>(
    stream: Pin<Box<dyn Stream<Item = T> + Send>>,
    items_per_second: usize,
) -> impl Stream<Item = T> {
    let interval = Duration::from_millis(1000 / items_per_second as u64);

    stream.then(move |item| async move {
        tokio::time::sleep(interval).await;
        item
    })
}

/// Creates a debounced stream (for real usage)
#[allow(dead_code)]
fn debounced_stream<T>(
    stream: Pin<Box<dyn Stream<Item = T> + Send>>,
    debounce_ms: u64,
) -> impl Stream<Item = T> {
    let debounce = Duration::from_millis(debounce_ms);

    stream.then(move |item| async move {
        tokio::time::sleep(debounce).await;
        item
    })
}

/// Creates a buffered stream with overflow handling (for real usage)
/// Note: This is a simplified example - in production you would use
/// buffer_unordered with futures or use a channels-based approach
#[allow(dead_code)]
fn buffered_with_overflow<T: Send + 'static>(
    _stream: Pin<Box<dyn Stream<Item = T> + Send>>,
    _buffer_size: usize,
) -> Pin<Box<dyn Stream<Item = T> + Send>> {
    // In real code, this would use a channel-based approach or
    // buffer_unordered with futures
    // Example pattern: stream.buffer_unordered(n)
    unimplemented!("This is a demonstration stub - implement with actual buffering logic")
}
