# Claude Agent SDK Rust - Project Overview

## Purpose
Rust SDK for Claude Agent, providing programmatic access to Claude's capabilities with type-safe, high-performance API. Achieves 98.3% feature parity with official Python/TypeScript SDKs.

## Tech Stack
- **Language**: Rust 1.85+
- **Edition**: 2024
- **Key Dependencies**:
  - tokio (async runtime)
  - serde/serde_json (serialization)
  - async-trait, futures, async-stream
  - thiserror, anyhow (error handling)
  - typed-builder (builder pattern)
  - semver (version checking)
  - reqwest (HTTP client)

## Architecture
Layered design:
1. **Transport Layer** - SubprocessTransport ↔ Claude Code CLI
2. **Client Layer** - ClaudeClient (bidirectional streaming)
3. **API Layer** - query(), query_stream(), V2 API
4. **Features** - Hooks, Skills, MCP, Subagents, Orchestration

## Module Structure
```
crates/claude-agent-sdk/src/
├── client.rs       # ClaudeClient
├── query.rs        # Simple query APIs
├── internal/       # Internal implementation
│   ├── transport/  # Subprocess transport
│   ├── client.rs   # Internal client logic
│   └── message_parser.rs
├── v2/             # V2 Session API
├── skills/         # Enhanced skills system
├── mcp/            # MCP integration
├── orchestration/  # Agent orchestration
├── types/          # Type definitions
└── errors.rs       # Error types
```

## Unique Features (vs Official SDKs)
- CLI auto-install capability
- Enhanced skills validation (12 fields)
- Security auditor (10 risk patterns)
- Progressive disclosure O(1) loading
- Hot reload support
- V2 API complete (TypeScript has only preview)