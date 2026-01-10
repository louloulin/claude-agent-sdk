---
name: hooks-test
description: Test skill demonstrating hooks functionality. Shows PreToolUse, PostToolUse, and Stop hooks in action.
version: "1.0.0"
author: "SDK Team <sdk@example.com>"
tags:
  - testing
  - hooks
  - development
dependencies: []

allowed_tools:
  - Read
  - Write
  - Bash
  - Grep

hooks:
  pre_tool_use:
    - matcher: "Bash"
      command: "echo '🔍 PreToolUse: About to execute Bash tool'"
      type: command
      once: false
    - matcher: "Write"
      command: "echo '📝 PreToolUse: About to write file - $TOOL_NAME'"
      type: command
      once: false
    - matcher: "*"
      command: "echo '⚡ PreToolUse: Tool $TOOL_NAME is about to be used'"
      type: command
      once: true
  post_tool_use:
    - matcher: "Bash"
      command: "echo '✅ PostToolUse: Bash tool executed successfully'"
      type: command
      once: false
    - matcher: "Write"
      command: "echo '💾 PostToolUse: File written successfully'"
      type: command
      once: false
  stop:
    - matcher: "*"
      command: "echo '🛑 Stop hook: Cleaning up resources...'"
      type: command
      once: false
---

# Hooks Test Skill

Demonstrates the complete hooks functionality in Claude Agent SDK. This skill showcases how PreToolUse, PostToolUse, and Stop hooks work together to provide event-driven automation.

## What are Hooks?

Hooks are event-driven commands that run at specific points during skill execution:

- **PreToolUse**: Runs BEFORE a tool is executed
- **PostToolUse**: Runs AFTER a tool completes successfully
- **Stop**: Runs when the skill execution stops or completes

## Hook Configuration

Each hook has:
- **matcher**: Pattern to match tool names (e.g., "Bash", "Write", "*")
- **command**: Shell command to execute
- **once**: If true, runs only once per session (default: false)
- **type**: Execution type - "command", "script", or "function"

## Testing the Hooks

### Test 1: Bash Tool Hooks

When you ask me to run a Bash command:

```
Please run: echo "Hello World"
```

**Expected Output**:
```
🔍 PreToolUse: About to execute Bash tool
[Bash command executes]
✅ PostToolUse: Bash tool executed successfully
```

### Test 2: Write Tool Hooks

When you ask me to write a file:

```
Create a file named test.txt with content "Hello"
```

**Expected Output**:
```
📝 PreToolUse: About to write file - Write
[File is written]
💾 PostToolUse: File written successfully
```

### Test 3: Wildcard Matcher

The `*` matcher with `once: true` will run only once for the first tool used:

```
Run any command
```

**Expected Output**:
```
⚡ PreToolUse: Tool <tool_name> is about to be used
[This message appears only once per session]
```

### Test 4: Stop Hook

When this skill finishes execution:

**Expected Output**:
```
🛑 Stop hook: Cleaning up resources...
```

## Hook Features Demonstrated

### 1. Tool-Specific Hooks
- PreToolUse and PostToolUse for specific tools (Bash, Write)
- Fine-grained control over tool execution

### 2. Wildcard Matching
- `*` matcher applies to all tools
- Useful for global hooks like logging

### 3. Once-Only Execution
- `once: true` ensures hook runs only once per session
- Perfect for initialization tasks

### 4. Multiple Hooks Per Event
- Can stack multiple hooks for the same event
- Run in sequence for chained operations

## Advanced Usage

### Pattern Matching

You can use patterns in the matcher field:

```yaml
hooks:
  PreToolUse:
    # Match specific tool
    - matcher: "Bash"
      hooks:
        - type: command
          command: "echo 'Bash tool detected'"

    # Match multiple tools with wildcard
    - matcher: "Bash*"
      hooks:
        - type: command
          command: "echo 'Any Bash-related tool'"

    # Match all tools
    - matcher: "*"
      hooks:
        - type: command
          command: "echo 'Any tool'"
```

