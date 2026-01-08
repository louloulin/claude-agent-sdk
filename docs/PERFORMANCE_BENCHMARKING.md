# 性能基准测试指南 (Performance Benchmarking Guide)

## 📊 概述 (Overview)

本指南介绍如何使用 Claude Agent SDK Rust 的性能基准测试套件来测量和分析性能。

This guide explains how to use the Claude Agent SDK Rust performance benchmarking suite to measure and analyze performance.

## 🚀 快速开始 (Quick Start)

### 安装依赖 (Install Dependencies)

```bash
# 安装 Criterion.rs 基准测试工具
cargo install cargo-criterion

# 或在 Cargo.toml 中添加
[dev-dependencies]
criterion = "0.5"
```

### 运行基准测试 (Run Benchmarks)

```bash
# 运行完整的基准测试套件
cargo criterion

# 运行特定基准测试
cargo criterion --bench benchmark_suite

# 生成图表（需要 gnuplot）
cargo criterion --plotting-format gnuplot
```

### 运行性能分析测试 (Run Performance Analysis)

```bash
# 运行手动性能分析
cargo test --test performance_analysis -- --nocapture --test-threads=1
```

## 📈 基准测试类型 (Benchmark Types)

### 1. 简单查询基准 (Simple Query Benchmark)

测试单个查询的延迟。

Measures latency for single queries.

```rust
// benches/benchmark_suite.rs
criterion_group!(benches, bench_simple_query);
```

**运行**:
```bash
cargo criterion --bench benchmark_suite
```

**输出示例**:
```
query/simple
                        time:   [2.3456 s 2.4567 s 2.5678 s]
                        thrpt:  [0.3894 elem/s 0.4072 elem/s 0.4261 elem/s]
```

### 2. 流式查询基准 (Streaming Query Benchmark)

比较 `query()` vs `query_stream()` 的性能。

Compares performance between `query()` and `query_stream()`.

```rust
criterion_group!(benches, bench_streaming_query, bench_query_comparison);
```

**关键指标**:
- **延迟 (Latency)**: 响应时间
- **吞吐量 (Throughput)**: 每秒请求数
- **内存使用 (Memory)**: 内存占用

### 3. 并发查询基准 (Concurrent Query Benchmark)

测试不同并发级别下的性能。

Tests performance at different concurrency levels.

```rust
criterion_group!(benches, bench_concurrent_queries);
```

**并发级别** (Concurrency Levels):
- 1 (顺序执行)
- 2 (2 个并发请求)
- 4 (4 个并发请求)
- 8 (8 个并发请求)

### 4. 内存模式基准 (Memory Pattern Benchmark)

分析内存使用模式。

Analyzes memory usage patterns.

```rust
criterion_group!(benches, bench_memory_patterns);
```

## 🔬 性能分析工具 (Performance Analysis Tools)

### 手动性能分析 (Manual Performance Analysis)

```bash
cargo test --test performance_analysis -- --nocapture
```

**分析项目** (Analysis Items):
1. ✅ 查询延迟分析 (Query Latency Analysis)
2. ✅ 流式查询分析 (Stream Latency Analysis)
3. ✅ 方法对比 (Method Comparison)
4. ✅ 并发性能 (Concurrent Performance)
5. ✅ 内存使用 (Memory Usage)
6. ✅ 回归测试 (Regression Tests)

### 性能指标 (Performance Metrics)

#### 延迟指标 (Latency Metrics)

| 指标 | 描述 | 目标值 |
|------|------|--------|
| 平均延迟 (Average) | 所有请求的平均响应时间 | < 2s |
| 最小延迟 (Min) | 最快的请求时间 | > 500ms |
| 最大延迟 (Max) | 最慢的请求时间 | < 5s |
| P50 | 中位数延迟 | < 1.5s |
| P95 | 95% 请求的延迟 | < 3s |
| P99 | 99% 请求的延迟 | < 4s |

#### 吞吐量指标 (Throughput Metrics)

| 指标 | 描述 | 目标值 |
|------|------|--------|
| 请求/秒 (RPS) | 每秒处理的请求数 | > 0.5 |
| 并发吞吐量 | 并发场景下的吞吐量 | 随并发增加 |
| 效率 | 实际吞吐量/理论吞吐量 | > 80% |

#### 内存指标 (Memory Metrics)

| 指标 | 描述 | 目标值 |
|------|------|--------|
| 峰值内存 | 最大内存使用 | < 100MB |
| 平均内存 | 平均内存使用 | < 50MB |
| 内存增长 | 内存泄漏检测 | 0 |

## 📊 结果解读 (Interpreting Results)

### Criterion 输出 (Criterion Output)

```
query/simple
                        time:   [2.3456 s 2.4567 s 2.5678 s]
                        thrpt:  [0.3894 elem/s 0.4072 elem/s 0.4261 elem/s]
                 change:
                        [-5.234% +2.345% +9.123%] (p = 0.03 < 0.05)
                        Performance has improved.
```

**解读说明** (Interpretation):
- **time**: 平均执行时间和置信区间
- **thrpt**: 吞吐量（每秒操作数）
- **change**: 与上次基准测试的对比
  - 绿色 ✅: 性能提升
  - 红色 ❌: 性能下降
  - 灰色 ⚪: 无显著变化

