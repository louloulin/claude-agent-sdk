# Documentation Index

Complete guide to all documentation in the Claude Agent SDK for Rust.

---

## 📚 Quick Start

1. [README.md](README.md) - Main project documentation
2. [Installation](README.md#installation) - Get started quickly
3. [Basic Usage](README.md#basic-usage) - First example
4. [API Key Setup](README.md#api-key-setup) - Security setup

---

## 🔒 Security

- [SECURITY.md](SECURITY.md) - **Start here for security**
  - API key management
  - Git security procedures
  - Production deployment guidelines
  - Vulnerability reporting

---

## 📖 Core Documentation

### API References
- [Public API Documentation](./docs/api/public-api.md) - Complete API reference
- [V2 API Guide](./docs/guides/v2-api-guide.md) - Session-based API guide

### Architecture
- [Architecture Overview](./docs/architecture/overview.md) - System design
- [Skills Architecture](./docs/architecture/skills.md) - Skills system design
- [Agent Orchestration](./docs/architecture/orchestration.md) - Orchestration patterns
- [Agents System](./docs/architecture/agents.md) - Agent types and usage

### Guides
- [Best Practices](./docs/guides/best-practices.md) - Usage recommendations
- [Subagent Guide](./docs/guides/subagent-guide.md) - Working with subagents
- [Troubleshooting](./docs/guides/troubleshooting.md) - Common issues and solutions
- [Real-World Skill Examples](./docs/guides/REAL_WORLD_SKILL_EXAMPLES.md) - Skill examples

### Development
- [Development Setup](./docs/development/setup.md) - Development environment
- [Contributing](CONTRIBUTING.md) - Contribution guidelines

---

## 🧪 Examples

### Example Index
- [Examples README](./crates/claude-agent-sdk/examples/README.md) - Complete examples guide
- [WASM Examples](./crates/claude-agent-sdk/examples/wasm/README.md) - WebAssembly examples
- [MCP Integration](./crates/claude-agent-sdk/examples/MCP_INTEGRATION.md) - MCP server integration

### Running Examples

```bash
# Basics
cargo run --example 01_hello_world
cargo run --example 02_limit_tool_use

# V2 API
cargo run --example 21_v2_api_simple
cargo run --example 22_v2_api_turns

# Hooks
cargo run --example 05_hooks_pretooluse
cargo run --example 15_hooks_comprehensive

# Advanced
cargo run --example 09_agents
cargo run --example 17_fallback_model
cargo run --example 18_max_budget_usd
```

---

## 📋 Reference

### Project Documentation
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
- [LICENSE.md](LICENSE.md) - MIT License

### Planning
- [Roadmap 2025](./docs/ROADMAP_2025.md) - Future plans
- [Plan 2.0](./docs/plan/plan2.0.md) - Implementation roadmap

---

## 🗂️ Module Documentation

### Core Modules
- **client.rs** - ClaudeClient for bidirectional streaming
- **query.rs** - Simple query() and query_stream() APIs
- **lib.rs** - Public API exports

### Feature Modules
- **commands/** - Slash command registration and execution
- **mcp/** - Model Context Protocol integration
- **observability/** - Logging and metrics
- **orchestration/** - Agent orchestration system
- **skills/** - Enhanced Skills system with validation
- **subagents/** - Subagent delegation
- **todos/** - Todo list management
- **types/** - Common type definitions
- **v2/** - V2 session-based API

---

## 🌐 Translations

- [中文文档](README.zh-CN.md) - Chinese documentation
- [中文文档索引](./docs/zh/README.md) - Chinese documentation index

---

## 🧪 Testing

- [Tests README](./tests/README.md) - Testing guide
- [Fixtures README](./fixtures/README.md) - Test fixtures guide
- [Tools README](./tools/README.md) - Development tools

---

## 🔗 External Resources

### Official Anthropic Resources
- [Claude Documentation](https://docs.anthropic.com/)
- [Claude Code CLI](https://docs.claude.com/claude-code)
- [Anthropic Console](https://console.anthropic.com/)

### Official SDKs
- [Python SDK](https://github.com/anthropics/claude-agent-sdk-python)
- [TypeScript SDK](https://github.com/anthropics/claude-agent-sdk-typescript)

### Standards
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [Anthropic API Reference](https://docs.anthropic.com/claude/reference/)

---

## 📞 Support

- **GitHub Issues**: [Report issues](https://github.com/louloulin/claude-agent-sdk/issues)
- **API Documentation**: [docs.rs](https://docs.rs/claude-agent-sdk-rs)
- **Security**: See [SECURITY.md](SECURITY.md)

---

**Last Updated**: 2026-01-13
**Version**: 0.7.0
