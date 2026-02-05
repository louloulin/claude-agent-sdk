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

    // === Validation Errors ===

    #[error("Name exceeds maximum length of 64 characters (got {0} characters)")]
    NameTooLong(usize),

    #[error("Name must contain only lowercase letters, numbers, and hyphens")]
    InvalidNameFormat,

    #[error("Name cannot contain reserved words 'anthropic' or 'claude'")]
    ReservedWord,

    #[error("Name cannot contain XML tags")]
    NameContainsXmlTags,

    #[error("Description cannot be empty")]
    DescriptionEmpty,

    #[error("Description exceeds maximum length of 1024 characters (got {0} characters)")]
    DescriptionTooLong(usize),

    #[error("Description cannot contain XML tags")]
    DescriptionContainsXmlTags,
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

impl SkillMdMetadata {
    /// Validate metadata according to Claude Skills specification
    ///
    /// Official requirements from:
    /// https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview
    ///
    /// Name requirements:
    /// - Maximum 64 characters
    /// - Only lowercase letters, numbers, and hyphens
    /// - Cannot contain XML tags
    /// - Cannot contain reserved words: "anthropic", "claude"
    ///
    /// Description requirements:
    /// - Must be non-empty
    /// - Maximum 1024 characters
    /// - Cannot contain XML tags
    pub fn validate(&self) -> Result<(), SkillMdError> {
        // Validate name
        self.validate_name()?;
        self.validate_description()?;
        Ok(())
    }

    fn validate_name(&self) -> Result<(), SkillMdError> {
        // Check length
        if self.name.len() > 64 {
            return Err(SkillMdError::NameTooLong(self.name.len()));
        }

        // Check format: lowercase letters, numbers, hyphens only
        if !self.name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(SkillMdError::InvalidNameFormat);
        }

        // Check for reserved words (case-insensitive)
        let lower_name = self.name.to_lowercase();
        if lower_name.contains("anthropic") || lower_name.contains("claude") {
            return Err(SkillMdError::ReservedWord);
        }

        // Check for XML tags
        if self.name.contains('<') || self.name.contains('>') {
            return Err(SkillMdError::NameContainsXmlTags);
        }

