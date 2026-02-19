# Scratchpad - cc-agent-sdk Documentation

## Objective
全面分析整个代码，按照章节编写cc-agent-sdk的文档

## Current Understanding

### Codebase Scope
The SDK is a Rust implementation of Claude Agent SDK with:
- **Source Files**: ~60 files in `crates/claude-agent-sdk/src/`
- **Key Modules**: client, query, v2, skills, mcp, orchestration, subagents
- **Architecture**: Layered design (Transport → Client → API → Features)

### Documentation Requirements
Need to create comprehensive documentation covering:
1. Getting Started (installation, basics)
2. Core API (ClaudeClient, query APIs)
3. V2 Session API
4. Skills System (extensive - 15+ files)
5. MCP Integration
6. Agent Orchestration
7. Subagents
8. Hooks System
9. Commands
10. Types Reference
11. Observability
12. Advanced Topics

## Plan Narrative
I will systematically review each module, starting from core components and moving to feature modules. For each section:
1. Read and analyze the source code
2. Identify public APIs and key types
3. Document usage patterns and examples
4. Note any important implementation details

## Progress
- [x] Explored codebase structure
- [x] Created documentation tasks
- [x] Reviewed Section 1 files (lib.rs, Cargo.toml, README.md, errors.rs)
- [ ] Emit review.section for detailed analysis

## Section 1: Getting Started - Review Complete

### Files Reviewed
- `crates/claude-agent-sdk/src/lib.rs` - Main library entry point with module docs
- `crates/claude-agent-sdk/Cargo.toml` - Package configuration and dependencies
- `README.md` - Comprehensive project documentation
- `crates/claude-agent-sdk/src/errors.rs` - Error type definitions

### Key Findings

**lib.rs Analysis:**
- Well-documented module with comprehensive doc comments
- Exports 4 API styles: Simple Query, Streaming, Bidirectional Client, V2 API
- Re-exports commonly used types from submodules
- Supports multimodal input (images via base64 or URL)
- Features: Hooks, Skills, MCP, Subagents, Orchestration

**Cargo.toml Analysis:**
- Package name: `cc-agent-sdk`
- Library name: `claude_agent_sdk`
- Key dependencies: tokio, serde, async-trait, thiserror, typed-builder
- Optional features: yaml, sandbox, hot-reload

**errors.rs Analysis:**
- Uses thiserror for error definitions
- Main error type: `ClaudeError` (enum with 15+ variants)
- Specific error types: ConnectionError, ProcessError, JsonDecodeError, etc.
- Result<T> type alias provided

**Documentation Structure Needs:**
1. Getting Started - Installation, basic usage
2. Core API - ClaudeClient, query APIs, errors
3. V2 Session API - Session management
4. Skills System - 15+ files to review
5. MCP Integration - Task management
6. Orchestration - Agent patterns
7. Types Reference - Messages, hooks, permissions

## Entry: 2026-02-19 - Initial Analysis
Completed codebase structure exploration. The SDK has a well-organized layered architecture. Ready to create tasks and begin section-by-section review.

## Entry: 2026-02-19 - Section 1 Complete
Reviewed Getting Started files. The codebase is well-documented with comprehensive doc comments. Ready to delegate to Deep Analyzer for detailed section documentation.

## Entry: 2026-02-19 - Section 2: Core API - Deep Analysis Complete

### Files Reviewed
- `crates/claude-agent-sdk/src/client.rs` - ClaudeClient for bidirectional streaming
- `crates/claude-agent-sdk/src/query.rs` - Simple query functions
- `crates/claude-agent-sdk/src/types/config.rs` - ClaudeAgentOptions and configuration types
- `crates/claude-agent-sdk/src/types/messages.rs` - Message types and content blocks

### Deep Analysis Findings

#### ClaudeClient (client.rs)

**API Design:**
- Bidirectional streaming client with session management
- 2 constructors: `new()` and `try_new()` (with early validation)
- Lifecycle: `connect()` → `query*()` → `receive*()` → `disconnect()`

