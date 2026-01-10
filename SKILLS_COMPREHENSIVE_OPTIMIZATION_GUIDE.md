# 🚀 Skills 全面优化指南

**更新日期**: 2025-01-10
**基于**: Claude Code Skills 官方文档最佳实践
**目标**: 将所有 23 个技能优化为生产级别

---

## 📋 目录

1. [优化概述](#优化概述)
2. [优化检查清单](#优化检查清单)
3. [已完成优化的技能](#已完成优化的技能)
4. [待优化技能列表](#待优化技能列表)
5. [批量优化工具](#批量优化工具)
6. [手动优化步骤](#手动优化步骤)

---

## 优化概述

### 优化维度

我们基于以下 7 个维度优化每个技能：

1. **元数据质量** ⭐
   - ✅ Name: 小写，kebab-case，< 64 字符
   - ✅ Description: 包含触发词，< 1024 字符
   - ✅ Tags: 相关且精确
   - ✅ Version: 遵循语义化版本

2. **工具限制** 🔧
   - ✅ 添加 `allowed-tools` 字段
   - ✅ 根据技能需求限制工具访问
   - ✅ 使用参数限制（如 `Bash(python:*)`）

3. **渐进式披露** 📚
   - ✅ SKILL.md: 核心信息（< 300 行）
   - ✅ reference.md: 完整 API 文档
   - ✅ examples.md: 实际示例
   - ✅ forms.md: 表单映射（如适用）
   - ✅ scripts/: 实用脚本

4. **描述触发词** 🎯
   - ✅ "Use when working with..."
   - ✅ "Use when user mentions..."
   - ✅ 具体场景描述

5. **最佳实践** ✨
   - ✅ DO/DON'T 列表
   - ✅ 常见模式
   - ✅ 故障排除
   - ✅ 代码示例

6. **工具脚本** 🛠️
   - ✅ validate.py - 验证脚本
   - ✅ test.py - 测试脚本
   - ✅ helper.py - 辅助功能

7. **文档完整性** 📖
   - ✅ 快速开始
   - ✅ 核心能力
   - ✅ 框架和工具
   - ✅ 资源链接

---

## 优化检查清单

### 元数据检查

- [ ] Name 是小写且使用 kebab-case
- [ ] Description 包含明确的触发词
- [ ] Description < 1024 字符
- [ ] Name < 64 字符
- [ ] Tags 相关且精确
- [ ] Dependencies 正确声明
- [ ] Version 遵循语义化版本

### 内容检查

- [ ] 快速开始部分（3-5 行代码）
- [ ] 核心能力列表
- [ ] DO/DON'T 最佳实践
- [ ] 故障排除部分
- [ ] 指向其他文件的链接

### 结构检查

- [ ] SKILL.md < 300 行（核心内容）
- [ ] reference.md 存在（详细文档）
- [ ] examples.md 存在（使用示例）
- [ ] scripts/ 目录存在（实用脚本）
- [ ] forms.md 存在（如需要）

### 高级功能检查

- [ ] `allowed-tools` 字段存在
- [ ] 工具限制合理（不严格也不过于宽松）
- [ ] 使用 `model` 字段（如需要特定模型）
- [ ] 使用 `hooks` 字段（如需要生命周期钩子）

---

## 已完成优化的技能

### 1. pdf-processor ✅

**状态**: 生产级别

**优化内容**:
- ✅ 完整元数据（包含 allowed-tools）
- ✅ 渐进式披露（SKILL.md + reference.md + examples.md）
- ✅ 3 个实用脚本
- ✅ 触发词优化
- ✅ 最佳实践指南

**文件结构**:
```
pdf-processor/
├── SKILL.md (156 行)
├── reference.md (400+ 行)
├── examples.md (350+ 行)
└── scripts/
    ├── validate.py
    ├── extract_forms.py
    └── merge.py
```

**关键特性**:
- 工具限制: `Read`, `Bash(python:*)`, `Grep`
- 描述触发词: "Use when working with PDF documents..."
- 完整的 API 文档
- 生产级脚本

### 2. backend-developer ✅

**状态**: 已优化核心 SKILL.md

**优化内容**:
- ✅ Name: "Backend Developer" → "backend-developer"
- ✅ Description: 添加触发词
- ✅ 添加 allowed-tools 字段
- ✅ 快速开始部分
- ✅ DO/DON'T 最佳实践
- ✅ 故障排除指南

**待完成**:
- [ ] 创建 reference.md
- [ ] 创建 examples.md
- [ ] 创建 scripts/

---

## 待优化技能列表

### 高优先级（> 500 行）

这些技能最需要渐进式披露：

1. **devops-engineer** (1,528 行) 🔴
   - 需要拆分为 SKILL.md + reference.md + examples.md
   - 添加 allowed-tools
   - 创建实用脚本

2. **mobile-developer** (1,445 行) 🔴
   - 需要拆分
   - 优化描述（添加触发词）
   - 添加工具限制

3. **frontend-developer** (959 行) 🔴
   - 需要拆分
   - 优化描述
   - 添加工具限制

4. **data-engineering** (927 行) 🔴
   - 需要拆分
   - 优化描述
   - 创建脚本

5. **cloud-infrastructure** (794 行) 🔴
   - 需要拆分
   - 优化描述
   - 添加工具限制

6. **content-marketing-specialist** (781 行) 🔴
   - 需要拆分
   - 优化描述
   - 添加工具限制

7. **technical-writer** (727 行) 🔴
   - 需要拆分
   - 优化描述
   - 添加工具限制

8. **performance-optimizer** (612 行) 🔴
   - 需要拆分
   - 优化描述
   - 创建性能分析脚本

9. **seo-specialist** (593 行) 🔴
   - 需要拆分
   - 优化描述
   - 添加工具限制

10. **machine-learning-engineer** (561 行) 🔴
    - 需要拆分
    - 优化描述
    - 创建模型训练脚本

### 中优先级（200-500 行）

11. **logging-monitoring** (555 行) 🟡
12. **security-auditor** (451 行) 🟡
13. **docker-helper** (374 行) 🟡
14. **deployment-automation** (317 行) 🟡
15. **data-analyst** (279 行) 🟡
16. **git-workflow** (272 行) 🟡
17. **skill-validator** (213 行) 🟡

### 低优先级（< 200 行）

18. **database-migrator** (136 行) 🟢
19. **api-tester** (108 行) 🟢
20. **code-reviewer** (77 行) 🟢
21. **example-calculator** (34 行) 🟢

---

## 批量优化工具

### 1. 分析工具

```bash
# 分析所有技能
python scripts/optimize_skills.py

# 输出:
# - SKILLS_OPTIMIZATION_REPORT.md
# - SKILLS_OPTIMIZATION_PRIORITY.md
```

### 2. 批量优化工具

```bash
# 预览更改（推荐先运行）
python scripts/batch_optimize_skills.py --dry-run

# 实际优化
python scripts/batch_optimize_skills.py

# 输出:
# - 自动优化所有技能的元数据
# - 创建 .md.backup 备份文件
# - 添加 allowed-tools 字段
# - 优化 name 为小写
# - 添加触发词到描述
```

### 3. 自定义目录

```bash
python scripts/batch_optimize_skills.py --skills-dir /path/to/skills
```

---

## 手动优化步骤

### 步骤 1: 优化元数据

```yaml
---
# 之前
name: "Backend Developer"
description: "后端开发专家，精通API设计、微服务架构、数据库设计和后端系统性能优化"

# 之后
name: backend-developer
description: Backend development expert specializing in API design, microservices, database architecture, and system performance. Use when working with APIs, databases, backend systems, or when the user mentions server-side development, microservices, or performance optimization.
version: "2.0.0"
allowed_tools:
  - Read
  - Write
  - Edit
  - Bash
  - Grep
---
```

### 步骤 2: 缩减 SKILL.md

将详细内容移到 reference.md 和 examples.md：

**SKILL.md** (目标: < 300 行):
- 快速开始（3-5 行代码）
- 核心能力列表
- DO/DON'T 最佳实践
- 指向其他文件的链接

**reference.md**:
- 完整 API 文档
- 所有类和方法
- 参数说明
- 高级模式

**examples.md**:
- 实际代码示例
- 真实场景
- 常见用例

### 步骤 3: 添加工具脚本

创建 `scripts/` 目录并添加实用脚本：

```bash
scripts/
├── validate.py      # 验证脚本
├── test.py          # 测试脚本
└── helper.py        # 辅助功能
```

示例脚本：

```python
#!/usr/bin/env python3
"""Validate skill configuration."""

import sys
import json

def validate_skill(skill_path):
    """Validate SKILL.md metadata."""
    # Implementation here
    pass

if __name__ == '__main__':
    validate_skill(sys.argv[1])
```

### 步骤 4: 添加最佳实践

在 SKILL.md 中添加 DO/DON'T 部分：

```markdown
## Best Practices

### DO (Recommended)

1. **API Design**
   - Use appropriate HTTP methods
   - Implement proper status codes
   - Version your APIs from the start

2. **Database**
   - Use transactions for multi-step operations
   - Create indexes on frequently queried columns

### DON'T (Avoid)

1. **API Design**
   - ❌ Return nested data inefficiently
   - ❌ Ignore HTTP status codes

2. **Database**
   - ❌ N+1 query problems
   - ❌ Unbounded queries without pagination
```

---

## 优化时间估算

### 自动化优化（使用批量工具）

- **元数据优化**: ~5 分钟（所有技能）
- **描述优化**: ~5 分钟（所有技能）
- **工具限制**: ~5 分钟（所有技能）

**总计**: ~15 分钟

### 手动优化（每个技能）

- **小型技能** (< 200 行): ~15 分钟
  - 优化元数据: 5 分钟
  - 添加最佳实践: 5 分钟
  - 创建脚本: 5 分钟

- **中型技能** (200-500 行): ~30 分钟
  - 优化元数据: 5 分钟
  - 添加最佳实践: 10 分钟
  - 拆分内容: 10 分钟
  - 创建脚本: 5 分钟

- **大型技能** (> 500 行): ~60 分钟
  - 优化元数据: 5 分钟
  - 添加最佳实践: 10 分钟
  - 拆分内容: 30 分钟
  - 创建 reference.md: 10 分钟
  - 创建 examples.md: 10 分钟
  - 创建脚本: 5 分钟

### 总时间估算

- **自动化优化**: 15 分钟
- **小型技能** (4 个): 1 小时
- **中型技能** (7 个): 3.5 小时
- **大型技能** (10 个): 10 小时

**总计**: ~15 小时（或 2 个工作日）

---

## 优先级建议

### 第 1 天：自动化 + 高价值技能

1. 运行批量优化工具（15 分钟）
2. 手动优化 top 5 技能（5 小时）
   - devops-engineer
   - mobile-developer
   - frontend-developer
   - backend-developer ✅（已完成）
   - data-engineering

### 第 2 天：中等优先级

3. 优化中优先级技能（5 小时）
   - cloud-infrastructure
   - content-marketing-specialist
   - technical-writer
   - performance-optimizer
   - seo-specialist

### 第 3 天（可选）：完善

4. 优化剩余技能（3 小时）
   - machine-learning-engineer
   - logging-monitoring
   - security-auditor
   - 其他

---

## 验证标准

优化后的技能应满足：

✅ **元数据标准**
- Name < 64 字符，小写，kebab-case
- Description < 1024 字符，包含触发词
- Version: "2.0.0" 或更高
- 包含 allowed-tools

✅ **内容标准**
- SKILL.md < 300 行
- 快速开始部分存在
- DO/DON'T 列表存在
- 故障排除部分存在

✅ **结构标准**
- reference.md 存在（如 SKILL.md > 200 行）
- examples.md 存在（如 SKILL.md > 200 行）
- scripts/ 目录存在（至少 1 个脚本）

✅ **质量标准**
- 所有代码示例可运行
- 所有链接有效
- 无拼写错误
- 格式一致

---

## 常见问题

### Q1: 为什么要拆分 SKILL.md？

**A**: 渐进式披露可以节省上下文窗口。SKILL.md 始终加载，应该保持精简。详细文档按需加载。

### Q2: allowed-tools 应该包含哪些工具？

**A**: 取决于技能需求：
- **只读技能**: Read, Grep
- **开发技能**: Read, Write, Edit, Bash, Grep
- **特定语言**: "Bash(python:*)", "Bash(node:*)"

### Q3: 描述应该多长？

**A**: 100-300 字符最佳，绝对不能超过 1024 字符。必须包含触发词。

### Q4: 什么时候需要创建 reference.md？

**A**: 当 SKILL.md 超过 200 行时，或者有大量 API 文档时。

### Q5: 脚本应该是可执行的吗？

**A**: 是的，脚本应该有执行权限：
```bash
chmod +x scripts/*.py
```

---

## 资源链接

- [Claude Code Skills 官方文档](https://code.claude.com/docs/en/skills)
- [Anthropic's Official Skills Repository](https://github.com/anthropics/skills)
- [Claude Agent SDK 文档](https://github.com/louloulin/claude-agent-sdk)
- [优化完成报告](./CLAUDE_CODE_SKILLS_OPTIMIZATION_COMPLETE.md)

---

**维护者**: Claude Agent SDK Team
**最后更新**: 2025-01-10
**许可证**: MIT
