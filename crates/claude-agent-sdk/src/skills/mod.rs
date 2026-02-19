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
pub mod discovery;
pub mod error;
pub mod hot_reload;
pub mod performance;
pub mod progressive_disclosure;
pub mod sandbox;
pub mod skill_md;
pub mod tags;
pub mod tool_restriction;
pub mod trait_impl;
pub mod types;
pub mod version;
pub mod vscode;

// Note: validator module is available as an example in examples/.claude/skills/skill-validator/

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

use std::path::Path;

// Re-export the canonical Skill trait and SkillBox from trait_impl module
pub use trait_impl::{Skill, SkillBox};

// Re-export discovery functions for convenience
pub use discovery::{
    discover_from_dir, discover_from_multiple_dirs, discover_skill_md_from_dir,
};

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
        discovery::discover_from_dir(dir)
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
        discovery::discover_skill_md_from_dir(dir)
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
        discovery::discover_from_multiple_dirs(dirs)
    }
}
