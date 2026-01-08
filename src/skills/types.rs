//! Type definitions for the Skills system

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use std::io::{self, Write};

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

impl SkillPackage {
    /// Save the skill package to a file in JSON format
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut file = fs::File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Load a skill package from a file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let package: SkillPackage = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(package)
    }

    /// Save the skill package to a file in YAML format (requires yaml feature)
    #[cfg(feature = "yaml")]
    pub fn save_to_yaml<P: AsRef<std::path::Path>>(&self, path: P) -> io::Result<()> {
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut file = fs::File::create(path)?;
        file.write_all(yaml.as_bytes())?;
        Ok(())
    }

    /// Load a skill package from a YAML file (requires yaml feature)
    #[cfg(feature = "yaml")]
    pub fn load_from_yaml<P: AsRef<std::path::Path>>(path: P) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let package: SkillPackage = serde_yaml::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(package)
    }
}
