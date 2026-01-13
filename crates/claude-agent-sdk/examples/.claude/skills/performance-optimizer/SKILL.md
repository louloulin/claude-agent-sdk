---
name: performance-optimizer
description: "Application and infrastructure performance analysis and optimization expert"
version: "2.0.0"
author: "Performance Team <performance@example.com>"
tags:
  - performance
  - optimization
  - profiling
  - monitoring
dependencies:
  - code-reviewer
  - docker-helper
---

# Performance Optimizer Skill

You are a performance optimization expert. Analyze and improve application performance.

## Performance Methodology

### The Optimization Process
```
1. Measure: Establish baseline metrics
2. Analyze: Identify bottlenecks
3. Optimize: Implement improvements
4. Verify: Measure impact
5. Iterate: Continue improvement
```

### Performance Metrics
```rust
// Key metrics to track
- Response time (p50, p95, p99)
- Throughput (requests per second)
- Error rate
- CPU usage
- Memory usage
- I/O operations
- Network bandwidth
- Database query time
- Cache hit rate
```

## Profiling Tools

### Application Profiling

#### Rust Profiling
```bash
# Flamegraph generation
cargo install flamegraph
cargo flamegraph

# Heap profiling
valgrind --tool=massif ./target/release/myapp

# CPU profiling
perf record -g ./target/release/myapp
perf report
```

#### Python Profiling
```bash
# cProfile
python -m cProfile -o profile.stats myapp.py

# Visualization
python -m pstats profile.stats

# Memory profiling
python -m memory_profiler myapp.py

# Line profiler
kernprof -l -v myapp.py
```

#### Node.js Profiling
```bash
# CPU profiling
node --prof app.js
node --prof-process isolate-0xnnnnnnnnnnnn-v8.log > processed.txt

# Memory profiling
node --heap-prof app.js

# Flamegraphs
0x --prof-legacy app.js
0x --prof-legacy --preprocess -j profile.json > processed.json
```

### Database Profiling
```sql
-- Slow query log (PostgreSQL)
SELECT * FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;

-- Query execution plan
EXPLAIN ANALYZE SELECT * FROM users WHERE email = 'test@example.com';

-- Index usage
SELECT schemaname, tablename, indexname, idx_scan
FROM pg_stat_user_indexes
ORDER BY idx_scan ASC;
```

## Optimization Strategies

### 1. Code Level

#### Algorithm Optimization
```rust
// ❌ O(n²) - Nested loops
fn find_duplicates(vec: &[i32]) -> Vec<i32> {
    let mut duplicates = Vec::new();
    for i in 0..vec.len() {
        for j in (i + 1)..vec.len() {
            if vec[i] == vec[j] {
                duplicates.push(vec[i]);
            }
        }
    }
    duplicates
}

// ✅ O(n) - HashSet
fn find_duplicates(vec: &[i32]) -> Vec<i32> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();

    for &item in vec {
        if !seen.insert(item) {
            duplicates.push(item);
        }
    }
    duplicates
}
```

#### Memory Optimization
```rust
// ❌ Unnecessary allocation
fn process_string(s: &str) -> String {
    let s2 = s.to_string(); // Unnecessary copy
    s2.to_uppercase()
}

// ✅ Avoid allocation
fn process_string(s: &str) -> String {
    s.to_uppercase() // Direct conversion
}

// ❌ Vec resizing in loop
let mut vec = Vec::new();
for i in 0..1000 {
    vec.push(i); // Multiple reallocations
}

// ✅ Pre-allocate
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 {
    vec.push(i); // No reallocations
}
```

#### Caching Strategies
```rust
use std::collections::HashMap;
use lru::LruCache;

// Memoization
fn fib(n: u64, cache: &mut HashMap<u64, u64>) -> u64 {
    if n <= 1 {
        return n;
    }

    if let Some(&result) = cache.get(&n) {
        return result;
    }

    let result = fib(n - 1, cache) + fib(n - 2, cache);
    cache.insert(n, result);
    result
}

// LRU Cache
use std::sync::Mutex;
use once_cell::sync::Lazy;

static CACHE: Lazy<Mutex<LruCache<String, String>>> =
    Lazy::new(|| Mutex::new(LruCache::new(1000)));

fn get_with_cache(key: &str) -> Option<String> {
    let mut cache = CACHE.lock().unwrap();
    cache.get(&key.to_string()).cloned()
}
```

### 2. Database Optimization

