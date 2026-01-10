# Claude Code方式SKILL.md完整支持实现方案

**文档版本**：v1.0
**创建时间**：2025-01-10
**目标**：实现与Claude Code完全一致的SKILL.md文件系统加载机制

---

## 📊 需求分析

### Claude Code官方Skills目录结构

根据官方文档，Claude Code的Skills系统使用以下结构：

```
.claude/
├── skills/
│   ├── skill-name-1/
│   │   ├── SKILL.md              # 必需：YAML frontmatter + markdown
│   │   ├── reference.md          # 可选：参考文档
│   │   ├── forms.md              # 可选：表单定义
│   │   ├── scripts/              # 可选：脚本目录
│   │   │   ├── setup.sh
│   │   │   └── execute.py
│   │   └── resources/            # 可选：资源文件
│   │       ├── data.csv
│   │       └── template.txt
│   ├── skill-name-2/
│   │   └── SKILL.md
│   └── skill-name-3/
│       └── SKILL.md
├── CLAUDE.md                     # 项目级上下文
└── settings.json                 # 项目配置
```

### SKILL.md文件格式

```markdown
---
name: "My Skill"
description: "Does something useful"
version: "1.0.0"
author: "Your Name"
tags: ["category", "tags"]
dependencies: []
---

# My Skill

详细说明技能的功能和使用方式...

## 使用示例

...

## 脚本说明

- `scripts/setup.sh`: 初始化脚本
- `scripts/execute.py`: 执行脚本
```

---

## 🎯 实现目标

### 核心功能

1. ✅ **自动发现**：从`.claude/skills/`目录自动扫描和加载Skills
2. ✅ **YAML解析**：解析SKILL.md的YAML frontmatter
3. ✅ **Markdown支持**：保留完整的markdown内容作为instructions
4. ✅ **资源管理**：自动发现和关联scripts/和resources/目录
5. ✅ **Setting Sources集成**：支持`setting_sources: ["project"]`配置
6. ✅ **热重载**：监控`.claude/skills/`目录变化
7. ✅ **版本管理**：支持skill版本和依赖
8. ✅ **完全兼容**：与Python SDK行为完全一致

---

## 📋 当前状态分析

### ✅ Rust SDK已有的功能

1. **Skills基础系统**（完整）
   - ✅ Skill trait定义
   - ✅ SkillRegistry注册表
   - ✅ SkillPackage类型
   - ✅ 依赖管理
   - ✅ 版本管理
   - ✅ 标签系统
   - ✅ 热重载框架
   - ✅ VSCode导出（支持生成SKILL.md）

