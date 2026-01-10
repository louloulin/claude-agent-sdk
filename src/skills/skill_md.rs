//! SKILL.md file parser for Claude Code compatibility
//!
//! This module provides functionality to parse and load SKILL.md files
//! in the same format as Claude Code CLI, including YAML frontmatter
//! and markdown content.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

// Use types from the current module's types.rs
use super::types::SkillPackage;

/// Errors that can occur when parsing SKILL.md files
#[derive(Debug, Error)]
pub enum SkillMdError {
    #[error("Failed to read SKILL.md: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    YamlError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid SKILL.md format: expected YAML frontmatter enclosed in ---")]
    InvalidFormat,

    #[error("Failed to parse skill package: {0}")]
    PackageError(String),
}

/// SKILL.md frontmatter metadata
///
/// Based on Claude Code Skills specification:
/// https://code.claude.com/docs/en/skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMdMetadata {
    // === Required Fields ===
    pub name: String,
    pub description: String,

    // === Standard Fields ===
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,

    // === Advanced Fields (Claude Code Official) ===

    /// Tool restrictions - limits which tools the skill can use
    /// Can include tool specifications like "Bash(python:*)"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<String>>,

    /// Specific model to use for this skill (e.g., "claude-sonnet-4-20250514")
    /// Defaults to the session's model if not specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Context mode - set to "fork" to run in isolated sub-agent context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<SkillContext>,

    /// Agent type when using context: fork
    /// Examples: "general-purpose", "Explore", "Plan", "code-reviewer"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// Lifecycle hooks for events during skill execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<SkillHooks>,

    /// Whether this skill appears in the / menu (default: true)
    /// Does not affect Skill tool invocation or auto-discovery
    #[serde(default = "default_user_invocable")]
    pub user_invocable: bool,

    /// Prevent model invocation via Skill tool
    /// Does not affect auto-discovery based on description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_model_invocation: Option<bool>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_user_invocable() -> bool {
    true
}

/// Context mode for skill execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkillContext {
    /// Run in isolated forked sub-agent context
    Fork,
}

/// Lifecycle hooks for skill execution events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillHooks {
    /// Hooks before tool use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_tool_use: Option<Vec<HookConfig>>,

    /// Hooks after tool use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_tool_use: Option<Vec<HookConfig>>,

    /// Hooks when skill stops
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<HookConfig>>,
}

/// Configuration for a single lifecycle hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// Tool/event matcher (e.g., "Bash", "Read", "*")
    pub matcher: String,

    /// Command/script to execute
    pub command: String,

    /// Only run this hook once per session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub once: Option<bool>,

    /// Hook type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<HookType>,
}

/// Type of hook execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HookType {
    /// Execute a shell command
    Command,
    /// Run a script file
    Script,
    /// Call a function
    Function,
}

/// Parsed SKILL.md file with all associated resources
#[derive(Debug, Clone)]
pub struct SkillMdFile {
    /// Metadata from YAML frontmatter
    pub metadata: SkillMdMetadata,
    /// Markdown content (instructions for Claude)
    pub content: String,
    /// Directory containing the skill
    pub skill_dir: PathBuf,
    /// Associated scripts from scripts/ directory
    pub scripts: Vec<PathBuf>,
    /// Associated resources from resources/ directory
    pub resources: Vec<PathBuf>,
    /// Reference file if exists
    pub reference: Option<PathBuf>,
    /// Forms file if exists
    pub forms: Option<PathBuf>,
}

impl SkillMdFile {
    /// Parse a SKILL.md file from the filesystem
    ///
    /// # Arguments
    ///
    /// * `skill_md_path` - Path to the SKILL.md file
    ///
    /// # Returns
    ///
    /// A parsed SkillMdFile with metadata, content, and discovered resources
    ///
    /// # Errors
    ///
    /// Returns SkillMdError if:
    /// - File cannot be read
    /// - YAML frontmatter is invalid
    /// - Required fields are missing
    ///
    /// # Example
    ///
    /// ```no_run
    /// use claude_agent_sdk_rs::skills::skill_md::SkillMdFile;
    ///
    /// let skill = SkillMdFile::parse(".claude/skills/my-skill/SKILL.md")?;
    /// println!("Loaded skill: {}", skill.metadata.name);
    /// ```
    pub fn parse<P: AsRef<Path>>(skill_md_path: P) -> Result<Self, SkillMdError> {
        let path = skill_md_path.as_ref();
        let skill_dir = path
            .parent()
            .ok_or_else(|| SkillMdError::InvalidFormat)?;

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
            skill_dir: skill_dir.to_path_buf(),
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

        // Split by "---" delimiter
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Err(SkillMdError::InvalidFormat);
        }

