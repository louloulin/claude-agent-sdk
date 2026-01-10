---
name: context-fork-skill
description: Demonstrates forked context execution. This skill runs in an isolated sub-agent context with its own conversation history and tool access.
version: "1.0.0"
author: "SDK Team <sdk@example.com>"
tags:
  - testing
  - subagent
  - isolation
  - context
dependencies: []

context: fork
agent: general-purpose

allowed_tools:
  - Read
  - Write
  - Bash
  - Grep
  - Glob
---

# Context Fork Skill

This skill demonstrates the `context: fork` feature, which allows a skill to run in an isolated sub-agent context with its own conversation history and state.

## What is Forked Context?

When a skill uses `context: fork`, it:

1. **Creates a new sub-agent**: A separate agent instance is created
2. **Isolates conversation history**: Has its own independent context
3. **Provides clean state**: No contamination from the main conversation
4. **Can specify agent type**: Uses the `agent` field to determine sub-agent behavior

## When to Use Forked Context

### ✅ Good Use Cases

1. **Complex Multi-Step Operations**
   - Long-running analyses that shouldn't clutter main conversation
   - Multi-phase processing with intermediate states

2. **Isolated Testing**
   - Test code without affecting main context
   - Experiment with different approaches

3. **Specialized Agent Behavior**
   - Need specific agent type (Plan, Explore, etc.)
   - Different tool access patterns

4. **State Management**
   - Keep temporary state isolated
   - Prevent unintended side effects

### ❌ Avoid When

1. Simple one-shot tasks (overhead not worth it)
2. Need to share results immediately (forked context is isolated)
3. Real-time collaboration required (context is separate)

## Configuration

### Skill Metadata

```yaml
context: fork           # Enables forked context
agent: general-purpose  # Specifies agent type
```

### Available Agent Types

- **general-purpose**: Default agent for general tasks
- **Plan**: Planning and design focused
- **Explore**: Codebase exploration and analysis
- **code-reviewer**: Code review specialist
- **custom**: Custom agents defined in `.claude/agents/`

## Testing Forked Context

### Test 1: Basic Isolation

Ask me to perform a complex analysis:

```
Analyze this codebase and create a detailed report
```

**Behavior**:
1. A new sub-agent is created
2. The analysis runs in isolation
3. The main conversation stays clean
4. Results are returned when complete

### Test 2: Conversation History

Test that the forked context has separate history:

```
First, tell me your name in the forked context
Then ask in the main conversation what we discussed
```

**Expected**: The main conversation won't see the forked context's internal messages

### Test 3: Different Agent Types

Test with different agent types:

```yaml
# For planning tasks
context: fork
agent: Plan

# For exploration
context: fork
agent: Explore

# For code review
context: fork
agent: code-reviewer
```

## Advanced Features

### 1. Combining with Hooks

Forked context works with hooks:

```yaml
context: fork
agent: general-purpose
hooks:
  PreToolUse:
    - matcher: "Bash"
      hooks:
        - type: command
          command: "echo 'Forked context: Running Bash'"
```

### 2. Tool Restrictions

Limit tools in forked context:

```yaml
context: fork
agent: general-purpose
allowed_tools:
  - Read
  - Grep
  # No Write - read-only analysis
```

### 3. Model Selection

Use specific model in forked context:

```yaml
context: fork
agent: general-purpose
model: claude-sonnet-4-20250514
```

## Comparison: Forked vs Non-Forked

| Aspect | Non-Forked (Default) | Forked Context |
|--------|---------------------|----------------|
| **Conversation History** | Shared with main | Separate/isolated |
| **State** | Global state | Independent state |
| **Tool Access** | Inherits from main | Configured independently |
| **Context Window** | Uses main context | Separate context window |
| **Overhead** | None | Sub-agent creation |
| **Use Case** | Simple tasks | Complex/isolated tasks |

## Implementation Details

### Architecture

