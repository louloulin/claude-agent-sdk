# Section 4: Skills System

The Skills System provides a comprehensive framework for creating, discovering, validating, and executing Claude Code skills. This module is fully compatible with Claude Code's Skills specification and includes several Rust-exclusive enhancements.

## 4.1 Overview

### Module Architecture

```
skills/
├── Core Types (types.rs, error.rs, trait_impl.rs)
├── Discovery (discovery.rs, skill_md.rs)
├── Security (auditor.rs, tool_restriction.rs, sandbox.rs)
├── Performance (performance.rs, progressive_disclosure.rs)
├── Integration (api.rs, vscode.rs, hot_reload.rs)
└── Utilities (tags.rs, version.rs, dependency.rs)
```

### Feature Flags

| Feature | Description |
|---------|-------------|
| `yaml` | Enable YAML serialization support |
| `hot-reload` | Enable file watching and hot reload |
| `sandbox` | Enable sandboxed skill execution |

### Claude Code Compatibility

This implementation is fully compatible with [Claude Code Skills specification](https://code.claude.com/docs/en/skills), supporting:
- SKILL.md YAML frontmatter parsing
- 12 metadata fields
- Progressive disclosure pattern
- Tool restrictions
- Lifecycle hooks

---

## 4.2 Core Types

### SkillMetadata

Metadata for a skill package:

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

**Fields:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | String | Yes | Unique identifier (e.g., "skill.my-skill") |
| `name` | String | Yes | Human-readable name |
| `description` | String | Yes | Skill description |
| `version` | String | Yes | Semantic version (default: "1.0.0") |
| `author` | Option<String> | No | Author information |
| `dependencies` | Vec<String> | No | Required skill dependencies |
| `tags` | Vec<String> | No | Tags for discovery |

### SkillPackage

A complete skill package with instructions and resources:

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

// Save to file
package.save_to_file("skill.json")?;

// Load from file
let loaded = SkillPackage::load_from_file("skill.json")?;
```

### SkillResources

Resources associated with a skill:

```rust
use claude_agent_sdk::skills::SkillResources;
use std::path::PathBuf;

let mut resources = SkillResources {
    folders: vec![PathBuf::from("./resources")],
    tools: vec!["search".to_string()],
    tests: vec!["test_basic".to_string()],
};

// Add resources
resources.add_folder("./data");
resources.add_tool("validate".to_string());
resources.add_test("test_advanced".to_string());

// Scan all folders
let files = resources.scan_folders()?;
for file in files {
    println!("Found: {:?}", file);
}
```

### SkillInput / SkillOutput

Execution types for skill invocation:

```rust
use claude_agent_sdk::skills::{SkillInput, SkillOutput};
use serde_json::json;

let input = SkillInput {
    params: json!({"query": "test"}),
};

// Create successful output
let output = SkillOutput::ok(json!({"result": "success"}));

// Create error output
let error_output = SkillOutput::err("Something went wrong");

// Add metadata
let output = output.with_metadata(json!({"duration_ms": 150}));
```

### SkillStatus

```rust
use claude_agent_sdk::skills::SkillStatus;

let status = SkillStatus::Ready;    // Ready to execute
let status = SkillStatus::Running;  // Currently executing
let status = SkillStatus::Completed; // Finished successfully
let status = SkillStatus::Failed;   // Execution failed
let status = SkillStatus::Disabled; // Disabled
```

---

## 4.3 Skill Trait

The `Skill` trait defines the interface for custom skills:

```rust
use claude_agent_sdk::skills::{Skill, SkillInput, SkillOutput, SkillError};
use async_trait::async_trait;

#[derive(Debug)]
struct MySkill;

#[async_trait]
impl Skill for MySkill {
    // Required methods
    fn name(&self) -> String {
        "my-skill".to_string()
    }

    fn description(&self) -> String {
        "Does something useful".to_string()
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        // Skill logic here
        Ok(SkillOutput::ok(serde_json::json!({"result": "done"})))
    }

    fn validate(&self) -> Result<(), SkillError> {
        // Validation logic
        Ok(())
    }

    // Optional methods
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

    // Lifecycle hooks
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

Wrapper for trait objects:

```rust
use claude_agent_sdk::skills::{Skill, SkillBox};

let skill = MySkill;
let boxed: SkillBox = SkillBox::new(skill);

// SkillBox implements Skill, so it can be used anywhere
let name = boxed.name();
boxed.validate()?;
let output = boxed.execute(SkillInput::default()).await?;

// Clone for sharing across async contexts
let cloned = boxed.clone();
```

---

## 4.4 SKILL.md Parser

The SKILL.md parser supports full YAML frontmatter with Claude Code specification compliance.

### Basic SKILL.md Format

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

### Advanced Metadata (12 Fields)

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

### Metadata Field Reference

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Skill name (max 64 chars, lowercase-hyphen) |
| `description` | String | Yes | Description (max 1024 chars) |
| `version` | String | No | Semantic version (default: "1.0.0") |
| `author` | String | No | Author information |
| `tags` | Vec<String> | No | Tags for discovery |
| `dependencies` | Vec<String> | No | Required skills |
| `allowed_tools` | Vec<String> | No | Tool restrictions |
| `model` | String | No | Specific model to use |
| `context` | String | No | "fork" for isolated context |
| `agent` | String | No | Agent type (e.g., "general-purpose") |
| `hooks` | Object | No | Lifecycle hooks |
| `user_invocable` | bool | No | Show in / menu (default: true) |

### Parsing SKILL.md Files

```rust
use claude_agent_sdk::skills::skill_md::{SkillMdFile, SkillsDirScanner, SkillMdError};

// Parse a single file
let skill = SkillMdFile::parse(".claude/skills/my-skill/SKILL.md")?;
println!("Loaded: {}", skill.metadata.name);
println!("Content: {}", skill.content);

// Access discovered resources
for script in &skill.scripts {
    println!("Script: {:?}", script);
}

// Progressive disclosure - get resource by name
if let Some(config) = skill.get_resource("config.json") {
    let content = std::fs::read_to_string(config)?;
}

// Scan a directory
let scanner = SkillsDirScanner::from_project_dir(".");
let skills = scanner.scan()?;
for skill in skills {
    println!("Found: {}", skill.metadata.name);
}

// Parallel scanning for large directories
let skills = scanner.scan_parallel().await?;
```

### Validation Rules

The parser enforces Claude Skills specification:

```rust
// Name validation
// - Max 64 characters
// - Only lowercase letters, numbers, hyphens
// - Cannot contain "anthropic" or "claude"
// - Cannot contain XML tags

let invalid_names = vec![
    "MySkill",           // Uppercase invalid
    "my_skill",          // Underscore invalid
    "my-skill!",         // Special chars invalid
    "claude-helper",     // Reserved word
    "skill<script>",     // XML tags invalid
];

// Description validation
// - Must be non-empty
// - Max 1024 characters
// - Cannot contain XML tags
```

---

## 4.5 Discovery System

### Discover from Directory

```rust
use claude_agent_sdk::skills::SkillRegistry;

// Discover JSON skill packages
let packages = SkillRegistry::discover_from_dir("./skills")?;

// Discover SKILL.md files
let packages = SkillRegistry::discover_skill_md_from_dir(".claude/skills")?;

// Multiple directories with priority
let packages = SkillRegistry::discover_from_multiple_dirs(vec![
    ".claude/skills",           // Project skills (highest priority)
    "~/.config/claude/skills",  // User skills
])?;
```

### SkillRegistry

Simple registry for managing skills:

```rust
use claude_agent_sdk::skills::{SkillRegistry, SkillBox};

let mut registry = SkillRegistry::new();

// Register a skill
let skill = MySkill;
registry.register(Box::new(skill))?;

// Get a skill by name
if let Some(skill) = registry.get("my-skill") {
    let output = skill.execute(SkillInput::default()).await?;
}

// List all registered skills
let names = registry.list();
for name in names {
    println!("Registered: {}", name);
}
```

---

## 4.6 Security Auditor

The security auditor detects potentially dangerous patterns in skill code.

### Risk Levels

| Level | Description |
|-------|-------------|
| `Safe` | No issues detected |
| `Low` | Minor concerns |
| `Medium` | Needs review |
| `High` | Dangerous, should not run |
| `Critical` | Malicious, block execution |

### Issue Types

1. **NetworkAccess** - HTTP/HTTPS, sockets, fetch
2. **DangerousCommand** - eval, exec, system calls
3. **FileAccess** - File read/write operations
4. **CodeExecution** - compile, importlib
5. **ExternalCommand** - subprocess, spawn
6. **SensitiveDataAccess** - /etc/, /home/, credentials
7. **Other** - Other security concerns

### Using the Auditor

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

// Filter by severity
let critical = report.critical_issues();
let high_risk = report.issues_by_severity(RiskLevel::High);
```

---

## 4.7 Tool Restrictions

Enforce `allowed_tools` from skill metadata:

```rust
use claude_agent_sdk::skills::tool_restriction::ToolRestriction;

// Create restriction
let restriction = ToolRestriction::new(Some(vec![
    "Read".to_string(),
    "Grep".to_string(),
    "Bash(python:*)".to_string(),  // Pattern restriction
]));

// Check tools
assert!(restriction.is_tool_allowed("Read"));
assert!(restriction.is_tool_allowed("Bash(python:script.py)"));
assert!(!restriction.is_tool_allowed("Write"));
assert!(!restriction.is_tool_allowed("Bash(node:app.js)"));

// Unrestricted (allow all)
let unrestricted = ToolRestriction::unrestricted();
assert!(unrestricted.is_tool_allowed("AnyTool"));
```

### Tool Specification Formats

```yaml
allowed_tools:
  - Read                    # Simple tool name
  - Grep                    # Simple tool name
  - "Bash(python:*)"        # Pattern: only python commands
  - "Bash(npm:*)"           # Pattern: only npm commands
  - "*"                     # Wildcard: all tools
```

---

## 4.8 Performance Optimizations

### IndexedSkillCollection

O(1) lookups by name and tag:

```rust
use claude_agent_sdk::skills::performance::IndexedSkillCollection;

let mut collection = IndexedSkillCollection::new();

// Add skills
collection.add(skill_package_1);
collection.add(skill_package_2);

// O(1) lookup by name
if let Some(skill) = collection.get_by_name("my-skill") {
    println!("Found: {}", skill.metadata.name);
}

// O(1) lookup by tag
let testing_skills = collection.get_by_tag("testing");

// Query with caching
let results = collection.query("tag:testing AND author:me");
```

### LruCache

Query result caching:

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
// ... after operations ...

println!("Cache hit rate: {:.2}%", stats.cache_hit_rate() * 100.0);
println!("Throughput: {:.2} items/sec", stats.throughput());
if let Some(avg) = stats.avg_time_per_operation() {
    println!("Avg time: {:?}", avg);
}
```

---

## 4.9 Progressive Disclosure

Lazy loading of supporting files to conserve context:

```rust
use claude_agent_sdk::skills::progressive_disclosure::ProgressiveSkillLoader;

// Load only SKILL.md
let loader = ProgressiveSkillLoader::load(".claude/skills/my-skill")?;

// Main content always available
let content = loader.get_main_content();

// Reference files loaded on-demand
if let Some(reference) = loader.load_reference("reference.md")? {
    println!("Reference docs: {}", reference);
}

// Load all references when needed
let all_refs = loader.load_all_references()?;
for (name, content) in all_refs {
    println!("=== {} ===\n{}", name, content);
}

// Check available references
println!("Available references: {}", loader.get_reference_count());
```

### File Structure for Progressive Disclosure

```
skill-directory/
├── SKILL.md          # Required, always loaded
├── reference.md      # Optional, detailed docs
├── examples.md       # Optional, usage examples
├── forms.md          # Optional, field mappings
└── scripts/          # Optional, utility scripts
    ├── helper.py
    └── validate.sh
```

---

## 4.10 Hot Reload

File watching for development (requires `hot-reload` feature):

```rust
use claude_agent_sdk::skills::hot_reload::{
    HotReloadManager, HotReloadConfig, HotReloadEvent
};

let config = HotReloadConfig {
    watch_paths: vec![".claude/skills".into()],
    debounce_ms: 500,
};

let manager = HotReloadManager::new(config);

// Start watching
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

## 4.11 Skills API Client

HTTP client for Anthropic Skills API:

```rust
use claude_agent_sdk::skills::api::{SkillsApiClient, SkillsError};

let client = SkillsApiClient::new("your-api-key")?;

// Upload a skill
let response = client.upload_skill(&skill_package).await?;
println!("Uploaded: {}", response.id);

// List skills
let skills = client.list_skills().await?;
for skill in skills.skills {
    println!("{}: {}", skill.name, skill.description);
}

// Get a specific skill
let skill = client.get_skill("skill-id").await?;

// Delete a skill
client.delete_skill("skill-id").await?;
```

---

## 4.12 Error Handling

```rust
use claude_agent_sdk::skills::error::{SkillError, SkillOutput, Result};

// Error variants
let errors = vec![
    SkillError::Validation("Invalid config".to_string()),
    SkillError::Execution("Runtime error".to_string()),
    SkillError::NotFound("Skill not found".to_string()),
    SkillError::AlreadyExists("Duplicate skill".to_string()),
    SkillError::InvalidMetadata("Bad metadata".to_string()),
    SkillError::VersionConflict("Incompatible version".to_string()),
];

// Result type alias
fn my_function() -> Result<SkillPackage> {
    // Returns Result<SkillPackage, SkillError>
}

// SkillOutput for execution results
let success = SkillOutput::ok(json!({"result": "done"}));
let failure = SkillOutput::err("Something went wrong");
```

---

## 4.13 Complete Example

```rust
use claude_agent_sdk::skills::{
    Skill, SkillBox, SkillRegistry, SkillInput, SkillOutput, SkillError,
    SkillAuditor, AuditConfig, SkillMdFile, SkillsDirScanner,
};
use async_trait::async_trait;
use serde_json::json;

// Define a custom skill
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
        // Process data...
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
    // 1. Create and register custom skill
    let mut registry = SkillRegistry::new();
    registry.register(Box::new(DataProcessor))?;

    // 2. Discover skills from directory
    let discovered = SkillRegistry::discover_skill_md_from_dir(".claude/skills")?;
    println!("Discovered {} skills", discovered.len());

    // 3. Audit a skill before using
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

    // 4. Execute the custom skill
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

## 4.14 Rust-Exclusive Features

The Rust SDK includes several enhancements not found in other SDKs:

| Feature | Description |
|---------|-------------|
| **Enhanced Validation** | 12 metadata fields with full specification compliance |
| **Security Auditor** | 7 issue types detected automatically |
| **Progressive Disclosure** | O(1) lazy loading with resource caching |
| **Hot Reload** | File watching for development |
| **Indexed Collections** | O(1) lookups by name and tag |
| **LRU Cache** | Query result caching |

---

## 4.15 API Reference

### Re-exports from `skills` module

```rust
// Core types
pub use types::{SkillInput, SkillMetadata, SkillPackage, SkillResources, SkillStatus};

// Trait and wrapper
pub use trait_impl::{Skill, SkillBox};

// Errors
pub use error::{SkillError, SkillOutput, SkillResult};

// Discovery
pub use discovery::{discover_from_dir, discover_from_multiple_dirs, discover_skill_md_from_dir};

// SKILL.md parsing
pub use skill_md::{SkillMdFile, SkillMdMetadata, SkillMdError, SkillsDirScanner, SkillHooks, HookConfig, HookType, SkillContext};

// Security
pub use auditor::{AuditConfig, AuditError, IssueType, RiskLevel, SkillAuditor, SkillAuditIssue, SkillAuditReport};

// Tool restrictions
pub use tool_restriction::{ToolRestriction, ToolRestrictionError};

// Performance
pub use performance::{BatchOperations, IndexedSkillCollection, LruCache, PerformanceStats};

// Progressive disclosure
pub use progressive_disclosure::ProgressiveSkillLoader;

// Hot reload (feature-gated)
pub use hot_reload::{HotReloadConfig, HotReloadEvent, HotReloadManager, HotReloadWatcher};

// API client
pub use api::{ListSkillsResponse, SkillApiInfo, SkillsApiClient, SkillsError, UploadSkillResponse};
```