### 性能回归检测 (Performance Regression Detection)

```bash
# 保存基线性能
cargo criterion --save-baseline main

# 与基线对比
cargo criterion --baseline main
```

## 🎯 性能优化建议 (Performance Optimization Tips)

### 1. 查询优化 (Query Optimization)

```rust
// ❌ 避免重复查询
for item in items {
    let result = query(&format!("Process {}", item), None).await;
}

// ✅ 使用批量查询
let combined = items.iter()
    .map(|i| format!("Process {}", i))
    .collect::<Vec<_>>()
    .join("\n");
let results = query(&combined, None).await;
```

### 2. 流式处理 (Stream Processing)

```rust
// ✅ 对于大响应使用流式处理
let mut stream = query_stream("Generate 100 items", None).await?;
while let Some(result) = stream.next().await {
    // 实时处理，O(1) 内存
}
```

### 3. 并发控制 (Concurrency Control)

```rust
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(10)); // 最多 10 个并发

let tasks = prompts.into_iter().map(|prompt| {
    let semaphore = semaphore.clone();
    async move {
        let _permit = semaphore.acquire().await.unwrap();
        query(&prompt, None).await
    }
});

futures::future::join_all(tasks).await;
```

### 4. 缓存策略 (Caching Strategy)

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

let cache = Arc::new(RwLock::new(HashMap::new()));

// 检查缓存
{
    let cache_read = cache.read().await;
    if let Some(cached) = cache_read.get(&prompt) {
        return Ok(cached.clone());
    }
}

// 查询并缓存
let result = query(&prompt, None).await?;

{
    let mut cache_write = cache.write().await;
    cache_write.insert(prompt, result.clone());
}
```

## 🔧 性能调优 (Performance Tuning)

### 1. Tokio 运行时配置 (Tokio Runtime Configuration)

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 默认配置适合大多数场景
    // 对于高并发场景，可以自定义
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)           // 工作线程数
        .max_blocking_threads(512)   // 最大阻塞线程数
        .enable_all()
        .build()?;

    runtime.block_on(async {
        // 你的代码
    })

    Ok(())
}
```

### 2. 超时配置 (Timeout Configuration)

```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(30),
    query("What is 2 + 2?", None)
).await??;
```

### 3. 重试策略 (Retry Strategy)

```rust
async fn retry_query(prompt: &str, max_retries: u32) -> Result<Vec<Message>> {
    let mut attempt = 0;

    loop {
        match query(prompt, None).await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                attempt += 1;
                tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempt))).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## 📝 性能测试最佳实践 (Best Practices)

### 1. 测试环境 (Test Environment)

- ✅ 使用一致的硬件配置
- ✅ 关闭其他应用程序
- ✅ 多次运行取平均值
- ✅ 监控系统资源使用

### 2. 测试数据 (Test Data)

```rust
// 使用真实场景的测试数据
const TEST_PROMPTS: &[&str] = &[
    "简单查询: 2 + 2 = ?",
    "中等查询: 列出 10 种编程语言",
    "复杂查询: 解释 Rust 所有权系统",
];
```

### 3. 统计显著性 (Statistical Significance)

- 🔬 Criterion 默认运行足够次数以获得统计显著性
- 📊 置信区间默认 95%
- 🎯 p < 0.05 认为有显著差异

### 4. 持续监控 (Continuous Monitoring)

```yaml
# .github/workflows/benchmark.yml
name: Benchmark

on:
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run benchmarks
        run: cargo criterion

      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/report/index.html
```

## 🐛 性能问题排查 (Troubleshooting)

### 问题 1: 性能突然下降

**排查步骤**:
1. 检查依赖版本是否更新
2. 对比基线性能: `cargo criterion --baseline main`
3. 查看系统资源使用: `htop`, `iotop`
4. 检查网络延迟

### 问题 2: 内存持续增长

**排查步骤**:
1. 使用 `valgrind` 检查内存泄漏
2. 检查是否有未释放的缓冲区
3. 使用 `memory_profiler` 分析内存分配

```bash
# Linux 上的内存分析
valgrind --leak-check=full cargo test
```

### 问题 3: 并发性能不佳

**排查步骤**:
1. 检查是否正确使用异步运行时
2. 确认没有阻塞操作
3. 调整并发级别
4. 检查系统资源限制

## 📚 相关资源 (Resources)

- [Criterion.rs 文档](https://bheisler.github.io/criterion.rs/book/index.html)
- [Tokio 性能优化指南](https://tokio.rs/tokio/topics/performance)
- [Rust 性能优化指南](https://nnethercote.github.io/perf-book/introduction.html)

## 🎯 总结 (Summary)

性能基准测试是确保 Claude Agent SDK Rust 高性能的关键工具。

Performance benchmarking is a key tool for ensuring the high performance of Claude Agent SDK Rust.

**关键要点** (Key Points):
- 🚀 定期运行基准测试以监控性能
- 📊 使用统计方法分析结果
- 🎯 设定性能目标和回归阈值
- 🔬 优化前先测量，优化后再验证
- 📝 记录基准测试结果以跟踪趋势

**下一步** (Next Steps):
1. 设置 CI/CD 中的基准测试
2. 建立性能基线
3. 定期审查性能指标
4. 优化识别的性能瓶颈