        Ok(())
    }

    fn validate_description(&self) -> Result<(), SkillMdError> {
        // Check empty
        if self.description.trim().is_empty() {
            return Err(SkillMdError::DescriptionEmpty);
        }

        // Check length
        if self.description.len() > 1024 {
            return Err(SkillMdError::DescriptionTooLong(self.description.len()));
        }

        // Check for XML tags
        if self.description.contains('<') || self.description.contains('>') {
            return Err(SkillMdError::DescriptionContainsXmlTags);
        }

        Ok(())
    }
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
    /// Associated resources from resources/ directory (for backward compatibility)
    pub resources: Vec<PathBuf>,
    /// Reference file if exists
    pub reference: Option<PathBuf>,
    /// Forms file if exists
    pub forms: Option<PathBuf>,
    /// Resource cache for progressive disclosure (maps name to path)
    _resource_cache: Option<std::collections::HashMap<String, PathBuf>>,
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
    /// use claude_agent_sdk::skills::skill_md::SkillMdFile;
    ///
    /// let skill = SkillMdFile::parse(".claude/skills/my-skill/SKILL.md")?;
    /// println!("Loaded skill: {}", skill.metadata.name);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse<P: AsRef<Path>>(skill_md_path: P) -> Result<Self, SkillMdError> {
        let path = skill_md_path.as_ref();
        let skill_dir = path
            .parent()
            .ok_or(SkillMdError::InvalidFormat)?;

        // Read the file
        let content = std::fs::read_to_string(path)?;

        // Split frontmatter and content
        let (metadata, content) = Self::parse_frontmatter(&content)?;

        // Discover associated files
        let scripts = Self::discover_scripts(skill_dir);
        let resources = Self::discover_resources(skill_dir);
        let reference = Self::check_file_exists(skill_dir, "reference.md");
        let forms = Self::check_file_exists(skill_dir, "forms.md");

        // Build resource cache for progressive disclosure
        let resource_cache = Self::build_resource_cache(&resources);

        Ok(Self {
            metadata,
            content,
            skill_dir: skill_dir.to_path_buf(),
            scripts,
            resources,
            reference,
            forms,
            _resource_cache: Some(resource_cache),
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

        // Validate metadata according to Claude Skills specification
        metadata.validate()?;

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

    /// Build a resource cache from discovered resources
    ///
    /// This creates a HashMap mapping resource names to their full paths
    /// for quick lookup via get_resource()
    fn build_resource_cache(resources: &[PathBuf]) -> std::collections::HashMap<String, PathBuf> {
        let mut cache = std::collections::HashMap::new();
        for resource_path in resources {
            if let Some(file_name) = resource_path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    cache.insert(name_str.to_string(), resource_path.clone());
                }
            }
        }
        cache
    }

    /// Get a resource by name from the resource cache
    ///
    /// This provides progressive disclosure - resources are indexed at parse time
    /// but can be efficiently retrieved by name without scanning.
    ///
    /// # Arguments
    ///
    /// * `name` - The resource filename (e.g., "config.json")
    ///
    /// # Returns
    ///
    /// * `Some(PathBuf)` - Full path to the resource if found
    /// * `None` - Resource not found
    ///
    /// # Example
    ///
    /// ```no_run
    /// use claude_agent_sdk::skills::skill_md::SkillMdFile;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let skill = SkillMdFile::parse(".claude/skills/my-skill/SKILL.md")?;
    /// if let Some(config_path) = skill.get_resource("config.json") {
    ///     let config_content = std::fs::read_to_string(config_path)?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_resource(&self, name: &str) -> Option<&PathBuf> {
        self._resource_cache.as_ref()?.get(name)
    }

    /// Get all resource names from the cache
    ///
    /// # Returns
    ///
    /// A vector of resource filenames available in this skill
    pub fn get_resource_names(&self) -> Vec<String> {
        self._resource_cache
            .as_ref()
            .map(|cache| cache.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if a resource exists by name
    ///
    /// This is a convenience method that combines resource cache lookup
    /// with existence checking.
    ///
    /// # Arguments
    ///
    /// * `name` - The resource filename to check
    ///
    /// # Returns
    ///
    /// * `true` - Resource exists
    /// * `false` - Resource does not exist
    pub fn has_resource(&self, name: &str) -> bool {
        self.get_resource(name).is_some()
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
    /// use claude_agent_sdk::skills::skill_md::SkillsDirScanner;
    ///
    /// let scanner = SkillsDirScanner::new("/path/to/skills");
    /// let skills = scanner.scan()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
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
    /// use claude_agent_sdk::skills::skill_md::SkillsDirScanner;
    ///
    /// let scanner = SkillsDirScanner::from_project_dir("/my/project");
    /// let skills = scanner.scan()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
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
    /// use claude_agent_sdk::skills::skill_md::SkillsDirScanner;
    ///
    /// let scanner = SkillsDirScanner::from_user_dir()?;
    /// let skills = scanner.scan()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
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
    /// use claude_agent_sdk::skills::skill_md::SkillsDirScanner;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let scanner = SkillsDirScanner::from_project_dir(".");
    /// let skills = scanner.scan()?;
    /// for skill in skills {
    ///     println!("Found skill: {}", skill.metadata.name);
    /// }
    /// # Ok(())
    /// # }
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
            .map_err(SkillMdError::IoError)?;

        for entry in entries {
            let entry = entry.map_err(SkillMdError::IoError)?;
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

    /// Scan the skills directory and load all SKILL.md files in parallel
    ///
    /// This is an asynchronous version that uses parallel processing for better performance
    /// when loading multiple skills. Each skill file is parsed in a separate task using
    /// `spawn_blocking` to avoid blocking the async runtime.
    ///
    /// Returns an empty Vec if the directory doesn't exist (not an error)
    ///
    /// # Returns
    ///
    /// A vector of successfully parsed SkillMdFile objects
    ///
    /// # Performance
    ///
    /// For directories with many skills, this method can provide significant speedup
    /// compared to the synchronous `scan()` method, as I/O operations and parsing
    /// happen concurrently.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use claude_agent_sdk::skills::skill_md::SkillsDirScanner;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let scanner = SkillsDirScanner::from_project_dir(".");
    /// let skills = scanner.scan_parallel().await?;
    /// for skill in skills {
    ///     println!("Found skill: {}", skill.metadata.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn scan_parallel(&self) -> Result<Vec<SkillMdFile>, SkillMdError> {
        if !self.base_dir.exists() {
            // Return empty if directory doesn't exist (not an error)
            tracing::debug!(
                "Skills directory does not exist: {:?}",
                self.base_dir
            );
            return Ok(Vec::new());
        }

        // Read entries in skills directory
        let entries = std::fs::read_dir(&self.base_dir)
            .map_err(SkillMdError::IoError)?;

        // Collect all skill directory paths
        let skill_dirs: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect();

        // Create parsing futures for each skill directory
        let parse_futures: Vec<_> = skill_dirs
            .into_iter()
            .filter_map(|skill_dir| {
                let skill_md = skill_dir.join("SKILL.md");
                if skill_md.exists() {
                    let skill_md_clone = skill_md.clone();
                    Some(async move {
                        tokio::task::spawn_blocking(move || {
                            SkillMdFile::parse(&skill_md)
                        })
                        .await
                        .unwrap_or_else(|e| {
                            tracing::error!(
                                "Task failed for {:?}: {}",
                                skill_md_clone, e
                            );
                            Err(SkillMdError::IoError(std::io::Error::other(
                                "Task execution failed"
                            )))
                        })
                    })
                } else {
                    tracing::debug!("No SKILL.md found in {:?}", skill_dir);
                    None
                }
            })
            .collect();

        // Execute all parsing tasks in parallel
        let results = futures::future::join_all(parse_futures).await;

        // Collect successfully parsed skills
        let mut skills = Vec::new();
        for result in results {
            match result {
                Ok(skill) => {
                    tracing::info!(
                        "Loaded skill '{}' from parallel scan",
                        skill.metadata.name
                    );
                    skills.push(skill);
                }
                Err(e) => {
                    tracing::warn!("Failed to load skill during parallel scan: {}", e);
                    // Continue loading other skills
                }
            }
        }

        tracing::info!(
            "Parallel scan completed: {} skills loaded",
            skills.len()
        );

        Ok(skills)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_skill_md() {
        let content = r#"---
name: test-skill
description: A test skill
version: 2.0.0
author: Test Author
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
        assert_eq!(metadata.name, "test-skill");
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
name: minimal-skill
description: Minimal skill
---

# Minimal

Content here.
"#;

        let (metadata, content) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "minimal-skill");
        assert_eq!(metadata.version, "1.0.0"); // default
        assert!(metadata.author.is_none());
        assert!(metadata.tags.is_empty());
        assert!(metadata.dependencies.is_empty());
        assert!(content.contains("Content here"));
    }

    #[test]
    fn test_parse_skill_md_with_content_containing_dashes() {
        let content = r#"---
name: test-skill
description: Test
---

# Content with dashes

---

Another section.
"#;

        let (metadata, content) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "test-skill");
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
        // Missing name - YAML parsing succeeds but validation should catch it
        let content1 = r#"---
description: Test
---

# Content
"#;
        let result1 = SkillMdFile::parse_frontmatter(content1);
        // serde_yaml will succeed with empty string, but our validation should catch it
        assert!(result1.is_err(), "Should fail when name is missing");

        // Missing description
        let content2 = r#"---
name: test-skill
---

# Content
"#;
        let result2 = SkillMdFile::parse_frontmatter(content2);
        assert!(result2.is_err(), "Should fail when description is missing");
    }

    #[test]
    fn test_parse_advanced_metadata_allowed_tools() {
        let content = r#"---
name: test-skill
description: Test with tool restrictions
allowed_tools:
  - Read
  - Grep
  - "Bash(python:*)"
---

# Content
"#;

        let (metadata, _) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "test-skill");
        assert!(metadata.allowed_tools.is_some());
        let tools = metadata.allowed_tools.unwrap();
        assert_eq!(tools, vec!["Read", "Grep", "Bash(python:*)"]);
    }

    #[test]
    fn test_parse_advanced_metadata_model() {
        let content = r#"---
name: test-skill
description: Test with specific model
model: claude-sonnet-4-20250514
---

# Content
"#;

        let (metadata, _) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.model, Some("claude-sonnet-4-20250514".to_string()));
    }

    #[test]
    fn test_parse_advanced_metadata_context_fork() {
        let content = r#"---
name: test-skill
description: Test with fork context
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
name: test-skill
description: Test with hooks
hooks:
  pre_tool_use:
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
name: test-skill
description: Test hidden from menu
user_invocable: false
---

# Content
"#;

        let (metadata1, _) = SkillMdFile::parse_frontmatter(content1).unwrap();
        assert_eq!(metadata1.user_invocable, false);

        // Default should be true
        let content2 = r#"---
name: test-skill-two
description: Test default user invocable
---

# Content
"#;

        let (metadata2, _) = SkillMdFile::parse_frontmatter(content2).unwrap();
        assert_eq!(metadata2.user_invocable, true);
    }

    #[test]
    fn test_parse_complete_advanced_metadata() {
        let content = r#"---
name: advanced-test-skill
description: Test all advanced fields. Use when working with advanced testing scenarios.
version: 2.0.0
author: Test Author <test@example.com>
tags:
  - advanced
  - testing
dependencies:
  - base-test
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

# Advanced Test Skill

This is a comprehensive test of all metadata fields.
"#;

        let (metadata, content) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "advanced-test-skill");
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

    // === Claude Skills Specification Validation Tests ===

    #[test]
    fn test_validation_valid_skill() {
        let content = r#"---
name: test-skill
description: A valid test skill
---

# Test Skill

Valid content.
"#;

        let (metadata, _) = SkillMdFile::parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, "test-skill");
        // Validation should pass
        metadata.validate().unwrap();
    }

    #[test]
    fn test_validation_name_too_long() {
        let name = "a".repeat(65); // 65 characters, exceeds 64 limit
        let content = format!(r#"---
name: {}
description: Test
---

# Content
"#, name);

        let result = SkillMdFile::parse_frontmatter(&content);
        assert!(matches!(result, Err(SkillMdError::NameTooLong(65))));
    }

    #[test]
    fn test_validation_name_exactly_64_chars() {
        let name = "a".repeat(64); // Exactly 64 characters, should pass
        let content = format!(r#"---
name: {}
description: Test
---

# Content
"#, name);

        let (metadata, _) = SkillMdFile::parse_frontmatter(&content).unwrap();
        metadata.validate().unwrap(); // Should succeed
    }

    #[test]
    fn test_validation_name_uppercase_invalid() {
        let content = r#"---
name: TestSkill
description: Test
---

# Content
"#;

        let result = SkillMdFile::parse_frontmatter(content);
        assert!(matches!(result, Err(SkillMdError::InvalidNameFormat)));
    }

    #[test]
    fn test_validation_name_with_spaces_invalid() {
        let content = r#"---
name: test skill
description: Test
---

# Content
"#;

        let result = SkillMdFile::parse_frontmatter(content);
        assert!(matches!(result, Err(SkillMdError::InvalidNameFormat)));
    }

    #[test]
    fn test_validation_name_with_special_chars_invalid() {
        let content = r#"---
name: test_skill!
description: Test
---

# Content
"#;

        let result = SkillMdFile::parse_frontmatter(content);
        assert!(matches!(result, Err(SkillMdError::InvalidNameFormat)));
    }

    #[test]
    fn test_validation_name_reserved_word_anthropic() {
        let content = r#"---
name: my-anthropic-tool
description: Test
---

# Content
"#;

        let result = SkillMdFile::parse_frontmatter(content);
        assert!(matches!(result, Err(SkillMdError::ReservedWord)));
    }

    #[test]
    fn test_validation_name_reserved_word_claude() {
        let content = r#"---
name: claude-helper
description: Test
---

# Content
"#;

        let result = SkillMdFile::parse_frontmatter(content);
        assert!(matches!(result, Err(SkillMdError::ReservedWord)));
    }

    #[test]
    fn test_validation_name_case_insensitive_reserved() {
        let content = r#"---
name: my-anthropic-tool
description: Test
---

# Content
"#;

        let result = SkillMdFile::parse_frontmatter(content);
        assert!(matches!(result, Err(SkillMdError::ReservedWord)));
    }

    #[test]
    fn test_validation_description_empty() {
        let content = r#"---
name: test-skill
description: "   "
---

# Content
"#;

        let result = SkillMdFile::parse_frontmatter(content);
        assert!(matches!(result, Err(SkillMdError::DescriptionEmpty)));
    }

    #[test]
    fn test_validation_description_too_long() {
        let description = "a".repeat(1025); // 1025 characters, exceeds 1024 limit
        let content = format!(r#"---
name: test-skill
description: {}
---

# Content
"#, description);

        let result = SkillMdFile::parse_frontmatter(&content);
        assert!(matches!(result, Err(SkillMdError::DescriptionTooLong(1025))));
    }

    #[test]
    fn test_validation_description_exactly_1024_chars() {
        let description = "a".repeat(1024); // Exactly 1024 characters, should pass
        let content = format!(r#"---
name: test-skill
description: {}
---

# Content
"#, description);

        let (metadata, _) = SkillMdFile::parse_frontmatter(&content).unwrap();
        metadata.validate().unwrap(); // Should succeed
    }

    #[test]
    fn test_validation_description_with_xml_tags() {
        let content = r#"---
name: test-skill
description: "A test <script>alert('xss')</script>"
---

# Content
"#;

        let result = SkillMdFile::parse_frontmatter(content);
        assert!(matches!(result, Err(SkillMdError::DescriptionContainsXmlTags)));
    }

    #[test]
    fn test_validation_multiple_errors_first_returned() {
        let content = r#"---
name: MyClaudeSkill<script>
description: "   "
---

# Content
"#;

        // Should return the first validation error (name format, not empty description)
        let result = SkillMdFile::parse_frontmatter(content);
        // The name validation runs before description, so we'll get a name-related error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_scan_parallel_nonexistent_directory() {
        let scanner = SkillsDirScanner::new("/nonexistent/path/to/skills");
        let result = scanner.scan_parallel().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_scan_parallel_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let scanner = SkillsDirScanner::new(temp_dir.path());
        let result = scanner.scan_parallel().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_scan_parallel_with_skills() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skills_dir = temp_dir.path();

        // Create multiple skill directories
        for i in 1..=3 {
            let skill_dir = skills_dir.join(format!("skill-{}", i));
            std::fs::create_dir(&skill_dir).unwrap();

            let skill_md_content = format!(
                r#"---
name: test-skill-{}
description: Test skill {}
version: 1.0.0
author: Test Author
---

# Test Skill {}

This is a test skill.
"#,
                i, i, i
            );

            std::fs::write(skill_dir.join("SKILL.md"), skill_md_content).unwrap();
        }

        let scanner = SkillsDirScanner::new(skills_dir);
        let result = scanner.scan_parallel().await;

        assert!(result.is_ok());
        let skills = result.unwrap();
        assert_eq!(skills.len(), 3);

        // Verify all skills were loaded
        let skill_names: Vec<_> = skills.iter().map(|s| s.metadata.name.clone()).collect();
        assert!(skill_names.contains(&"test-skill-1".to_string()));
        assert!(skill_names.contains(&"test-skill-2".to_string()));
        assert!(skill_names.contains(&"test-skill-3".to_string()));
    }

    #[tokio::test]
    async fn test_scan_parallel_handles_invalid_skills() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skills_dir = temp_dir.path();

        // Create a valid skill
        let valid_skill_dir = skills_dir.join("valid-skill");
        std::fs::create_dir(&valid_skill_dir).unwrap();
        std::fs::write(
            valid_skill_dir.join("SKILL.md"),
            r#"---
name: valid-skill
description: A valid skill
version: 1.0.0
---

# Valid Skill
"#,
        )
        .unwrap();

        // Create an invalid skill (missing required fields)
        let invalid_skill_dir = skills_dir.join("invalid-skill");
        std::fs::create_dir(&invalid_skill_dir).unwrap();
        std::fs::write(
            invalid_skill_dir.join("SKILL.md"),
            r#"---
name: ""
---

# Invalid Skill
"#,
        )
        .unwrap();

        let scanner = SkillsDirScanner::new(skills_dir);
        let result = scanner.scan_parallel().await;

        assert!(result.is_ok());
        let skills = result.unwrap();
        // Should only load the valid skill
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].metadata.name, "valid-skill");
    }

    #[tokio::test]
    async fn test_scan_parallel_vs_sync_consistency() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skills_dir = temp_dir.path();

        // Create multiple skills
        for i in 1..=5 {
            let skill_dir = skills_dir.join(format!("skill-{}", i));
            std::fs::create_dir(&skill_dir).unwrap();

            let skill_md_content = format!(
                r#"---
name: skill-{}
description: Skill number {}
version: 1.0.0
---

# Skill {}
"#,
                i, i, i
            );

            std::fs::write(skill_dir.join("SKILL.md"), skill_md_content).unwrap();
        }

        let scanner = SkillsDirScanner::new(skills_dir);

        // Run sync scan
        let sync_result = scanner.scan();
        assert!(sync_result.is_ok());
        let sync_skills = sync_result.unwrap();

        // Run parallel scan (directly await, no new runtime)
        let parallel_result = scanner.scan_parallel().await;
        assert!(parallel_result.is_ok());
        let parallel_skills = parallel_result.unwrap();

        // Both should load the same number of skills
        assert_eq!(sync_skills.len(), parallel_skills.len());
        assert_eq!(sync_skills.len(), 5);

        // Skill names should match (order might differ)
        let sync_names: std::collections::HashSet<_> =
            sync_skills.iter().map(|s| s.metadata.name.clone()).collect();
        let parallel_names: std::collections::HashSet<_> =
            parallel_skills.iter().map(|s| s.metadata.name.clone()).collect();

        assert_eq!(sync_names, parallel_names);
    }

    #[test]
    fn test_progressive_disclosure_resource_cache() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        // Create resources directory with some files
        let resources_dir = skill_dir.join("resources");
        std::fs::create_dir(&resources_dir).unwrap();
        std::fs::write(resources_dir.join("config.json"), "{}").unwrap();
        std::fs::write(resources_dir.join("data.txt"), "data").unwrap();

        // Create SKILL.md
        std::fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: test-skill
