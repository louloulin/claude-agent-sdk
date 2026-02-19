# Section 10: Observability & Commands

This section documents the observability, commands, and todos modules of the Claude Agent SDK for Rust.

## 10.1 Overview

The SDK provides comprehensive observability and utility features:

```
observability/
├── mod.rs        # Module exports (42 lines)
├── logger.rs     # Structured logging (505 lines, 5 tests)
└── metrics.rs    # Metrics collection (747 lines, 11 tests)

commands/
└── mod.rs        # Slash commands system (467 lines, 21 tests)

todos/
└── mod.rs        # Todo list management (769 lines, 24 tests)
```

**Key Features**:
- **Structured Logging**: Context-aware logging with JSON/text formats
- **Metrics Collection**: Counters, gauges, histograms with Prometheus export
- **Slash Commands**: Extensible command registration and execution
- **Todo Management**: Task tracking with status management

## 10.2 Observability Module

### 10.2.1 Module Exports

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

## 10.3 Logger

### 10.3.1 LogLevel Enum

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

**String Parsing**:
```rust
use std::str::FromStr;

let level = LogLevel::from_str("INFO").unwrap();
let level = LogLevel::from_str("error").unwrap(); // Case-insensitive
```

### 10.3.2 LogEntry Structure

```rust
pub struct LogEntry {
    pub timestamp: u64,              // Milliseconds since epoch
    pub level: LogLevel,
    pub context: String,             // Logger name
    pub message: String,
    pub metadata: Vec<(String, String)>,
    pub error: Option<String>,
}
```

**Creating Entries**:
```rust
let entry = LogEntry::new(LogLevel::Info, "MyAgent", "Processing started")
    .with_field("task_id", "123")
    .with_field("user_id", "abc")
    .with_fields(&[("key1", "value1"), ("key2", "value2")])
    .with_error("Connection failed");

// Output formats
let json = entry.to_json();  // JSON format
let text = entry.to_text();  // Human-readable format
```

**JSON Output**:
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

**Text Output**:
```
[2024-02-19 12:34:56.678] INFO MyAgent: Processing started task_id=123 user_id=abc error=Connection failed
```

### 10.3.3 LogObserver Trait

```rust
pub trait LogObserver: Send + Sync {
    fn on_log(&self, entry: &LogEntry);
}
```

**ConsoleLogObserver**:
```rust
pub struct ConsoleLogObserver {
    format: LogFormat,
    min_level: LogLevel,
}

pub enum LogFormat {
    Text,  // Human-readable
    Json,  // JSON format
}

// Create observers
let text_observer = ConsoleLogObserver::text(LogLevel::Info);
let json_observer = ConsoleLogObserver::json(LogLevel::Debug);
```

**Custom Observer**:
```rust
struct FileLogObserver {
    file: std::fs::File,
}

impl LogObserver for FileLogObserver {
    fn on_log(&self, entry: &LogEntry) {
        // Write to file
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

**Creating Loggers**:
```rust
// Basic logger
let logger = Logger::new("MyAgent");

// With minimum level
let logger = Logger::new("MyAgent")
    .with_min_level(LogLevel::Debug);

// With observer
let observer = Arc::new(ConsoleLogObserver::text(LogLevel::Info));
let logger = Logger::new("MyAgent")
    .with_min_level(LogLevel::Debug)
    .with_observer(observer);
```

**Logging Methods**:
```rust
// Basic logging
logger.trace("Trace message", &[("key", "value")]);
logger.debug("Debug message", &[("request_id", "123")]);
logger.info("Info message", &[("status", "success")]);
logger.warn("Warning message", &[("retry_count", "3")]);
logger.error("Error message", Some(&anyhow::anyhow!("Connection failed")));