**Public Methods (15):**
1. `new(options)` - Create client
2. `try_new(options)` - Create with early validation
3. `connect()` - Establish connection
4. `query(prompt)` - Send text query
5. `query_with_session(prompt, session_id)` - Query with session context
6. `query_with_content(content)` - Multimodal query
7. `query_with_content_and_session(content, session_id)` - Full multimodal
8. `receive_messages()` - Continuous stream
9. `receive_response()` - Stream until ResultMessage
10. `interrupt()` - Stop current operation
11. `set_permission_mode(mode)` - Dynamic permission change
12. `set_model(model)` - Dynamic model change
13. `rewind_files(user_message_id)` - File checkpoint restore
14. `get_server_info()` - Server capabilities
15. `new_session(session_id, prompt)` - Start new session
16. `disconnect()` - Clean shutdown

**Correctness Issues:**
- Line 826-832: Drop impl warns but can't cleanup async resources
- Users MUST call disconnect() explicitly

**Edge Cases:**
- `connect()` is idempotent (checks `connected` flag)
- `disconnect()` is idempotent (early return if not connected)
- Empty content validation in `query_with_content`

**Security:**
- No input sanitization on prompts
- Session IDs are user-controlled (no validation)
- Direct stdin access bypasses transport lock (intentional design)

**Performance:**
- Lock contention pattern: acquire lock → clone Arc → drop lock → use clone
- 100ms sleep on disconnect for background task cleanup
- Arc<Mutex<>> pattern for shared state

**Testability:**
- Well-isolated concerns
- Clear error boundaries
- Uses trait objects for transport abstraction

#### Query Functions (query.rs)

**4 Functions:**
1. `query(prompt, options)` - Simple one-shot query
2. `query_stream(prompt, options)` - Streaming one-shot
3. `query_with_content(content, options)` - Multimodal one-shot
4. `query_stream_with_content(content, options)` - Full multimodal streaming

**Correctness:**
- All functions create new InternalClient per call
- Stream uses async_stream! macro for lifetime management
- Error propagation through yield

**Performance:**
- query(): O(n) memory (collects all messages)
- query_stream(): O(1) per message

#### Configuration Types (config.rs)

**ClaudeAgentOptions - 40+ fields with TypedBuilder:**
- Tools configuration (allowed/disallowed)
- System prompt (text or preset)
- MCP servers
- Permission mode
- Session management (resume, continue, fork)
- Budget and token limits
- Hooks, callbacks, plugins
- Sandbox settings
- Skills auto-discovery

**Key Types:**
- `SystemPrompt` - Text or Preset
- `PermissionMode` - Default, AcceptEdits, Plan, BypassPermissions
- `SettingSource` - User, Project, Local (precedence order)
- `AgentDefinition` - Custom agent config
- `Tools` - List or Preset
- `SandboxSettings` - Isolation configuration

**Pattern:** Uses `#[builder(default, setter(strip_option))]` for optional fields

#### Message Types (messages.rs)

**Message Enum (6 variants):**
- Assistant, System, Result, StreamEvent, User, ControlCancelRequest

**Content Blocks:**
- Text, Thinking, ToolUse, ToolResult, Image
- UserContentBlock for input (Text, Image)

**Image Support:**
- MIME types: jpeg, png, gif, webp
- Max base64 size: 15MB
- Validation in `UserContentBlock::image_base64()`

**Testing:** 30+ unit tests covering serialization/deserialization

### Documentation Content Ready for Section 2

```markdown
## Section 2: Core API

### 2.1 ClaudeClient - Bidirectional Streaming
- Constructor methods
- Lifecycle management
- Query methods (4 variants)
- Response streaming
- Dynamic control (interrupt, permissions, model)
- Session management
- File checkpointing

### 2.2 Simple Query Functions
- query() - Basic queries
- query_stream() - Memory-efficient streaming
- query_with_content() - Multimodal
- query_stream_with_content() - Full features

### 2.3 Configuration (ClaudeAgentOptions)
- All 40+ fields documented
- Builder pattern usage
- Common configurations

### 2.4 Message Types
- Message enum variants
- Content blocks
- Image handling
- Tool use patterns

### 2.5 Error Handling
- ClaudeError variants
- Result type alias
- Error recovery patterns
```


## Entry: 2026-02-19 - Code Review: Section 2 Core API

### Review Summary
**Status**: APPROVE