description: Test skill
version: 1.0.0
---

# Test Skill
"#,
        )
        .unwrap();

        // Parse the skill
        let skill = SkillMdFile::parse(skill_dir.join("SKILL.md")).unwrap();

        // Resource cache should be built
        assert!(skill._resource_cache.is_some());

        // Test get_resource
        assert!(skill.get_resource("config.json").is_some());
        assert!(skill.get_resource("data.txt").is_some());
        assert!(skill.get_resource("nonexistent.json").is_none());
    }

    #[test]
    fn test_get_resource_names() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        // Create resources directory
        let resources_dir = skill_dir.join("resources");
        std::fs::create_dir(&resources_dir).unwrap();
        std::fs::write(resources_dir.join("file1.txt"), "content1").unwrap();
        std::fs::write(resources_dir.join("file2.txt"), "content2").unwrap();

        // Create SKILL.md
        std::fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: test-skill
description: Test skill
version: 1.0.0
---

# Test Skill
"#,
        )
        .unwrap();

        let skill = SkillMdFile::parse(skill_dir.join("SKILL.md")).unwrap();

        // Get resource names
        let names = skill.get_resource_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"file1.txt".to_string()));
        assert!(names.contains(&"file2.txt".to_string()));
    }

    #[test]
    fn test_has_resource() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        // Create resources directory
        let resources_dir = skill_dir.join("resources");
        std::fs::create_dir(&resources_dir).unwrap();
        std::fs::write(resources_dir.join("exists.txt"), "content").unwrap();

        // Create SKILL.md
        std::fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: test-skill
