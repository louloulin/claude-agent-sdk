//! Comprehensive performance benchmarking suite for Claude Agent SDK.
//!
//! This benchmark suite measures:
//! - Query latency and throughput
//! - Memory usage patterns
//! - Streaming vs non-streaming performance
//! - Concurrent request handling
//! - Tool execution overhead

use claude_agent_sdk::{ContentBlock, Message, query, query_stream};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use futures::stream::StreamExt;
use std::time::Duration;

/// Benchmark simple query performance
fn bench_simple_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("query");

    group.bench_function("simple", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| query(black_box("What is 2 + 2?"), None));
    });

    group.finish();
}

/// Benchmark query with different prompt sizes
fn bench_query_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_by_size");

    for size in [10, 50, 100, 500].iter() {
        let prompt = format!("Repeat the word 'test' {} times: ", size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| query(black_box(&prompt), None));
        });
    }

    group.finish();
}

/// Benchmark streaming query performance
fn bench_streaming_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming");

    group.bench_function("stream_query", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                let stream_future = async {
                    let mut stream = query_stream(black_box("What is 2 + 2?"), None)
                        .await
                        .unwrap();
                    while let Some(_) = stream.next().await {
                        // Consume stream
                    }
                };
                stream_future
            });
    });

    group.finish();
}

/// Compare query() vs query_stream() performance
fn bench_query_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_comparison");

    group.bench_function("query_collect", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| query(black_box("List 10 programming languages"), None));
    });

    group.bench_function("query_stream", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                let stream_future = async {
                    let mut stream = query_stream(black_box("List 10 programming languages"), None)
                        .await
                        .unwrap();
                    while let Some(_) = stream.next().await {
                        // Consume stream
                    }
                };
                stream_future
            });
    });

    group.finish();
}

/// Benchmark concurrent queries
fn bench_concurrent_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent");
    group.measurement_time(Duration::from_secs(30));

    for concurrency in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| {
                        let futures: Vec<_> = (0..concurrency)
                            .map(|_| query(black_box("What is 2 + 2?"), None))
                            .collect();

                        async move {
                            let results = futures::future::join_all(futures).await;
                            black_box(results);
                        }
                    });
            },
        );
    }

    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    group.bench_function("large_response", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                query(
                    black_box("Explain quantum computing in great detail with examples"),
                    None,
                )
            });
    });

    group.bench_function("streaming_large_response", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                let stream_future = async {
                    let mut stream = query_stream(
                        black_box("Explain quantum computing in great detail with examples"),
                        None,
                    )
                    .await
                    .unwrap();
                    while let Some(_) = stream.next().await {
                        // Consume stream
                    }
                };
                stream_future
            });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_query,
    bench_query_by_size,
    bench_streaming_query,
    bench_query_comparison,
    bench_concurrent_queries,
    bench_memory_patterns
);

criterion_main!(benches);
