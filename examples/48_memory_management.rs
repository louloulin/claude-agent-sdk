//! Memory management and optimization techniques.
//!
//! Demonstrates:
//! - Efficient memory usage patterns
//! - Stream processing for large datasets
//! - Buffer size optimization
//! - Memory profiling and monitoring

use claude_agent_sdk_rs::{ContentBlock, Message, query, query_stream};
use futures::stream::StreamExt;
use std::time::Instant;

/// Measure memory usage (platform-specific)
#[cfg(target_os = "linux")]
fn get_memory_usage() -> usize {
    use std::fs;
    let status = fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(kb) = parts.get(1) {
                return kb.parse().unwrap_or(0);
            }
        }
    }
    0
}

#[cfg(not(target_os = "linux"))]
fn get_memory_usage() -> usize {
    // Placeholder for non-Linux platforms
    0
}

/// Format memory size in human-readable format
fn format_memory(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    const GB: usize = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Compare memory usage between query() and query_stream()
async fn compare_memory_usage(prompt: &str) -> anyhow::Result<()> {
    println!("📊 Memory Comparison: query() vs query_stream()\n");
    println!("Prompt: {}\n", prompt);

    // Measure query() memory usage
    let mem_before = get_memory_usage();
    let start = Instant::now();

    let messages = query(prompt, None).await?;

    let elapsed = start.elapsed();
    let mem_after = get_memory_usage();
    let mem_used = if mem_after > mem_before {
        mem_after - mem_before
    } else {
        0
    };

    println!("query() Results:");
    println!("  Time: {:.2}s", elapsed.as_secs_f64());
    println!("  Memory used: {}", format_memory(mem_used * 1024));
    println!("  Messages: {}", messages.len());

    let total_chars: usize = messages
        .iter()
        .filter_map(|m| {
            if let Message::Assistant(msg) = m {
                Some(
                    msg.message
                        .content
                        .iter()
                        .filter_map(|b| {
                            if let ContentBlock::Text(t) = b {
                                Some(t.text.len())
                            } else {
                                Some(0)
                            }
                        })
                        .sum::<usize>(),
                )
            } else {
                None
            }
        })
        .sum();

    println!("  Total characters: {}\n", total_chars);

    // Measure query_stream() memory usage
    let mem_before_stream = get_memory_usage();
    let start_stream = Instant::now();

    let mut stream = query_stream(prompt, None).await?;
    let mut stream_messages = 0;
    let mut stream_chars = 0;

    while let Some(result) = stream.next().await {
        match result? {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        stream_chars += text.text.len();
                    }
                }
                stream_messages += 1;
            },
            _ => {},
        }
    }

    let elapsed_stream = start_stream.elapsed();
    let mem_after_stream = get_memory_usage();
    let mem_used_stream = if mem_after_stream > mem_before_stream {
        mem_after_stream - mem_before_stream
    } else {
        0
    };

    println!("query_stream() Results:");
    println!("  Time: {:.2}s", elapsed_stream.as_secs_f64());
    println!("  Memory used: {}", format_memory(mem_used_stream * 1024));
    println!("  Messages: {}", stream_messages);
    println!("  Total characters: {}\n", stream_chars);

    // Comparison
    println!("Comparison:");
    println!(
        "  Time difference: {:.2}%",
        ((elapsed.as_secs_f64() - elapsed_stream.as_secs_f64()) / elapsed_stream.as_secs_f64())
            * 100.0
    );
    println!(
        "  Memory savings: {:.2}%",
        if mem_used > 0 {
            ((mem_used - mem_used_stream) as f64 / mem_used as f64) * 100.0
        } else {
            0.0
        }
    );

    Ok(())
}

