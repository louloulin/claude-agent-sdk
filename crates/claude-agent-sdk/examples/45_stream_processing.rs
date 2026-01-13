//! Advanced stream processing example demonstrating memory-efficient handling
//! of large conversations and real-time message processing.

use claude_agent_sdk::{ContentBlock, Message, query_stream};
use futures::stream::StreamExt;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Advanced Stream Processing Example\n");

    // Example 1: Basic stream processing with real-time output
    println!("📡 Example 1: Real-time Stream Processing");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut stream = query_stream(
        "Count from 1 to 10, but take a breath between each number",
        None,
    )
    .await?;

    let mut message_count = 0;
    let start_time = Instant::now();

    while let Some(result) = stream.next().await {
        match result? {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        print!("{}", text.text);
                        // Flush to ensure immediate output
                        use std::io::Write;
                        std::io::stdout().flush()?;
                    }
                }
                message_count += 1;
            },
            Message::Result(result) if result.is_error => {
                eprintln!("\n❌ Error result received");
                break;
            },
            _ => {},
        }
    }

    let elapsed = start_time.elapsed();
    println!(
        "\n✅ Processed {} messages in {:.2}s\n",
        message_count,
        elapsed.as_secs_f64()
    );

    // Example 2: Stream with backpressure handling
    println!("📊 Example 2: Stream with Backpressure Control");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut stream = query_stream(
        "List 20 Python best practices with brief explanations",
        None,
    )
    .await?;

    let mut item_count = 0;
    let start_time = Instant::now();

    while let Some(result) = stream.next().await {
        match result? {
            Message::Assistant(msg) => {
                // Simulate processing time
                sleep(Duration::from_millis(50)).await;

                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        // Process text in chunks
                        let words: Vec<&str> = text.text.split_whitespace().collect();
                        for (i, word) in words.iter().enumerate() {
                            if i % 5 == 0 {
                                print!("\n  "); // New line every 5 words
                            }
                            print!("{} ", word);
                            item_count += 1;

                            // Simulate complex processing
                            if item_count % 50 == 0 {
                                sleep(Duration::from_millis(10)).await;
                            }
                        }
                    }
                }
            },
            Message::Result(result) if result.is_error => {
                eprintln!("\n❌ Error result received");
                break;
            },
            _ => {},
        }

        // Print progress every few messages
        if item_count % 100 == 0 {
            let elapsed = start_time.elapsed();
            let rate = item_count as f64 / elapsed.as_secs_f64();
            println!(
                "\n  📈 Progress: {} items ({:.1} items/s)",
                item_count, rate
            );
        }
    }

    let elapsed = start_time.elapsed();
    let rate = item_count as f64 / elapsed.as_secs_f64();
    println!(
        "\n✅ Processed {} items in {:.2}s ({:.1} items/s)\n",
        item_count,
        elapsed.as_secs_f64(),
        rate
    );

    // Example 3: Stream filtering and transformation
    println!("🔍 Example 3: Stream Filtering and Transformation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let query = "Explain the differences between Rust and Go in terms of:
1. Memory management
2. Concurrency model
3. Error handling
4. Learning curve";

    let mut stream = query_stream(query, None).await?;
    let mut filtered_content = Vec::new();
    let mut total_blocks = 0;

    while let Some(result) = stream.next().await {
        match result? {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        total_blocks += 1;

                        // Filter: Only keep blocks with substantial content
                        if text.text.len() > 20 {
                            // Transform: Add numbering
                            let transformed =
                                format!("[{}] {}", filtered_content.len() + 1, text.text);
                            filtered_content.push(transformed);
                            println!(
                                "  ✓ Captured block {} ({} chars)",
                                filtered_content.len(),
                                text.text.len()
                            );
                        } else {
                            println!("  ⊘ Skipped short block ({} chars)", text.text.len());
                        }
                    }
                }
            },
            Message::Result(result) if result.is_error => {
                eprintln!("❌ Error result received");
                break;
            },
            _ => {},
        }
    }

    println!("\n📊 Filtering Summary:");
    println!("  Total blocks: {}", total_blocks);
    println!("  Filtered blocks: {}", filtered_content.len());
    println!(
        "  Filter rate: {:.1}%",
        (filtered_content.len() as f64 / total_blocks as f64) * 100.0
    );

    // Example 4: Memory-efficient stream aggregation
    println!("\n💾 Example 4: Memory-Efficient Aggregation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut stream = query_stream(
        "Generate 5 tips for each: Python, JavaScript, Rust, and Go programming",
        None,
    )
    .await?;

    // Aggregate by language without storing all messages
    let mut tips_by_language = std::collections::HashMap::new();
    let mut current_language = String::new();

    while let Some(result) = stream.next().await {
        match result? {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        // Parse language from text
                        let text_lower = text.text.to_lowercase();
                        for lang in ["python", "javascript", "rust", "go"] {
                            if text_lower.contains(lang) && current_language != lang {
                                current_language = lang.to_string();
                                break;
                            }
                        }

                        // Count tips for current language
                        if !current_language.is_empty() {
                            *tips_by_language
                                .entry(current_language.clone())
                                .or_insert(0) += 1;
                        }
                    }
                }
            },
            Message::Result(result) if result.is_error => {
                eprintln!("❌ Error result received");
                break;
            },
            _ => {},
        }
    }

    println!("📊 Tips by Language:");
    for (lang, count) in &tips_by_language {
        println!("  {}: {} tips", lang, count);
    }

    // Summary
    let separator = "=".repeat(50);
    println!("\n{}", separator);
    println!("✅ Stream Processing Examples Completed");
    println!("{}", separator);
    println!("\nKey Takeaways:");
    println!("  🎯 Real-time processing reduces perceived latency");
    println!("  💾 O(1) memory per message vs O(n) for query()");
    println!("  🔄 Backpressure control prevents resource exhaustion");
    println!("  🔍 Stream filtering enables selective processing");
    println!("  📊 Aggregation can be done without storing all data");

    Ok(())
}
