//! # Agent Skills System for Claude Agent SDK
//!
//! This module provides a comprehensive skills system based on Claude Code Skills specification.
//!
//! ## Features
//!
//! - **SKILL.md Parsing**: Full support for YAML frontmatter and markdown content
//! - **Progressive Disclosure**: Lazy loading of supporting files to save context
//! - **Tool Restrictions**: Enforce allowed-tools from skill metadata
//! - **Advanced Metadata**: Support for all Claude Code skill fields
//!
//! Based on: https://code.claude.com/docs/en/skills

pub mod api;
pub mod auditor;
pub mod dependency;
pub mod error;
pub mod hot_reload;
pub mod performance;
pub mod progressive_disclosure;
pub mod sandbox;
pub mod skill_md;
pub mod tags;
pub mod tool_restriction;
pub mod types;
pub mod version;
pub mod vscode;

// Note: validator module is available as an example in examples/.claude/skills/skill-validator/

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

use async_trait::async_trait;
use std::path::Path;

pub use api::{ListSkillsResponse, SkillApiInfo, SkillsApiClient, SkillsError, UploadSkillResponse};
pub use auditor::{
    AuditConfig, AuditError, IssueType, RiskLevel, SkillAuditor, SkillAuditIssue, SkillAuditReport,
};
pub use dependency::{Dependency, DependencyResolver, ResolutionResult};
pub use error::{SkillError, SkillOutput, SkillResult};
pub use hot_reload::{HotReloadConfig, HotReloadEvent, HotReloadManager, HotReloadWatcher};
pub use performance::{BatchOperations, IndexedSkillCollection, LruCache, PerformanceStats};
pub use progressive_disclosure::ProgressiveSkillLoader;
pub use sandbox::{SandboxConfig, SandboxExecutor, SandboxResult, SandboxUtils};
pub use skill_md::{HookConfig, HookType, SkillContext, SkillHooks, SkillMdError, SkillMdFile, SkillMdMetadata, SkillsDirScanner};
pub use tags::{TagFilter, TagOperator, TagQueryBuilder, TagUtils};
pub use tool_restriction::{ToolRestriction, ToolRestrictionError};
pub use types::{SkillInput, SkillMetadata, SkillPackage, SkillResources, SkillStatus};
pub use version::{CompatibilityResult, VersionManager};
pub use vscode::{VsCodeExportConfig, VsCodeUtils, export_batch_to_vscode, export_to_vscode};

/// The core Skill trait
#[async_trait]
pub trait Skill: Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> String;
    async fn execute(&self, input: SkillInput) -> SkillResult;
    fn validate(&self) -> Result<(), SkillError>;
}

