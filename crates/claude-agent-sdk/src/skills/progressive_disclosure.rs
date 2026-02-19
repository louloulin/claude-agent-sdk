//! Progressive Disclosure for Skills
//!
//! This module implements the progressive disclosure pattern from Claude Code,
//! where SKILL.md contains essential information and supporting files are loaded
//! on-demand to save context window space.
//!
//! Based on: https://code.claude.com/docs/en/skills

use crate::skills::skill_md::{SkillMdError, SkillMdFile};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors for progressive disclosure
#[derive(Debug, Error)]
pub enum ProgressiveError {
    #[error("SKILL.md error: {0}")]
    SkillMdError(#[from] SkillMdError),

    #[error("Referenced file not found: {0}")]
    FileNotFound(PathBuf, #[source] std::io::Error),

    #[error("Invalid reference format: {0}")]
    InvalidReference(String),
}

/// Progressive skill loader implementing lazy loading of supporting files
///
/// # Principles
///
/// 1. **SKILL.md is always loaded** - Contains essential information
/// 2. **Supporting files are on-demand** - Loaded only when needed
/// 3. **Context window conservation** - Reduces token usage
/// 4. **Rich documentation** - Can have extensive docs without cost
///
/// # File Structure
///
/// ```text
/// skill-directory/
/// ├── SKILL.md          # Required, always loaded
/// ├── reference.md      # Optional, detailed docs
/// ├── examples.md       # Optional, usage examples
/// ├── forms.md          # Optional, field mappings
/// └── scripts/          # Optional, utility scripts (executed only)
///     ├── helper.py
///     └── validate.sh
/// ```
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk::skills::progressive_disclosure::ProgressiveSkillLoader;
///
/// let loader = ProgressiveSkillLoader::load("/path/to/skill")?;
///
/// // Essential content is always available
/// println!("{}", loader.get_main_content());
///
/// // Load detailed reference only when needed
/// if let Some(reference) = loader.load_reference("reference.md")? {
///     println!("{}", reference);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct ProgressiveSkillLoader {
    /// Path to the skill directory
    #[allow(dead_code)]
    skill_dir: PathBuf,
    /// Content from SKILL.md (always loaded)
    main_content: String,
    /// Referenced supporting files (discovered but not loaded)
    referenced_files: HashMap<String, PathBuf>,
    /// Available scripts in scripts/ directory
    available_scripts: Vec<PathBuf>,
}

impl ProgressiveSkillLoader {
    /// Load a skill and scan for references without loading supporting files
    ///
    /// # Arguments
    ///
    /// * `skill_dir` - Path to the skill directory containing SKILL.md
    ///
    /// # Returns
    ///
    /// A ProgressiveSkillLoader with main content loaded and references scanned
    ///
    /// # Errors
    ///
    /// Returns ProgressiveError if:
    /// - SKILL.md cannot be read or parsed
    /// - Skill directory doesn't exist
    ///
    /// # Example
    ///
    /// ```no_run
    /// use claude_agent_sdk::skills::progressive_disclosure::ProgressiveSkillLoader;
    ///
    /// let loader = ProgressiveSkillLoader::load(".claude/skills/my-skill")?;
    /// println!("Loaded skill with {} references",
    ///          loader.get_reference_count());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load<P: AsRef<Path>>(skill_dir: P) -> Result<Self, ProgressiveError> {
        let skill_dir = skill_dir.as_ref();
        let skill_md = skill_dir.join("SKILL.md");

        // Parse SKILL.md
        let skill_file = SkillMdFile::parse(&skill_md)?;

        // Scan for references in main content
        let referenced_files = Self::scan_references(&skill_file.content, skill_dir);

        // Discover available scripts
        let available_scripts = Self::discover_scripts(skill_dir);

        Ok(Self {
            skill_dir: skill_dir.to_path_buf(),
            main_content: skill_file.content,
            referenced_files,
            available_scripts,
        })
    }

    /// Get the main SKILL.md content (always available)
    ///
    /// # Returns
    ///
    /// Reference to the main markdown content
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::skills::progressive_disclosure::ProgressiveSkillLoader;
    /// # let loader = ProgressiveSkillLoader::load(".").unwrap();
    /// let content = loader.get_main_content();
    /// println!("Skill content:\n{}", content);
    /// ```
    pub fn get_main_content(&self) -> &str {
        &self.main_content
    }