description: Test skill
version: 1.0.0
---

# Test Skill
"#,
        )
        .unwrap();

        let skill = SkillMdFile::parse(skill_dir.join("SKILL.md")).unwrap();

        // Test has_resource
        assert!(skill.has_resource("exists.txt"));
        assert!(!skill.has_resource("missing.txt"));
    }

    #[test]
    fn test_progressive_disclosure_no_resources() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        // Create SKILL.md without resources
        std::fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: test-skill
description: Test skill
version: 1.0.0
---

# Test Skill
"#,
        )
        .unwrap();

        let skill = SkillMdFile::parse(skill_dir.join("SKILL.md")).unwrap();

        // Resource cache should still be built but empty
        assert!(skill._resource_cache.is_some());
        assert_eq!(skill.get_resource_names().len(), 0);
        assert!(skill.get_resource("anything").is_none());
        assert!(!skill.has_resource("anything"));
    }

    #[test]
    fn test_backward_compatible_resources_list() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        // Create resources directory
        let resources_dir = skill_dir.join("resources");
        std::fs::create_dir(&resources_dir).unwrap();
        std::fs::write(resources_dir.join("res1.txt"), "content1").unwrap();
        std::fs::write(resources_dir.join("res2.txt"), "content2").unwrap();

        // Create SKILL.md
        std::fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: test-skill
description: Test skill
version: 1.0.0
---

# Test Skill
"#,
        )
        .unwrap();

        let skill = SkillMdFile::parse(skill_dir.join("SKILL.md")).unwrap();

        // Old API still works
        assert!(!skill.resources.is_empty());
        assert_eq!(skill.resources.len(), 2);

        // New API provides the same resources
        assert_eq!(skill.get_resource_names().len(), 2);
    }
}
