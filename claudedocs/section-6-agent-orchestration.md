# Section 6: Agent Orchestration

The Agent Orchestration module provides a comprehensive framework for coordinating multiple AI agents to collaborate on complex tasks. It supports various orchestration patterns including sequential and parallel execution.

## 6.1 Overview

### Module Architecture

```
orchestration/
├── Core Types (agent.rs, orchestrator.rs)
├── Context Management (context.rs)
├── Error Handling (errors.rs)
├── Agent Registry (registry.rs)
└── Patterns
    ├── Sequential (patterns/sequential.rs)
    └── Parallel (patterns/parallel.rs)
```

### Feature Flags

| Feature | Description |
|---------|-------------|
| Default | All orchestration features enabled |
| `tracing` | Enable execution tracing support |

### Key Features

- **Flexible Orchestration Patterns**: Sequential and parallel execution modes
- **Type-Safe Agent Interface**: Strongly typed agent definitions with Rust's trait system
- **Async-First Design**: Full async/await support with Tokio
- **Execution Tracking**: Comprehensive execution traces for debugging and monitoring
- **Error Recovery**: Built-in retry logic with exponential backoff
- **Extensible**: Easy to add custom agents and orchestrators

---

## 6.2 Core Types

### AgentInput

Input to an agent:

```rust
use claude_agent_sdk::orchestration::AgentInput;

let input = AgentInput::new("Research quantum computing")
    .with_context(serde_json::json!({"depth": "advanced"}))
    .with_metadata("priority", "high");
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `content` | String | Main content/prompt for the agent |
| `context` | serde_json::Value | Additional context data |
| `metadata` | HashMap<String, String> | Metadata key-value pairs |

### AgentOutput

Output from an agent:

```rust
use claude_agent_sdk::orchestration::AgentOutput;

let output = AgentOutput::new("Research complete: 5 articles found")
    .with_confidence(0.95)
    .with_data(serde_json::json!({"articles": 5}))
    .with_metadata("duration_ms", "1500");