### Environment Variables

Hooks can access environment variables:

```yaml
hooks:
  PreToolUse:
    - matcher: "*"
      hooks:
        - type: command
          command: "echo 'Tool: $TOOL_NAME, Input: $TOOL_INPUT'"
```

### Script Hooks

Instead of inline commands, use scripts:

```yaml
hooks:
  PreToolUse:
    - matcher: "Write"
      hooks:
        - type: script
          command: "scripts/validate_write.sh"
```

With `scripts/validate_write.sh`:
```bash
#!/bin/bash
echo "Validating write operation..."
echo "Tool: $TOOL_NAME"
echo "Input: $TOOL_INPUT"
# Add validation logic here
```

## Common Use Cases

### 1. Security Validation

```yaml
hooks:
  PreToolUse:
    - matcher: "Bash"
      hooks:
        - type: script
          command: "scripts/security_check.sh"
```

### 2. Audit Logging

```yaml
hooks:
  PostToolUse:
    - matcher: "*"
      hooks:
        - type: command
          command: "echo '$(date): Tool $TOOL_NAME used' >> audit.log"
```

### 3. Resource Cleanup

```yaml
hooks:
  Stop:
    - matcher: "*"
      hooks:
        - type: script
          command: "scripts/cleanup.sh"
```

### 4. Performance Monitoring

```yaml
hooks:
  PreToolUse:
    - matcher: "*"
      hooks:
        - type: command
          command: "echo $(date +%s%N) > /tmp/start_time"
  PostToolUse:
    - matcher: "*"
      hooks:
        - type: command
          command: |
            START=$(cat /tmp/start_time)
            END=$(date +%s%N)
            echo "Tool execution time: $((($END - $START) / 1000000))ms"
```

## Testing Checklist

Test each hook type:

- [ ] PreToolUse fires before Bash
- [ ] PreToolUse fires before Write
- [ ] PostToolUse fires after Bash
- [ ] PostToolUse fires after Write
- [ ] Wildcard matcher works for all tools
- [ ] `once: true` prevents repeat execution
- [ ] Stop hook fires on skill completion
- [ ] Multiple hooks execute in sequence
- [ ] Hook failures don't stop main execution

## Troubleshooting

### Hook Not Firing

**Problem**: Hook command doesn't execute

**Solutions**:
1. Check YAML syntax (use spaces, not tabs)
2. Verify matcher pattern matches tool name
3. Ensure command is executable
4. Check hook event type (PreToolUse vs PostToolUse)

### Once Flag Not Working

**Problem**: Hook with `once: true` keeps running

**Solution**: The `once` flag applies per session. Restart the session to test.

### Hook Failing Silently

**Problem**: Hook errors not visible

**Solution**: Add error handling:
```yaml
hooks:
  PreToolUse:
    - matcher: "*"
      hooks:
        - type: command
          command: "python scripts/hook.py 2>&1 || echo 'Hook failed'"
```

## Best Practices

### DO ✅

1. Use specific matchers when possible
2. Keep hook commands simple and fast
3. Add proper error handling
4. Use scripts for complex logic
5. Test hooks in isolation
6. Log hook execution for debugging

### DON'T ❌

1. Don't put long-running commands in hooks
2. Don't use hooks for core logic (use skill execution)
3. Don't forget to make scripts executable
4. Don't assume environment variables exist
5. Don't create circular dependencies with hooks

## Summary

This skill demonstrates:
- ✅ PreToolUse hooks for tool preparation
- ✅ PostToolUse hooks for tool completion
- ✅ Stop hooks for cleanup
- ✅ Pattern matching with specific tools
- ✅ Wildcard matching for all tools
- ✅ Once-only execution
- ✅ Multiple hooks per event

Try the tests above to see hooks in action!

---

**Version**: 1.0.0
**Last Updated**: 2026-01-10
**Maintainer**: SDK Team