#### Query Optimization
```sql
-- ❌ N+1 query problem
SELECT * FROM users;
-- For each user:
SELECT * FROM orders WHERE user_id = ?;

-- ✅ JOIN instead
SELECT u.*, o.*
FROM users u
LEFT JOIN orders o ON o.user_id = u.id;

-- ✅ Or use bulk fetch
SELECT * FROM orders WHERE user_id IN (?, ?, ?);
```

#### Indexing Strategy
```sql
-- Create indexes on frequently queried columns
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_created_at ON orders(created_at DESC);

-- Composite index for multi-column queries
CREATE INDEX idx_orders_user_status_date
ON orders(user_id, status, created_at);

-- Partial index for specific conditions
CREATE INDEX idx_active_users
ON users(email)
WHERE active = true;
```

#### Connection Pooling
```rust
// Use connection pooling
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(20) // Optimal pool size
    .min_connections(5)
    .connect_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect("postgres://localhost/db").await?;
```

### 3. Caching Architecture

#### Multi-Level Caching
```
Level 1: Application Cache (L1)
- Fastest access
- Limited size
- In-memory (e.g., Redis, Memcached)

Level 2: Database Cache (Query Cache)
- Fast but slower than L1
- Larger capacity
- Database-level caching

Level 3: CDN/Edge Cache
- Geographically distributed
- For static content
- High latency tolerance

Level 4: Browser Cache
- Client-side caching
- HTTP caching headers
- Long-lived assets
```

#### Cache Patterns
```rust
// Cache-Aside Pattern
async fn get_user(id: u64) -> Result<User> {
    // Try cache first
    if let Some(user) = cache.get(&id).await? {
        return Ok(user);
    }

    // Cache miss - fetch from database
    let user = db.fetch_user(id).await?;

    // Store in cache
    cache.set(id, &user, TTL::Hour).await?;

    Ok(user)
}

// Write-Through Pattern
async fn update_user(user: User) -> Result<()> {
    // Update database
    db.update_user(&user).await?;

    // Update cache synchronously
    cache.set(user.id, &user, TTL::Hour).await?;

    Ok(())
}
```

### 4. Concurrency & Parallelism

#### Async/Await
```rust
// ❌ Sequential operations
async fn fetch_data() -> Vec<Data> {
    let data1 = fetch_api1().await;
    let data2 = fetch_api2().await;
    let data3 = fetch_api3().await;
    vec![data1, data2, data3]
}

// ✅ Concurrent operations
async fn fetch_data() -> Vec<Data> {
    let (data1, data2, data3) = tokio::join!(
        fetch_api1(),
        fetch_api2(),
        fetch_api3()
    );
    vec![data1, data2, data3]
}
```

#### Thread Pool
```rust
use rayon::prelude::*;

// Parallel iteration
fn process_large_dataset(data: Vec<i32>) -> Vec<i32> {
    data.par_iter() // Parallel iterator
        .map(|x| x * 2)
        .collect()
}

// Parallel processing
fn calculate_statistics(data: &[f64]) -> (f64, f64, f64) {
    use rayon::prelude::*;

    let mean = data.par_iter().sum::<f64>() / data.len() as f64;
    let variance = data.par_iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / data.len() as f64;
    let stddev = variance.sqrt();

    (mean, variance, stddev)
}
```

### 5. I/O Optimization

#### Batch Processing
```rust
// ❌ Individual I/O operations
for item in items {
    db.save(item).await?;
}

// ✅ Batch operations
db.save_batch(&items).await?;
```

#### Streaming
```rust
// ❌ Load entire file into memory
let data = fs::read_to_string("large_file.txt")?;

// ✅ Stream processing
use std::fs::File;
use std::io::{BufRead, BufReader};

let file = File::open("large_file.txt")?;
let reader = BufReader::new(file);

for line in reader.lines() {
    process_line(line?);
}
```

#### Compression
```rust
// Compress large data before transmission
use flate2::write::GzEncoder;
use flate2::Compression;

let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
encoder.write_all(data.as_bytes())?;
let compressed = encoder.finish()?;
```

## Performance Monitoring

### Application Performance Monitoring (APM)
```rust
// Metrics collection
use prometheus::{Counter, Histogram, Registry};

let request_duration = Histogram::with_opts(
    HistogramOpts::new("http_request_duration_seconds", "Request duration")
)?;

let request_counter = Counter::new("http_requests_total", "Total requests")?;

// Record metrics
let start = Instant::now();
// ... handle request ...
request_duration.observe(start.elapsed().as_secs_f64());
request_counter.inc();
```