        // Second part is YAML frontmatter
        let yaml_content = parts[1].trim();

        // Third part is markdown content (everything after the second "---")
        let markdown_content = if parts.len() > 3 {
            // Join remaining parts with "---" in case content contains "---"
            parts[2..].join("---")
        } else {
            parts[2].to_string()
        };

        // Parse YAML frontmatter
        let metadata: SkillMdMetadata = serde_yaml::from_str(yaml_content)
            .map_err(|e| SkillMdError::YamlError(e.to_string()))?;

        // Validate required fields
        if metadata.name.is_empty() {
            return Err(SkillMdError::MissingField("name".to_string()));
        }
        if metadata.description.is_empty() {
            return Err(SkillMdError::MissingField("description".to_string()));
        }

        Ok((metadata, markdown_content))
    }

    /// Discover scripts in scripts/ directory
    fn discover_scripts(skill_dir: &Path) -> Vec<PathBuf> {
        let scripts_dir = skill_dir.join("scripts");
        if !scripts_dir.exists() {
            return Vec::new();
        }

        std::fs::read_dir(&scripts_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.is_file())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Discover resources in resources/ directory (recursive)
    fn discover_resources(skill_dir: &Path) -> Vec<PathBuf> {
        let resources_dir = skill_dir.join("resources");
        if !resources_dir.exists() {
            return Vec::new();
        }

        let mut resources = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&resources_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    resources.push(path.clone());
                } else if path.is_dir() {
                    // Recursively scan subdirectories
                    if let Ok(sub_entries) = std::fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if sub_path.is_file() {
                                resources.push(sub_path);
                            }
                        }
                    }
                }
            }
        }

        resources
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

    /// Convert to SkillPackage for use with the SDK
    pub fn to_skill_package(&self) -> crate::skills::types::SkillPackage {
        use crate::skills::types::{SkillMetadata, SkillResources};

        // Collect all resource folder paths
        let mut resource_folders = Vec::new();
        if self.skill_dir.join("resources").exists() {
            resource_folders.push(self.skill_dir.join("resources"));
        }

        SkillPackage {
            metadata: SkillMetadata {
                id: format!(
                    "skill.{}",
                    self.metadata.name.to_lowercase().replace(' ', "-")
                ),
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
                folders: resource_folders,
                tools: vec![],
                tests: vec![],
            },
        }
    }
}

/// Scanner for discovering skills from .claude/skills/ directories
pub struct SkillsDirScanner {
    base_dir: PathBuf,
}

impl SkillsDirScanner {
    /// Create a new scanner with a custom base directory
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Path to the skills directory
    ///
    /// # Example
    ///
    /// ```no_run
    /// use claude_agent_sdk_rs::skills::skill_md::SkillsDirScanner;
    ///
    /// let scanner = SkillsDirScanner::new("/path/to/skills");
    /// let skills = scanner.scan()?;
    /// ```
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Create a new scanner for project .claude/skills/ directory
    ///
    /// # Arguments
    ///
    /// * `project_dir` - Path to the project root directory
    ///
    /// # Example
    ///
    /// ```no_run
    /// use claude_agent_sdk_rs::skills::skill_md::SkillsDirScanner;
    ///
    /// let scanner = SkillsDirScanner::from_project_dir("/my/project");
    /// let skills = scanner.scan()?;
    /// ```
    pub fn from_project_dir<P: AsRef<Path>>(project_dir: P) -> Self {
        Self {
            base_dir: project_dir.as_ref().join(".claude").join("skills"),
        }
    }

