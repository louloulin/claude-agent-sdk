//! # VS Code Skills Format Export
//!
//! This module provides functionality to export skills in VS Code's SKILL.md format,
//! which includes YAML frontmatter with metadata and markdown content.

use crate::skills::error::SkillError;
use crate::skills::types::SkillPackage;
use std::fs;
use std::io::Write;
use std::path::Path;

/// VS Code Skill format configuration
#[derive(Debug, Clone)]
pub struct VsCodeExportConfig {
    /// Include dependencies in the export
    pub include_dependencies: bool,

    /// Include resource references in the export
    pub include_resources: bool,

    /// Add usage examples section
    pub include_examples: bool,

    /// Custom footer to add to the markdown
    pub footer: Option<String>,
}

impl Default for VsCodeExportConfig {
    fn default() -> Self {
        Self {
            include_dependencies: true,
            include_resources: true,
            include_examples: true,
            footer: None,
        }
    }
}

impl VsCodeExportConfig {
    /// Create a new export config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to include dependencies
    pub fn with_dependencies(mut self, include: bool) -> Self {
        self.include_dependencies = include;
        self
    }

    /// Set whether to include resources
    pub fn with_resources(mut self, include: bool) -> Self {
        self.include_resources = include;
        self
    }

    /// Set whether to include examples
    pub fn with_examples(mut self, include: bool) -> Self {
        self.include_examples = include;
        self
    }

    /// Set custom footer
    pub fn with_footer(mut self, footer: String) -> Self {
        self.footer = Some(footer);
        self
    }
}

/// Utility functions for VS Code Skills format
pub struct VsCodeUtils;

