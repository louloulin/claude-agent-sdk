# 第10章：可观测性与命令系统

本章文档介绍 Claude Agent SDK for Rust 的可观测性、命令和待办事项模块。

## 10.1 概述

SDK 提供全面的可观测性和实用功能：

```
observability/
├── mod.rs        # 模块导出 (42 行)
├── logger.rs     # 结构化日志 (505 行, 5 个测试)
└── metrics.rs    # 指标收集 (747 行, 11 个测试)

commands/
└── mod.rs        # 斜杠命令系统 (467 行, 21 个测试)

todos/
└── mod.rs        # 待办事项管理 (769 行, 24 个测试)
```

**核心功能**：
- **结构化日志**：支持 JSON/文本格式的上下文感知日志
- **指标收集**：计数器、仪表、直方图，支持 Prometheus 导出
- **斜杠命令**：可扩展的命令注册和执行系统
- **待办事项管理**：带状态管理的任务跟踪

## 10.2 可观测性模块

### 10.2.1 模块导出

```rust
pub use logger::{
    ConsoleLogObserver, GlobalLogger, LogEntry, LogFormat,
    LogLevel, LogObserver, Logger,
};
pub use metrics::{
    Histogram, HistogramBuckets, LabeledMetric, MetricKind,
    MetricStorage, MetricsCollector, TimerGuard,
};
```

## 10.3 Logger 日志器

### 10.3.1 LogLevel 枚举

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}
```

**字符串解析**：
```rust
use std::str::FromStr;

let level = LogLevel::from_str("INFO").unwrap();
let level = LogLevel::from_str("error").unwrap(); // 大小写不敏感
```

### 10.3.2 LogEntry 结构

```rust
pub struct LogEntry {
    pub timestamp: u64,              // 自纪元以来的毫秒数
    pub level: LogLevel,
    pub context: String,             // 日志器名称
    pub message: String,
    pub metadata: Vec<(String, String)>,
    pub error: Option<String>,
}
```

**创建日志条目**：
```rust
let entry = LogEntry::new(LogLevel::Info, "MyAgent", "Processing started")
    .with_field("task_id", "123")
    .with_field("user_id", "abc")
    .with_fields(&[("key1", "value1"), ("key2", "value2")])
    .with_error("Connection failed");

// 输出格式
let json = entry.to_json();  // JSON 格式
let text = entry.to_text();  // 人类可读格式
```

**JSON 输出**：
```json
{
    "timestamp": 1708312345678,
    "level": "INFO",
    "context": "MyAgent",
    "message": "Processing started",
    "task_id": "123",
    "user_id": "abc",
    "error": "Connection failed"
}
```

**文本输出**：
```
[2024-02-19 12:34:56.678] INFO MyAgent: Processing started task_id=123 user_id=abc error=Connection failed
```

### 10.3.3 LogObserver Trait

```rust
pub trait LogObserver: Send + Sync {
    fn on_log(&self, entry: &LogEntry);
}
```

**ConsoleLogObserver**：
```rust
pub struct ConsoleLogObserver {
    format: LogFormat,
    min_level: LogLevel,
}

pub enum LogFormat {
    Text,  // 人类可读
    Json,  // JSON 格式
}

// 创建观察者
let text_observer = ConsoleLogObserver::text(LogLevel::Info);
let json_observer = ConsoleLogObserver::json(LogLevel::Debug);
```

**自定义观察者**：
```rust
struct FileLogObserver {
    file: std::fs::File,
}

impl LogObserver for FileLogObserver {
    fn on_log(&self, entry: &LogEntry) {
        // 写入文件
        let _ = writeln!(self.file, "{}", entry.to_json());
    }
}
```

### 10.3.4 Logger

```rust
pub struct Logger {
    context: String,
    min_level: LogLevel,
    observers: Vec<Arc<dyn LogObserver>>,
}
```

**创建日志器**：
```rust
// 基本日志器
let logger = Logger::new("MyAgent");

