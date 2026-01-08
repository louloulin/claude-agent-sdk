//! Skill registry for managing and discovering skills

use super::{Skill, SkillBox, SkillError, SkillPackage, SkillStatus};
use crate::skills::error::Result;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Registry for managing skills
#[derive(Clone)]
pub struct SkillRegistry {
    inner: Arc<RwLock<RegistryInner>>,
}

struct RegistryInner {
    skills: HashMap<String, RegisteredSkill>,
    skill_packages: HashMap<String, SkillPackage>,
    skill_indices: SkillIndices,
}

struct RegisteredSkill {
    skill: SkillBox,
    status: SkillStatus,
    registered_at: std::time::SystemTime,
}

struct SkillIndices {
    by_tag: HashMap<String, HashSet<String>>,
    by_capability: HashMap<String, HashSet<String>>,
    dependencies: HashMap<String, HashSet<String>>,
}

impl Default for SkillIndices {
    fn default() -> Self {
        SkillIndices {
            by_tag: HashMap::new(),
            by_capability: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }
}

impl SkillRegistry {
    /// Create a new skill registry
    pub fn new() -> Self {
        SkillRegistry {
            inner: Arc::new(RwLock::new(RegistryInner {
                skills: HashMap::new(),
                skill_packages: HashMap::new(),
                skill_indices: SkillIndices::default(),
            })),
        }
    }

    /// Register a skill
    pub async fn register_skill(&self, skill: SkillBox) -> Result<()> {
        let name = skill.name();
        skill.validate()?;

        let mut inner = self.inner.write().await;

        if inner.skills.contains_key(&name) {
            return Err(SkillError::AlreadyExists(name));
        }

        let registered = RegisteredSkill {
            skill,
            status: SkillStatus::Ready,
            registered_at: std::time::SystemTime::now(),
        };

        let skill_name = registered.skill.name().clone();
        let tags = registered.skill.tags();
        let dependencies = registered.skill.dependencies();

        for tag in &tags {
            inner.skill_indices.by_tag
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(skill_name.clone());
        }

        for dep in &dependencies {
            inner.skill_indices.dependencies
                .entry(skill_name.clone())
                .or_insert_with(HashSet::new)
                .insert(dep.clone());
        }

        inner.skills.insert(skill_name.clone(), registered);
        tracing::debug!("Registered skill: {}", skill_name);

        Ok(())
    }

    /// Get a skill by name
    pub async fn get_skill(&self, name: &str) -> Option<SkillBox> {
        let inner = self.inner.read().await;
        inner.skills.get(name).map(|s| s.skill.clone())
    }

    /// Check if a skill exists
    pub async fn has_skill(&self, name: &str) -> bool {
        let inner = self.inner.read().await;
        inner.skills.contains_key(name)
    }

    /// Get all skill names
    pub async fn list_skills(&self) -> Vec<String> {
        let inner = self.inner.read().await;
        inner.skills.keys().cloned().collect()
    }

    /// Find skills by tag
    pub async fn find_by_tag(&self, tag: &str) -> Vec<SkillBox> {
        let inner = self.inner.read().await;

        if let Some(names) = inner.skill_indices.by_tag.get(tag) {
            names
                .iter()
                .filter_map(|name| inner.skills.get(name).map(|s| s.skill.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get skill status
    pub async fn get_status(&self, name: &str) -> Option<SkillStatus> {
        let inner = self.inner.read().await;
        inner.skills.get(name).map(|s| s.status)
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