impl VsCodeUtils {
    /// Normalize a skill name to VS Code format
    ///
    /// Rules:
    /// - Lowercase only
    /// - Only alphanumeric characters and hyphens
    /// - Must start with a letter
    /// - Maximum 64 characters
    ///
    /// # Examples
    /// ```
    /// # use claude_agent_sdk::skills::vscode::VsCodeUtils;
    /// assert_eq!(VsCodeUtils::normalize_name("My Skill 123"), "my-skill-123");
    /// assert_eq!(VsCodeUtils::normalize_name("Test_API"), "test-api");
    /// ```
    pub fn normalize_name(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .filter_map(|c| {
                if c.is_alphanumeric() {
                    Some(c)
                } else if c.is_whitespace() || c == '_' || c == '-' {
                    Some('-')
                } else {
                    None
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
            .trim_start_matches(|c: char| !c.is_alphabetic())
            .to_string()
    }

    /// Validate a skill name according to VS Code rules
    pub fn validate_name(name: &str) -> Result<(), SkillError> {
        if name.is_empty() {
            return Err(SkillError::Validation("Name cannot be empty".to_string()));
        }

        if name.len() > 64 {
            return Err(SkillError::Validation(
                "Name must be 64 characters or less".to_string(),
            ));
        }

        if !name
            .chars()
            .next()
            .map(|c| c.is_alphabetic())
            .unwrap_or(false)
        {
            return Err(SkillError::Validation(
                "Name must start with a letter".to_string(),
            ));
        }

        if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(SkillError::Validation(
                "Name can only contain lowercase letters, numbers, and hyphens".to_string(),
            ));
        }

        if !name
            .chars()
            .all(|c| c.is_lowercase() || c.is_numeric() || c == '-')
        {
            return Err(SkillError::Validation("Name must be lowercase".to_string()));
        }

        Ok(())
    }

    /// Validate description length (should be concise)
    pub fn validate_description(description: &str) -> Result<(), SkillError> {
        if description.is_empty() {
            return Err(SkillError::Validation(
                "Description cannot be empty".to_string(),
            ));
        }

        if description.len() > 200 {
            return Err(SkillError::Validation(
                "Description should be 200 characters or less for clarity".to_string(),
            ));
        }

        Ok(())
    }
}

/// Export a skill package to VS Code SKILL.md format
pub fn export_to_vscode<P: AsRef<Path>>(
    skill: &SkillPackage,
    output_path: P,
    config: &VsCodeExportConfig,
) -> Result<(), SkillError> {
    let output_path = output_path.as_ref();

    // Normalize and validate name
    let normalized_name = VsCodeUtils::normalize_name(&skill.metadata.name);
    VsCodeUtils::validate_name(&normalized_name)?;

    // Validate description
    let description = skill.metadata.description.clone();
    VsCodeUtils::validate_description(&description)?;

    // Build the SKILL.md content
    let mut content = String::new();

    // YAML Frontmatter
    content.push_str("---\n");
    content.push_str(&format!("name: {}\n", normalized_name));
    content.push_str(&format!("description: {}\n", description));

    // Add version if available
    if !skill.metadata.version.is_empty() {
        content.push_str(&format!("version: {}\n", skill.metadata.version));
    }

    // Add author if available
    if let Some(ref author) = skill.metadata.author {
        content.push_str(&format!("author: {}\n", author));
    }

    // Add tags if available
    if !skill.metadata.tags.is_empty() {
        content.push_str(&format!("tags: [{}]\n", skill.metadata.tags.join(", ")));
    }

    content.push_str("---\n\n");

    // Instructions section
    if !skill.instructions.is_empty() {
        content.push_str("# Instructions\n\n");
        content.push_str(&skill.instructions);
        content.push_str("\n\n");
    }

    // Scripts section
    if !skill.scripts.is_empty() && config.include_resources {
        content.push_str("## Scripts\n\n");
        for (i, script) in skill.scripts.iter().enumerate() {
            content.push_str(&format!("### Script {}\n\n", i + 1));
            content.push_str("```");
            // Try to detect language from shebang or extension
            if script.contains("#!/bin/bash") || script.contains("#!/bin/sh") {
                content.push_str("bash");
            } else if script.contains("#!/usr/bin/env python") {
                content.push_str("python");
            } else if script.contains("fn ") && script.contains("{") {
                content.push_str("rust");
            } else {
                content.push_str("text");
            }
            content.push('\n');
            content.push_str(script);
            content.push_str("\n```\n\n");
        }
    }

    // Dependencies section
    if !skill.metadata.dependencies.is_empty() && config.include_dependencies {
        content.push_str("## Dependencies\n\n");
        content.push_str("This skill requires the following dependencies:\n\n");
        for dep in &skill.metadata.dependencies {
            content.push_str(&format!("- {}\n", dep));
        }
        content.push('\n');
    }

    // Resources section
    if config.include_resources {
        let has_folders = !skill.resources.folders.is_empty();
        let has_tools = !skill.resources.tools.is_empty();
        let has_tests = !skill.resources.tests.is_empty();

        if has_folders || has_tools || has_tests {
            content.push_str("## Resources\n\n");

            if has_folders {
                content.push_str("### Folders\n\n");
                for folder in &skill.resources.folders {
                    content.push_str(&format!("- `{}`\n", folder.display()));
                }
                content.push('\n');
            }

            if has_tools {
                content.push_str("### Tools\n\n");
                for tool in &skill.resources.tools {
                    content.push_str(&format!("- {}\n", tool));
                }
                content.push('\n');
            }

            if has_tests {
                content.push_str("### Tests\n\n");
                for test in &skill.resources.tests {
                    content.push_str(&format!("- {}\n", test));
                }
                content.push('\n');
            }
        }
    }

    // Examples section
    if config.include_examples {
        content.push_str("## Usage Examples\n\n");
        content.push_str("### Basic Usage\n\n");
        content.push_str("```rust\n");
        content.push_str("// Import the skill\n");
        content.push_str("use claude_agent_sdk::skills::SkillPackage;\n\n");
        content.push_str("// Load the skill from the installed directory\n");
        content.push_str("let skill = SkillPackage::load(\"");
        content.push_str(&skill.metadata.name);
        content.push_str("\").unwrap();\n\n");
        content.push_str("// Use the skill with an agent\n");
        content.push_str("let agent = Agent::new()\n");
        content.push_str("    .with_skill(skill);\n");
        content.push_str("```\n\n");
    }

    // Footer
    if let Some(ref footer) = config.footer {
        content.push_str("---\n\n");
        content.push_str(footer);
        content.push('\n');
    }

    // Write to file
    let mut file = fs::File::create(output_path)
        .map_err(|e| SkillError::Io(format!("Failed to create SKILL.md file: {}", e)))?;

    file.write_all(content.as_bytes())
        .map_err(|e| SkillError::Io(format!("Failed to write SKILL.md file: {}", e)))?;

    Ok(())
}

/// Export multiple skills to a directory
pub fn export_batch_to_vscode<P: AsRef<Path>>(
    skills: &[SkillPackage],
    output_dir: P,
    config: &VsCodeExportConfig,
) -> Result<Vec<String>, SkillError> {
    let output_dir = output_dir.as_ref();

    // Create output directory if it doesn't exist
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)
            .map_err(|e| SkillError::Io(format!("Failed to create output directory: {}", e)))?;
    }

