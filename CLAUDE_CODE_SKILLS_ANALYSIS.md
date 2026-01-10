# 🔬 Claude Code Skills 官方文档深度分析

**分析日期**: 2025-01-10
**文档来源**: https://code.claude.com/docs/en/skills
**目标**: 基于官方文档优化现有 Skills 实现

---

## 📋 官方文档核心要点

### 1. Skills 的本质

```
核心定义:
  Skills 是模型调用的
  Claude 根据请求自动决定使用哪个 Skill
  不需要显式调用，自动匹配和应用

触发方式:
  1. 手动调用: 用户输入 /skill-name
  2. 编程调用: Claude 调用 Skill tool
  3. 自动发现: Claude 根据描述自动匹配
```

### 2. 文件结构规范

#### 必需文件

```
SKILL.md (必须)
├── YAML frontmatter (--- 包裹)
├── Markdown 内容
└── 大小写敏感: SKILL.md (全大写)
```

#### 可选文件

```
skill-directory/
├── SKILL.md          # 必需，概览和导航
├── reference.md      # 详细文档，按需加载
├── examples.md       # 使用示例，按需加载
├── forms.md          # 表单字段映射
└── scripts/          # 工具脚本，执行但不加载
    ├── helper.py    # 实用脚本
    └── validate.py  # 验证脚本
```

### 3. 完整的元数据字段

#### 基础字段 (必需)

```yaml
---
name: your-skill-name                    # 必需，小写字母、数字、连字符，最大64字符
description: Brief description          # 必需，最大1024字符
                                           # Claude 用此决定何时应用 Skill
---
```

#### 高级字段 (可选)

```yaml
---
allowed-tools:                            # 工具限制，逗号分隔或YAML列表
  - Read
  - Grep
  - Bash(python:*)                       # 支持参数

model: claude-sonnet-4-20250514         # 指定模型

context: fork                            # 在子代理上下文中运行

agent: general-purpose                   # 配合 context: fork 使用

hooks:                                   # 生命周期钩子
  PreToolUse:
    - matcher: "Bash"
      hooks:
        - type: command
          command: "./scripts/security-check.sh $TOOL_INPUT"
          once: true                      # 只运行一次

user-invocable: false                    # 控制是否在 / 菜单显示
                                           # 不影响 Skill tool 或自动发现
disable-model-invocation: true          # 阻止通过 Skill tool 调用
---
```

### 4. 元数据字段详解

| 字段 | 必需 | 描述 | 验证规则 |
|------|------|------|----------|
| `name` | ✅ | 技能名称 | 小写、数字、连字符，最大64字符，必须匹配目录名 |
| `description` | ✅ | 技能描述和用途 | 最大1024字符，Claude用此决定何时使用 |
| `allowed-tools` | ❌ | 允许的工具列表 | 逗号分隔或YAML列表，工具需要权限时才询问 |
| `model` | ❌ | 使用的模型 | 如 claude-sonnet-4-20250514，默认使用会话模型 |
| `context` | ❌ | 上下文模式 | 设置为 fork 在隔离子代理中运行 |
| `agent` | ❌ | 代理类型 | 配合 context: fork，如 Explore, Plan, general-purpose |
| `hooks` | ❌ | 生命周期钩子 | PreToolUse, PostToolUse, Stop 事件 |
| `user-invocable` | ❌ | 斜杠菜单可见性 | 默认 true，false 时隐藏但仍可自动发现 |
| `disable-model-invocation` | ❌ | 禁止模型调用 | 阻止 Skill tool 调用，但仍可自动发现 |

### 5. 技能优先级规则

```
优先级顺序 (从高到低):
  1. Enterprise (企业级) - �盖所有
  2. Personal (个人)   - 覆盖项目和插件
  3. Project (项目)   - 覆盖插件
  4. Plugin (插件)    - 基础级别

同优先级时:
  - 名称相同时，高优先级覆盖低优先级
```

### 6. 渐进式披露原则

```
核心原则:
  SKILL.md: 必备信息，始终加载
  其他文件: 详细信息，按需加载

好处:
  - 节省上下文窗口
  - 聚焦核心功能
  - 丰富文档但不消耗token

实现方式:
  在 SKILL.md 中链接到其他文件:
  - See [reference.md](reference.md) for details
  - See [examples.md](examples.md) for usage
  - Run: python scripts/validate.py
```

### 7. 描述字段最佳实践

#### 好的描述

```yaml
# ✅ 具体、可操作、有触发词
description: Extract text and tables from PDF files, fill forms, merge documents. Use when working with PDF files or when the user mentions PDFs, forms, or document extraction.
```

**回答两个问题**:
1. 这个 Skill 做什么？
2. Claude 应该何时使用它？

#### 差的描述

```yaml
# ❌ 模糊、无触发词
description: Helps with documents
```

### 8. 技能与子代理的关系

#### 两种结合方式

**方式1: 给子代理分配 Skills**

```markdown
# .claude/agents/code-reviewer.md
---
name: code-reviewer
description: Review code for quality and best practices
skills: pr-review, security-check    # 列出技能名称
---
```