/// Simple skill registry
pub struct SkillRegistry {
    skills: std::collections::HashMap<String, Box<dyn Skill>>,
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: Box<dyn Skill>) -> Result<(), SkillError> {
        let name = skill.name();
        skill.validate()?;
        self.skills.insert(name, skill);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&dyn Skill> {
        self.skills.get(name).map(|s| s.as_ref())
    }

    pub fn list(&self) -> Vec<String> {
        self.skills.keys().cloned().collect()
    }

    /// Discover and load skill packages from a directory
    ///
    /// This method searches for `.json` files in the given directory,
    /// attempts to load them as SkillPackages, and returns the loaded packages.
    ///
    /// # Arguments
    /// * `dir` - Path to the directory containing skill package files
    ///
    /// # Returns
    /// A vector of successfully loaded SkillPackages
    ///
    /// # Examples
    /// ```no_run
    /// use claude_agent_sdk::skills::SkillRegistry;
    ///
    /// let packages = SkillRegistry::discover_from_dir("/path/to/skills")?;
    /// for package in packages {
    ///     println!("Found skill: {}", package.metadata.name);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn discover_from_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<SkillPackage>, SkillError> {
        let dir = dir.as_ref();

        if !dir.exists() {
            return Err(SkillError::Io(format!(
                "Directory does not exist: {:?}",
                dir
            )));
        }

        if !dir.is_dir() {
            return Err(SkillError::Io(format!(
                "Path is not a directory: {:?}",
                dir
            )));
        }

        let entries = std::fs::read_dir(dir)
            .map_err(|e| SkillError::Io(format!("Failed to read directory: {}", e)))?;

        let mut packages = Vec::new();

        for entry in entries {
            let entry = entry
                .map_err(|e| SkillError::Io(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            // Only process .json files
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // Try to load as SkillPackage
            match SkillPackage::load_from_file(&path) {
                Ok(package) => {
                    tracing::info!(
                        "Loaded skill package: {} from {:?}",
                        package.metadata.name,
                        path
                    );
                    packages.push(package);
                },
                Err(e) => {
                    tracing::warn!("Failed to load skill package from {:?}: {}", path, e);
                    // Continue loading other files instead of failing completely
                    continue;
                },
            }
        }

        Ok(packages)
    }

    /// Discover and load SKILL.md files from a skills directory
    ///
    /// This method searches for subdirectories containing `SKILL.md` files,
    /// parses them with full YAML frontmatter support, and returns the loaded
    /// packages.
    ///
    /// # Arguments
    /// * `dir` - Path to the skills directory (e.g., `.claude/skills/`)
    ///
    /// # Returns
    /// A vector of successfully loaded SkillPackages
    ///
    /// # Examples
    /// ```no_run
    /// use claude_agent_sdk::skills::SkillRegistry;
    ///
    /// let packages = SkillRegistry::discover_skill_md_from_dir(".claude/skills")?;
    /// for package in packages {
    ///     println!("Found skill: {} from SKILL.md", package.metadata.name);
    /// }
    /// # Ok::<(), claude_agent_sdk::skills::SkillError>(())
    /// ```
    pub fn discover_skill_md_from_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<SkillPackage>, SkillError> {
        let dir = dir.as_ref();

        if !dir.exists() {
            // Return empty vec instead of error for missing directories
            tracing::debug!("Skills directory does not exist: {:?}", dir);
            return Ok(Vec::new());
        }

        if !dir.is_dir() {
            return Err(SkillError::Io(format!(
                "Path is not a directory: {:?}",
                dir
            )));
        }

        // Use SkillsDirScanner to discover all SKILL.md files
        let scanner = crate::skills::SkillsDirScanner::new(dir);
        let skill_md_files = scanner.scan()
            .map_err(|e| SkillError::Io(format!("Failed to scan skills directory: {}", e)))?;

        // Convert all SkillMdFile to SkillPackage
        let mut packages = Vec::new();
        for skill_md in skill_md_files {
            let package = skill_md.to_skill_package();
            tracing::info!(
                "Loaded SKILL.md: {} from {:?}",
                package.metadata.name,
                skill_md.skill_dir
            );
            packages.push(package);
        }

        Ok(packages)
    }

    /// Discover and load skills from multiple directories with priority
    ///
    /// Searches multiple directories in order, merging results. Later directories
    /// override earlier ones if skills have the same ID.
    ///
    /// # Arguments
    /// * `dirs` - Vector of directory paths to search (in priority order)
    ///
    /// # Returns
    /// A vector of successfully loaded SkillPackages
    ///
    /// # Examples
    /// ```no_run
    /// use claude_agent_sdk::skills::SkillRegistry;
    ///
    /// let packages = SkillRegistry::discover_from_multiple_dirs(vec![
    ///     ".claude/skills",
    ///     "~/.config/claude/skills",
    /// ])?;
    /// # Ok::<(), claude_agent_sdk::skills::SkillError>(())
    /// ```
    pub fn discover_from_multiple_dirs<P: AsRef<Path>>(dirs: Vec<P>) -> Result<Vec<SkillPackage>, SkillError> {
        let mut all_packages = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for dir in dirs {
            let dir = dir.as_ref();

            // Try SKILL.md discovery first (modern format)
            if let Ok(mut packages) = Self::discover_skill_md_from_dir(dir) {
                // Filter out duplicates (keep first occurrence)
                packages.retain(|p| seen_ids.insert(p.metadata.id.clone()));
                all_packages.extend(packages);
            }

            // Fall back to JSON discovery (legacy format)
            if let Ok(mut packages) = Self::discover_from_dir(dir) {
                // Filter out duplicates (keep first occurrence)
                packages.retain(|p| seen_ids.insert(p.metadata.id.clone()));
                all_packages.extend(packages);
            }
        }

        Ok(all_packages)
    }
}
