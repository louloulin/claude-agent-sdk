# Scratchpad - Extension Crate Architecture Refinement

## Current Analysis

### What's Already Implemented (In Core)
1. **internal/pool.rs** - ConnectionPool, PoolConfig, PooledWorker
2. **internal/message_parser.rs** - ZeroCopyMessageParser, MessageKind
3. **observability/** - TracingConfig, MetricsCollector, histogram support
4. **skills/** - Skill trait, SkillRegistry, discovery
5. **orchestration/** - Agent, Orchestrator, SequentialOrchestrator, ParallelOrchestrator
6. **subagents/** - Subagent, SubagentExecutor

### Objective
Update PROMPT.md to add separate extension crates instead of implementing in current crate. Focus on:
1. Minimal changes - extract existing code, not rewrite
2. Maximize code reuse
3. Clear migration path

### Plan: Minimal Extraction

**Phase 0: Create Extension Crate Skeletons**
- Create directory structure for each extension crate
- Empty Cargo.toml with dependencies

**Phase 1: Extract pool → claude-agent-sdk-pool**
- Move internal/pool.rs to new crate
- Update core to depend on pool crate (optional dep)
- Minimal changes, just re-export

**Phase 2: Extract observability → claude-agent-sdk-observability**
- Move observability/ directory to new crate
- Keep tracing basics in core, advanced metrics in extension

**Phase 3: Extract batch → claude-agent-sdk-batch** (New)
- Create BatchExecutor using existing pool
- Parallel query execution

**Phase 4: Extract agents → claude-agent-sdk-agents** (New)
- Move subagents/ to new crate
- Add prebuilt agents (CodeReviewer, etc.)

**Phase 5: session, cost, mcp crates** (Future)

### Key Decisions
1. **Feature flags in core for backward compatibility**: Users can still use `features=["pool"]` in core crate
2. **Re-exports from extensions**: Core re-exports extension types when feature enabled
3. **Independent versioning**: Each crate can version independently

### Updated PROMPT.md Section
Need to update:
1. Phase 0 task list - already have pool.rs, observability in core
2. Make extraction plan clearer - what moves, what stays
3. Add code reuse strategy
4. Update timeline based on current state

## Current Task
Update PROMPT.md with refined plan showing:
1. What already exists in core
2. What needs to be extracted
3. Minimal extraction approach
4. Code reuse strategy