// Empty fields
const EMPTY: &[(&str, &str)] = &[];
logger.info("Simple message", EMPTY);
```

**Default Behavior**:
- If no observers are registered, falls back to `tracing` crate
- Uses `tracing::info!`, `tracing::debug!`, etc.

### 10.3.5 GlobalLogger

```rust
pub struct GlobalLogger {
    loggers: RwLock<HashMap<String, Logger>>,
}
```

**Usage**:
```rust
// Get global instance
let global = GlobalLogger::instance();

// Get or create logger
let logger = global.get("MyAgent");

// Register a logger
global.register(Logger::new("MyAgent").with_min_level(LogLevel::Debug));

// Set minimum level for all loggers
global.set_min_level(LogLevel::Trace);

// Convenience function
let logger = logger("MyAgent");
```

## 10.4 MetricsCollector

### 10.4.1 MetricKind Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum MetricKind {
    Counter,    // Cumulative value that only increases
    Gauge,      // Point-in-time value that can go up or down
    Histogram,  // Distribution of values
    Summary,    // Similar to histogram with configurable quantiles
}
```

### 10.4.2 LabeledMetric Structure

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

### 10.4.3 Histogram

```rust
pub struct Histogram {
    pub buckets: Vec<u64>,     // Bucket counts
    pub sum: f64,              // Total sum
    pub count: u64,            // Total observations
    pub boundaries: Vec<f64>,  // Bucket boundaries
}
```

**HistogramBuckets**:
```rust
pub struct HistogramBuckets {
    pub boundaries: Vec<f64>,
}

// Predefined buckets
let latency = HistogramBuckets::latency();  // [1, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000] ms
let size = HistogramBuckets::size();        // [1KB, 10KB, 100KB, 1MB, 10MB, 100MB]

// Custom buckets
let custom = HistogramBuckets::custom(vec![0.1, 0.5, 1.0, 5.0, 10.0]);
```

**Histogram Methods**:
```rust
let mut hist = Histogram::new(HistogramBuckets::latency());

hist.observe(50.0);
hist.observe(150.0);
hist.observe(500.0);

// Calculate percentile
let p50 = hist.percentile(50.0);   // 50th percentile
let p95 = hist.percentile(95.0);   // 95th percentile
let p99 = hist.percentile(99.0);   // 99th percentile

// Get average
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

### 10.4.5 MetricsCollector Usage

```rust
// Create collector
let metrics = MetricsCollector::new();

// With prefix
let metrics = MetricsCollector::with_prefix("myapp");

// Record counter
metrics.increment("api_requests", &[("endpoint", "/api/users"), ("method", "GET")]);
metrics.increment_by("bytes_sent", 1024.0, &[("service", "cdn")]);

// Set gauge
metrics.set_gauge("active_connections", 5.0, &[("pool", "database")]);
metrics.set_gauge("temperature", 22.5, &[("sensor", "room1")]);

// Record histogram
metrics.record("request_duration_ms", MetricKind::Histogram, 150.0,
    &[("endpoint", "/api/search")]);

// Record timing
let duration = std::time::Duration::from_millis(100);
metrics.record_timing("operation_duration", duration, &[("op", "query")]);

// Time a block
let (duration, result) = metrics.time("expensive_operation", &[("type", "batch")], || {
    // Do work
    42
});

// Timer guard (records on drop)
{
    let _guard = metrics.start_timer("request", &[("path", "/api/data")]);
    // Do work
} // Timer recorded here
```

**Getting Metrics**:
```rust
// Get counter
let count = metrics.get_counter("api_requests", &[("endpoint", "/api/users")]);

// Get gauge
let temp = metrics.get_gauge("temperature", &[("sensor", "room1")]);

// Get histogram
let hist = metrics.get_histogram("request_duration_ms", &[]);
if let Some(h) = hist {
    println!("Avg: {}, P95: {}", h.avg(), h.percentile(95.0));
}

// Get all metrics
let all = metrics.get_all_metrics();
```

**Export Formats**:
```rust
// Prometheus format
let prometheus = metrics.export_prometheus();
// Output:
// # TYPE api_requests counter
// api_requests{endpoint="/api/users"} 5
// # TYPE active_connections gauge
// active_connections{pool="database"} 5