    /// Create a new scanner for user ~/.config/claude/skills/ directory
    ///
    /// # Errors
    ///
    /// Returns an error if home directory cannot be determined
    ///
    /// # Example
    ///
    /// ```no_run
    /// use claude_agent_sdk_rs::skills::skill_md::SkillsDirScanner;
    ///
    /// let scanner = SkillsDirScanner::from_user_dir()?;
    /// let skills = scanner.scan()?;
    /// ```
    pub fn from_user_dir() -> Result<Self, SkillMdError> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| SkillMdError::IoError(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Home directory not found"
                )
            ))?;

        Ok(Self {
            base_dir: PathBuf::from(home)
                .join(".config")
                .join("claude")
                .join("skills"),
        })
    }

    /// Scan the skills directory and load all SKILL.md files
    ///
    /// Returns an empty Vec if the directory doesn't exist (not an error)
    ///
    /// # Returns
    ///
    /// A vector of successfully parsed SkillMdFile objects
    ///
    /// # Example
    ///
    /// ```no_run
    /// let scanner = SkillsDirScanner::from_project_dir(".");
    /// let skills = scanner.scan()?;
    /// for skill in skills {
    ///     println!("Found skill: {}", skill.metadata.name);
    /// }
    /// ```
    pub fn scan(&self) -> Result<Vec<SkillMdFile>, SkillMdError> {
        if !self.base_dir.exists() {
            // Return empty if directory doesn't exist (not an error)
            tracing::debug!(
                "Skills directory does not exist: {:?}",
                self.base_dir
            );
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();

        // Read entries in skills directory
        let entries = std::fs::read_dir(&self.base_dir)
            .map_err(|e| SkillMdError::IoError(e))?;

        for entry in entries {
            let entry = entry.map_err(|e| SkillMdError::IoError(e))?;
            let skill_dir = entry.path();

            // Skip if not a directory
            if !skill_dir.is_dir() {
                continue;
            }

            // Look for SKILL.md file
            let skill_md = skill_dir.join("SKILL.md");
            if skill_md.exists() {
                match SkillMdFile::parse(&skill_md) {
                    Ok(skill) => {
                        tracing::info!(
                            "Loaded skill '{}' from {:?}",
                            skill.metadata.name,
                            skill_md
                        );
                        skills.push(skill);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load skill from {:?}: {}",
                            skill_md, e
                        );
                        // Continue loading other skills
                    }
                }
            } else {
                tracing::debug!(
                    "No SKILL.md found in {:?}",
                    skill_dir
                );
            }
        }

        Ok(skills)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_skill_md() {
        let content = r#"---
name: "Test Skill"
description: "A test skill"
version: "2.0.0"
author: "Test Author"
tags:
  - test
  - example
dependencies:
  - other-skill
---

# Test Skill

This is a test skill with some content.

## Features

- Feature 1
- Feature 2
"#;

        let (metadata, content) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "Test Skill");
        assert_eq!(metadata.description, "A test skill");
        assert_eq!(metadata.version, "2.0.0");
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert_eq!(metadata.tags, vec!["test", "example"]);
        assert_eq!(metadata.dependencies, vec!["other-skill"]);
        assert!(content.contains("This is a test skill"));
        assert!(content.contains("Feature 1"));
    }

    #[test]
    fn test_parse_minimal_skill_md() {
        let content = r#"---
name: "Minimal"
description: "Minimal skill"
---

# Minimal

Content here.
"#;

        let (metadata, content) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "Minimal");
        assert_eq!(metadata.version, "1.0.0"); // default
        assert!(metadata.author.is_none());
        assert!(metadata.tags.is_empty());
        assert!(metadata.dependencies.is_empty());
        assert!(content.contains("Content here"));
    }

    #[test]
    fn test_parse_skill_md_with_content_containing_dashes() {
        let content = r#"---
name: "Test"
description: "Test"
---

# Content with dashes

---

Another section.
"#;

        let (metadata, content) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "Test");
        assert!(content.contains("Content with dashes"));
        assert!(content.contains("Another section"));
    }

    #[test]
    fn test_parse_invalid_no_frontmatter() {
        let content = r#"# Invalid
No frontmatter here.
"#;

        let result = SkillMdFile::parse_frontmatter(content);
        assert!(matches!(result, Err(SkillMdError::InvalidFormat)));
    }

    #[test]
    fn test_parse_missing_required_fields() {
        // Missing name
        let content1 = r#"---
description: "Test"
---

# Content
"#;
        let result1 = SkillMdFile::parse_frontmatter(content1);
        assert!(matches!(result1, Err(SkillMdError::MissingField(_))));

        // Missing description
        let content2 = r#"---
name: "Test"
---

# Content
"#;
        let result2 = SkillMdFile::parse_frontmatter(content2);
        assert!(matches!(result2, Err(SkillMdError::MissingField(_))));
    }

    #[test]
    fn test_parse_advanced_metadata_allowed_tools() {
        let content = r#"---
name: "Test"
description: "Test with tool restrictions"
allowed_tools:
  - Read
  - Grep
  - "Bash(python:*)"
---

# Content
"#;

        let (metadata, _) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "Test");
        assert!(metadata.allowed_tools.is_some());
        let tools = metadata.allowed_tools.unwrap();
        assert_eq!(tools, vec!["Read", "Grep", "Bash(python:*)"]);
    }

    #[test]
    fn test_parse_advanced_metadata_model() {
        let content = r#"---
name: "Test"
description: "Test with specific model"
model: "claude-sonnet-4-20250514"
---

# Content
"#;

        let (metadata, _) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.model, Some("claude-sonnet-4-20250514".to_string()));
    }

    #[test]
    fn test_parse_advanced_metadata_context_fork() {
        let content = r#"---
name: "Test"
description: "Test with fork context"
context: fork
agent: general-purpose
---

# Content
"#;

        let (metadata, _) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.context, Some(SkillContext::Fork));
        assert_eq!(metadata.agent, Some("general-purpose".to_string()));
    }

    #[test]
    fn test_parse_advanced_metadata_hooks() {
        let content = r#"---
name: "Test"
description: "Test with hooks"
hooks:
  PreToolUse:
    - matcher: "Bash"
      command: "./scripts/security-check.sh $TOOL_INPUT"
      once: true
      type: command
---

# Content
"#;

        let (metadata, _) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert!(metadata.hooks.is_some());
        let hooks = metadata.hooks.unwrap();
        assert!(hooks.pre_tool_use.is_some());
        let pre_hooks = hooks.pre_tool_use.unwrap();
        assert_eq!(pre_hooks.len(), 1);
        assert_eq!(pre_hooks[0].matcher, "Bash");
        assert_eq!(pre_hooks[0].command, "./scripts/security-check.sh $TOOL_INPUT");
        assert_eq!(pre_hooks[0].once, Some(true));
        assert_eq!(pre_hooks[0].r#type, Some(HookType::Command));
    }

    #[test]
    fn test_parse_advanced_metadata_user_invocable() {
        let content1 = r#"---
name: "Test"
description: "Test hidden from menu"
user-invocable: false
---

# Content
"#;

        let (metadata1, _) = SkillMdFile::parse_frontmatter(content1).unwrap();
        assert_eq!(metadata1.user_invocable, false);

        // Default should be true
        let content2 = r#"---
name: "Test"
description: "Test default user invocable"
---

# Content
"#;

        let (metadata2, _) = SkillMdFile::parse_frontmatter(content2).unwrap();
        assert_eq!(metadata2.user_invocable, true);
    }

    #[test]
    fn test_parse_complete_advanced_metadata() {
        let content = r#"---
name: "Advanced Test"
description: "Test all advanced fields. Use when working with advanced testing scenarios."
version: "2.0.0"
author: "Test Author <test@example.com>"
tags:
  - advanced
  - testing
dependencies:
  - base-test
allowed_tools:
  - Read
  - Grep
  - "Bash(python:*)"
model: "claude-sonnet-4-20250514"
context: fork
agent: general-purpose
hooks:
  PreToolUse:
    - matcher: "Bash"
      command: "./scripts/check.sh"
      once: true
  PostToolUse:
    - matcher: "*"
      command: "./scripts/notify.sh"
user-invocable: true
disable-model-invocation: false
---

# Advanced Test Skill

This is a comprehensive test of all metadata fields.
"#;

        let (metadata, content) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "Advanced Test");
        assert!(metadata.description.contains("advanced testing"));
        assert_eq!(metadata.version, "2.0.0");
        assert_eq!(metadata.author, Some("Test Author <test@example.com>".to_string()));
        assert_eq!(metadata.tags, vec!["advanced", "testing"]);
        assert_eq!(metadata.dependencies, vec!["base-test"]);
        assert!(metadata.allowed_tools.is_some());
        assert_eq!(metadata.model, Some("claude-sonnet-4-20250514".to_string()));
        assert_eq!(metadata.context, Some(SkillContext::Fork));
        assert_eq!(metadata.agent, Some("general-purpose".to_string()));
        assert!(metadata.hooks.is_some());
        assert_eq!(metadata.user_invocable, true);
        assert_eq!(metadata.disable_model_invocation, Some(false));
        assert!(content.contains("comprehensive test"));
    }
}