- 子代理不继承主会话的 Skills
- 必须明确指定 `skills` 字段
- 完整内容注入到子代理

**方式2: 在子代理上下文中运行 Skill**

```yaml
---
name: code-analysis
context: fork                      # 在 forked 上下文中运行
agent: general-purpose            # 指定代理类型
---
```

- Skill 在隔离上下文中运行
- 有独立的对话历史
- 不影响主会话

---

## 🎯 与现有实现的对比分析

### 我们的实现状态

#### ✅ 已实现的功能

1. **基础结构**
   - ✅ YAML frontmatter 解析
   - ✅ Markdown 内容分离
   - ✅ 必需字段
   - ✅ 可选字段支持

2. **文件发现**
   - ✅ SKILL.md 解析
   - ✅ 目录扫描
   - ✅ 资源发现
   - ✅ 多文件支持

3. **转换机制**
   - ✅ SkillPackage 转换
   - ✅ 元数据映射
   - ✅ SDK 集成

#### ❌ 缺失的高级功能

1. **高级元数据**
   - ❌ `allowed-tools` 支持
   - ❌ `model` 字段支持
   - ❌ `context: fork` 支持
   - ❌ `agent` 字段支持
   - ❌ `hooks` 支持
   - ❌ `user-invocable` 支持
   - ❌ `disable-model-invocation` 支持

2. **子代理集成**
   - ❌ Skills 字段支持
   - ❌ Subagent 技能加载

3. **工具限制**
   - ❌ 工具白名单机制
   - ❌ Bash 参数支持

---

## 🚀 优化改造方案

### 方案1: 扩展元数据结构

```rust
// src/skills/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMdMetadata {
    // 现有字段
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,

    // 新增字段
    #[serde(default)]
    pub allowed_tools: Option<Vec<String>>,

    #[serde(default)]
    pub model: Option<String>,

    #[serde(default)]
    pub context: Option<SkillContext>,

    #[serde(default)]
    pub agent: Option<String>,

    #[serde(default)]
    pub hooks: Option<SkillHooks>,

    #[serde(default = "default_user_invocable")]
    pub user_invocable: bool,

    #[serde(default)]
    pub disable_model_invocation: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillContext {
    Fork,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillHooks {
    pub pre_tool_use: Option<Vec<HookConfig>>,
    pub post_tool_use: Option<Vec<HookConfig>>,
    pub stop: Option<Vec<HookConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub r#type: HookType,
    pub matcher: String,
    pub command: String,
    pub once: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HookType {
    Command,
    Script,
    Function,
}

fn default_user_invocable() -> bool {
    true
}
```

### 方案2: 实现渐进式披露

```rust
// src/skills/progressive_disclosure.rs
use std::path::PathBuf;
use std::collections::HashMap;

pub struct ProgressiveSkillLoader {
    skill_dir: PathBuf,
    main_content: String,
    referenced_files: HashMap<String, PathBuf>,
}

impl ProgressiveSkillLoader {
    pub fn load(skill_dir: PathBuf) -> Result<Self, SkillError> {
        // 1. 始终加载 SKILL.md
        let skill_md = skill_dir.join("SKILL.md");
        let content = std::fs::read_to_string(&skill_md)?;

        // 2. 扫描引用的其他文件
        let references = Self::scan_references(&content, &skill_dir);

        Ok(Self {
            skill_dir,
            main_content: content,
            referenced_files: references,
        })
    }

    pub fn get_main_content(&self) -> &str {
        &self.main_content
    }

    pub fn load_reference(&self, filename: &str) -> Result<String, SkillError> {
        if let Some(path) = self.referenced_files.get(filename) {
            std::fs::read_to_string(path)
                .map_err(|e| SkillError::FileNotFound(path.clone(), e.to_string()))
        } else {
            Err(SkillError::ReferenceNotFound(filename.to_string()))
        }
    }

    fn scan_references(content: &str, base_dir: &PathBuf) -> HashMap<String, PathBuf> {
        let mut refs = HashMap::new();

        // Markdown 链接格式
        let link_pattern = regex::Regex::new(r"\[(?P<title>[^\]]+)\]\((?P<file>[^)]+\.md\)").unwrap();

        for cap in link_pattern.captures_iter(content) {
            let file = cap.name("file").unwrap().as_str();
            let full_path = base_dir.join(file);

            if full_path.exists() {
                refs.insert(file.to_string(), full_path);
            }
        }

        refs
    }
}
```

### 方案3: 实现工具限制

