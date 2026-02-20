# 第4章：技能系统

技能系统提供了一个全面的框架，用于创建、发现、验证和执行 Claude Code 技能。该模块完全兼容 Claude Code 的技能规范，并包含多项 Rust 专属增强功能。

## 4.1 概述

### 模块架构

```
skills/
├── 核心类型 (types.rs, error.rs, trait_impl.rs)
├── 发现机制 (discovery.rs, skill_md.rs)
├── 安全机制 (auditor.rs, tool_restriction.rs, sandbox.rs)
├── 性能优化 (performance.rs, progressive_disclosure.rs)
├── 集成功能 (api.rs, vscode.rs, hot_reload.rs)
└── 工具类 (tags.rs, version.rs, dependency.rs)
```

### 功能标志

| 功能 | 描述 |
|---------|-------------|
| `yaml` | 启用 YAML 序列化支持 |
| `hot-reload` | 启用文件监视和热重载 |
| `sandbox` | 启用沙箱化的技能执行 |

### Claude Code 兼容性

此实现完全兼容 [Claude Code 技能规范](https://code.claude.com/docs/en/skills)，支持：
- SKILL.md YAML 前置内容解析
- 12 个元数据字段
- 渐进式披露模式
- 工具限制
- 生命周期钩子

---

## 4.2 核心类型

### SkillMetadata

技能包的元数据：

```rust
use claude_agent_sdk::skills::SkillMetadata;

let metadata = SkillMetadata {
    id: "skill.my-skill".to_string(),
    name: "my-skill".to_string(),
    description: "A test skill".to_string(),
    version: "1.0.0".to_string(),
    author: Some("Author Name".to_string()),
    dependencies: vec!["other-skill".to_string()],
    tags: vec!["test".to_string(), "example".to_string()],
};
```

**字段：**
| 字段 | 类型 | 必需 | 描述 |
|-------|------|----------|-------------|
| `id` | String | 是 | 唯一标识符（例如 "skill.my-skill"） |
| `name` | String | 是 | 人类可读的名称 |
| `description` | String | 是 | 技能描述 |
| `version` | String | 是 | 语义版本（默认："1.0.0"） |
| `author` | Option<String> | 否 | 作者信息 |
| `dependencies` | Vec<String> | 否 | 所需的技能依赖 |
| `tags` | Vec<String> | 否 | 用于发现的标签 |

### SkillPackage

包含指令和资源的完整技能包：

```rust
use claude_agent_sdk::skills::{SkillPackage, SkillMetadata, SkillResources};

let package = SkillPackage {
    metadata: SkillMetadata {
        id: "skill.example".to_string(),
        name: "example-skill".to_string(),
        description: "Example skill".to_string(),
        version: "1.0.0".to_string(),
        author: None,
        dependencies: vec![],
        tags: vec!["example".to_string()],
    },
    instructions: "This skill does X, Y, Z...".to_string(),
    scripts: vec!["scripts/helper.sh".to_string()],
    resources: SkillResources::default(),
};

// 保存到文件
package.save_to_file("skill.json")?;

// 从文件加载
let loaded = SkillPackage::load_from_file("skill.json")?;
```

### SkillResources

与技能关联的资源：

```rust
use claude_agent_sdk::skills::SkillResources;
use std::path::PathBuf;

let mut resources = SkillResources {
    folders: vec![PathBuf::from("./resources")],
    tools: vec!["search".to_string()],
    tests: vec!["test_basic".to_string()],
};

// 添加资源
resources.add_folder("./data");
resources.add_tool("validate".to_string());
resources.add_test("test_advanced".to_string());

// 扫描所有文件夹
let files = resources.scan_folders()?;
for file in files {
    println!("Found: {:?}", file);
}
```

### SkillInput / SkillOutput

技能调用的执行类型：

```rust
use claude_agent_sdk::skills::{SkillInput, SkillOutput};
use serde_json::json;

let input = SkillInput {
    params: json!({"query": "test"}),
};

// 创建成功输出
let output = SkillOutput::ok(json!({"result": "success"}));

// 创建错误输出
let error_output = SkillOutput::err("Something went wrong");

// 添加元数据
let output = output.with_metadata(json!({"duration_ms": 150}));
```

### SkillStatus

```rust
use claude_agent_sdk::skills::SkillStatus;

let status = SkillStatus::Ready;    // 准备执行
let status = SkillStatus::Running;  // 正在执行
let status = SkillStatus::Completed; // 成功完成
let status = SkillStatus::Failed;   // 执行失败
let status = SkillStatus::Disabled; // 已禁用
```

---

## 4.3 Skill Trait

`Skill` trait 定义了自定义技能的接口：

```rust
use claude_agent_sdk::skills::{Skill, SkillInput, SkillOutput, SkillError};
use async_trait::async_trait;

#[derive(Debug)]
struct MySkill;

#[async_trait]
impl Skill for MySkill {
    // 必需方法
    fn name(&self) -> String {
        "my-skill".to_string()
    }

    fn description(&self) -> String {
        "Does something useful".to_string()
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        // 技能逻辑
        Ok(SkillOutput::ok(serde_json::json!({"result": "done"})))
    }

    fn validate(&self) -> Result<(), SkillError> {
        // 验证逻辑
        Ok(())
    }

    // 可选方法
    fn version(&self) -> String {
        "2.0.0".to_string()
    }

    fn author(&self) -> Option<String> {
        Some("Author Name".to_string())
    }

    fn tags(&self) -> Vec<String> {
        vec!["utility".to_string()]
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["base-skill".to_string()]
    }

    fn supports(&self, capability: &str) -> bool {
        capability == "async"
    }

    // 生命周期钩子
    async fn before_execute(&self, _input: &SkillInput) -> Result<(), SkillError> {
        println!("Starting execution...");
        Ok(())
    }

    async fn after_execute(&self, _input: &SkillInput, _output: &SkillOutput) -> Result<(), SkillError> {
        println!("Execution complete!");
        Ok(())
    }

    async fn on_error(&self, _input: &SkillInput, error: &SkillError) -> SkillError {
        SkillError::Execution(format!("Wrapped error: {}", error))
    }
}
```

### SkillBox

用于 trait 对象的包装器：

```rust
use claude_agent_sdk::skills::{Skill, SkillBox};

let skill = MySkill;
let boxed: SkillBox = SkillBox::new(skill);

// SkillBox 实现了 Skill，因此可以在任何地方使用
let name = boxed.name();
boxed.validate()?;
let output = boxed.execute(SkillInput::default()).await?;

// 克隆以便在异步上下文中共享
let cloned = boxed.clone();
```

---

## 4.4 SKILL.md 解析器

SKILL.md 解析器支持完整的 YAML 前置内容，并符合 Claude Code 规范。

### 基本 SKILL.md 格式

```markdown
---
name: my-skill
description: A useful skill for X
version: 1.0.0
author: Your Name
tags:
  - utility
  - example
dependencies:
  - base-skill
---

# My Skill

This skill helps with X, Y, and Z.

## Usage

Instructions for using the skill...
```

### 高级元数据（12 个字段）

```markdown
---
name: advanced-skill
description: An advanced skill with all metadata fields
version: 2.0.0
author: Author <author@example.com>
tags:
  - advanced
  - testing
dependencies:
  - base-skill
allowed_tools:
  - Read
  - Grep
  - "Bash(python:*)"
model: claude-sonnet-4-20250514
context: fork
agent: general-purpose
hooks:
  pre_tool_use:
    - matcher: "Bash"
      command: "./scripts/check.sh"
      once: true
  post_tool_use:
    - matcher: "*"
      command: "./scripts/notify.sh"
user_invocable: true
disable_model_invocation: false
---

# Advanced Skill Content
...
```

### 元数据字段参考

| 字段 | 类型 | 必需 | 描述 |
|-------|------|----------|-------------|
| `name` | String | 是 | 技能名称（最多 64 字符，小写-连字符格式） |
| `description` | String | 是 | 描述（最多 1024 字符） |
| `version` | String | 否 | 语义版本（默认："1.0.0"） |
| `author` | String | 否 | 作者信息 |
| `tags` | Vec<String> | 否 | 用于发现的标签 |
| `dependencies` | Vec<String> | 否 | 所需的技能 |
| `allowed_tools` | Vec<String> | 否 | 工具限制 |
| `model` | String | 否 | 使用的特定模型 |
| `context` | String | 否 | "fork" 表示隔离的上下文 |
| `agent` | String | 否 | 代理类型（例如 "general-purpose"） |
| `hooks` | Object | 否 | 生命周期钩子 |
| `user_invocable` | bool | 否 | 在 / 菜单中显示（默认：true） |

### 解析 SKILL.md 文件

```rust
use claude_agent_sdk::skills::skill_md::{SkillMdFile, SkillsDirScanner, SkillMdError};

// 解析单个文件
let skill = SkillMdFile::parse(".claude/skills/my-skill/SKILL.md")?;
println!("Loaded: {}", skill.metadata.name);
println!("Content: {}", skill.content);

// 访问发现的资源
for script in &skill.scripts {
    println!("Script: {:?}", script);
}

// 渐进式披露 - 按名称获取资源
if let Some(config) = skill.get_resource("config.json") {
    let content = std::fs::read_to_string(config)?;
}

// 扫描目录
let scanner = SkillsDirScanner::from_project_dir(".");
let skills = scanner.scan()?;
for skill in skills {
    println!("Found: {}", skill.metadata.name);
}

// 大型目录的并行扫描
let skills = scanner.scan_parallel().await?;
```

### 验证规则

解析器强制执行 Claude Skills 规范：

```rust
// 名称验证
// - 最多 64 个字符
// - 仅限小写字母、数字、连字符
// - 不能包含 "anthropic" 或 "claude"
// - 不能包含 XML 标签

let invalid_names = vec![
    "MySkill",           // 大写无效
    "my_skill",          // 下划线无效
    "my-skill!",         // 特殊字符无效
    "claude-helper",     // 保留字
    "skill<script>",     // XML 标签无效
];

// 描述验证
// - 必须非空
// - 最多 1024 个字符
// - 不能包含 XML 标签
```

---

## 4.5 发现系统

### 从目录发现

```rust
use claude_agent_sdk::skills::SkillRegistry;

// 发现 JSON 技能包
let packages = SkillRegistry::discover_from_dir("./skills")?;

// 发现 SKILL.md 文件
let packages = SkillRegistry::discover_skill_md_from_dir(".claude/skills")?;

// 多目录发现（带优先级）
let packages = SkillRegistry::discover_from_multiple_dirs(vec![
    ".claude/skills",           // 项目技能（最高优先级）
    "~/.config/claude/skills",  // 用户技能
])?;
```

### SkillRegistry

用于管理技能的简单注册表：

```rust
use claude_agent_sdk::skills::{SkillRegistry, SkillBox};

let mut registry = SkillRegistry::new();

// 注册技能
let skill = MySkill;
registry.register(Box::new(skill))?;

// 按名称获取技能
if let Some(skill) = registry.get("my-skill") {
    let output = skill.execute(SkillInput::default()).await?;
}

// 列出所有已注册的技能
let names = registry.list();
for name in names {
    println!("Registered: {}", name);
}
```

---

## 4.6 安全审计器

安全审计器检测技能代码中潜在的危险模式。

### 风险级别

| 级别 | 描述 |
|-------|-------------|
| `Safe` | 未检测到问题 |
| `Low` | 轻微问题 |
| `Medium` | 需要审查 |
| `High` | 危险，不应运行 |
| `Critical` | 恶意，阻止执行 |

### 问题类型

1. **NetworkAccess** - HTTP/HTTPS、套接字、fetch
2. **DangerousCommand** - eval、exec、系统调用
3. **FileAccess** - 文件读写操作
4. **CodeExecution** - compile、importlib
5. **ExternalCommand** - subprocess、spawn
6. **SensitiveDataAccess** - /etc/、/home/、凭据
7. **Other** - 其他安全问题

### 使用审计器

```rust
use claude_agent_sdk::skills::auditor::{
    SkillAuditor, AuditConfig, RiskLevel, IssueType
};
use claude_agent_sdk::skills::skill_md::SkillMdFile;

let config = AuditConfig {
    strict_mode: true,
    allow_network: false,
    check_scripts: true,
    check_resources: true,
};

let auditor = SkillAuditor::new(config);
let skill = SkillMdFile::parse(".claude/skills/suspicious/SKILL.md")?;
let report = auditor.audit(&skill)?;

if report.safe {
    println!("Skill is safe");
} else {
    println!("Risk level: {}", report.risk_level);

    for issue in &report.issues {
        println!(
            "[{}] {} at line {:?}: {}",
            issue.severity,
            issue.issue_type,
            issue.line,
            issue.message
        );
    }
}

// 按严重程度筛选
let critical = report.critical_issues();
let high_risk = report.issues_by_severity(RiskLevel::High);
```

---

## 4.7 工具限制

强制执行技能元数据中的 `allowed_tools`：

```rust
use claude_agent_sdk::skills::tool_restriction::ToolRestriction;

// 创建限制
let restriction = ToolRestriction::new(Some(vec![
    "Read".to_string(),
    "Grep".to_string(),
    "Bash(python:*)".to_string(),  // 模式限制
]));

// 检查工具
assert!(restriction.is_tool_allowed("Read"));
assert!(restriction.is_tool_allowed("Bash(python:script.py)"));
assert!(!restriction.is_tool_allowed("Write"));
assert!(!restriction.is_tool_allowed("Bash(node:app.js)"));

// 无限制（允许所有）
let unrestricted = ToolRestriction::unrestricted();
assert!(unrestricted.is_tool_allowed("AnyTool"));
```

### 工具规范格式

```yaml
allowed_tools:
  - Read                    # 简单工具名称
  - Grep                    # 简单工具名称
  - "Bash(python:*)"        # 模式：仅 python 命令
  - "Bash(npm:*)"           # 模式：仅 npm 命令
  - "*"                     # 通配符：所有工具
```

---

## 4.8 性能优化

### IndexedSkillCollection

按名称和标签的 O(1) 查找：

```rust
use claude_agent_sdk::skills::performance::IndexedSkillCollection;

let mut collection = IndexedSkillCollection::new();

// 添加技能
collection.add(skill_package_1);
collection.add(skill_package_2);

// 按名称 O(1) 查找
if let Some(skill) = collection.get_by_name("my-skill") {
    println!("Found: {}", skill.metadata.name);
}

// 按标签 O(1) 查找
let testing_skills = collection.get_by_tag("testing");

// 带缓存的查询
let results = collection.query("tag:testing AND author:me");
```

### LruCache

查询结果缓存：

```rust
use claude_agent_sdk::skills::performance::LruCache;

let mut cache = LruCache::new(100);

cache.put("query:result", vec![1, 2, 3]);

if let Some(results) = cache.get(&"query:result".to_string()) {
    println!("Cache hit!");
}
```

### PerformanceStats

```rust
use claude_agent_sdk::skills::performance::PerformanceStats;

let stats = PerformanceStats::new();
// ... 操作后 ...

println!("Cache hit rate: {:.2}%", stats.cache_hit_rate() * 100.0);
println!("Throughput: {:.2} items/sec", stats.throughput());
if let Some(avg) = stats.avg_time_per_operation() {
    println!("Avg time: {:?}", avg);
}
```

---

## 4.9 渐进式披露

延迟加载支持文件以节省上下文：

```rust
use claude_agent_sdk::skills::progressive_disclosure::ProgressiveSkillLoader;

// 仅加载 SKILL.md
let loader = ProgressiveSkillLoader::load(".claude/skills/my-skill")?;

// 主要内容始终可用
let content = loader.get_main_content();

// 参考文件按需加载
if let Some(reference) = loader.load_reference("reference.md")? {
    println!("Reference docs: {}", reference);
}

// 需要时加载所有参考文件
let all_refs = loader.load_all_references()?;
for (name, content) in all_refs {
    println!("=== {} ===\n{}", name, content);
}

// 检查可用的参考文件
println!("Available references: {}", loader.get_reference_count());
```

### 渐进式披露的文件结构

```
skill-directory/
├── SKILL.md          # 必需，始终加载
├── reference.md      # 可选，详细文档
├── examples.md       # 可选，使用示例
├── forms.md          # 可选，字段映射
└── scripts/          # 可选，工具脚本
    ├── helper.py
    └── validate.sh
```

---

## 4.10 热重载

开发时的文件监视（需要 `hot-reload` 功能）：

```rust
use claude_agent_sdk::skills::hot_reload::{
    HotReloadManager, HotReloadConfig, HotReloadEvent
};

let config = HotReloadConfig {
    watch_paths: vec![".claude/skills".into()],
    debounce_ms: 500,
};

let manager = HotReloadManager::new(config);

// 开始监视
manager.start(|event| {
    match event {
        HotReloadEvent::Created(path) => {
            println!("New skill: {:?}", path);
        }
        HotReloadEvent::Modified(path) => {
            println!("Skill updated: {:?}", path);
        }
        HotReloadEvent::Deleted(path) => {
            println!("Skill removed: {:?}", path);
        }
        HotReloadEvent::Error(err) => {
            eprintln!("Watch error: {}", err);
        }
    }
})?;
```

---

## 4.11 Skills API 客户端

Anthropic Skills API 的 HTTP 客户端：

```rust
use claude_agent_sdk::skills::api::{SkillsApiClient, SkillsError};

let client = SkillsApiClient::new("your-api-key")?;

// 上传技能
let response = client.upload_skill(&skill_package).await?;
println!("Uploaded: {}", response.id);

// 列出技能
let skills = client.list_skills().await?;
for skill in skills.skills {
    println!("{}: {}", skill.name, skill.description);
}

// 获取特定技能
let skill = client.get_skill("skill-id").await?;

// 删除技能
client.delete_skill("skill-id").await?;
```

---

## 4.12 错误处理

```rust
use claude_agent_sdk::skills::error::{SkillError, SkillOutput, Result};

// 错误变体
let errors = vec![
    SkillError::Validation("Invalid config".to_string()),
    SkillError::Execution("Runtime error".to_string()),
    SkillError::NotFound("Skill not found".to_string()),
    SkillError::AlreadyExists("Duplicate skill".to_string()),
    SkillError::InvalidMetadata("Bad metadata".to_string()),
    SkillError::VersionConflict("Incompatible version".to_string()),
];

// 结果类型别名
fn my_function() -> Result<SkillPackage> {
    // 返回 Result<SkillPackage, SkillError>
}

// 执行结果的 SkillOutput
let success = SkillOutput::ok(json!({"result": "done"}));
let failure = SkillOutput::err("Something went wrong");
```

---

## 4.13 完整示例

```rust
use claude_agent_sdk::skills::{
    Skill, SkillBox, SkillRegistry, SkillInput, SkillOutput, SkillError,
    SkillAuditor, AuditConfig, SkillMdFile, SkillsDirScanner,
};
use async_trait::async_trait;
use serde_json::json;

// 定义自定义技能
#[derive(Debug)]
struct DataProcessor;

#[async_trait]
impl Skill for DataProcessor {
    fn name(&self) -> String {
        "data-processor".to_string()
    }

    fn description(&self) -> String {
        "Processes and transforms data".to_string()
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let data = input.params.get("data").cloned().unwrap_or(json!(null));
        // 处理数据...
        Ok(SkillOutput::ok(json!({
            "processed": true,
            "input": data
        })))
    }

    fn validate(&self) -> Result<(), SkillError> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建并注册自定义技能
    let mut registry = SkillRegistry::new();
    registry.register(Box::new(DataProcessor))?;

    // 2. 从目录发现技能
    let discovered = SkillRegistry::discover_skill_md_from_dir(".claude/skills")?;
    println!("Discovered {} skills", discovered.len());

    // 3. 使用前审计技能
    let scanner = SkillsDirScanner::from_project_dir(".");
    let skills = scanner.scan()?;

    if let Some(skill) = skills.first() {
        let auditor = SkillAuditor::new(AuditConfig::default());
        let report = auditor.audit(skill)?;

        if report.safe {
            println!("Skill '{}' is safe to use", skill.metadata.name);
        } else {
            println!("Warning: Risk level = {}", report.risk_level);
        }
    }

    // 4. 执行自定义技能
    if let Some(skill) = registry.get("data-processor") {
        let input = SkillInput {
            params: json!({"data": [1, 2, 3]}),
        };
        let output = skill.execute(input).await?;
        println!("Result: {}", output);
    }

    Ok(())
}
```

---

## 4.14 Rust 专属功能

Rust SDK 包含其他 SDK 中没有的多项增强功能：

| 功能 | 描述 |
|---------|-------------|
| **增强验证** | 12 个元数据字段，完全符合规范 |
| **安全审计器** | 自动检测 7 种问题类型 |
| **渐进式披露** | O(1) 延迟加载，带资源缓存 |
| **热重载** | 开发时的文件监视 |
| **索引集合** | 按名称和标签的 O(1) 查找 |
| **LRU 缓存** | 查询结果缓存 |

---

## 4.15 API 参考

### 从 `skills` 模块重新导出

```rust
// 核心类型
pub use types::{SkillInput, SkillMetadata, SkillPackage, SkillResources, SkillStatus};

// Trait 和包装器
pub use trait_impl::{Skill, SkillBox};

// 错误
pub use error::{SkillError, SkillOutput, SkillResult};

// 发现
pub use discovery::{discover_from_dir, discover_from_multiple_dirs, discover_skill_md_from_dir};

// SKILL.md 解析
pub use skill_md::{SkillMdFile, SkillMdMetadata, SkillMdError, SkillsDirScanner, SkillHooks, HookConfig, HookType, SkillContext};

// 安全
pub use auditor::{AuditConfig, AuditError, IssueType, RiskLevel, SkillAuditor, SkillAuditIssue, SkillAuditReport};

// 工具限制
pub use tool_restriction::{ToolRestriction, ToolRestrictionError};

// 性能
pub use performance::{BatchOperations, IndexedSkillCollection, LruCache, PerformanceStats};

// 渐进式披露
pub use progressive_disclosure::ProgressiveSkillLoader;

// 热重载（功能门控）
pub use hot_reload::{HotReloadConfig, HotReloadEvent, HotReloadManager, HotReloadWatcher};

// API 客户端
pub use api::{ListSkillsResponse, SkillApiInfo, SkillsApiClient, SkillsError, UploadSkillResponse};
```
