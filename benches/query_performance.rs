// Rust SDK性能基准测试
// 使用Criterion库进行精确的性能测量

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use claude_agent_sdk::{query, query_stream, UserContentBlock};
use futures::StreamExt;

// 基准测试配置
const SMALL_ITERATIONS: u64 = 50;    // 小测试迭代次数
const MEDIUM_ITERATIONS: u64 = 20;   // 中等测试迭代次数
const LARGE_ITERATIONS: u64 = 10;    // 大测试迭代次数

/// 测试简单查询的性能
fn bench_simple_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_query");

    // 测试不同大小的prompt
    for prompt_size in [10, 50, 100, 500].iter() {
        let prompt = "What is 2 + 2? ".repeat(*prompt_size);
        let iterations = if *prompt_size < 100 { SMALL_ITERATIONS } else { MEDIUM_ITERATIONS };

        group.bench_with_input(
            BenchmarkId::from_parameter(prompt_size),
            &prompt,
            |b, p| {
                b.iter(|| {
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(query(black_box(p.clone()), None))
                        .unwrap()
                })
            },
        );
    }

    group.finish();
}

/// 测试流式查询的性能
fn bench_streaming_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_query");

    group.bench_function("small_prompt", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    let mut stream = query_stream("What is 2 + 2?", None).await.unwrap();
                    while let Some(result) = stream.next().await {
                        result.unwrap();
                    }
                })
        })
    });

    group.finish();
}

/// 测试多模态查询性能（包含图片）
fn bench_multimodal_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("multimodal_query");

    // 模拟一个小的base64图片
    let fake_base64_image = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

    group.bench_function("text_only", |b| {
        let content = vec![UserContentBlock::text("What is 2 + 2?")];
        b.iter(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(claude_agent_sdk::query_with_content(
                    black_box(content.clone()),
                    None,
                ))
                .unwrap()
        })
    });

    group.bench_function("text_with_image", |b| {
        let content = vec![
            UserContentBlock::text("What's in this image?"),
            UserContentBlock::image_base64("image/png", black_box(fake_base64_image)).unwrap(),
        ];
        b.iter(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(claude_agent_sdk::query_with_content(
                    black_box(content.clone()),
                    None,
                ))
                .unwrap()
        })
    });

    group.finish();
}

/// 测试并发查询性能
fn bench_concurrent_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_queries");

    for num_queries in [1, 2, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_queries),
            num_queries,
            |b, &n| {
                b.iter(|| {
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(async {
                            let tasks: Vec<_> = (0..n)
                                .map(|_| {
                                    tokio::spawn(async {
                                        query("What is 2 + 2?", None).await.unwrap()
                                    })
                                })
                                .collect();

                            for task in tasks {
                                task.await.unwrap();
                            }
                        })
                })
            },
        );
    }

    group.finish();
}

/// 测试内存分配
fn bench_memory_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    group.throughput(Throughput::Elements(1));

    group.bench_function("single_query_memory", |b| {
        b.iter(|| {
            let prompt = "Explain quantum computing in one paragraph";
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(query(black_box(prompt), None))
                .unwrap()
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_query,
    bench_streaming_query,
    bench_multimodal_query,
    bench_concurrent_queries,
    bench_memory_allocations
);
criterion_main!(benches);