// 带最小级别
let logger = Logger::new("MyAgent")
    .with_min_level(LogLevel::Debug);

// 带观察者
let observer = Arc::new(ConsoleLogObserver::text(LogLevel::Info));
let logger = Logger::new("MyAgent")
    .with_min_level(LogLevel::Debug)
    .with_observer(observer);
```

**日志方法**：
```rust
// 基本日志
logger.trace("Trace message", &[("key", "value")]);
logger.debug("Debug message", &[("request_id", "123")]);
logger.info("Info message", &[("status", "success")]);
logger.warn("Warning message", &[("retry_count", "3")]);
logger.error("Error message", Some(&anyhow::anyhow!("Connection failed")));

// 空字段
const EMPTY: &[(&str, &str)] = &[];
logger.info("Simple message", EMPTY);
```

**默认行为**：
- 如果没有注册观察者，回退到 `tracing` crate
- 使用 `tracing::info!`、`tracing::debug!` 等

### 10.3.5 GlobalLogger

```rust
pub struct GlobalLogger {
    loggers: RwLock<HashMap<String, Logger>>,
}
```

**使用方法**：
```rust
// 获取全局实例
let global = GlobalLogger::instance();

// 获取或创建日志器
let logger = global.get("MyAgent");

// 注册日志器
global.register(Logger::new("MyAgent").with_min_level(LogLevel::Debug));

// 为所有日志器设置最小级别
global.set_min_level(LogLevel::Trace);

// 便捷函数
let logger = logger("MyAgent");
```

## 10.4 MetricsCollector 指标收集器

### 10.4.1 MetricKind 枚举

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum MetricKind {
    Counter,    // 只增不减的累计值
    Gauge,      // 可增可减的瞬时值
    Histogram,  // 值的分布
    Summary,    // 类似直方图，支持可配置的分位数
}
```

### 10.4.2 LabeledMetric 结构

```rust
pub struct LabeledMetric {
    pub name: String,
    pub kind: MetricKind,
    pub labels: Vec<(String, String)>,
    pub value: f64,
    pub timestamp: u64,
}

impl LabeledMetric {
    pub fn new(
        name: impl Into<String>,
        kind: MetricKind,
        value: f64,
        labels: Vec<(String, String)>,
    ) -> Self;

    pub fn get_label(&self, key: &str) -> Option<&String>;
}
```

### 10.4.3 Histogram 直方图

```rust
pub struct Histogram {
    pub buckets: Vec<u64>,     // 桶计数
    pub sum: f64,              // 总和
    pub count: u64,            // 总观测数
    pub boundaries: Vec<f64>,  // 桶边界
}
```

**HistogramBuckets**：
```rust
pub struct HistogramBuckets {
    pub boundaries: Vec<f64>,
}

// 预定义桶
let latency = HistogramBuckets::latency();  // [1, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000] ms
let size = HistogramBuckets::size();        // [1KB, 10KB, 100KB, 1MB, 10MB, 100MB]

// 自定义桶
let custom = HistogramBuckets::custom(vec![0.1, 0.5, 1.0, 5.0, 10.0]);
```

**Histogram 方法**：
```rust
let mut hist = Histogram::new(HistogramBuckets::latency());

hist.observe(50.0);
hist.observe(150.0);
hist.observe(500.0);

// 计算分位数
let p50 = hist.percentile(50.0);   // 50th 分位数
let p95 = hist.percentile(95.0);   // 95th 分位数
let p99 = hist.percentile(99.0);   // 99th 分位数

// 获取平均值
let avg = hist.avg();
```

### 10.4.4 MetricStorage Trait

```rust
pub trait MetricStorage: Send + Sync {
    fn record(&self, metric: LabeledMetric);
    fn get_counter(&self, name: &str, labels: &[(String, String)]) -> f64;
    fn get_gauge(&self, name: &str, labels: &[(String, String)]) -> f64;
    fn get_histogram(&self, name: &str, labels: &[(String, String)]) -> Option<Histogram>;
    fn get_all_metrics(&self) -> Vec<LabeledMetric>;
}
```