// JSON format
let json = metrics.export_json();
// Output: [{"name":"api_requests","kind":"counter",...}]
```

## 10.5 Commands System

### 10.5.1 CommandError Enum

```rust
pub enum CommandError {
    NotFound(String),         // Command not found
    ExecutionFailed(String),  // Execution failed
    InvalidName(String),      // Invalid name format
    AlreadyRegistered(String), // Duplicate registration
}

impl std::error::Error for CommandError {}
impl fmt::Display for CommandError { ... }
```

### 10.5.2 CommandHandler Type

```rust
pub type CommandHandler = Arc<
    dyn Fn(&str, Vec<String>) ->
        Pin<Box<dyn Future<Output = Result<String, CommandError>> + Send>>
        + Send + Sync,
>;
```

### 10.5.3 SlashCommand Structure

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

**Creating Commands**:
```rust
// Simple command
let cmd = SlashCommand::new(
    "echo",
    "Echo arguments back",
    Arc::new(|_name, args| {
        Box::pin(async move {
            Ok(format!("Echo: {:?}", args))
        })
    }),
);

// Command with logic
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

// Async command with error
let cmd = SlashCommand::new(
    "fetch",
    "Fetch data from URL",
    Arc::new(|_name, args| {
        Box::pin(async move {
            if args.is_empty() {
                return Err(CommandError::ExecutionFailed("URL required".to_string()));
            }
            let url = &args[0];
            // Fetch data...
            Ok(format!("Fetched from: {}", url))
        })
    }),
);
```

**Name Validation**:
```rust
// Valid names
"help", "status", "deploy-app", "runTest"

// Invalid names
"", "test command", "123test"  // Empty, contains space, starts with number
```

### 10.5.4 CommandRegistry

```rust
pub struct CommandRegistry {
    commands: HashMap<String, SlashCommand>,
}

impl Default for CommandRegistry { ... }
```

**Registry Operations**:
```rust
let mut registry = CommandRegistry::new();

// Register command
registry.register(cmd)?;  // Returns AlreadyRegistered error if duplicate

// Execute command
let result = registry.execute("echo", vec!["hello".to_string()]).await?;

// Check existence
if registry.exists("help") { ... }

// Get command
let cmd = registry.get("help");

// List commands
let names = registry.list_names();     // Vec<String>
let commands = registry.list_all();    // Vec<&SlashCommand>

// Count
let count = registry.len();
let empty = registry.is_empty();

// Unregister
registry.unregister("help")?;  // Returns NotFound error if not found

// Clear all
registry.clear();
```

## 10.6 Todos Module

### 10.6.1 TodoStatus Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TodoStatus {
    Pending,     // Not yet started
    InProgress,  // Currently being worked on
    Completed,   // Finished
}

impl TodoStatus {
    pub fn is_completed(&self) -> bool;
    pub fn is_active(&self) -> bool;  // Pending or InProgress
}
```

### 10.6.2 TodoItem Structure

```rust
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub status: TodoStatus,
    pub created_at: DateTime<Utc>,
}
```

**Creating Items**:
```rust
let item = TodoItem::new("123", "Write documentation");
// id: "123", content: "Write documentation", status: Pending

// Status transitions
item.start();     // Pending -> InProgress
item.complete();  // Any -> Completed
item.reset();     // Any -> Pending
```

### 10.6.3 TodoList Structure

```rust
pub struct TodoList {
    pub id: String,
    pub name: String,
    pub items: Vec<TodoItem>,
}
```

**Creating Lists**:
```rust
let mut list = TodoList::new("Project Tasks");
// Auto-generates UUID for id
```

**Adding Items**:
```rust
let item = list.add("Implement feature X");
// Auto-generates UUID for item.id
// Returns reference to the new item
```

**Status Operations**:
```rust
// Change status
list.start(&item.id)?;      // Returns NotFound error if not found
list.complete(&item.id)?;
list.reset(&item.id)?;
```