/// Process large datasets efficiently with streaming
async fn process_large_dataset() -> anyhow::Result<()> {
    println!("\n🔄 Large Dataset Processing with Streaming\n");

    // Generate a large prompt
    let large_prompt = format!(
        "Generate a comprehensive list of 50 programming best practices, \
         each with a brief explanation. Organize by category: \
         Code Quality, Performance, Security, Testing, and Documentation."
    );

    let mem_before = get_memory_usage();
    let start = Instant::now();

    let mut stream = query_stream(&large_prompt, None).await?;
    let mut categories = std::collections::HashMap::new();
    let mut total_items = 0;

    println!("Processing stream...");

    while let Some(result) = stream.next().await {
        match result? {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        // Categorize items without storing all text
                        let text_lower = text.text.to_lowercase();
                        for category in [
                            "code quality",
                            "performance",
                            "security",
                            "testing",
                            "documentation",
                        ] {
                            if text_lower.contains(category) {
                                *categories.entry(category).or_insert(0) += 1;
                                total_items += 1;
                                break;
                            }
                        }

                        // Print progress every few items
                        if total_items % 10 == 0 {
                            let elapsed = start.elapsed();
                            println!(
                                "  Processed {} items ({:.1} items/s)",
                                total_items,
                                total_items as f64 / elapsed.as_secs_f64()
                            );
                        }
                    }
                }
            },
            _ => {},
        }
    }

    let elapsed = start.elapsed();
    let mem_after = get_memory_usage();
    let mem_used = mem_after.saturating_sub(mem_before);

    println!("\nResults:");
    println!("  Total items: {}", total_items);
    println!("  Time: {:.2}s", elapsed.as_secs_f64());
    println!("  Memory used: {}", format_memory(mem_used * 1024));
    println!(
        "  Throughput: {:.1} items/s\n",
        total_items as f64 / elapsed.as_secs_f64()
    );

    println!("Categories:");
    for (category, count) in &categories {
        println!("  {}: {} items", category, count);
    }

    Ok(())
}

/// Buffer size optimization
async fn optimize_buffer_size() -> anyhow::Result<()> {
    println!("\n⚙️  Buffer Size Optimization\n");

    let test_sizes = vec![1, 10, 50, 100];
    let test_prompt = "List 20 tips for Rust programming beginners";

    for buffer_size in test_sizes {
        let start = Instant::now();

        let mut stream = query_stream(test_prompt, None).await?;
        let mut count = 0;
        let mut buffer = Vec::with_capacity(buffer_size);

        while let Some(result) = stream.next().await {
            if let Ok(Message::Assistant(msg)) = result {
                for block in msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        buffer.push(text.text);
                        count += 1;

                        // Process buffer when full
                        if buffer.len() >= buffer_size {
                            // Simulate processing
                            let _: Vec<_> = buffer.drain(..).collect();
                        }
                    }
                }
            }
        }

        let elapsed = start.elapsed();
        println!(
            "Buffer size {}: {:.2}s ({} items)",
            buffer_size,
            elapsed.as_secs_f64(),
            count
        );
    }

    Ok(())
}

/// Memory-efficient text processing
async fn efficient_text_processing() -> anyhow::Result<()> {
    println!("\n💾 Memory-Efficient Text Processing\n");

    let prompt =
        "Explain the following in detail: memory management in Rust, Go, Python, and JavaScript";

    let mut stream = query_stream(prompt, None).await?;

    // Process without storing all messages
    let mut word_count = 0;
    let mut language_counts = std::collections::HashMap::new();

    println!("Processing languages mentioned:");

    while let Some(result) = stream.next().await {
        match result? {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        // Count words without storing
                        word_count += text.text.split_whitespace().count();

                        // Track language mentions
                        let text_lower = text.text.to_lowercase();
                        for lang in ["rust", "go", "python", "javascript"] {
                            if text_lower.contains(lang) {
                                *language_counts.entry(lang).or_insert(0) += 1;
                            }
                        }
                    }
                }
            },
            _ => {},
        }
    }

    println!("\nResults:");
    println!("  Total words: {}", word_count);
    println!("  Language mentions:");
    for (lang, count) in &language_counts {
        println!("    {}: {} times", lang, count);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("💾 Memory Management and Optimization\n");
    println!("{}", "=".repeat(50));

    // Example 1: Memory comparison
    compare_memory_usage("Explain the differences between Rust and Go in 3 paragraphs").await?;

    // Example 2: Large dataset processing
    process_large_dataset().await?;

    // Example 3: Buffer size optimization
    optimize_buffer_size().await?;

    // Example 4: Efficient text processing
    efficient_text_processing().await?;

    // Summary
    println!("\n{}", "=".repeat(50));
    println!("✅ Memory Management Examples Completed");
    println!("{}", "=".repeat(50));
    println!("\nKey Takeaways:");
    println!("  🔄 Use streaming for large responses (O(1) memory per message)");
    println!("  📊 query() collects all data (O(n) memory)");
    println!("  ⚙️  Optimize buffer sizes for your use case");
    println!("  💾 Process data incrementally without storing everything");
    println!("  📈 Monitor memory usage in production");

    Ok(())
}
