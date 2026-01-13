//! Manual performance analysis and profiling tool.
//!
//! Run with: cargo test --test performance_analysis -- --nocapture --test-threads=1

use claude_agent_sdk::{ContentBlock, Message, query, query_stream};
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct PerformanceMetrics {
    operation_name: String,
    total_time: Duration,
    min_time: Duration,
    max_time: Duration,
    avg_time: Duration,
    iterations: usize,
    throughput_ops_per_sec: f64,
}

impl PerformanceMetrics {
    fn new(operation_name: String, times: Vec<Duration>) -> Self {
        let iterations = times.len();
        let total_time: Duration = times.iter().sum();
        let min_time = *times.iter().min().unwrap_or(&Duration::ZERO);
        let max_time = *times.iter().max().unwrap_or(&Duration::ZERO);
        let avg_time = if iterations > 0 {
            total_time / iterations as u32
        } else {
            Duration::ZERO
        };

        let throughput_ops_per_sec = if total_time.as_secs_f64() > 0.0 {
            iterations as f64 / total_time.as_secs_f64()
        } else {
            0.0
        };

        Self {
            operation_name,
            total_time,
            min_time,
            max_time,
            avg_time,
            iterations,
            throughput_ops_per_sec,
        }
    }

    fn print(&self) {
        println!("┌─ {} Performance", self.operation_name);
        println!("├─ Iterations: {}", self.iterations);
        println!("├─ Total time: {:.2}s", self.total_time.as_secs_f64());
        println!("├─ Average: {:.2}ms", self.avg_time.as_millis());
        println!("├─ Min: {:.2}ms", self.min_time.as_millis());
        println!("├─ Max: {:.2}ms", self.max_time.as_millis());
        println!("└─ Throughput: {:.2} ops/sec", self.throughput_ops_per_sec);
    }
}

