# SKILL.md 文件系统支持 - 用户指南

**版本**: 1.0.0
**日期**: 2025-01-10
**状态**: ✅ 完全实现

---

## 📋 目录

1. [简介](#简介)
2. [快速开始](#快速开始)
3. [SKILL.md 格式说明](#skillmd-格式说明)
4. [目录结构](#目录结构)
5. [API 参考](#api-参考)
6. [集成到 ClaudeAgentOptions](#集成到-claudeagentoptions)
7. [示例](#示例)
8. [最佳实践](#最佳实践)
9. [故障排除](#故障排除)

---

## 简介

SKILL.md 文件系统支持为 Claude Agent SDK 提供了与 Claude Code 完全兼容的技能定义方式。技能以人类可读的 Markdown 文件存储，支持 YAML frontmatter 元数据和资源自动发现。

### 主要特性

✅ **YAML Frontmatter** - 结构化的元数据定义
✅ **Markdown 内容** - 人类可读的技能说明
✅ **资源自动发现** - 自动加载 scripts/ 和 resources/ 目录
✅ **多级别配置** - 支持 Project/User/Local 配置层级
✅ **完整兼容性** - 与 Claude Code 100% 兼容
✅ **类型安全** - Rust 的完整类型检查

---

## 快速开始

### 1. 创建 SKILL.md 文件

在项目的 `.claude/skills/my-skill/` 目录中创建 `SKILL.md`:

```markdown
---
name: "My Skill"
description: "A brief description of what this skill does"
version: "1.0.0"
author: "Your Name"
tags:
  - example
  - utility
dependencies: []
---

# My Skill

You are a helpful assistant that does X, Y, and Z.

## Instructions

Detailed instructions for using this skill...
```

### 2. 自动发现技能

```rust
use claude_agent_sdk_rs::skills::SkillRegistry;

// 从目录自动发现所有 SKILL.md 文件
let packages = SkillRegistry::discover_skill_md_from_dir(".claude/skills")?;

for package in packages {
    println!("Loaded skill: {}", package.metadata.name);
}
```

### 3. 启用自动发现（配置）

```rust
use claude_agent_sdk_rs::{ClaudeAgentOptions, ClaudeClient};

let options = ClaudeAgentOptions::builder()
    .auto_discover_skills(true)  // 启用自动发现
    .build();

let client = ClaudeClient::new(options);
```

---

## SKILL.md 格式说明

### 文件结构

SKILL.md 文件由两部分组成：

1. **YAML Frontmatter** - 元数据（由 `---` 分隔）
2. **Markdown 内容** - 技能指令和说明

### 完整示例

```markdown
---
name: "Calculator"
description: "Performs mathematical calculations"
version: "1.0.0"
author: "Math Team <math@example.com>"
tags:
  - math
  - calculator
  - utility
dependencies:
  - logger
---

# Calculator Skill

You are a calculator assistant. Evaluate mathematical expressions
and provide step-by-step explanations.

## Supported Operations

- Addition (+)
- Subtraction (-)
- Multiplication (*)
- Division (/)

## Usage

Simply provide a mathematical expression, and I'll calculate the result.
```

### 元数据字段

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `name` | `String` | ✅ | 技能名称 |
| `description` | `String` | ✅ | 技能描述 |
| `version` | `String` | ❌ | 版本号（默认 "1.0.0"） |
| `author` | `String` | ❌ | 作者信息 |
| `tags` | `Vec<String>` | ❌ | 标签列表 |
| `dependencies` | `Vec<String>` | ❌ | 依赖的其他技能 |

### 内容格式

SKILL.md 的内容部分使用标准 Markdown 格式：

- **标题** (`#`, `##`) - 组织内容结构
- **列表** (`-`, `*`) - 列出功能和说明
- **代码块** (```) - 提供示例代码
- **链接** (`[text](url)`) - 引用外部资源

---

## 目录结构

### 标准技能目录结构

```
.claude/skills/my-skill/
├── SKILL.md              # 主文件（必需）
├── reference.md          # 参考文档（可选）
├── forms.md              # 表单定义（可选）
├── scripts/              # 脚本目录（可选）
│   ├── setup.sh
│   └── deploy.py
└── resources/            # 资源目录（可选）
    ├── template.txt
    ├── config.json
    └── images/
        └── logo.png
```

### 配置层级

Claude Agent SDK 支持多级别技能配置，按优先级从高到低：

1. **Local** - `.claude/skills/` (项目级，最高优先级)
2. **User** - `~/.config/claude/skills/` (用户级)
3. **Project** - `.claude/skills/` (项目级，团队共享)

---

## API 参考

### SkillMdFile

解析单个 SKILL.md 文件。

```rust
use claude_agent_sdk_rs::skills::SkillMdFile;

// 解析 SKILL.md 文件
let skill = SkillMdFile::parse(".claude/skills/my-skill/SKILL.md")?;

// 访问元数据
println!("Name: {}", skill.metadata.name);
println!("Version: {}", skill.metadata.version);

// 转换为 SkillPackage
let package = skill.to_skill_package();
```

### SkillsDirScanner

扫描技能目录并发现所有 SKILL.md 文件。

```rust
use claude_agent_sdk_rs::skills::SkillsDirScanner;

// 方法 1: 使用自定义目录
let scanner = SkillsDirScanner::new("/path/to/skills");
let skills = scanner.scan()?;

// 方法 2: 从项目目录扫描 (.claude/skills/)
let scanner = SkillsDirScanner::from_project_dir("/my/project");
let skills = scanner.scan()?;

// 方法 3: 从用户目录扫描 (~/.config/claude/skills/)
let scanner = SkillsDirScanner::from_user_dir()?;
let skills = scanner.scan()?;
```

### SkillRegistry

发现和加载技能包。

```rust
use claude_agent_sdk_rs::skills::SkillRegistry;

// 方法 1: 从单个目录发现 SKILL.md
let packages = SkillRegistry::discover_skill_md_from_dir(".claude/skills")?;

// 方法 2: 从多个目录发现（支持优先级）
let packages = SkillRegistry::discover_from_multiple_dirs(vec![
    ".claude/skills",           // 项目级
    "~/.config/claude/skills",  // 用户级
])?;

// 方法 3: 传统 JSON 格式（向后兼容）
let packages = SkillRegistry::discover_from_dir(".claude/skills")?;
```

---

## 集成到 ClaudeAgentOptions

### 启用自动发现

```rust
use claude_agent_sdk_rs::{ClaudeAgentOptions, ClaudeClient};

let options = ClaudeAgentOptions::builder()
    .auto_discover_skills(true)  // 启用自动发现
    .build();

let client = ClaudeClient::new(options);
```

### 自定义技能目录

```rust
let options = ClaudeAgentOptions::builder()
    .auto_discover_skills(true)
    .project_skills_dir("/custom/project/skills")  // 自定义项目目录
    .user_skills_dir("/custom/user/skills")        // 自定义用户目录
    .build();
```

### 配置选项

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `auto_discover_skills` | `bool` | `false` | 是否自动发现技能 |
| `project_skills_dir` | `Option<PathBuf>` | `.claude/skills` | 项目技能目录 |
| `user_skills_dir` | `Option<PathBuf>` | `~/.config/claude/skills` | 用户技能目录 |

---

## 示例

### 示例 1: 简单计算器技能

**SKILL.md**:
```markdown
---
name: "Calculator"
description: "Performs basic mathematical calculations"
version: "1.0.0"
---

# Calculator

You are a calculator. Evaluate mathematical expressions like "2 + 2" or "5 * 10".
```

**使用代码**:
```rust
use claude_agent_sdk_rs::skills::SkillMdFile;

let calc = SkillMdFile::parse(".claude/skills/calculator/SKILL.md")?;
println!("Loaded: {}", calc.metadata.name);
```

### 示例 2: 带资源的翻译技能

**目录结构**:
```
.claude/skills/translator/
├── SKILL.md
├── reference.md
└── resources/
    └── dictionary.json
```

**SKILL.md**:
```markdown
---
name: "Translator"
description: "Translates text between languages"
version: "2.0.0"
tags:
  - translation
  - i18n
---

# Translator

Translate text between languages while preserving meaning and tone.
```

**reference.md**:
```markdown
# Translation Reference

Common phrases and idioms for translation.
```

### 示例 3: 批量发现技能

```rust
use claude_agent_sdk_rs::skills::SkillRegistry;

let packages = SkillRegistry::discover_from_multiple_dirs(vec![
    ".claude/skills",
    "~/.config/claude/skills",
])?;

println!("Discovered {} skills", packages.len());

for package in packages {
    println!("- {} (v{})", package.metadata.name, package.metadata.version);
}
```

---

## 最佳实践

### 1. 版本管理

使用语义化版本（Semantic Versioning）：

```yaml
version: "1.0.0"  # Major.Minor.Patch
```

- **Major** - 破坏性变更
- **Minor** - 新功能
- **Patch** - Bug 修复

### 2. 依赖声明

明确声明依赖关系：

```yaml
dependencies:
  - logger          # 基础依赖
  - data-processor  # 数据处理依赖
```

### 3. 标签使用

使用有意义的标签：

```yaml
tags:
  - category:math    # 分类
  - type:utility     # 类型
  - language:rust    # 技术栈
```

### 4. 内容组织

使用清晰的 Markdown 结构：

```markdown
# Skill Name

Brief one-line description.

## Overview

Detailed explanation...

## Features

- Feature 1
- Feature 2

## Usage

Step-by-step instructions...

## Examples

Code examples...
```

### 5. 资源管理

保持资源目录结构清晰：

```
scripts/
├── install.sh        # 安装脚本
├── setup.sh          # 配置脚本
└── utils/
    └── helper.sh     # 辅助脚本

resources/
├── config/
│   └── settings.json
├── templates/
│   └── template.txt
└── docs/
    └── guide.md
```

---

## 故障排除

### 问题 1: SKILL.md 未被加载

**症状**: 技能目录存在但技能未被加载。

**解决方案**:
1. 检查文件名是否为 `SKILL.md`（大写）
2. 确认 YAML frontmatter 以 `---` 开始和结束
3. 验证必填字段（`name`, `description`）存在

```bash
# 检查文件结构
ls -la .claude/skills/my-skill/SKILL.md

# 验证 YAML 语法
cat .claude/skills/my-skill/SKILL.md
```

### 问题 2: YAML 解析错误

**症状**: 加载时出现 `YamlError`。

**解决方案**:
- 检查 YAML 缩进（使用空格，不要使用 Tab）
- 确保列表格式正确（使用 `-` 前缀）
- 验证字符串引用（必要时使用引号）

```yaml
# ❌ 错误
tags: math, calculator

# ✅ 正确
tags:
  - math
  - calculator
```

### 问题 3: 资源未发现

**症状**: `scripts/` 或 `resources/` 目录为空。

**解决方案**:
- 确认目录在技能目录内（与 SKILL.md 同级）
- 验证目录名称拼写正确
- 检查文件权限

```bash
# 检查目录结构
.claude/skills/my-skill/
├── SKILL.md
├── scripts/         # 必须在 my-skill/ 内
└── resources/       # 必须在 my-skill/ 内
```

### 问题 4: 权限错误

**症状**: `IoError: Permission denied`

**解决方案**:
- 检查文件和目录权限
- 确保用户有读取权限
- 验证目录所有者

```bash
# 修复权限
chmod -R u+r .claude/skills/
```

### 问题 5: 编码问题

**症状**: 特殊字符显示不正确。

**解决方案**:
- 使用 UTF-8 编码保存 SKILL.md 文件
- 避免使用 BOM（Byte Order Mark）

```bash
# 检查文件编码
file -i .claude/skills/my-skill/SKILL.md

# 转换为 UTF-8
iconv -f ISO-8859-1 -t UTF-8 input.md > SKILL.md
```

---

## 高级用法

### 自定义错误处理

```rust
use claude_agent_sdk_rs::skills::{SkillMdFile, SkillMdError};

match SkillMdFile::parse("path/to/SKILL.md") {
    Ok(skill) => {
        println!("Loaded: {}", skill.metadata.name);
    },
    Err(SkillMdError::MissingField(field)) => {
        eprintln!("Missing required field: {}", field);
    },
    Err(SkillMdError::YamlError(msg)) => {
        eprintln!("YAML parsing error: {}", msg);
    },
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

### 批量处理

```rust
use claude_agent_sdk_rs::skills::SkillRegistry;

let packages = SkillRegistry::discover_skill_md_from_dir(".claude/skills")?;

// 过滤特定标签
let math_skills: Vec<_> = packages
    .into_iter()
    .filter(|p| p.metadata.tags.iter().any(|t| t.contains("math")))
    .collect();

println!("Found {} math skills", math_skills.len());
```

---

## 总结

SKILL.md 文件系统支持为 Claude Agent SDK 提供了强大而灵活的技能定义方式：

✅ **简单易用** - 人类可读的 Markdown 格式
✅ **功能完整** - YAML frontmatter + 资源自动发现
✅ **类型安全** - Rust 的完整类型检查
✅ **完全兼容** - 与 Claude Code 100% 兼容
✅ **生产就绪** - 完整的错误处理和测试覆盖

开始使用 SKILL.md，让您的技能定义更加清晰和易于维护！

---

**文档版本**: 1.0.0
**最后更新**: 2025-01-10
**维护者**: Claude Agent SDK Team