### Files Reviewed
- ✅ crates/claude-agent-sdk/src/client.rs (835 lines)
- ✅ crates/claude-agent-sdk/src/query.rs (261 lines)
- ✅ crates/claude-agent-sdk/src/types/config.rs (analyzed by Deep Analyzer)
- ✅ crates/claude-agent-sdk/src/types/messages.rs (analyzed by Deep Analyzer)

### Verification Results

**ClaudeClient Method Count Verification:**
- Scratchpad claims: "15 public methods"
- Actual count: 16 methods
- **Discrepancy**: Under-counted by 1

**Actual Methods:**
1. new(options) ✅
2. try_new(options) ✅
3. connect() ✅
4. query(prompt) ✅
5. query_with_session(prompt, session_id) ✅
6. query_with_content(content) ✅
7. query_with_content_and_session(content, session_id) ✅
8. receive_messages() ✅
9. receive_response() ✅
10. interrupt() ✅
11. set_permission_mode(mode) ✅
12. set_model(model) ✅
13. rewind_files(user_message_id) ✅
14. get_server_info() ✅
15. new_session(session_id, prompt) ✅
16. disconnect() ✅

**Critical Issues: None** ✅

**Correctness Analysis:**
- Line 826-832: Drop implementation correctly warns users ✅
- Users MUST call disconnect() explicitly ✅
- Lock contention pattern verified ✅
- Arc<Mutex<>> usage verified ✅

**Edge Cases Documented:**
- connect() idempotency verified ✅
- disconnect() idempotency verified ✅
- Empty content validation present in query.rs:172, 230 ✅

**Security Observations:**
- No input sanitization on prompts (documented) ✅
- Session IDs user-controlled (documented) ✅
- This is intentional design for flexibility

**Performance Characteristics:**
- query(): O(n) memory - collects all messages ✅
- query_stream(): O(1) per message - streaming ✅
- async_stream! macro usage verified ✅

### Recommendations

**For Deep Analyzer:**
1. Update method count from "15" to "16" in documentation
2. All other findings are accurate and well-documented

**Positive Notes:**
- Code is well-documented with comprehensive doc comments
- Error handling is consistent and idiomatic
- Test coverage visible in messages.rs (30+ tests)
- Builder pattern usage is idiomatic Rust
- Security trade-offs are documented
- Performance characteristics clearly documented

### Next Steps
Ready for review.complete event → Ralph coordinates next section

## Entry: 2026-02-19 - Section 4: Skills System Review Complete

### Section 4: Skills System - Review Summary
**Status**: APPROVE ✅

### Files Reviewed (12 core files analyzed):
1. ✅ mod.rs (175 lines) - Module entry point with SkillRegistry
2. ✅ api.rs (518 lines) - SkillsApiClient for Anthropic API
3. ✅ types.rs (480 lines) - Core types: SkillMetadata, SkillPackage, SkillResources
4. ✅ error.rs (111 lines) - SkillError, SkillOutput
5. ✅ discovery.rs (265 lines) - Skill discovery from filesystem
6. ✅ trait_impl.rs (149 lines) - Skill trait and SkillBox wrapper
7. ✅ skill_md.rs (1614 lines) - SKILL.md parser with full YAML frontmatter
8. ✅ auditor.rs (668 lines) - Security auditing with 10 risk patterns
9. ✅ progressive_disclosure.rs (442 lines) - O(1) lazy loading pattern
10. ✅ hot_reload.rs (359 lines) - File watching with optional feature
11. ✅ performance.rs (576 lines) - LRU cache, indexed collections
12. ✅ tool_restriction.rs (538 lines) - Tool restriction enforcement

### Architecture Analysis

**Module Organization:**
```
skills/
├── Core Types (types.rs, error.rs, trait_impl.rs)
├── Discovery (discovery.rs, skill_md.rs)
├── Security (auditor.rs, tool_restriction.rs, sandbox.rs)
├── Performance (performance.rs, progressive_disclosure.rs)
├── Integration (api.rs, vscode.rs, hot_reload.rs)
└── Utilities (tags.rs, version.rs, dependency.rs)
```