    /// Load a referenced file on-demand
    ///
    /// # Arguments
    ///
    /// * `filename` - Name of the file to load (e.g., "reference.md", "examples.md")
    ///
    /// # Returns
    ///
    /// Ok(Some(content)) if file exists and was loaded
    /// Ok(None) if reference doesn't exist
    /// Err if file exists but cannot be read
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::skills::progressive_disclosure::ProgressiveSkillLoader;
    /// # let loader = ProgressiveSkillLoader::load(".").unwrap();
    /// // Load detailed reference documentation
    /// match loader.load_reference("reference.md") {
    ///     Ok(Some(reference)) => println!("Reference: {}", reference),
    ///     Ok(None) => println!("No reference.md found"),
    ///     Err(e) => eprintln!("Error loading reference: {}", e),
    /// }
    /// ```
    pub fn load_reference(&self, filename: &str) -> Result<Option<String>, ProgressiveError> {
        if let Some(path) = self.referenced_files.get(filename) {
            std::fs::read_to_string(path)
                .map(Some)
                .map_err(|e| ProgressiveError::FileNotFound(path.clone(), e))
        } else {
            Ok(None)
        }
    }

    /// Load all referenced files
    ///
    /// # Returns
    ///
    /// HashMap of filename -> content for all discovered references
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::skills::progressive_disclosure::ProgressiveSkillLoader;
    /// # let loader = ProgressiveSkillLoader::load(".").unwrap();
    /// let all_refs = loader.load_all_references()?;
    /// for (filename, content) in all_refs {
    ///     println!("=== {} ===\n{}", filename, content);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load_all_references(&self) -> Result<HashMap<String, String>, ProgressiveError> {
        let mut loaded = HashMap::new();

        for (filename, path) in &self.referenced_files {
            let content = std::fs::read_to_string(path)
                .map_err(|e| ProgressiveError::FileNotFound(path.clone(), e))?;
            loaded.insert(filename.clone(), content);
        }

