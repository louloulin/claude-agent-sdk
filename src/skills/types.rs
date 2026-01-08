//! Type definitions for the Skills system

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Metadata for a Skill
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Resources associated with a Skill
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SkillResources {
    #[serde(default)]
    pub folders: Vec<PathBuf>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub tests: Vec<String>,
}

/// Input for skill execution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillInput {
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Status of a skill
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillStatus {
    Ready,
    Running,
    Completed,
    Failed,
    Disabled,
}

/// A complete Skill package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPackage {
    pub metadata: SkillMetadata,
    pub instructions: String,
    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(default)]
    pub resources: SkillResources,
}