```
Main Conversation
    ↓
    User requests task using context-fork-skill
    ↓
Skill Engine
    ↓
    Detects context: fork
    ↓
Sub-Agent Factory
    ↓
    Creates new agent instance
    ↓
Forked Context
    ↓
    Executes task in isolation
    ↓
    Returns results
    ↓
Main Conversation (receives results)
```

### Memory Considerations

Forked context:
- ✅ Has its own context window
- ✅ Doesn't compete with main context for tokens
- ⚠️ Uses additional memory (sub-agent instance)
- ⚠️ Results still consume tokens when returned

### Performance

Forked context overhead:
- Creation: ~100-200ms
- Execution: Same as normal (no penalty)
- Cleanup: ~50ms

**Total overhead**: ~150-250ms per invocation

## Best Practices

### DO ✅

1. **Use for complex tasks**
   - Multi-step analyses
   - Long-running operations
   - Stateful processing

2. **Choose appropriate agent**
   - Plan for design tasks
   - Explore for code analysis
   - general-purpose for general tasks

3. **Combine with tool restrictions**
   - Read-only for analysis
   - Specific tools for specific tasks

4. **Document the isolation**
   - Explain why forked context is used
   - Clarify what's isolated

### DON'T ❌

1. **Don't use for simple tasks**
   - One-shot queries don't need isolation
   - Adds unnecessary overhead

2. **Don't forget about isolation**
   - Results need to be explicitly returned
   - Can't access main conversation state

3. **Don't overuse**
   - Each fork consumes resources
   - Multiple concurrent forks can be expensive

4. **Don't assume shared state**
   - Forked context starts fresh
   - No memory of previous interactions

## Testing Checklist

Verify forked context behavior:

- [ ] Sub-agent is created
- [ ] Conversation history is isolated
- [ ] Agent type is respected
- [ ] Tool restrictions work
- [ ] Results are returned correctly
- [ ] No contamination of main context
- [ ] Cleanup happens after completion
- [ ] Works with hooks
- [ ] Works with model selection

## Troubleshooting

### Fork Not Creating

**Problem**: Skill runs in main context

**Solutions**:
1. Verify `context: fork` is in YAML frontmatter
2. Check YAML syntax (no tabs, proper indentation)
3. Ensure agent field is specified

### Wrong Agent Type

**Problem**: Uses default agent instead of specified one

**Solutions**:
1. Check agent name spelling
2. Verify agent exists (for custom agents)
3. Use built-in agent names (general-purpose, Plan, Explore)

### Results Not Returned

**Problem**: Forked context completes but no results

**Solutions**:
1. Ensure skill returns output
2. Check for errors in forked context
3. Verify execution completed successfully

## Examples

### Example 1: Code Analysis

```yaml
---
name: deep-code-analysis
description: Perform deep code analysis in isolation
context: fork
agent: Explore
allowed_tools:
  - Read
  - Grep
  - Glob
---
```

**Usage**:
```
Analyze the authentication system deeply
```

### Example 2: Security Audit

```yaml
---
name: security-audit
description: Run security audit in isolated context
context: fork
agent: general-purpose
allowed_tools:
  - Read
  - Grep
  - Bash
hooks:
  PreToolUse:
    - matcher: "Bash"
      hooks:
        - type: command
          command: "echo 'Security audit: Executing command'"
---
```

### Example 3: Planning Task

```yaml
---
name: architecture-planner
description: Create architecture plan in isolation
context: fork
agent: Plan
---
```

**Usage**:
```
Create a detailed plan for refactoring the payment system
```

## Summary

Forked context provides:
- ✅ Isolated execution environment
- ✅ Separate conversation history
- ✅ Configurable agent types
- ✅ Clean state management
- ✅ No contamination of main context

Use when you need isolation, complex processing, or specialized agent behavior.

---

**Version**: 1.0.0
**Last Updated**: 2026-01-10
**Maintainer**: SDK Team