        Ok(loaded)
    }

    /// Get list of discovered reference filenames (without loading them)
    ///
    /// # Returns
    ///
    /// Vector of filenames that were referenced in SKILL.md
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::skills::progressive_disclosure::ProgressiveSkillLoader;
    /// # let loader = ProgressiveSkillLoader::load(".").unwrap();
    /// let refs = loader.list_references();
    /// println!("Available references: {:?}", refs);
    /// ```
    pub fn list_references(&self) -> Vec<String> {
        self.referenced_files.keys().cloned().collect()
    }

    /// Get count of discovered references
    pub fn get_reference_count(&self) -> usize {
        self.referenced_files.len()
    }

    /// Get list of available scripts (without loading them)
    pub fn list_scripts(&self) -> &[PathBuf] {
        &self.available_scripts
    }

    /// Check if a specific reference exists
    pub fn has_reference(&self, filename: &str) -> bool {
        self.referenced_files.contains_key(filename)
    }

    /// Scan markdown content for file references
    ///
    /// Detects patterns like:
    /// - `[Text](filename.md)` - Markdown links
    /// - `See [reference.md](reference.md)` - Self-referencing links
    fn scan_references(content: &str, base_dir: &Path) -> HashMap<String, PathBuf> {
        let mut refs = HashMap::new();

        // Markdown link pattern: [text](filename.md)
        let link_pattern = Regex::new(r"\[(?P<title>[^\]]+)\]\((?P<file>[^)]+\.md)\)").unwrap();

        for cap in link_pattern.captures_iter(content) {
            let file = cap.name("file").unwrap().as_str();
            let full_path = base_dir.join(file);

            if full_path.exists() && !refs.contains_key(file) {
                refs.insert(file.to_string(), full_path);
            }
        }

        // Also check for standard supporting files even if not linked
        let standard_files = ["reference.md", "examples.md", "forms.md"];
        for standard in standard_files {
            let path = base_dir.join(standard);
            if path.exists() && !refs.contains_key(standard) {
                refs.insert(standard.to_string(), path);
            }
        }

        refs
    }

    /// Discover executable scripts in scripts/ directory
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

    /// Get summary statistics
    ///
    /// # Returns
    ///
    /// String with statistics about the skill structure
    pub fn get_summary(&self) -> String {
        format!(
            "ProgressiveSkillLoader:\n\
             - Main content: {} bytes\n\
             - References: {} files\n\
             - Scripts: {} files",
            self.main_content.len(),
            self.referenced_files.len(),
            self.available_scripts.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn create_test_skill(temp_dir: &Path) {
        // Create SKILL.md with references
        let skill_md = temp_dir.join("SKILL.md");
        let content = r#"---
name: test-skill
description: A test skill for progressive disclosure
---

# Test Skill

Main content here.

See [reference.md](reference.md) for details.
See [examples.md](examples.md) for usage.
"#;
        fs::write(&skill_md, content).unwrap();

        // Create reference.md
        let reference = temp_dir.join("reference.md");
        fs::write(&reference, "Detailed reference documentation").unwrap();

        // Create examples.md
        let examples = temp_dir.join("examples.md");
        fs::write(&examples, "Usage examples").unwrap();

        // Create scripts directory with a script
        let scripts_dir = temp_dir.join("scripts");
        fs::create_dir_all(&scripts_dir).unwrap();
        let script = scripts_dir.join("helper.sh");
        File::create(&script).unwrap();
    }

    #[test]
    fn test_progressive_loader_load() {
        let temp_dir = TempDir::new().unwrap();
        create_test_skill(temp_dir.path());

        let loader = ProgressiveSkillLoader::load(temp_dir.path()).unwrap();
        assert!(loader.get_main_content().contains("Main content here"));
        assert_eq!(loader.get_reference_count(), 2);
    }

    #[test]
    fn test_progressive_loader_load_reference() {
        let temp_dir = TempDir::new().unwrap();
        create_test_skill(temp_dir.path());

        let loader = ProgressiveSkillLoader::load(temp_dir.path()).unwrap();

        // Load reference.md
        let reference = loader.load_reference("reference.md").unwrap();
        assert!(reference.is_some());
        assert!(reference.unwrap().contains("Detailed reference"));

        // Load examples.md
        let examples = loader.load_reference("examples.md").unwrap();
        assert!(examples.is_some());
        assert!(examples.unwrap().contains("Usage examples"));

        // Try to load non-existent file
        let missing = loader.load_reference("nonexistent.md").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn test_progressive_loader_list_references() {
        let temp_dir = TempDir::new().unwrap();
        create_test_skill(temp_dir.path());

        let loader = ProgressiveSkillLoader::load(temp_dir.path()).unwrap();
        let refs = loader.list_references();

        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&"reference.md".to_string()));
        assert!(refs.contains(&"examples.md".to_string()));
    }

    #[test]
    fn test_progressive_loader_has_reference() {
        let temp_dir = TempDir::new().unwrap();
        create_test_skill(temp_dir.path());

        let loader = ProgressiveSkillLoader::load(temp_dir.path()).unwrap();
        assert!(loader.has_reference("reference.md"));
        assert!(loader.has_reference("examples.md"));
        assert!(!loader.has_reference("nonexistent.md"));
    }

    #[test]
    fn test_progressive_loader_load_all_references() {
        let temp_dir = TempDir::new().unwrap();
        create_test_skill(temp_dir.path());

        let loader = ProgressiveSkillLoader::load(temp_dir.path()).unwrap();
        let all_refs = loader.load_all_references().unwrap();

        assert_eq!(all_refs.len(), 2);
        assert!(all_refs.get("reference.md").unwrap().contains("Detailed"));
        assert!(all_refs.get("examples.md").unwrap().contains("Usage"));
    }

    #[test]
    fn test_progressive_loader_scripts() {
        let temp_dir = TempDir::new().unwrap();
        create_test_skill(temp_dir.path());

        let loader = ProgressiveSkillLoader::load(temp_dir.path()).unwrap();
        let scripts = loader.list_scripts();

        assert_eq!(scripts.len(), 1);
        assert!(scripts[0].ends_with("helper.sh"));
    }

    #[test]
    fn test_progressive_loader_summary() {
        let temp_dir = TempDir::new().unwrap();
        create_test_skill(temp_dir.path());

        let loader = ProgressiveSkillLoader::load(temp_dir.path()).unwrap();
        let summary = loader.get_summary();

        assert!(summary.contains("ProgressiveSkillLoader"));
        assert!(summary.contains("References: 2 files"));
        assert!(summary.contains("Scripts: 1 files"));
    }
}
