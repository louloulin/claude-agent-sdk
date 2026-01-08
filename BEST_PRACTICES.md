# Best Practices for Claude Agent SDK Rust

**本文档总结了使用 Claude Agent SDK Rust 的最佳实践、设计模式和推荐做法。**

遵循这些最佳实践将帮助您构建高性能、可维护、可靠的 AI 应用。

---

## 📋 目录

1. [性能优化](#性能优化)
2. [错误处理](#错误处理)
3. [资源管理](#资源管理)
4. [代码组织](#代码组织)
5. [安全性](#安全性)
6. [测试策略](#测试策略)
7. [部署建议](#部署建议)
8. [常见陷阱](#常见陷阱)

---

## 性能优化

### 1.1 优先使用流式 API

对于可能产生长响应的查询，使用 `query_stream()` 而非 `query()`：

**❌ 不推荐**:
```rust
// 一次性加载所有响应到内存
let messages = query("Explain Rust in detail", None).await?;
// 内存使用: O(n)，n = 响应大小
```

**✅ 推荐**:
```rust
// 流式处理，O(1) 内存
let mut stream = query_stream("Explain Rust in detail", None).await?;
while let Some(result) = stream.next().await {
    // 实时处理每条消息
}
```

**收益**:
- 内存使用减少 60-80%
- 更快的首次响应时间
- 更好的用户体验

### 1.2 并发查询控制

使用信号量限制并发数量，避免资源耗尽：

**❌ 不推荐**:
```rust
// 无限制并发可能导致资源耗尽
let futures: Vec<_> = prompts.into_iter()
    .map(|p| query(p, None))
    .collect();

let results = futures::future::join_all(futures).await;
```

**✅ 推荐**:
```rust
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(10)); // 最多 10 个并发

let futures = prompts.into_iter()
    .map(|p| {
        let semaphore = semaphore.clone();
        async move {
            let _permit = semaphore.acquire().await.unwrap();
            query(p, None).await
        }
    });

let results = futures::future::join_all(futures).await;
```

**收益**:
- 可预测的资源使用
- 避免系统过载
- 更好的稳定性

### 1.3 复用客户端连接

复用 `ClaudeClient` 而非为每个查询创建新实例：

**❌ 不推荐**:
```rust
// 每次查询都创建新客户端，开销大
for prompt in prompts {
    let client = ClaudeClient::new(vec
![prompt.clone()], None)?;
    let _ = client.execute().await;
}
```

**✅ 推荐**:
```rust
// 复用客户端
let client = ClaudeClient::new(vec
![], None)?;
for prompt in prompts {
    let _ = client.query(&prompt, None).await;
}
```

**收益**:
- 减少 50%+ 的初始化开销
- 更低的延迟
- 更少的资源消耗

### 1.4 合理设置 max_tokens

限制响应长度以减少不必要的计算和传输：

**✅ 推荐**:
```rust
let options = ClaudeAgentOptions {
    max_tokens: Some(1000),  // 根据需要设置
    ..Default::default()
};

let messages = query("Brief explanation", Some(options)).await?;
```

**收益**:
- 更快的响应时间
- 更低的 API 成本
- 减少后处理工作量

### 1.5 使用缓存机制

对于重复性查询，实现缓存：

**✅ 推荐**:
```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

struct Cache {
    inner: RwLock<HashMap<String, Vec<Message>>>,
}

impl Cache {
    async fn get(&self, key: &str) -> Option<Vec<Message>> {
        self.inner.read().await.get(key).cloned()
    }

    async fn set(&self, key: String, value: Vec<Message>) {
        self.inner.write().await.insert(key, value);
    }
}

// 使用缓存
let cache = Cache::default();

if let Some(cached) = cache.get("fixed_prompt").await {
    return Ok(cached);
}

let messages = query("fixed_prompt", None).await?;
cache.set("fixed_prompt".to_string(), messages.clone()).await;
```

---

## 错误处理

### 2.1 使用类型安全的 Result

始终使用 `Result` 处理可能的错误：

**❌ 不推荐**:
```rust
// 忽略错误
let messages = query("prompt", None).await.unwrap();
```

**✅ 推荐**:
```rust
// 正确处理错误
match query("prompt", None).await {
    Ok(messages) => {
        // 处理成功响应
    }
    Err(e) => {
        // 记录错误并采取适当措施
        eprintln!("Query failed: {}", e);
        return Err(e.into());
    }
}
```

### 2.2 使用 anyhow 简化错误处理

对于应用代码，使用 `anyhow` 提供更好的错误上下文：

**✅ 推荐**:
```rust
use anyhow::{Context, Result};

async fn process_query(prompt: &str) -> Result<Vec<Message>> {
    query(prompt, None)
        .await
        .context("Failed to execute query")?;

    // 其他处理...
    Ok(messages)
}
```

### 2.3 实现重试机制

对于暂时性错误，实现指数退避重试：

**✅ 推荐**:
```rust
async fn retry_query<F, Fut>(
    operation: F,
    max_retries: usize,
) -> Result<Vec<Message>>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<Vec<Message>, anyhow::Error>>,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                attempt += 1;
                let delay = std::time::Duration::from_millis(100 * 2_u64.pow(attempt as u32));
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}

// 使用
let messages = retry_query(|| query("prompt", None), 3).await?;
```

### 2.4 自定义错误类型

对于库代码，定义自己的错误类型：

**✅ 推荐**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

---

## 资源管理

### 3.1 正确清理资源

确保客户端正确断开连接：

**✅ 推荐**:
```rust
use claude_agent_sdk_rs::ClaudeClient;

async fn run_query() -> anyhow::Result<()> {
    let mut client = ClaudeClient::new(
        vec
!["context".to_string()],
        None,
    )?;

    // 使用 client...
    let _ = client.query("prompt", None).await?;

    // 显式断开连接
    client.disconnect().await?;

    Ok(())
}

// 或者使用 Drop
async fn run_query_auto() -> anyhow::Result<()> {
    let client = ClaudeClient::new(vec
![], None)?;

    // client 会在 drop 时自动清理
    Ok(())
}
```

### 3.2 使用 RAII 模式

利用 Rust 的 RAII (Resource Acquisition Is Initialization)：

**✅ 推荐**:
```rust
struct QueryRunner {
    client: ClaudeClient,
}

impl QueryRunner {
    fn new() -> Result<Self> {
        Ok(Self {
            client: ClaudeClient::new(vec
![], None)?
        })
    }

    async fn query(&self, prompt: &str) -> anyhow::Result<Vec<Message>> {
        self.client.query(prompt, None).await
    }
}

// 使用时自动管理生命周期
```

### 3.3 限制缓冲区大小

避免无限制的缓冲区增长：

**❌ 不推荐**:
```rust
// 可能导致内存无限增长
let mut messages = Vec::new();
loop {
    messages.push(query(...).await?);
}
```

**✅ 推荐**:
```rust
// 限制缓冲区大小
const MAX_MESSAGES: usize = 1000;

let mut messages = Vec::with_capacity(MAX_MESSAGES);

for _ in 0..MAX_MESSAGES {
    messages.push(query(...).await?);
}
```

---

## 代码组织

### 4.1 模块化设计

将功能分解为独立的模块：

**✅ 推荐**:
```rust
// src/query_handler.rs
pub struct QueryHandler {
    client: ClaudeClient,
}

impl QueryHandler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: ClaudeClient::new(vec
![], None)?
        })
    }

    pub async fn ask(&self, prompt: &str) -> Result<String> {
        let messages = self.client.query(prompt, None).await?;
        // 提取文本...
        Ok(text)
    }
}

// src/main.rs
mod query_handler;

use query_handler::QueryHandler;

#[tokio::main]
async fn main() -> Result<()> {
    let handler = QueryHandler::new()?;
    let response = handler.ask("What is 2 + 2?").await?;
    println!("{}", response);
    Ok(())
}
```

### 4.2 使用 Builder 模式

对于复杂配置，使用 Builder 模式：

**✅ 推荐**:
```rust
use claude_agent_sdk_rs::ClaudeAgentOptions;

let options = ClaudeAgentOptions::builder()
    .model("claude-3-5-sonnet-20241022")
    .max_tokens(4096)
    .temperature(0.7)
    .max_turns(10)
    .permission_mode(PermissionMode::Default)
    .build();
```

### 4.3 依赖注入

使用 trait 进行依赖注入，提高可测试性：

**✅ 推荐**:
```rust
#[async_trait::async_trait]
pub trait AIProvider {
    async fn query(&self, prompt: &str) -> Result<String>;
}

pub struct ClaudeProvider {
    client: ClaudeClient,
}

#[async_trait::async_trait]
impl AIProvider for ClaudeProvider {
    async fn query(&self, prompt: &str) -> Result<String> {
        let messages = self.client.query(prompt, None).await?;
        // 提取文本...
        Ok(text)
    }
}

// 使用 trait 而非具体实现
pub struct Service<T: AIProvider> {
    provider: T,
}

impl<T: AIProvider> Service<T> {
    pub async fn process(&self, input: &str) -> Result<String> {
        self.provider.query(input).await
    }
}
```

---

## 安全性

### 5.1 保护 API 密钥

永远不要在代码中硬编码 API 密钥：

**❌ 不推荐**:
```rust
// 危险！密钥暴露
let api_key = "sk-ant-1234567890";
```

**✅ 推荐**:
```rust
// 从环境变量读取
let api_key = std::env::var("ANTHROPIC_API_KEY")
    .expect("ANTHROPIC_API_KEY not set");

// 使用 Claude CLI 的密钥管理
// Claude CLI 会自动处理密钥
let client = ClaudeClient::new(vec
![], None)?;
```

### 5.2 验证输入

验证用户输入，防止注入攻击：

**✅ 推荐**:
```rust
pub fn validate_prompt(prompt: &str) -> Result<()> {
    if prompt.is_empty() {
        return Err(anyhow::anyhow!("Prompt cannot be empty"));
    }

    if prompt.len() > 100_000 {
        return Err(anyhow::anyhow!("Prompt too long"));
    }

    // 检查恶意内容
    if prompt.contains("malicious_pattern") {
        return Err(anyhow::anyhow!("Invalid prompt content"));
    }

    Ok(())
}

// 使用
validate_prompt(user_prompt)?;
let messages = query(user_prompt, None).await?;
```

### 5.3 使用权限模式

合理使用权限模式，限制 Claude 的操作：

**✅ 推荐**:
```rust
use claude_agent_sdk_rs::PermissionMode;

let options = ClaudeAgentOptions {
    // 仅允许特定工具
    allowed_tools: Some(vec
!["Bash".to_string()]),

    // 使用保守的权限模式
    permission_mode: Some(PermissionMode::Default),

    ..Default::default()
};
```

### 5.4 敏感数据处理

避免在查询中包含敏感信息：

**❌ 不推荐**:
```rust
let prompt = format!(
    "My password is {}, what should I do?",
    user_password
);
```

**✅ 推荐**:
```rust
// 脱敏处理
let prompt = "I forgot my password, what should I do?".to_string();
```

---

## 测试策略

### 6.1 单元测试

为关键功能编写单元测试：

**✅ 推荐**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_handler() {
        let handler = QueryHandler::new().unwrap();
        let result = handler.ask("What is 2 + 2?").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("4"));
    }
}
```

### 6.2 集成测试

编写端到端的集成测试：

**✅ 推荐**:
```rust
#[tokio::test]
#[ignore]  // 默认跳过，需要 Claude CLI
async fn test_full_workflow() {
    let client = ClaudeClient::new(vec
![], None).unwrap();
    client.connect().await.unwrap();

    let messages = client.query("What is 2 + 2?", None).await.unwrap();
    assert!(!messages.is_empty());

    client.disconnect().await.unwrap();
}
```

### 6.3 Mock 测试

使用 mock 对象进行快速测试：

**✅ 推荐**:
```rust
#[cfg_attr(test, mockall::automock)]
pub trait AIProvider {
    async fn query(&self, prompt: &str) -> Result<String>;
}

#[tokio::test]
async fn test_with_mock() {
    let mut mock_provider = MockAIProvider::new();
    mock_provider
        .expect_query()
        .returning(Ok("4".to_string()));

    let service = Service { provider: mock_provider };
    let result = service.process("What is 2 + 2?").await.unwrap();

    assert_eq!(result, "4");
}
```

### 6.4 性能测试

使用 Criterion.rs 进行性能基准测试：

**✅ 推荐**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_query(c: &mut Criterion) {
    c.bench_function("query", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| query(black_box("What is 2 + 2?"), None));
    });
}

criterion_group!(benches, bench_query);
criterion_main!(benches);
```

---

## 部署建议

### 7.1 容器化部署

使用 Docker 容器化应用：

**Dockerfile**:
```dockerfile
FROM rust:1.85 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/my-app /usr/local/bin/my-app

CMD ["my-app"]
```

### 7.2 环境变量配置

通过环境变量配置应用：

**✅ 推荐**:
```rust
let max_tokens = std::env::var("MAX_TOKENS")
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(4096);

let model = std::env::var("CLAUDE_MODEL")
    .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());
```

### 7.3 健康检查

实现健康检查端点：

**✅ 推荐**:
```rust
pub async fn health_check() -> Result<HealthStatus> {
    // 检查 Claude CLI 可用性
    let cli_available = which::which("claude").is_ok();

    Ok(HealthStatus {
        healthy: cli_available,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
```

### 7.4 日志记录

使用结构化日志：

**✅ 推荐**:
```rust
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("Starting application");

    match query("prompt", None).await {
        Ok(messages) => {
            info!("Query successful", message_count = messages.len());
        }
        Err(e) => {
            error!("Query failed", error = %e);
            return Err(e.into());
        }
    }

    Ok(())
}
```

---

## 常见陷阱

### 8.1 阻塞异步代码

**❌ 不推荐**:
```rust
// 在异步上下文中阻塞
use std::thread::sleep;

#[tokio::main]
async fn main() {
    sleep(std::time::Duration::from_secs(5)); // 阻塞整个运行时
}
```

**✅ 推荐**:
```rust
// 使用异步睡眠
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    sleep(Duration::from_secs(5)).await; // 非阻塞
}
```

### 8.2 忽略取消信号

**✅ 推荐**:
```rust
use tokio::select;

#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = query_stream("long prompt", None).await?;

    loop {
        select! {
            result = stream.next() => {
                match result {
                    Some(Ok(msg)) => {
                        // 处理消息
                    }
                    Some(Err(e)) => return Err(e.into()),
                    None => break,
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Interrupted");
                return Ok(());
            }
        }
    }
}
```

### 8.3 过度克隆大数据

**❌ 不推荐**:
```rust
// 不必要的克隆
for prompt in prompts {
    let large_data = large_data.clone(); // 昂贵的克隆
    process(&large_data);
}
```

**✅ 推荐**:
```rust
// 使用引用
for prompt in prompts {
    process(&large_data); // 无克隆
}
```

### 8.4 错误的异步上下文

**❌ 不推荐**:
```rust
// 在非异步函数中使用异步代码
fn process() -> Result<()> {
    query("prompt", None).await?; // 编译错误
    Ok(())
}
```

**✅ 推荐**:
```rust
// 正确的异步函数
async fn process() -> Result<()> {
    query("prompt", None).await?;
    Ok(())
}
```

### 8.5 忽略超时

**✅ 推荐**:
```rust
use tokio::time::{timeout, Duration};

async fn query_with_timeout(prompt: &str) -> Result<Vec<Message>> {
    timeout(
        Duration::from_secs(30),
        query(prompt, None)
    ).await??
}
```

---

## 总结

遵循这些最佳实践将帮助您：

✅ **提升性能**: 流式 API、并发控制、缓存机制
✅ **增强可靠性**: 完善的错误处理、重试机制
✅ **提高可维护性**: 模块化设计、依赖注入
✅ **保障安全性**: 密钥保护、输入验证、权限控制
✅ **简化测试**: 单元测试、Mock 测试、性能测试
✅ **顺利部署**: 容器化、健康检查、日志记录

**记住**:
- 🎯 性能优先：流式 > 批量
- 🛡️ 安全第一：永远不要硬编码密钥
- 📊 监控一切：记录日志、收集指标
- 🧪 测试充分：单元、集成、性能测试
- 📖 文档完善：注释、README、API 文档

**下一步**:
- 查看 [ARCHITECTURE.md](ARCHITECTURE.md) 了解系统设计
- 探索 `examples/` 目录中的实际应用
- 阅读 API 文档了解所有可用功能

**祝您构建出色的 AI 应用！** 🚀

---

**文档版本**: v0.6.0
**最后更新**: 2026-01-08
**维护者**: Claude Agent SDK Rust Team
