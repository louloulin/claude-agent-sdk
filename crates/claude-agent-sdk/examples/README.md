# Claude Agent SDK - Examples

Welcome to the Claude Agent SDK examples directory! This collection contains 70 examples demonstrating all features of the SDK.

## 📋 Prerequisites

Before running any examples, make sure you have:

1. **Rust**: Version 1.90 or higher
2. **Claude Code CLI**: Version 2.0.0 or higher
3. **API Key**: Set your Anthropic API key
   ```bash
   export ANTHROPIC_API_KEY=your_api_key_here
   ```

## 🚀 Quick Start

### Run Your First Example
```bash
cargo run --example 01_hello_world
```

### Run Any Example
```bash
cargo run --example <example_name>
```

## 📚 Examples by Category

### 1. Basic Core Features (01-23)
Beginner-friendly examples covering SDK fundamentals

- 01_hello_world - Simple query example
- 02_limit_tool_use - Restrict tool usage
- 06_bidirectional_client - Bidirectional streaming
- 09_agents - Agent orchestration
- ...and 19 more

### 2. Agent Skills System (30-41)
Advanced Skills features and patterns

- 30_agent_skills - Skills overview
- 32_agent_skills_discovery - Discovery mechanism
- 38_agent_skills_hot_reload - Hot reloading
- ...and 10 more

### 3. Advanced Patterns (42-54)
Production-ready patterns and real-world usage

- 42_mcp_async_tasks - Async MCP tasks
- 44_concurrent_queries - Concurrency patterns
- 48_performance_benchmarking - Performance testing
- 52_batch_operations - Batch processing patterns
- 53_error_recovery - Error recovery and retry patterns
- 54_advanced_streaming - Backpressure and stream composition
- ...and more

### 4. Production & Integration (50-55)
Enterprise features and deployment

- 50_production_deployment - Deployment guide
- 51_orchestration - Orchestration patterns
- 55_real_skill_md_verification - Verification

### 5. New API Modules (56-62)
Modern API patterns and utilities

- 56_v2_api - Simplified TypeScript-style V2 API
- 57_todos - Todo list management
- 58_commands - Slash command registration and execution
- 59_subagents - Specialized subagent system
- 60_observability - Structured logging and metrics collection
- 61_multi_modal - Comprehensive multimodal input handling
- 62_advanced_v2_api - Advanced V2 API features and middleware

### 6. Integration & CLI (63-67)
Advanced integration patterns and CLI tools

- 65_v2_middleware - Middleware patterns (retry, caching, rate limiting, metrics)
- 66_full_integration - Combining multiple SDK features together
- 67_cli_tools - Building interactive CLI tools with the SDK

## 📖 Learning Path

1. **Start Here** (Beginner)
   - 01_hello_world
   - 02_limit_tool_use
   - 13_system_prompt

2. **Core Features** (Intermediate)
   - 06_bidirectional_client
   - 14_streaming_mode
   - 05_hooks_pretooluse

3. **Advanced Topics** (Advanced)
   - 09_agents
   - 30_agent_skills
   - 42_mcp_async_tasks
   - 52_batch_operations
   - 53_error_recovery

4. **New APIs** (Modern SDK)
   - 56_v2_api - Simplified API
   - 58_commands - Custom commands
   - 60_observability - Logging and metrics
   - 61_multi_modal - Image and multimodal handling
   - 62_advanced_v2_api - Middleware and advanced patterns

5. **Integration Patterns** (Production)
   - 65_v2_middleware - Build middleware chains
   - 66_full_integration - Full-stack integration example
   - 67_cli_tools - CLI application patterns

## 🆕 New Examples (65-67)

### 65_v2_middleware.rs
Demonstrates middleware patterns for the V2 API:
- **RetryMiddleware**: Exponential backoff retry logic
- **LoggingMiddleware**: Request/response logging
- **CachingMiddleware**: Response caching with TTL
- **RateLimitMiddleware**: Request rate limiting
- **MetricsMiddleware**: Performance metrics collection
- **TransformationMiddleware**: Request/response transformation

### 66_full_integration.rs
Shows how to combine multiple SDK features:
- Application state management
- Session management with persistence
- Command registry for slash commands
- Subagent dispatching
- Multi-modal content building
- Streaming response handling
- Cost tracking

### 67_cli_tools.rs
Building interactive CLI applications:
- Interactive REPL engine
- Command parsing and handling
- Progress indicators
- Rich output formatting (Text/Markdown/JSON)
- Session management
- Configuration management
- Error handling and recovery

## 🔧 Common Issues

### "ANTHROPIC_API_KEY not set"
```bash
export ANTHROPIC_API_KEY=your_key_here
```

### Example compiles but doesn't run
This is expected - examples need valid API key and network connection.

## 📞 Getting Help

- **Main README**: [../../README.md](../../README.md)
- **Analysis Report**: [../../EXAMPLES_ANALYSIS_REPORT.md](../../EXAMPLES_ANALYSIS_REPORT.md)
- **GitHub**: [louloulin/claude-agent-sdk](https://github.com/louloulin/claude-agent-sdk)

---

**Happy Coding! 🚀**