**Query Operations**:
```rust
// Get by ID
let item = list.get(&id);  // Option<&TodoItem>

// Filter by status
let pending = list.filter_by_status(TodoStatus::Pending);
let in_progress = list.filter_by_status(TodoStatus::InProgress);
let completed = list.filter_by_status(TodoStatus::Completed);

// Count by status
let counts = list.count_by_status();  // HashMap<TodoStatus, usize>
let completed_count = list.completed_count();

// Completion percentage
let percentage = list.completion_percentage();  // 0.0 - 100.0
```

**Item Management**:
```rust
// Remove item
list.remove(&id)?;  // Returns NotFound error if not found

// Count
let total = list.len();
let empty = list.is_empty();
```

### 10.6.4 TodoError Enum

```rust
pub enum TodoError {
    NotFound(String),    // Item not found
    InvalidInput(String), // Invalid input
}

impl std::error::Error for TodoError {}
impl fmt::Display for TodoError { ... }
```

## 10.7 Test Coverage

| Module | Tests | Focus Areas |
|--------|-------|-------------|
| logger.rs | 5 | Level parsing, entry creation, observer pattern |
| metrics.rs | 11 | Counters, gauges, histograms, timers, export |
| commands/mod.rs | 21 | Registration, execution, validation, errors |
| todos/mod.rs | 24 | Status transitions, list operations, errors |
| **Total** | **61** | |

## 10.8 API Reference

### Observability Re-exports

```rust
use claude_agent_sdk::observability::{
    // Logging
    Logger, LogLevel, LogEntry, LogFormat, LogObserver,
    ConsoleLogObserver, GlobalLogger,

    // Metrics
    MetricsCollector, MetricKind, LabeledMetric, Histogram,
    HistogramBuckets, MetricStorage, TimerGuard,
};
```

### Commands Re-exports

```rust
use claude_agent_sdk::commands::{
    SlashCommand, CommandRegistry, CommandError, CommandHandler,
};
```

### Todos Re-exports

```rust
use claude_agent_sdk::todos::{
    TodoItem, TodoList, TodoStatus, TodoError,
};
```

## 10.9 Usage Examples

### Complete Observability Setup

```rust
use claude_agent_sdk::observability::*;

// Create logger with observer
let observer = Arc::new(ConsoleLogObserver::json(LogLevel::Debug));
let logger = Logger::new("MyAgent")
    .with_min_level(LogLevel::Debug)
    .with_observer(observer);

// Create metrics collector
let metrics = MetricsCollector::with_prefix("myapp");

// Use in application
logger.info("Starting agent", &[("version", "1.0.0")]);
let _timer = metrics.start_timer("agent_execution", &[("agent", "researcher")]);

// ... do work ...

metrics.increment("tasks_completed", &[("type", "research")]);
logger.info("Agent completed", &[("duration_ms", "150")]);
```

### Command System Setup

```rust
use claude_agent_sdk::commands::*;

let mut registry = CommandRegistry::new();

// Register commands
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

// Execute
let result = registry.execute("help", vec![]).await?;
```

### Todo List Workflow

```rust
use claude_agent_sdk::todos::*;

let mut list = TodoList::new("Sprint Tasks");

// Add tasks
list.add("Implement authentication");
list.add("Write tests");
list.add("Update documentation");

// Start first task
let first_id = list.items[0].id.clone();
list.start(&first_id)?;

// Complete some tasks
list.complete(&first_id)?;
list.complete(&list.items[1].id)?;

// Check progress
println!("Progress: {:.1}%", list.completion_percentage());
println!("Completed: {}/{}", list.completed_count(), list.len());
```

## 10.10 Integration Patterns

### Logger + Metrics Integration

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

### Command with Logging

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
                // Deploy...
                logger.info("Deployment complete", &[("env", env)]);
                Ok(format!("Deployed to {}", env))
            })
        }
    }),
);
```