// Check if output indicates success
if output.is_successful() {
    println!("Agent succeeded with confidence: {}", output.confidence);
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `content` | String | Main response content |
| `data` | serde_json::Value | Additional structured data |
| `confidence` | f64 | Confidence score (0.0 - 1.0) |
| `metadata` | HashMap<String, String> | Metadata key-value pairs |

### OrchestratorInput / OrchestratorOutput

Input and output types for orchestrators:

```rust
use claude_agent_sdk::orchestration::{OrchestratorInput, OrchestratorOutput};

// Create orchestrator input
let input = OrchestratorInput::new("Analyze market trends")
    .with_context(serde_json::json!({"region": "US"}))
    .with_metadata("request_id", "12345");

// Check output
if output.is_successful() {
    println!("Final result: {}", output.result);
    println!("Agents executed: {}", output.agent_outputs.len());
}
```

---

## 6.3 Agent Trait

The `Agent` trait defines the interface for participating in orchestration:

```rust
use claude_agent_sdk::orchestration::{Agent, AgentInput, AgentOutput, Result};
use async_trait::async_trait;

#[derive(Debug)]
struct ResearchAgent;

#[async_trait]
impl Agent for ResearchAgent {
    fn name(&self) -> &str {
        "ResearchAgent"
    }

    fn description(&self) -> &str {
        "Researches topics and gathers information"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // Perform research logic
        let research_result = format!("Found 10 articles about: {}", input.content);

        Ok(AgentOutput::new(research_result)
            .with_confidence(0.9)
            .with_data(serde_json::json!({"sources": 10})))
    }
}
```

### SimpleAgent

Convenient wrapper for creating agents from closures:

```rust
use claude_agent_sdk::orchestration::agent::SimpleAgent;

let agent = SimpleAgent::new(
    "QuickAgent",
    "A simple agent for quick tasks",
    |input| {
        Ok(AgentOutput::new(format!("Processed: {}", input.content)))
    }
);
```

---

## 6.4 Orchestration Patterns

### Sequential Orchestration

Agents execute one after another, with each agent's output becoming the input for the next:

```text
Input → Agent A → Agent B → Agent C → Output
```

**Use cases:**
- Data processing pipelines
- Multi-step reasoning
- Content generation and refinement

```rust
use claude_agent_sdk::orchestration::{
    SequentialOrchestrator, Orchestrator, OrchestratorInput, Agent
};

// Create sequential orchestrator
let orchestrator = SequentialOrchestrator::new()
    .with_max_retries(3);

// Define pipeline agents
let agents: Vec<Box<dyn Agent>> = vec![
    create_researcher(),
    create_writer(),
    create_editor(),
];

// Execute pipeline
let input = OrchestratorInput::new("Climate Change");
let output = orchestrator.orchestrate(agents, input).await?;

println!("Final result: {}", output.result);
// Each agent's output is passed to the next:
// Researcher: "Found 5 articles about Climate Change"
//   → Writer: "Article draft based on Found 5 articles..."
//     → Editor: "Final version: Article draft..."
```

### Parallel Orchestration

Multiple agents execute concurrently, and their outputs are aggregated:

```text
        → Agent A ─┐
Input ─┼→ Agent B ─┼→ Aggregator → Output
        → Agent C ─┘
```

**Use cases:**
- Multi-angle analysis
- Parallel task processing
- Performance optimization

```rust
use claude_agent_sdk::orchestration::{
    ParallelOrchestrator, Orchestrator, OrchestratorInput, Agent
};

// Create parallel orchestrator with concurrency control
let orchestrator = ParallelOrchestrator::new()
    .with_max_retries(2)
    .with_parallel_limit(5);  // Max 5 concurrent agents

// Define parallel agents
let agents: Vec<Box<dyn Agent>> = vec![
    create_analyzer("technical"),
    create_analyzer("business"),
    create_analyzer("security"),
];

// Execute in parallel
let input = OrchestratorInput::new("New Architecture Proposal");
let output = orchestrator.orchestrate(agents, input).await?;

// Results are aggregated
println!("Aggregated results:\n{}", output.result);
// Output:
// Parallel execution results:
// 1. Technical Analysis: ...
// 2. Business Analysis: ...
// 3. Security Analysis: ...
```

---

## 6.5 Execution Context

### ExecutionConfig

Configuration for orchestration execution:

```rust
use claude_agent_sdk::orchestration::context::ExecutionConfig;
use std::time::Duration;

let config = ExecutionConfig::new()
    .with_timeout(Duration::from_secs(120))   // Max 2 minutes
    .with_max_retries(5)                      // 5 retries per agent
    .with_parallel_limit(20)                  // Max 20 parallel agents
    .with_logging(true)                       // Enable logging
    .with_tracing(true);                      // Enable tracing
```

**Default values:**
| Setting | Default |
|---------|---------|
| timeout | 300 seconds (5 minutes) |
| max_retries | 3 |
| parallel_limit | 10 |
| enable_logging | true |
| enable_tracing | true |

### ExecutionContext

Manages orchestration state:

```rust
use claude_agent_sdk::orchestration::context::ExecutionContext;

let ctx = ExecutionContext::new(config);

// Store state
ctx.set_state("key", serde_json::json!("value")).await;

// Retrieve state
if let Some(value) = ctx.get_state("key").await {
    println!("State: {}", value);
}

// Get execution trace
let trace = ctx.get_trace().await;
println!("Executions: {}", trace.agent_executions.len());
```

### ExecutionTrace

Records execution history:

```rust
use claude_agent_sdk::orchestration::context::ExecutionTrace;

let trace = output.execution_trace;

println!("Started: {}", trace.start_time);
println!("Duration: {:?}", trace.duration());

// Access individual agent executions
for exec in &trace.agent_executions {
    println!(
        "Agent: {} - Success: {} - Duration: {:?}ms",
        exec.agent_name,
        exec.success,
        exec.duration_ms
    );
}
```

### AgentExecution

Record of a single agent execution:

```rust
use claude_agent_sdk::orchestration::context::AgentExecution;

// Execution records are created automatically
// and accessible via ExecutionTrace.agent_executions

for exec in &output.execution_trace.agent_executions {
    println!("Agent: {}", exec.agent_name);
    println!("  Input: {}", exec.input.content);
    if let Some(ref output) = exec.output {
        println!("  Output: {}", output.content);
    }
    if let Some(ref error) = exec.error {
        println!("  Error: {}", error);
    }
}
```

---

## 6.6 Agent Registry

The registry provides centralized management for agent definitions:

### AgentMetadata

Rich metadata for agent definitions:

```rust
use claude_agent_sdk::orchestration::registry::AgentMetadata;

let metadata = AgentMetadata::new(
    "research-agent",
    "Academic Researcher",
    "Expert in academic research",
    "research"
)
.with_tool("web-search")
.with_tool("read-file")
.with_skill("citation")
.with_tag("academic")
.with_version("2.0.0")
.with_max_retries(5)
.with_timeout(120);
```

**Metadata Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Unique identifier |
| `name` | String | Human-readable name |
| `description` | String | What the agent does |
| `category` | String | Domain (e.g., "research", "analysis") |
| `version` | String | Semantic version |
| `tools` | Vec<String> | Tools this agent can use |
| `skills` | Vec<String> | Skills this agent possesses |
| `tags` | Vec<String> | Tags for filtering |
| `max_retries` | usize | Maximum retries |
| `timeout_secs` | u64 | Timeout in seconds |
| `enabled` | bool | Whether agent is enabled |

### AgentRegistry

Centralized registry for agent management:

```rust
use claude_agent_sdk::orchestration::registry::{AgentRegistry, AgentMetadata};

let registry = AgentRegistry::new();

// Register an agent
let agent = SimpleAgent::new("researcher", "Academic researcher", |input| {
    Ok(AgentOutput::new(format!("Researched: {}", input.content)))
});

let metadata = AgentMetadata::new(
    "researcher",
    "researcher",  // Must match agent.name()
    "Academic researcher",
    "research"
);

registry.register(Box::new(agent), metadata).await?;

// Check registration
assert!(registry.contains("researcher").await);

// Execute agent by ID
let output = registry.execute_agent(
    "researcher",
    AgentInput::new("Quantum computing")
).await?;
```

### AgentFilter

Filter criteria for searching agents:

```rust
use claude_agent_sdk::orchestration::registry::AgentFilter;

// Find agents by criteria
let filter = AgentFilter::new()
    .with_category("research")
    .with_tool("web-search")
    .with_tag("academic")
    .enabled_only();

let matching = registry.find(&filter).await;

for metadata in matching {
    println!("Found: {} - {}", metadata.name, metadata.description);
}
```

### AgentRegistryBuilder

Builder pattern for registry creation:

```rust
use claude_agent_sdk::orchestration::registry::AgentRegistryBuilder;

let registry = AgentRegistryBuilder::new()
    .with_agent(
        Box::new(SimpleAgent::new("agent1", "First", |input| {
            Ok(AgentOutput::new(format!("1: {}", input.content)))
        })),
        AgentMetadata::new("agent1", "agent1", "First agent", "test")
    ).await?
    .with_agent(
        Box::new(SimpleAgent::new("agent2", "Second", |input| {
            Ok(AgentOutput::new(format!("2: {}", input.content)))
        })),
        AgentMetadata::new("agent2", "agent2", "Second agent", "test")
    ).await?
    .build();
```

---

## 6.7 Error Handling

### OrchestrationError

Error types for orchestration operations:

```rust
use claude_agent_sdk::orchestration::errors::OrchestrationError;

match orchestrator.orchestrate(agents, input).await {
    Ok(output) => println!("Success: {}", output.result),
    Err(OrchestrationError::AgentFailed(name, reason)) => {
        eprintln!("Agent {} failed: {}", name, reason);
    }
    Err(OrchestrationError::Timeout(msg)) => {
        eprintln!("Operation timed out: {}", msg);
    }
    Err(OrchestrationError::InvalidConfig(msg)) => {
        eprintln!("Configuration error: {}", msg);
    }
    Err(OrchestrationError::PartialSuccess(count)) => {
        eprintln!("{} agents failed", count);
    }
    Err(e) => eprintln!("Other error: {}", e),
}

// Check if error is retryable
if error.is_retryable() {
    // Retry the operation
}
```

**Error Variants:**
| Variant | Description | Retryable |
|---------|-------------|-----------|
| `AgentFailed` | Agent execution failed | Yes |
| `AgentError` | Generic agent error | No |
| `OrchestratorFailed` | Orchestrator failed | No |
| `Timeout` | Operation timed out | Yes |
| `InvalidConfig` | Invalid configuration | No |
| `Cancelled` | Execution cancelled | No |
| `PartialSuccess` | Some agents failed | No |

---

## 6.8 Complete Example

```rust
use claude_agent_sdk::orchestration::{
    Agent, AgentInput, AgentOutput,
    SequentialOrchestrator, ParallelOrchestrator,
    Orchestrator, OrchestratorInput,
    AgentRegistry, AgentMetadata,
};
use async_trait::async_trait;

// Define custom agents
struct Researcher;
struct Writer;
struct Editor;

#[async_trait]
impl Agent for Researcher {
    fn name(&self) -> &str { "Researcher" }
    fn description(&self) -> &str { "Gathers information" }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk::orchestration::Result<AgentOutput> {
        Ok(AgentOutput::new(format!("Research: {} - Found 10 sources", input.content))
            .with_confidence(0.95))
    }
}

#[async_trait]
impl Agent for Writer {
    fn name(&self) -> &str { "Writer" }
    fn description(&self) -> &str { "Creates content" }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk::orchestration::Result<AgentOutput> {
        Ok(AgentOutput::new(format!("Draft: {}", input.content))
            .with_confidence(0.90))
    }
}

#[async_trait]
impl Agent for Editor {
    fn name(&self) -> &str { "Editor" }
    fn description(&self) -> &str { "Refines content" }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk::orchestration::Result<AgentOutput> {
        Ok(AgentOutput::new(format!("Final: {}", input.content))
            .with_confidence(0.92))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Sequential Pipeline
    println!("=== Sequential Pipeline ===");

    let sequential = SequentialOrchestrator::new();
    let agents: Vec<Box<dyn Agent>> = vec![
        Box::new(Researcher),
        Box::new(Writer),
        Box::new(Editor),
    ];

    let input = OrchestratorInput::new("AI Trends 2024");
    let output = sequential.orchestrate(agents, input).await?;

    println!("Success: {}", output.is_successful());
    println!("Steps: {}", output.agent_outputs.len());
    println!("Final: {}", output.result);
    println!("Duration: {:?}", output.execution_trace.duration());

    // Example 2: Parallel Analysis
    println!("\n=== Parallel Analysis ===");

    let parallel = ParallelOrchestrator::new()
        .with_parallel_limit(5);

    let analyzers: Vec<Box<dyn Agent>> = vec![
        Box::new(Researcher),  // Reuse as analyzers
    ];

    let input = OrchestratorInput::new("Market Analysis");
    let output = parallel.orchestrate(analyzers, input).await?;

    println!("Success: {}", output.is_successful());
    println!("Aggregated: {}", output.result);

    Ok(())
}
```

---

## 6.9 API Reference

### Re-exports from `orchestration` module

```rust
// Core types
pub use agent::{Agent, AgentInput, AgentOutput, AgentError, SimpleAgent};
pub use orchestrator::{Orchestrator, OrchestratorInput, OrchestratorOutput, BaseOrchestrator};

// Context
pub use context::{ExecutionConfig, ExecutionContext, ExecutionTrace, AgentExecution};

// Errors
pub use errors::{OrchestrationError, Result};

// Registry
pub use registry::{
    AgentRegistry, AgentRegistryBuilder,
    AgentMetadata, AgentFilter, RegistryError
};

// Patterns
pub use patterns::{SequentialOrchestrator, ParallelOrchestrator};
```

---

## 6.10 Best Practices

### Agent Design

1. **Single Responsibility**: Each agent should do one thing well
2. **Clear Contracts**: Define expected input/output formats clearly
3. **Confidence Scoring**: Provide meaningful confidence scores (0.0-1.0)
4. **Error Handling**: Return appropriate errors for retryable vs non-retryable failures

### Pattern Selection

| Use Case | Recommended Pattern |
|----------|-------------------|
| Data pipelines | Sequential |
| Multi-step workflows | Sequential |
| Multi-perspective analysis | Parallel |
| Batch processing | Parallel |
| Mixed dependencies | Sequential first, then Parallel |

### Performance

1. **Parallel Limit**: Set appropriate `parallel_limit` based on resources
2. **Retry Strategy**: Use `with_max_retries()` for flaky operations
3. **Timeout**: Set reasonable timeouts for long-running agents
4. **Registry**: Use `AgentRegistry` for dynamic agent management

### Monitoring

1. **Enable Tracing**: Always enable tracing in production
2. **Check Execution Traces**: Review agent execution times and success rates
3. **Log Failures**: Use logging to capture agent failures for debugging