/// Benchmark single query latency
async fn benchmark_query_latency(prompt: &str, iterations: usize) -> PerformanceMetrics {
    println!("\n🔬 Benchmarking: Query Latency");
    println!("Prompt: {}", prompt);

    let mut times = Vec::with_capacity(iterations);

    for i in 0..iterations {
        let start = Instant::now();
        let _result = query(prompt, None).await;
        let elapsed = start.elapsed();

        times.push(elapsed);

        if (i + 1) % 10 == 0 {
            println!("  Completed {}/{}", i + 1, iterations);
        }

        // Small delay to avoid overwhelming the system
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    PerformanceMetrics::new("Query Latency".to_string(), times)
}

/// Benchmark streaming query latency
async fn benchmark_stream_latency(prompt: &str, iterations: usize) -> PerformanceMetrics {
    println!("\n🔬 Benchmarking: Stream Latency");
    println!("Prompt: {}", prompt);

    let mut times = Vec::with_capacity(iterations);

    for i in 0..iterations {
        let start = Instant::now();

        let mut stream = query_stream(prompt, None).await.unwrap();
        while let Some(_) = stream.next().await {
            // Consume stream
        }

        let elapsed = start.elapsed();
        times.push(elapsed);

        if (i + 1) % 10 == 0 {
            println!("  Completed {}/{}", i + 1, iterations);
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    PerformanceMetrics::new("Stream Latency".to_string(), times)
}

/// Compare query() vs query_stream()
async fn compare_query_methods(
    prompt: &str,
    iterations: usize,
) -> HashMap<String, PerformanceMetrics> {
    let mut results = HashMap::new();

    println!("\n🔬 Comparing: query() vs query_stream()");

    // Benchmark query()
    let query_metrics = benchmark_query_latency(prompt, iterations).await;
    query_metrics.print();
    results.insert("query()".to_string(), query_metrics);

    // Benchmark query_stream()
    let stream_metrics = benchmark_stream_latency(prompt, iterations).await;
    stream_metrics.print();
    results.insert("query_stream()".to_string(), stream_metrics);

    // Comparison
    println!("\n📊 Comparison:");
    let query_avg = results.get("query()").unwrap().avg_time.as_millis();
    let stream_avg = results.get("query_stream()").unwrap().avg_time.as_millis();

    if query_avg > stream_avg {
        println!(
            "  query_stream() is {:.2}% faster",
            ((query_avg - stream_avg) as f64 / query_avg as f64) * 100.0
        );
    } else if stream_avg > query_avg {
        println!(
            "  query() is {:.2}% faster",
            ((stream_avg - query_avg) as f64 / stream_avg as f64) * 100.0
        );
    } else {
        println!("  Both methods have similar performance");
    }

    results
}

/// Benchmark concurrent queries
async fn benchmark_concurrent_queries(
    prompt: &str,
    concurrency_levels: Vec<usize>,
    iterations: usize,
) -> HashMap<usize, PerformanceMetrics> {
    let mut results = HashMap::new();

    println!("\n🔬 Benchmarking: Concurrent Queries");

    for concurrency in concurrency_levels {
        println!("\n  Concurrency level: {}", concurrency);

        let mut total_times = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();

            let futures: Vec<_> = (0..concurrency).map(|_| query(prompt, None)).collect();

            let _results = futures::future::join_all(futures).await;

            let elapsed = start.elapsed();
            total_times.push(elapsed);
        }

        let metrics = PerformanceMetrics::new(format!("Concurrent({})", concurrency), total_times);

        metrics.print();
        results.insert(concurrency, metrics);
    }

    results
}

/// Analyze memory usage patterns
async fn analyze_memory_usage(prompt: &str) -> anyhow::Result<()> {
    println!("\n🔬 Analyzing: Memory Usage Patterns");

    #[cfg(target_os = "linux")]
    {
        let mem_before = get_memory_usage();

        // Measure query() memory
        let start = Instant::now();
        let messages = query(prompt, None).await?;
        let elapsed_query = start.elapsed();

        let mem_after_query = get_memory_usage();

        println!("  query():");
        println!("    Time: {:.2}ms", elapsed_query.as_millis());
        println!(
            "    Memory delta: {} KB",
            mem_after_query.saturating_sub(mem_before)
        );

        // Calculate response size
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

        println!("    Response size: {} characters", total_chars);

        // Measure query_stream() memory
        let mem_before_stream = get_memory_usage();
        let start = Instant::now();

        let mut stream = query_stream(prompt, None).await?;
        let mut total_chars_stream = 0;

        while let Some(result) = stream.next().await {
            if let Ok(Message::Assistant(msg)) = result {
                for block in msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        total_chars_stream += text.text.len();
                    }
                }
            }
        }

        let elapsed_stream = start.elapsed();
        let mem_after_stream = get_memory_usage();

        println!("  query_stream():");
        println!("    Time: {:.2}ms", elapsed_stream.as_millis());
        println!(
            "    Memory delta: {} KB",
            mem_after_stream.saturating_sub(mem_before_stream)
        );
        println!("    Response size: {} characters", total_chars_stream);
    }

    #[cfg(not(target_os = "linux"))]
    {
        println!("  Memory analysis only available on Linux");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn get_memory_usage() -> i64 {
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

/// Performance regression testing
async fn regression_test() -> anyhow::Result<()> {
    println!("\n🔬 Running: Performance Regression Tests");

    let test_cases = vec![
        ("What is 2 + 2?", 1000, "Simple query"),
        ("List 10 programming languages", 2000, "Medium query"),
        ("Explain Rust ownership in detail", 3000, "Complex query"),
    ];

    let mut all_passed = true;

    for (prompt, max_time_ms, test_name) in test_cases {
        let start = Instant::now();
        let _result = query(prompt, None).await;
        let elapsed = start.elapsed();

        let passed = elapsed.as_millis() <= max_time_ms;

        println!(
            "  {}: {} ({}ms / {}ms)",
            if passed { "✅" } else { "❌" },
            test_name,
            elapsed.as_millis(),
            max_time_ms
        );

        if !passed {
            all_passed = false;
        }
    }

    if all_passed {
        println!("\n✅ All regression tests passed");
    } else {
        println!("\n⚠️  Some regression tests failed");
    }

    Ok(())
}

#[tokio::test]
#[ignore = "Performance analysis test requires significant time and API calls. Run with: cargo test --test performance_analysis -- --ignored"]
async fn test_full_performance_analysis() -> anyhow::Result<()> {
    let separator = "=".repeat(60);
    println!("\n{}", separator);
    println!("🔬 Claude Agent SDK - Performance Analysis Suite");
    println!("{}", separator);

    // Test parameters
    let iterations = 20;
    let test_prompt = "What is 2 + 2? Answer with just the number.";

    // 1. Query latency
    let query_metrics = benchmark_query_latency(test_prompt, iterations).await;
    query_metrics.print();

    // 2. Stream latency
    let stream_metrics = benchmark_stream_latency(test_prompt, iterations).await;
    stream_metrics.print();

    // 3. Comparison
    let comparison = compare_query_methods(test_prompt, 10).await;

    // 4. Concurrent queries
    let concurrency_levels = vec![1, 2, 4];
    let concurrent_results =
        benchmark_concurrent_queries("What is 2 + 2?", concurrency_levels, 5).await;

    // 5. Memory usage
    analyze_memory_usage("Explain memory management in programming").await?;

    // 6. Regression tests
    regression_test().await?;

    // Summary
    let separator = "=".repeat(60);
    println!("\n{}", separator);
    println!("📊 Performance Analysis Summary");
    println!("{}", separator);

    println!("\n🎯 Key Findings:");
    println!(
        "  • query() avg: {:.2}ms",
        comparison.get("query()").unwrap().avg_time.as_millis()
    );
    println!(
        "  • query_stream() avg: {:.2}ms",
        comparison
            .get("query_stream()")
            .unwrap()
            .avg_time
            .as_millis()
    );

    println!("\n⚡ Concurrent Performance:");
    for (level, metrics) in &concurrent_results {
        println!(
            "  • {}: {:.2} ops/sec",
            level, metrics.throughput_ops_per_sec
        );
    }

    let separator = "=".repeat(60);
    println!("\n{}", separator);
    println!("✅ Performance Analysis Complete");
    println!("{}", separator);

    Ok(())
}