### 10.4.5 MetricsCollector 使用方法

```rust
// 创建收集器
let metrics = MetricsCollector::new();

// 带前缀
let metrics = MetricsCollector::with_prefix("myapp");

// 记录计数器
metrics.increment("api_requests", &[("endpoint", "/api/users"), ("method", "GET")]);
metrics.increment_by("bytes_sent", 1024.0, &[("service", "cdn")]);

// 设置仪表
metrics.set_gauge("active_connections", 5.0, &[("pool", "database")]);
metrics.set_gauge("temperature", 22.5, &[("sensor", "room1")]);

// 记录直方图
metrics.record("request_duration_ms", MetricKind::Histogram, 150.0,
    &[("endpoint", "/api/search")]);

// 记录时间
let duration = std::time::Duration::from_millis(100);
metrics.record_timing("operation_duration", duration, &[("op", "query")]);

// 计时代码块
let (duration, result) = metrics.time("expensive_operation", &[("type", "batch")], || {
    // 执行工作
    42
});

// 计时器守卫（drop 时记录）
{
    let _guard = metrics.start_timer("request", &[("path", "/api/data")]);
    // 执行工作
} // 计时器在此记录
```

**获取指标**：
```rust
// 获取计数器
let count = metrics.get_counter("api_requests", &[("endpoint", "/api/users")]);

// 获取仪表
let temp = metrics.get_gauge("temperature", &[("sensor", "room1")]);

// 获取直方图
let hist = metrics.get_histogram("request_duration_ms", &[]);
if let Some(h) = hist {
    println!("Avg: {}, P95: {}", h.avg(), h.percentile(95.0));
}

// 获取所有指标
let all = metrics.get_all_metrics();
```

**导出格式**：
```rust
// Prometheus 格式
let prometheus = metrics.export_prometheus();
// 输出：
// # TYPE api_requests counter
// api_requests{endpoint="/api/users"} 5
// # TYPE active_connections gauge
// active_connections{pool="database"} 5

// JSON 格式
let json = metrics.export_json();
// 输出：[{"name":"api_requests","kind":"counter",...}]
```

## 10.5 命令系统

### 10.5.1 CommandError 枚举

```rust
pub enum CommandError {
    NotFound(String),         // 命令未找到
    ExecutionFailed(String),  // 执行失败
    InvalidName(String),      // 名称格式无效
    AlreadyRegistered(String), // 重复注册
}

impl std::error::Error for CommandError {}
impl fmt::Display for CommandError { ... }
```

### 10.5.2 CommandHandler 类型

```rust
pub type CommandHandler = Arc<
    dyn Fn(&str, Vec<String>) ->
        Pin<Box<dyn Future<Output = Result<String, CommandError>> + Send>>
        + Send + Sync,
>;
```

### 10.5.3 SlashCommand 结构

```rust
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub handler: CommandHandler,
}

impl SlashCommand {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        handler: CommandHandler,
    ) -> Self;
}
```

**创建命令**：
```rust
// 简单命令
let cmd = SlashCommand::new(
    "echo",
    "Echo arguments back",
    Arc::new(|_name, args| {
        Box::pin(async move {
            Ok(format!("Echo: {:?}", args))
        })
    }),
);

// 带逻辑的命令
let cmd = SlashCommand::new(
    "sum",
    "Sum numbers",
    Arc::new(|_name, args| {
        Box::pin(async move {
            let sum: i32 = args
                .iter()
                .map(|s| s.parse::<i32>().unwrap_or(0))
                .sum();
            Ok(format!("Sum: {}", sum))
        })
    }),
);

// 带错误的异步命令
let cmd = SlashCommand::new(
    "fetch",
    "Fetch data from URL",
    Arc::new(|_name, args| {
        Box::pin(async move {
            if args.is_empty() {
                return Err(CommandError::ExecutionFailed("URL required".to_string()));
            }
            let url = &args[0];
            // 获取数据...
            Ok(format!("Fetched from: {}", url))
        })
    }),
);
```

