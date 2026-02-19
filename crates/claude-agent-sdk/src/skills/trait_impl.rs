//! Core Skill trait implementation

use async_trait::async_trait;
use std::fmt;
use std::sync::Arc;
use super::{SkillInput};
use crate::skills::error::{Result, SkillError, SkillOutput};

/// The core Skill trait - implement this to create custom skills
#[async_trait]
pub trait Skill: fmt::Debug + Send + Sync {
    /// Returns the unique name/identifier for this skill
    fn name(&self) -> String;

    /// Returns a human-readable description of what this skill does
    fn description(&self) -> String;

    /// Execute the skill with the given input
    async fn execute(&self, input: SkillInput) -> Result<SkillOutput>;

    /// Validate the skill's configuration and dependencies
    fn validate(&self) -> Result<()>;

    /// Optional: Get the skill version
    fn version(&self) -> String {
        "1.0.0".to_string()
    }

    /// Optional: Get the skill author
    fn author(&self) -> Option<String> {
        None
    }

    /// Optional: Get tags for discovery
    fn tags(&self) -> Vec<String> {
        Vec::new()
    }

    /// Optional: Get skill dependencies
    fn dependencies(&self) -> Vec<String> {
        Vec::new()
    }

    /// Optional: Check if the skill supports a given capability
    fn supports(&self, _capability: &str) -> bool {
        false
    }

    /// Optional: Called before execution
    async fn before_execute(&self, _input: &SkillInput) -> Result<()> {
        Ok(())
    }

    /// Optional: Called after execution
    async fn after_execute(&self, _input: &SkillInput, _output: &SkillOutput) -> Result<()> {
        Ok(())
    }

    /// Optional: Called on execution error
    async fn on_error(&self, _input: &SkillInput, error: &SkillError) -> SkillError {
        error.clone()
    }
}

/// Wrapper to convert any type implementing Skill into a trait object
///
/// Uses `Arc<dyn Skill>` internally for efficient cloning and sharing
/// across async contexts.
pub struct SkillBox {
    pub inner: Arc<dyn Skill>,
}

impl SkillBox {
    /// Create a new boxed skill
    pub fn new<S: Skill + 'static>(skill: S) -> Self {
        SkillBox {
            inner: Arc::new(skill),
        }
    }
}

#[async_trait]
impl Skill for SkillBox {
    fn name(&self) -> String {
        self.inner.name()
    }

    fn description(&self) -> String {
        self.inner.description()
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput> {
        self.inner.execute(input).await
    }

    fn validate(&self) -> Result<()> {
        self.inner.validate()
    }

    fn version(&self) -> String {
        self.inner.version()
    }

    fn author(&self) -> Option<String> {
        self.inner.author()
    }

    fn tags(&self) -> Vec<String> {
        self.inner.tags()
    }

    fn supports(&self, capability: &str) -> bool {
        self.inner.supports(capability)
    }

    fn dependencies(&self) -> Vec<String> {
        self.inner.dependencies()
    }

    async fn before_execute(&self, input: &SkillInput) -> Result<()> {
        self.inner.before_execute(input).await
    }

    async fn after_execute(&self, input: &SkillInput, output: &SkillOutput) -> Result<()> {
        self.inner.after_execute(input, output).await
    }

    async fn on_error(&self, input: &SkillInput, error: &SkillError) -> SkillError {
        self.inner.on_error(input, error).await
    }
}

impl fmt::Debug for SkillBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SkillBox")
            .field("name", &self.inner.name())
            .field("version", &self.inner.version())
            .finish()
    }
}

impl Clone for SkillBox {
    fn clone(&self) -> Self {
        SkillBox {
            inner: Arc::clone(&self.inner),
        }
    }
}
