//! SKILL.md file parser for Claude Code compatibility
//!
//! This module provides functionality to parse and load SKILL.md files
//! in the same format as Claude Code CLI, including YAML frontmatter
//! and markdown content.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

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
}