**名称验证**：
```rust
// 有效名称
"help", "status", "deploy-app", "runTest"

// 无效名称
"", "test command", "123test"  // 空、包含空格、以数字开头
```

### 10.5.4 CommandRegistry

```rust
pub struct CommandRegistry {
    commands: HashMap<String, SlashCommand>,
}

impl Default for CommandRegistry { ... }
```

**注册表操作**：
```rust
let mut registry = CommandRegistry::new();

// 注册命令
registry.register(cmd)?;  // 如果重复则返回 AlreadyRegistered 错误

// 执行命令
let result = registry.execute("echo", vec!["hello".to_string()]).await?;

// 检查存在
if registry.exists("help") { ... }

// 获取命令
let cmd = registry.get("help");

// 列出命令
let names = registry.list_names();     // Vec<String>
let commands = registry.list_all();    // Vec<&SlashCommand>

// 计数
let count = registry.len();
let empty = registry.is_empty();

// 注销
registry.unregister("help")?;  // 如果未找到则返回 NotFound 错误

// 清空所有
registry.clear();
```

## 10.6 Todos 待办事项模块

### 10.6.1 TodoStatus 枚举

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TodoStatus {
    Pending,     // 尚未开始
    InProgress,  // 正在进行
    Completed,   // 已完成
}

impl TodoStatus {
    pub fn is_completed(&self) -> bool;
    pub fn is_active(&self) -> bool;  // Pending 或 InProgress
}
```

### 10.6.2 TodoItem 结构

```rust
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub status: TodoStatus,
    pub created_at: DateTime<Utc>,
}
```

**创建条目**：
```rust
let item = TodoItem::new("123", "Write documentation");
// id: "123", content: "Write documentation", status: Pending

// 状态转换
item.start();     // Pending -> InProgress
item.complete();  // 任意 -> Completed
item.reset();     // 任意 -> Pending
```

### 10.6.3 TodoList 结构

```rust
pub struct TodoList {
    pub id: String,
    pub name: String,
    pub items: Vec<TodoItem>,
}
```

**创建列表**：
```rust
let mut list = TodoList::new("Project Tasks");
// 自动生成 UUID 作为 id
```

**添加条目**：
```rust
let item = list.add("Implement feature X");
// 自动生成 UUID 作为 item.id
// 返回新条目的引用
```

**状态操作**：
```rust
// 更改状态
list.start(&item.id)?;      // 如果未找到则返回 NotFound 错误
list.complete(&item.id)?;
list.reset(&item.id)?;
```

**查询操作**：
```rust
// 按 ID 获取
let item = list.get(&id);  // Option<&TodoItem>

// 按状态过滤
let pending = list.filter_by_status(TodoStatus::Pending);
let in_progress = list.filter_by_status(TodoStatus::InProgress);
let completed = list.filter_by_status(TodoStatus::Completed);

// 按状态计数
let counts = list.count_by_status();  // HashMap<TodoStatus, usize>
let completed_count = list.completed_count();

// 完成百分比
let percentage = list.completion_percentage();  // 0.0 - 100.0
```

**条目管理**：
```rust
// 移除条目
list.remove(&id)?;  // 如果未找到则返回 NotFound 错误

// 计数
let total = list.len();
let empty = list.is_empty();
```

### 10.6.4 TodoError 枚举

```rust
pub enum TodoError {
    NotFound(String),    // 条目未找到
    InvalidInput(String), // 无效输入
}