### Distributed Tracing
```rust
use opentelemetry::trace::{TraceContextExt, Tracer};
use opentelemetry::global;

let tracer = global::tracer("my_app");
let span = tracer.start("process_request");
let cx = opentelemetry::Context::current_with_span(span);

// ... do work ...
tracer.span(&cx).end();
```

### Logging Strategy
```rust
// Structured logging
use tracing::{info, warn, error, instrument};

#[instrument(skip(password))]
async fn login(username: &str, password: &str) -> Result<User> {
    info!(username = %username, "Login attempt");

    match authenticate(username, password).await {
        Ok(user) => {
            info!(user_id = %user.id, "Login successful");
            Ok(user)
        }
        Err(e) => {
            warn!(error = %e, username = %username, "Login failed");
            Err(e)
        }
    }
}
```

## Performance Testing

### Load Testing
```bash
# Apache Bench
ab -n 10000 -c 100 http://localhost:3000/api/users

# wrk
wrk -t12 -c400 -d30s http://localhost:3000/api/users

# Locust (Python)
locust -f locustfile.py --host=http://localhost:3000
```

### Benchmarking
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

## Common Performance Issues

### 1. N+1 Query Problem
```rust
// ❌ N+1 queries
let users = db.get_users().await?;
for user in &users {
    let orders = db.get_orders_by_user(user.id).await?; // N queries
}

// ✅ Single query with JOIN
let users_with_orders = db.get_users_with_orders().await?;
```

### 2. Memory Leaks
```rust
// ❌ Memory leak - growing collection
static GLOBAL_DATA: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());

fn process_data(data: Vec<u8>) {
    GLOBAL_DATA.lock().unwrap().push(data); // Never cleared
}

// ✅ Use bounded cache
static GLOBAL_CACHE: Mutex<LruCache<u64, Vec<u8>>> =
    Mutex::new(LruCache::new(1000)); // Max 1000 items
```

### 3. Unnecessary Serialization
```rust
// ❌ Serialize/Deserialize unnecessarily
let json = serde_json::to_string(&data)?;
let data2 = serde_json::from_str::<Data>(&json)?;

// ✅ Pass references
fn process(data: &Data) { }
process(&data);
```

### 4. Synchronous I/O in Async Context
```rust
// ❌ Blocking in async context
async fn fetch_data() -> Result<Data> {
    let data = std::fs::read("file.txt")?; // Blocking!
    Ok(data)
}

// ✅ Use async I/O
async fn fetch_data() -> Result<Data> {
    let data = tokio::fs::read("file.txt").await?;
    Ok(data)
}
```

## Performance Targets

### Response Time Targets
```
P50 (median):  < 100ms
P95:           < 500ms
P99:           < 1s
P99.9:         < 5s
```

### Throughput Targets
```
REST API:      > 1000 req/s
GraphQL:       > 500 req/s
WebSocket:     > 10k connections
```

### Resource Limits
```
CPU:           < 70% average
Memory:        < 80% of limit
Error Rate:    < 0.1%
```

## Optimization Checklist

### Code Review
- [ ] Algorithm complexity optimized
- [ ] Memory allocations minimized
- [ ] Caching implemented appropriately
- [ ] Async/await used correctly
- [ ] No blocking operations in async context
- [ ] Connection pooling configured
- [ ] Batch operations used

### Infrastructure
- [ ] CDN configured for static assets
- [ ] Load balancing configured
- [ ] Database indexes optimized
- [ ] Connection pools sized correctly
- [ ] Caching layers configured
- [ ] Compression enabled
- [ ] HTTP/2 enabled

### Monitoring
- [ ] APM configured
- [ ] Metrics collected
- [ ] Alerts configured
- [ ] Dashboards set up
- [ ] Log aggregation
- [ ] Distributed tracing

## Tools & Resources

### Profiling Tools
- **Flamegraph**: Visualization of CPU usage
- **Valgrind**: Memory profiling
- **perf**: Linux performance analysis
- **pprof**: Go profiler

### Monitoring Tools
- **Prometheus**: Metrics collection
- **Grafana**: Visualization
- **Jaeger**: Distributed tracing
- **ELK Stack**: Log aggregation

### Load Testing Tools
- **wrk**: HTTP benchmarking
- **Locust**: Python load testing
- **k6**: Modern load testing
- **Apache Bench**: Simple benchmarking

### Documentation
- [Google SRE Book](https://sre.google/sre-book/table-of-contents/)
- [Performance Budgets](https://web.dev/performance-budgets-101/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
