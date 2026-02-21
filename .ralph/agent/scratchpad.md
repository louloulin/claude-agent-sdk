# Scratchpad: PROMPT.md 扩展包重构计划

## 2026-02-21 - 完成更新

### ✅ 任务完成

已将 PROMPT.md 完全重构为**扩展包架构**设计:

#### 新架构设计
- **核心 crate**: `claude-agent-sdk` - 只包含基础 API (query/prompt/Agent/Transport/Error)
- **7 个扩展包**:
  1. `claude-agent-sdk-pool` - 连接池
  2. `claude-agent-sdk-batch` - 批量操作
  3. `claude-agent-sdk-agents` - 预构建 Agent (CodeReviewer, DataAnalyst, etc.)
  4. `claude-agent-sdk-mcp` - MCP 协议
  5. `claude-agent-sdk-observability` - 可观测性 (Prometheus, OpenTelemetry)
  6. `claude-agent-sdk-session` - 会话管理
  7. `claude-agent-sdk-cost` - 成本追踪

#### 架构优势
- 核心精简: 快速编译、小二进制
- 按需加载: 用户只引入需要的扩展
- 版本独立: 各扩展包可独立迭代

#### 文档内容
- 详细的 crate 结构和依赖关系图
- 每个扩展包的 API 设计示例
- 验证指标和测试要求
- 更新的实现时间线 (Phase 0-4)

### 提交信息
```
docs(plan): refactor to extension crate architecture
commit: 4106231
```