impl std::error::Error for TodoError {}
impl fmt::Display for TodoError { ... }
```

## 10.7 测试覆盖

| 模块 | 测试数 | 重点领域 |
|------|--------|----------|
| logger.rs | 5 | 级别解析、条目创建、观察者模式 |
| metrics.rs | 11 | 计数器、仪表、直方图、计时器、导出 |
| commands/mod.rs | 21 | 注册、执行、验证、错误 |
| todos/mod.rs | 24 | 状态转换、列表操作、错误 |
| **总计** | **61** | |

## 10.8 API 参考

### 可观测性重导出

```rust
use claude_agent_sdk::observability::{
    // 日志
    Logger, LogLevel, LogEntry, LogFormat, LogObserver,
    ConsoleLogObserver, GlobalLogger,

    // 指标
    MetricsCollector, MetricKind, LabeledMetric, Histogram,
    HistogramBuckets, MetricStorage, TimerGuard,
};
```

### 命令重导出

```rust
use claude_agent_sdk::commands::{
    SlashCommand, CommandRegistry, CommandError, CommandHandler,
};
```

### 待办事项重导出

```rust
use claude_agent_sdk::todos::{
    TodoItem, TodoList, TodoStatus, TodoError,
};
```

## 10.9 使用示例

### 完整的可观测性设置

```rust
use claude_agent_sdk::observability::*;

// 创建带观察者的日志器
let observer = Arc::new(ConsoleLogObserver::json(LogLevel::Debug));
let logger = Logger::new("MyAgent")
    .with_min_level(LogLevel::Debug)
    .with_observer(observer);

// 创建指标收集器
let metrics = MetricsCollector::with_prefix("myapp");

// 在应用中使用
logger.info("Starting agent", &[("version", "1.0.0")]);
let _timer = metrics.start_timer("agent_execution", &[("agent", "researcher")]);

// ... 执行工作 ...

metrics.increment("tasks_completed", &[("type", "research")]);
logger.info("Agent completed", &[("duration_ms", "150")]);
```

### 命令系统设置

```rust
use claude_agent_sdk::commands::*;

let mut registry = CommandRegistry::new();

// 注册命令
registry.register(SlashCommand::new(
    "help",
    "Show available commands",
    Arc::new(|_name, _args| {
        Box::pin(async { Ok("Available commands: help, status, deploy".to_string()) })
    }),
))?;

registry.register(SlashCommand::new(
    "status",
    "Show system status",
    Arc::new(|_name, _args| {
        Box::pin(async { Ok("System: operational".to_string()) })
    }),
))?;

// 执行
let result = registry.execute("help", vec![]).await?;
```

### 待办事项列表工作流

```rust
use claude_agent_sdk::todos::*;

let mut list = TodoList::new("Sprint Tasks");

// 添加任务
list.add("Implement authentication");
list.add("Write tests");
list.add("Update documentation");

// 开始第一个任务
let first_id = list.items[0].id.clone();
list.start(&first_id)?;

// 完成一些任务
list.complete(&first_id)?;
list.complete(&list.items[1].id)?;

// 检查进度
println!("Progress: {:.1}%", list.completion_percentage());
println!("Completed: {}/{}", list.completed_count(), list.len());
```

## 10.10 集成模式

### 日志器 + 指标集成

```rust
fn process_request(metrics: &MetricsCollector, logger: &Logger, id: &str) {
    let _timer = metrics.start_timer("request_duration", &[("id", id)]);
    logger.info("Processing request", &[("id", id)]);

    match do_work() {
        Ok(result) => {
            metrics.increment("requests_success", &[]);
            logger.info("Request completed", &[("id", id)]);
            result
        }
        Err(e) => {
            metrics.increment("requests_failed", &[]);
            logger.error("Request failed", Some(&e));
            Err(e)
        }
    }
}
```

### 带日志的命令

```rust
let cmd = SlashCommand::new(
    "deploy",
    "Deploy to environment",
    Arc::new({
        let logger = logger.clone();
        move |_name, args| {
            let logger = logger.clone();
            Box::pin(async move {
                let env = args.get(0).map(|s| s.as_str()).unwrap_or("staging");
                logger.info("Starting deployment", &[("env", env)]);
                // 部署...
                logger.info("Deployment complete", &[("env", env)]);
                Ok(format!("Deployed to {}", env))
            })
        }
    }),
);
```
