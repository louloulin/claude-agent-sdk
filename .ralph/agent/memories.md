# Memories

## Patterns

### mem-1771657857-12e0
> Zero-copy JSON parsing: Added ZeroCopyMessageParser with parse() and parse_bytes() methods. Added MessageKind enum for fast type detection using substring matching. Eliminates intermediate serde_json::Value allocation. 14 tests passing in message_parser.rs.
<!-- tags: rust-sdk, performance, json | created: 2026-02-21 -->

### mem-1771655310-f1c6
> Structured logging with tracing: Added tracing_setup.rs module with TracingConfig (production/development/testing presets), init_tracing(), generate_request_id(), span macros (query_span!, skill_span!, etc.), error category logging macros, metrics logging helpers. Added tracing-subscriber dependency. 9 tests passing.
<!-- tags: rust-sdk, observability, tracing | created: 2026-02-21 -->

### mem-1771654452-b5fd
> Error type categories: Added ErrorCategory enum (Network, Process, Parsing, Configuration, Validation, Permission, Resource, Internal, External), HttpStatus enum (12 codes), ErrorContext struct. Methods on ClaudeError: category(), error_code(), is_retryable(), http_status(), to_error_context(). Error codes format: ENET001, EPROC001, etc. Enables structured logging, metrics aggregation, retry logic, and HTTP API error responses.
<!-- tags: rust-sdk, error-handling, observability | created: 2026-02-21 -->

### mem-1771654114-4571
> Dynamic buffer implementation: Added DynamicBufferConfig with initial_size (64KB), max_message_size (50MB), growth_factor (2x). Uses AtomicBufferMetrics for thread-safe metrics tracking. Per-message size check instead of cumulative tracking. Backward compatible with max_buffer_size option. Access metrics via transport.get_buffer_metrics().
<!-- tags: rust-sdk, performance, buffer | created: 2026-02-21 -->

### mem-1771653309-f14f
> Lock contention optimization: SubprocessTransport now uses direct Option<T> for stdin/stdout instead of Arc<Mutex<Option<T>>>. For bidirectional mode (QueryFull), use take_stdin_arc() to get shared reference. Saves ~50-100ns per operation, no cache line bouncing.
<!-- tags: performance, rust-sdk, optimization | created: 2026-02-21 -->

### mem-1771652651-4b58
> Connection pool implementation: Created internal/pool.rs with PoolConfig, PooledWorker, WorkerGuard, ConnectionPool. Uses channel-based distribution with semaphore control. Pool disabled by default. Added pool_config to ClaudeAgentOptions. Target: reduce query latency from 300ms to <100ms by reusing CLI processes.
<!-- tags:  | created: 2026-02-21 -->

## Decisions

### mem-1771650972-6b60
> Claude Agent SDK商业化研究: Vercel B/200M ARR (平台+AI工具), LangChain .25B/12-16M ARR (开源+企业版), Claude Code B+ ARR (按结果付费). Anthropic 70-75%收入来自API调用, 36%使用量为编程任务. 四大商业化类型: Finance/PersonalAssistant/CustomerSupport/DeepResearch Agent.
<!-- tags: commercial, research, sdk | created: 2026-02-21 -->

### mem-1771460429-291f
> Rust SDK improvement plan: Phase 1 (1-2w): connection pooling, lock optimization, error refinement. Phase 2 (2-4w): batch API, dynamic buffers, zero-copy. Phase 3 (ongoing): fix examples, Clippy warnings, doc TODOs, 80%+ test coverage.
<!-- tags: roadmap, rust-sdk, planning | created: 2026-02-19 -->

### mem-1771460427-85e9
> Rust SDK vs Official SDKs: Rust has 100% feature parity + unique features: CLI auto-install, enhanced skills validation (12 fields), security auditor (10 risk patterns), progressive disclosure O(1) loading, hot reload support. V2 API is complete in Rust while TypeScript has only preview.
<!-- tags: comparison, sdk-features, rust-sdk | created: 2026-02-19 -->

## Fixes

### mem-1771649925-bbb6
> Fixed redundant_closure Clippy warning: use unwrap_or_else(String::new) instead of unwrap_or_else(|| String::new()) - pass the function directly instead of wrapping in closure
<!-- tags: clippy, rust-sdk, code-quality | created: 2026-02-21 -->

### mem-1771649691-8385
> Fixed unnecessary_sort_by Clippy warning: use sort_by_key(|b| std::cmp::Reverse(b.1)) instead of sort_by(|a, b| b.1.cmp(&a.1)) for descending sort by value - more idiomatic
<!-- tags: clippy, rust-sdk, code-quality | created: 2026-02-21 -->

### mem-1771649314-52af
> Fixed int_plus_one Clippy warning: change `current + 1 <= current_max` to `current < current_max` for simpler idiomatic comparison - mathematically equivalent and clearer
<!-- tags: clippy, rust-sdk, code-quality | created: 2026-02-21 -->

### mem-1771648950-be23
> Fixed bool_assert_comparison Clippy warning: use assert!(!x) instead of assert_eq!(x, false) for boolean assertions - better readability and Clippy compliant
<!-- tags: clippy, rust-sdk, code-quality | created: 2026-02-21 -->

### mem-1771646335-5a58
> Example 39 Path import: Fixed missing conditional import. Use #[cfg(feature = "sandbox")] use std::path::Path when Path is only used in sandbox feature code.
<!-- tags: examples, rust-sdk | created: 2026-02-21 -->

### mem-1771460414-2da8
> Rust SDK known issues: 1) No connection pooling - spawns new CLI per query 2) Lock contention in client.rs hot path 3) Fixed 10MB buffer may be insufficient 4) V2 prompt() creates new client per call 5) 7 example compilation errors 6) 17 Clippy warnings
<!-- tags: rust-sdk, issues, performance | created: 2026-02-19 -->

## Context

### mem-1771651507-3598
> Claude Agent SDK commercialization examples: Claude Cowork (2h=2months work), v0.dev-like web builder, Design System UI generator, Excalidraw (10min feature). Enterprise: Cognizant multi-agent, ServiceNow 44xM ACV, Salesforce AgentForce /conversation 213% ROI. Validation: SWE-bench Verified, Terminal-Bench, Azure AI Evaluation SDK.
<!-- tags: commercial, research, sdk | created: 2026-02-21 -->

### mem-1771460413-b297
> Rust SDK Performance: 1.5x faster than Python (300ms vs 500ms simple query), 6x faster concurrent (800ms vs 5000ms for 10 queries), 10x less memory (5MB vs 50MB idle). Bottlenecks: process spawn ~50-100ms, JSON serialization, lock contention.
<!-- tags: performance, benchmark, rust-sdk | created: 2026-02-19 -->

### mem-1771460410-b54b
> Claude Agent SDK Rust architecture: Layered design with transport (subprocess.rs), client (client.rs), V2 API (v2/mod.rs), skills system, orchestration. Uses typed-builder, async-trait, thiserror, async_stream. Key pattern: subprocess transport spawns CLI process per query.
<!-- tags: architecture, rust-sdk, codebase | created: 2026-02-19 -->