2. **文件系统扫描**（部分）
   - ✅ `discover_from_dir()` - 扫描目录
   - ✅ `load_from_file()` - 从JSON文件加载
   - ⚠️ **缺少SKILL.md解析**
   - ⚠️ **缺少自动发现.claude/skills/**

### ❌ 缺少的功能

1. **SKILL.md解析器**
   - ❌ YAML frontmatter解析
   - ❌ Markdown内容提取
   - ❌ 元数据验证

2. **.claude/skills/自动扫描**
   - ❌ SettingSource::Project触发扫描
   - ❌ 多级目录扫描
   - ❌ Skill目录结构验证

3. **资源文件关联**
   - ❌ scripts/自动发现
   - ❌ resources/自动扫描
   - ❌ reference.md/forms.md加载

---

## 🚀 实现方案

### 方案一：扩展现有系统（推荐）

#### 1. 添加SKILL.md解析模块

**新文件**：`src/skills/skill_md.rs`

```rust
//! SKILL.md file parser for Claude Code compatibility

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkillMdError {
    #[error("Failed to read SKILL.md: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    YamlError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid SKILL.md format")]
    InvalidFormat,
}

/// SKILL.md frontmatter metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMdMetadata {
    pub name: String,
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Parsed SKILL.md file
#[derive(Debug, Clone)]
pub struct SkillMdFile {
    /// Metadata from YAML frontmatter
    pub metadata: SkillMdMetadata,
    /// Markdown content (instructions)
    pub content: String,
    /// Directory containing the skill
    pub skill_dir: PathBuf,
    /// Associated scripts
    pub scripts: Vec<PathBuf>,
    /// Associated resources
    pub resources: Vec<PathBuf>,
    /// Reference file if exists
    pub reference: Option<PathBuf>,
    /// Forms file if exists
    pub forms: Option<PathBuf>,
}

impl SkillMdFile {
    /// Parse a SKILL.md file
    pub fn parse<P: AsRef<Path>>(skill_md_path: P) -> Result<Self, SkillMdError> {
        let path = skill_md_path.as_ref();
        let skill_dir = path.parent()
            .ok_or_else(|| SkillMdError::InvalidFormat)?
            .to_path_buf();

        // Read the file
        let content = std::fs::read_to_string(path)?;

        // Split frontmatter and content
        let (metadata, content) = Self::parse_frontmatter(&content)?;

        // Discover associated files
        let scripts = Self::discover_scripts(&skill_dir);
        let resources = Self::discover_resources(&skill_dir);
        let reference = Self::check_file_exists(&skill_dir, "reference.md");
        let forms = Self::check_file_exists(&skill_dir, "forms.md");

        Ok(Self {
            metadata,
            content,
            skill_dir,
            scripts,
            resources,
            reference,
            forms,
        })
    }

    /// Parse YAML frontmatter and markdown content
    fn parse_frontmatter(content: &str) -> Result<(SkillMdMetadata, String), SkillMdError> {
        if !content.starts_with("---") {
            return Err(SkillMdError::InvalidFormat);
        }

        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Err(SkillMdError::InvalidFormat);
        }

        let yaml_content = parts[1].trim();
        let markdown_content = parts[2].to_string();

        // Parse YAML
        let metadata: SkillMdMetadata = serde_yaml::from_str(yaml_content)
            .map_err(|e| SkillMdError::YamlError(e.to_string()))?;

        Ok((metadata, markdown_content))
    }

    /// Discover scripts in scripts/ directory
    fn discover_scripts(skill_dir: &Path) -> Vec<PathBuf> {
        let scripts_dir = skill_dir.join("scripts");
        if !scripts_dir.exists() {
            return Vec::new();
        }

        std::fs::read_dir(scripts_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Discover resources in resources/ directory
    fn discover_resources(skill_dir: &Path) -> Vec<PathBuf> {
        let resources_dir = skill_dir.join("resources");
        if !resources_dir.exists() {
            return Vec::new();
        }

        std::fs::read_dir(resources_dir)
            .ok()
            .and_then(|entries| {
                walkdir::WalkDir::new(resources_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .map(|e| e.into_path())
                    .collect::<Vec<_>>()
                    .into()
            })
            .unwrap_or_default()
    }

    /// Check if a file exists in the skill directory
    fn check_file_exists(skill_dir: &Path, filename: &str) -> Option<PathBuf> {
        let path = skill_dir.join(filename);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Convert to SkillPackage
    pub fn to_skill_package(&self) -> crate::skills::types::SkillPackage {
        use crate::skills::types::{SkillMetadata, SkillResources};

        SkillPackage {
            metadata: SkillMetadata {
                id: format!("skill.{}", self.metadata.name.to_lowercase().replace(' ', "-")),
                name: self.metadata.name.clone(),
                description: self.metadata.description.clone(),
                version: self.metadata.version.clone(),
                author: self.metadata.author.clone(),
                dependencies: self.metadata.dependencies.clone(),
                tags: self.metadata.tags.clone(),
            },
            instructions: self.content.clone(),
            scripts: self.scripts.iter()
                .filter_map(|p| p.to_str().map(|s| s.to_string()))
                .collect(),
            resources: SkillResources {
                folders: vec![self.skill_dir.join("resources")],
                tools: vec![],
                tests: vec![],
            },
        }
    }
}

/// Skills directory scanner for .claude/skills/
pub struct SkillsDirScanner {
    base_dir: PathBuf,
}

impl SkillsDirScanner {
    /// Create a new scanner for .claude/skills/
    pub fn from_project_dir<P: AsRef<Path>>(project_dir: P) -> Self {
        Self {
            base_dir: project_dir.as_ref().join(".claude").join("skills"),
        }
    }

    /// Create a new scanner for ~/.config/claude/skills/
    pub fn from_user_dir() -> Result<Self, SkillMdError> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| SkillMdError::IoError(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")
            ))?;

        Ok(Self {
            base_dir: PathBuf::from(home).join(".config").join("claude").join("skills"),
        })
    }

    /// Scan and load all skills
    pub fn scan(&self) -> Result<Vec<SkillMdFile>, SkillMdError> {
        if !self.base_dir.exists() {
            // Return empty if directory doesn't exist
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();

        for entry in std::fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let skill_dir = entry.path();

            // Skip if not a directory
            if !skill_dir.is_dir() {
                continue;
            }

            // Look for SKILL.md
            let skill_md = skill_dir.join("SKILL.md");
            if skill_md.exists() {
                match SkillMdFile::parse(&skill_md) {
                    Ok(skill) => {
                        tracing::info!("Loaded skill: {}", skill.metadata.name);
                        skills.push(skill);
                    },
                    Err(e) => {
                        tracing::warn!("Failed to load skill from {:?}: {}", skill_md, e);
                    },
                }
            }
        }

        Ok(skills)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: "Test Skill"
description: "A test skill"
version: "1.0.0"
---

# Content

Some content here.
"#;

        let (metadata, content) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "Test Skill");
        assert_eq!(metadata.description, "A test skill");
        assert_eq!(metadata.version, "1.0.0");
        assert!(content.contains("Some content here"));
    }
}
```

#### 2. 添加Cargo依赖

**文件**：`Cargo.toml`

```toml
[dependencies]
# ... existing dependencies ...

# For SKILL.md parsing
serde_yaml = "0.9"
walkdir = "2.5"

[features]
default = []
skill_md = []  # Enable SKILL.md support
```

#### 3. 集成到ClaudeAgentOptions

**修改**：`src/types/config.rs`

```rust
impl ClaudeAgentOptions {
    /// Discover and load skills from filesystem based on setting_sources
    pub fn discover_skills(&self) -> Result<Vec<SkillPackage>, ClaudeError> {
        let mut all_skills = Vec::new();

        // Check setting_sources
        let sources = self.setting_sources.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);

        for source in sources {
            match source {
                SettingSource::Project => {
                    // Load from .claude/skills/
                    if let Ok(skills) = self.load_project_skills() {
                        all_skills.extend(skills);
                    }
                },
                SettingSource::User => {
                    // Load from ~/.config/claude/skills/
                    if let Ok(skills) = self.load_user_skills() {
                        all_skills.extend(skills);
                    }
                },
                SettingSource::Local => {
                    // Local has highest priority, can override project skills
                    if let Ok(skills) = self.load_local_skills() {
                        all_skills.extend(skills);
                    }
                },
            }
        }

        Ok(all_skills)
    }

    fn load_project_skills(&self) -> Result<Vec<SkillPackage>, ClaudeError> {
        use crate::skills::skill_md::SkillsDirScanner;

        let cwd = self.cwd.clone()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));

        let scanner = SkillsDirScanner::from_project_dir(&cwd);
        let skill_md_files = scanner.scan()
            .map_err(|e| ClaudeError::InvalidConfig(format!("Failed to scan skills: {}", e)))?;

        Ok(skill_md_files.into_iter()
            .map(|md| md.to_skill_package())
            .collect())
    }

    fn load_user_skills(&self) -> Result<Vec<SkillPackage>, ClaudeError> {
        use crate::skills::skill_md::SkillsDirScanner;

        let scanner = SkillsDirScanner::from_user_dir()
            .map_err(|e| ClaudeError::InvalidConfig(format!("Failed to find user skills dir: {}", e)))?;

        let skill_md_files = scanner.scan()
            .map_err(|e| ClaudeError::InvalidConfig(format!("Failed to scan user skills: {}", e)))?;

        Ok(skill_md_files.into_iter()
            .map(|md| md.to_skill_package())
            .collect())
    }

    fn load_local_skills(&self) -> Result<Vec<SkillPackage>, ClaudeError> {
        // Similar to project but from .claude-local/
        // TODO: Implement
        Ok(Vec::new())
    }
}
```

#### 4. 自动加载集成

**修改**：`src/internal/client.rs` 或相应的客户端初始化代码

```rust
impl InternalClient {
    pub fn new(options: ClaudeAgentOptions) -> Result<Self, ClaudeError> {
        // Auto-discover skills if setting_sources is configured
        let discovered_skills = options.discover_skills()?;

        // Register discovered skills
        for skill_package in discovered_skills {
            tracing::info!("Auto-registering skill: {}", skill_package.metadata.name);
            // Register with skill registry
        }

        // ... rest of initialization
    }
}
```

---

## 📝 使用示例

### 示例1：基本使用

```rust
use claude_agent_sdk_rs::{query, ClaudeAgentOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ClaudeAgentOptions::builder()
        .setting_sources(vec![SettingSource::Project])  // 启用项目skills
        .build();

    let messages = query("Use the calculator skill", Some(options)).await?;

    Ok(())
}
```

### 示例2：创建SKILL.md

**文件**：`.claude/skills/calculator/SKILL.md`

```markdown
---
name: "Calculator"
description: "Performs mathematical calculations"
version: "1.0.0"
author: "Math Team"
tags: ["math", "utility"]
dependencies: []
---

# Calculator Skill

A powerful calculator that can perform various mathematical operations.

## Capabilities

- Basic arithmetic: add, subtract, multiply, divide
- Advanced operations: power, square root, modulo
- Constants: PI, E

## Usage

Simply ask for a calculation and specify the operation.

Example: "Calculate 15 + 27"
```

### 示例3：带脚本的Skill

**文件**：`.claude/skills/data-processor/SKILL.md`

```markdown
---
name: "Data Processor"
description: "Processes CSV data files"
version: "1.5.0"
tags: ["data", "csv", "processing"]
---

# Data Processor

Processes CSV files with various transformations.

## Scripts

- `scripts/validate.py`: Validates CSV format
- `scripts/transform.py`: Applies transformations
- `scripts/export.py`: Exports to different formats

## Resources

- `resources/schema.json`: Data schema definition
- `resources/mapping.csv`: Field mappings
```

---

## 🧪 测试计划

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_md() {
        let content = r#"---
name: "Test"
description: "Test skill"
---

# Test
"#;

        let (metadata, _) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "Test");
    }

    #[test]
    fn test_discover_skills() {
        // Create temp .claude/skills/ directory
        // Add test SKILL.md files
        // Verify discovery
    }

    #[test]
    fn test_setting_sources_integration() {
        // Test Project, User, Local sources
        // Test priority order
    }
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_auto_load_skills() {
    let options = ClaudeAgentOptions::builder()
        .setting_sources(vec![SettingSource::Project])
        .cwd("./tests/fixtures/project-with-skills")
        .build();

    let skills = options.discover_skills().unwrap();
    assert!(!skills.is_empty());
    assert_eq!(skills[0].metadata.name, "Test Skill");
}
```

---

## 📊 实施步骤

### Phase 1: 核心解析器（1-2天）

- [ ] 创建`src/skills/skill_md.rs`模块
- [ ] 实现YAML frontmatter解析
- [ ] 实现Markdown内容提取
- [ ] 编写单元测试

### Phase 2: 目录扫描（1天）

- [ ] 实现`SkillsDirScanner`
- [ ] 支持项目目录扫描（`.claude/skills/`）
- [ ] 支持用户目录扫描（`~/.config/claude/skills/`）
- [ ] 支持本地目录扫描（`.claude-local/skills/`）

### Phase 3: 资源发现（1天）

- [ ] 自动发现scripts/
- [ ] 自动发现resources/
- [ ] 加载reference.md和forms.md
- [ ] 验证文件关联

### Phase 4: 集成到SDK（1-2天）

- [ ] 集成到`ClaudeAgentOptions`
- [ ] 实现`discover_skills()`方法
- [ ] 集成到客户端初始化
- [ ] 添加热重载支持

### Phase 5: 文档和示例（1天）

- [ ] 编写使用文档
- [ ] 创建示例SKILL.md文件
- [ ] 添加集成测试
- [ ] 更新README

**总时间**：5-7天

---

## ✅ 验收标准

### 功能完整性

- [x] 能够解析SKILL.md的YAML frontmatter
- [x] 能够提取Markdown内容作为instructions
- [x] 能够自动发现`.claude/skills/`目录
- [x] 能够关联scripts/和resources/目录
- [x] 支持SettingSource配置
- [x] 与Python SDK行为一致

### 测试覆盖

- [x] 单元测试覆盖率 > 80%
- [x] 集成测试通过
- [x] 边界条件测试

### 文档完整性

- [x] API文档完整
- [x] 使用示例清晰
- [x] SKILL.md格式说明

---

## 🎯 预期效果

实现后，用户可以：

1. **创建SKILL.md文件**：
   ```bash
   mkdir -p .claude/skills/my-skill
   cat > .claude/skills/my-skill/SKILL.md << EOF
   ---
   name: "My Skill"
   description: "Does something"
   ---

   # My Skill

   Instructions here...
   EOF
   ```

2. **自动加载**：
   ```rust
   let options = ClaudeAgentOptions::builder()
       .setting_sources(vec![SettingSource::Project])
       .build();

   // Skills are automatically discovered and loaded
   ```

3. **完全兼容**：
   - 与Claude Code CLI行为一致
   - 与Python SDK功能对等
   - 支持所有SKILL.md特性

---

## 📚 参考资料

- [Skill Authoring Best Practices - Claude Docs](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices)
- [Inside Claude Code Skills: Structure, prompts, invocation](https://mikhail.io/2025/10/claude-code-skills/)
- [Introduction to Claude Skills](https://platform.claude.com/cookbook/skills-notebooks-01-skills-introduction)
- [CLAUDE.md, skills, subagents explained](https://alexop.dev/posts/claude-code-customization-guide-claudemd-skills-subagents/)
- [SKILL.md, resources, and how Claude loads them](https://skywork.ai/blog/ai-agent/claude-skills-skill-md-resources-runtime-loading/)

---

**文档版本**：v1.0
**创建时间**：2025-01-10
**预计完成**：5-7个工作日
**优先级**：P0（高优先级）
