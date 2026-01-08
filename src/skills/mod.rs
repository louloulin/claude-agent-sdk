//! # Agent Skills System for Claude Agent SDK

pub mod dependency;
pub mod error;
pub mod hot_reload;
pub mod performance;
pub mod sandbox;
pub mod tags;
pub mod types;
pub mod version;
pub mod vscode;

#[cfg(test)]
mod tests;

use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub use dependency::{Dependency, DependencyResolver, ResolutionResult};
pub use error::{SkillError, SkillOutput, SkillResult};
pub use hot_reload::{HotReloadConfig, HotReloadEvent, HotReloadManager, HotReloadWatcher};
pub use performance::{BatchOperations, IndexedSkillCollection, LruCache, PerformanceStats};
pub use sandbox::{SandboxConfig, SandboxExecutor, SandboxResult, SandboxUtils};
pub use tags::{TagFilter, TagOperator, TagQueryBuilder, TagUtils};
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
    /// use claude_agent_sdk_rs::skills::SkillRegistry;
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
                }
                Err(e) => {
                    tracing::warn!("Failed to load skill package from {:?}: {}", path, e);
                    // Continue loading other files instead of failing completely
                    continue;
                }
            }
        }

        Ok(packages)
    }
}