**Key Design Patterns:**
1. **Trait-based extensibility**: Skill trait with lifecycle hooks (before/after execute)
2. **Progressive disclosure**: Main content always loaded, supporting files on-demand
3. **Security auditing**: 10 pattern types detected, risk levels Safe→Critical
4. **Tool restrictions**: Pattern-based (e.g., `Bash(python:*)`)
5. **LRU caching**: Query result caching for performance

**Claude Code Compatibility:**
- Full SKILL.md YAML frontmatter support
- 12 metadata fields supported (name, description, version, author, tags, dependencies, allowed_tools, model, context, agent, hooks, user_invocable)
- Validation per Claude Skills specification (name: 64 chars, lowercase-hyphen only, no XML tags)

### Verification Results

**Test Coverage:**
- skill_md.rs: 30+ tests (validation, parsing, parallel scan)
- types.rs: 15+ tests (serialization, resources)
- tool_restriction.rs: 15+ tests (patterns, wildcards)
- performance.rs: 20+ tests (LRU cache, indexed collections)
- auditor.rs: 10+ tests (security patterns, risk levels)
- hot_reload.rs: 5+ tests (config, events)

**Code Quality:**
- Well-documented with comprehensive doc comments ✅
- Uses thiserror for error definitions ✅
- Feature-gated: yaml, hot-reload, sandbox ✅
- Async support via tokio ✅
- Idiomatic Rust patterns ✅

### Critical Issues: None ✅

### Suggestions (Should Consider):
1. **api.rs:354-396** - `zip_skill()` implementation is placeholder:
   - Should use proper zip library (e.g., `zip` crate) for production
   - Current implementation writes text markers, not actual zip

2. **LruCache O(n) removal** - performance.rs:98-100:
   - Consider using `LinkedHashMap` or `lru` crate for O(1) operations

### Positive Notes
- Excellent Claude Code specification compliance
- Comprehensive validation (name length, format, reserved words, XML tags)
- Progressive disclosure pattern well-implemented
- Security auditor covers 10 risk patterns
- Good test coverage across all modules
- Feature flags for optional dependencies
- Clear separation of concerns

### Documentation Content Ready for Section 4

```markdown
## Section 4: Skills System

### 4.1 Overview
- Module architecture
- Feature flags (yaml, hot-reload, sandbox)
- Claude Code compatibility

### 4.2 Core Types
- SkillMetadata - 7 fields (id, name, description, version, author, dependencies, tags)
- SkillPackage - Complete skill with instructions, scripts, resources
- SkillResources - Folders, tools, tests configuration
- SkillInput/SkillOutput - Execution types
- SkillStatus enum

### 4.3 Skill Trait
- Required methods (name, description, execute, validate)
- Optional methods (version, author, tags, dependencies, supports)
- Lifecycle hooks (before_execute, after_execute, on_error)
- SkillBox wrapper for trait objects

### 4.4 SKILL.md Parser
- YAML frontmatter parsing
- 12 metadata fields supported
- Validation rules (name: 64 chars, lowercase-hyphen, no XML tags)
- Progressive disclosure resource caching
- Parallel scanning support

### 4.5 Discovery System
- discover_from_dir() - JSON packages
- discover_skill_md_from_dir() - SKILL.md files
- discover_from_multiple_dirs() - Priority-based merging
- SkillsDirScanner for batch operations

### 4.6 Security Auditor
- Risk levels: Safe, Low, Medium, High, Critical
- 10 pattern types: NetworkAccess, DangerousCommand, FileAccess, CodeExecution, etc.
- AuditConfig for customization
- SkillAuditReport with severity filtering

### 4.7 Tool Restrictions
- Pattern-based restrictions (Bash(python:*))
- Wildcard support (*)
- ToolRestriction API

### 4.8 Performance Optimizations
- IndexedSkillCollection (O(1) lookups by name/tag)
- LruCache for query results
- BatchOperations utilities
- Progressive disclosure (lazy loading)

### 4.9 Hot Reload
- File system monitoring (feature-gated)
- HotReloadEvent types (Created, Modified, Deleted, Error)
- HotReloadManager for state management

### 4.10 Skills API Client
- HTTP client for Anthropic Skills API
- upload_skill(), list_skills(), get_skill(), delete_skill()
- SkillsError types
```

### Next Steps
Emit `review.section` → Deep Analyzer creates detailed documentation