    let mut exported = Vec::new();

    for skill in skills {
        let normalized_name = VsCodeUtils::normalize_name(&skill.metadata.name);
        let file_name = format!("{}.md", normalized_name);
        let file_path = output_dir.join(&file_name);

        export_to_vscode(skill, &file_path, config)?;

        exported.push(file_path.to_string_lossy().to_string());
    }

    Ok(exported)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::types::{SkillMetadata, SkillResources};
    use uuid::Uuid;

    fn create_test_skill(name: &str, description: &str) -> SkillPackage {
        SkillPackage {
            metadata: SkillMetadata {
                id: Uuid::new_v4().to_string(),
                name: name.to_string(),
                description: description.to_string(),
                version: "1.0.0".to_string(),
                author: Some("Test Author".to_string()),
                dependencies: vec!["dep1".to_string(), "dep2".to_string()],
                tags: vec!["rust".to_string(), "api".to_string()],
            },
            instructions: "This is a test skill with instructions.".to_string(),
            scripts: vec!["#!/bin/bash\necho 'Hello'".to_string()],
            resources: {
                let mut res = SkillResources::default();
                res.folders.push("/tmp/test".into());
                res.tools.push("test-tool".to_string());
                res
            },
        }
    }

    #[test]
    fn test_normalize_name_basic() {
        assert_eq!(VsCodeUtils::normalize_name("My Skill"), "my-skill");
        assert_eq!(VsCodeUtils::normalize_name("TestAPI"), "testapi");
        assert_eq!(VsCodeUtils::normalize_name("hello_world"), "hello-world");
        assert_eq!(VsCodeUtils::normalize_name("My Skill 123"), "my-skill-123");
    }

    #[test]
    fn test_normalize_name_special_chars() {
        assert_eq!(VsCodeUtils::normalize_name("Test@#$API"), "testapi");
        assert_eq!(
            VsCodeUtils::normalize_name("  multiple  spaces  "),
            "multiple-spaces"
        );
        assert_eq!(VsCodeUtils::normalize_name("Test___API"), "test-api");
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(VsCodeUtils::validate_name("my-skill").is_ok());
        assert!(VsCodeUtils::validate_name("test").is_ok());
        assert!(VsCodeUtils::validate_name("my-skill-123").is_ok());
        assert!(VsCodeUtils::validate_name("a").is_ok());
    }

    #[test]
    fn test_validate_name_invalid_empty() {
        let result = VsCodeUtils::validate_name("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_name_invalid_too_long() {
        let result = VsCodeUtils::validate_name(&"a".repeat(65));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_name_invalid_start() {
        let result = VsCodeUtils::validate_name("123-skill");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_name_invalid_uppercase() {
        let result = VsCodeUtils::validate_name("MySkill");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_name_invalid_special_chars() {
        let result = VsCodeUtils::validate_name("my_skill");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_description_valid() {
        assert!(VsCodeUtils::validate_description("A valid description").is_ok());
    }

    #[test]
    fn test_validate_description_invalid_empty() {
        let result = VsCodeUtils::validate_description("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_description_invalid_too_long() {
        let result = VsCodeUtils::validate_description(&"x".repeat(201));
        assert!(result.is_err());
    }

    #[test]
    fn test_export_config_default() {
        let config = VsCodeExportConfig::default();
        assert!(config.include_dependencies);
        assert!(config.include_resources);
        assert!(config.include_examples);
        assert!(config.footer.is_none());
    }

    #[test]
    fn test_export_config_builder() {
        let config = VsCodeExportConfig::new()
            .with_dependencies(false)
            .with_resources(false)
            .with_examples(false)
            .with_footer("Custom footer".to_string());

        assert!(!config.include_dependencies);
        assert!(!config.include_resources);
        assert!(!config.include_examples);
        assert_eq!(config.footer, Some("Custom footer".to_string()));
    }
}
