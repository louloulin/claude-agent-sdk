//! # Agent Skills System for Claude Agent SDK

pub mod error;
pub mod types;

use async_trait::async_trait;

pub use error::{SkillError, SkillOutput, SkillResult};
pub use types::{SkillInput, SkillMetadata, SkillPackage, SkillResources, SkillStatus};

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
}