```rust
// src/skills/tool_restriction.rs
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ToolRestriction {
    pub allowed_tools: Option<HashSet<String>>,
}

impl ToolRestriction {
    pub fn new(allowed_tools: Option<Vec<String>>) -> Self {
        Self {
            allowed_tools: allowed_tools.map(|tools| {
                tools.into_iter().collect()
            }),
        }
    }

    pub fn is_tool_allowed(&self, tool_name: &str) -> bool {
        match &self.allowed_tools {
            Some(allowed) => {
                // 支持带参数的工具
                if tool_name.contains('(') {
                    let base_tool = tool_name.split('(').next().unwrap_or(tool_name);
                    allowed.contains(&base_tool.to_string())
                } else {
                    allowed.contains(&tool_name.to_string())
                }
            }
            None => true, // 没有限制，允许所有工具
        }
    }

    pub fn parse_tool_spec(tool_spec: &str) -> (String, Option<String>) {
        // 解析工具规范: "Bash(python:*)" -> ("Bash", Some("python:*"))
        if let Some(params) = tool_spec.strip_suffix(')') {
            if let Some((base, args)) = params.split_once('(') {
                return (base.to_string(), Some(args.to_string()));
            }
        }
        (tool_spec.to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_restriction() {
        let restriction = ToolRestriction::new(Some(vec![
            "Read".to_string(),
            "Grep".to_string(),
            "Bash(python:*)".to_string(),
        ]));

        assert!(restriction.is_tool_allowed("Read"));
        assert!(restriction.is_tool_allowed("Grep"));
        assert!(restriction.is_tool_allowed("Bash(python:script.py)"));
        assert!(!restriction.is_tool_allowed("Write"));
    }
}
```

### 方案4: 优化现有技能示例

基于官方文档，我们需要优化现有技能的元数据：

#### 优化模板

```yaml
---
name: skill-name                    # 小写，最大64字符
description: Specific description.   # 具体，触发词明确
                                   # 说明做什么 + 何时使用
version: "1.0.0"
author: "Team <email>"
tags:
  - tag1
  - tag2

# 高级配置
allowed-tools:                      # 可选：工具限制
  - Read
  - Bash(python:*)

model: claude-sonnet-4-20250514  # 可选：指定模型

context: fork                       # 可选：隔离上下文
agent: general-purpose             # 可选：代理类型

hooks:                             # 可选：生命周期钩子
  PreToolUse:
    - matcher: "Bash"
      command: "./scripts/check.sh"
      once: true

user-invocable: true               # 可选：菜单可见性
disable-model-invocation: false   # 可选：程序调用控制
---
```

---

## 📝 优化后的技能示例

### 示例1: PDF Processing Skill

```yaml
---
name: pdf-processing
description: Extract text, fill forms, and merge PDF files. Use when working with PDF documents, forms, or when users mention PDF processing.
version: "2.0.0"
author: "Doc Team <docs@example.com>"
tags:
  - pdf
  - documents
  - forms
dependencies: []

allowed-tools:
  - Read
  - Bash(python:*)
  - Grep
---
```

**SKILL.md**:

```markdown
# PDF Processing

## Quick Start

Extract text from PDF:
\`\`\`python
import pdfplumber
with pdfplumber.open("doc.pdf") as pdf:
    page = pdf.pages[0]
    text = page.extract_text()
    print(text)
\`\`\`

## Additional Resources

### Form Filling
For detailed form field mappings and instructions, see [forms.md](forms.md).

### API Reference
For complete API documentation, see [reference.md](reference.md).

## Utility Scripts

To validate PDF files, run:
```bash
python scripts/validate.py document.pdf
```

To extract form data:
```bash
python scripts/extract_forms.py document.pdf
```

## Requirements

Ensure required packages are installed:
```bash
pip install pypdf pdfplumber
```

## Troubleshooting

### Common Issues

**Problem**: Script not found
**Solution**: Ensure scripts have execute permissions: `chmod +x scripts/*.py`

**Problem**: Package not installed
**Solution**: Run pip install with required packages

**Problem**: PDF is encrypted
**Solution**: Unlock the PDF first or provide the password
```

### 示例2: Code Review Skill

```yaml
---
name: pr-review
description: Review pull requests for code quality, security vulnerabilities, and adherence to team standards. Use when reviewing PRs, when users ask to review changes, or mention code review.
version: "1.5.0"
author: "Code Review Team <review@example.com>"
tags:
  - code-review
  - pull-request
  - quality
  - security
dependencies:
  - security-auditor

allowed-tools:
  - Read
  - Grep
  - Bash

context: fork                       # 在隔离上下文中运行
agent: general-purpose

hooks:
  PreToolUse:
    - matcher: "Bash"
      command: "./scripts/pre-review-check.sh"
      once: true
---
```

---

## ✅ 实施检查清单

### 代码改进

- [ ] 扩展 SkillMdMetadata 结构
- [ ] 添加 allowed-tools 支持
- [ ] 添加 model 字段支持
- [ ] 添加 context/agent 支持
- [ ] 添加 hooks 支持
- [ ] 添加 user-invocable 支持
- [ ] 实现渐进式披露加载器
- [ ] 实现工具限制检查器
- [ ] 添加子代理技能集成

### 文档优化

- [ ] 更新所有现有技能的元数据
- [ ] 添加渐进式披露示例
- [ ] 添加工具限制示例
- [ ] 添加钩子示例
- [ ] 优化描述字段
- [ ] 添加官方文档链接

### 测试验证

- [ ] 单元测试：元数据解析
- [ ] 单元测试：工具限制
- [ ] 单元测试：渐进式加载
- [ ] 集成测试：完整流程
- [ ] 端到端测试：实际使用场景

---

**分析完成**: 2025-01-10
**下一步**: 基于此分析实施改造
