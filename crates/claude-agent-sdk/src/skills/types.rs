//! Type definitions for the Skills system

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
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

impl SkillResources {
    /// Scan folders and return a list of all files found
    ///
    /// This method recursively scans all folders configured in this SkillResources
    /// and returns a list of all file paths found. Invalid or inaccessible folders
    /// are skipped with warnings logged.
    ///
    /// # Returns
    /// A vector of PathBuf representing all files found in the configured folders
    ///
    /// # Examples
    /// ```no_run
    /// use claude_agent_sdk::skills::SkillResources;
    ///
    /// let resources = SkillResources {
    ///     folders: vec!["./resources".into()],
    ///     ..Default::default()
    /// };
    ///
    /// let files = resources.scan_folders().unwrap();
    /// for file in files {
    ///     println!("Found resource: {:?}", file);
    /// }
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn scan_folders(&self) -> io::Result<Vec<PathBuf>> {
        let mut all_files = Vec::new();

        for folder in &self.folders {
            if !folder.exists() {
                tracing::warn!("Resource folder does not exist: {:?}", folder);
                continue;
            }

            if !folder.is_dir() {
                tracing::warn!("Resource path is not a directory: {:?}", folder);
                continue;
            }

            self.scan_folder_recursive(folder, &mut all_files)?;
        }

        Ok(all_files)
    }

    /// Recursively scan a folder and collect all file paths
    fn scan_folder_recursive(&self, dir: &PathBuf, files: &mut Vec<PathBuf>) -> io::Result<()> {
        let entries = fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.scan_folder_recursive(&path, files)?;
            } else if path.is_file() {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Validate that all configured folders exist and are accessible
    ///
    /// # Returns
    /// Ok(()) if all folders are valid, Err otherwise with details about invalid folders
    ///
    /// # Examples
    /// ```no_run
    /// use claude_agent_sdk::skills::SkillResources;
    ///
    /// let resources = SkillResources {
    ///     folders: vec!["./resources".into()],
    ///     ..Default::default()
    /// };
    ///
    /// match resources.validate_folders() {
    ///     Ok(_) => println!("All folders are valid"),
    ///     Err(e) => eprintln!("Invalid folders: {}", e),
    /// }
    /// ```
    pub fn validate_folders(&self) -> io::Result<()> {
        let mut invalid_folders = Vec::new();

        for folder in &self.folders {
            if !folder.exists() {
                invalid_folders.push(format!("{:?} does not exist", folder));
            } else if !folder.is_dir() {
                invalid_folders.push(format!("{:?} is not a directory", folder));
            }
        }

        if !invalid_folders.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid folders: {}", invalid_folders.join(", ")),
            ));
        }

        Ok(())
    }

    /// Add a folder to the resources
    ///
    /// # Examples
    /// ```
    /// use claude_agent_sdk::skills::SkillResources;
    ///
    /// let mut resources = SkillResources::default();
    /// resources.add_folder("./resources");
    /// assert_eq!(resources.folders.len(), 1);
    /// ```
    pub fn add_folder<P: AsRef<std::path::Path>>(&mut self, path: P) {
        let path = path.as_ref().to_path_buf();
        if !self.folders.contains(&path) {
            self.folders.push(path);
        }
    }

    /// Add a tool to the resources
    ///
    /// # Examples
    /// ```
    /// use claude_agent_sdk::skills::SkillResources;
    ///
    /// let mut resources = SkillResources::default();
    /// resources.add_tool("search".to_string());
    /// assert_eq!(resources.tools.len(), 1);
    /// ```
    pub fn add_tool(&mut self, tool: String) {
        if !self.tools.contains(&tool) {
            self.tools.push(tool);
        }
    }

    /// Add a test to the resources
    ///
    /// # Examples
    /// ```
    /// use claude_agent_sdk::skills::SkillResources;
    ///
    /// let mut resources = SkillResources::default();
    /// resources.add_test("test_basic_functionality".to_string());
    /// assert_eq!(resources.tests.len(), 1);
    /// ```
    pub fn add_test(&mut self, test: String) {
        if !self.tests.contains(&test) {
            self.tests.push(test);
        }
    }
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
            .map_err(|e| io::Error::other(e))?;

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
        let yaml =
            serde_yaml::to_string(self).map_err(|e| io::Error::other(e))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_metadata_creation() {
        let metadata = SkillMetadata {
            id: "test-skill".to_string(),
            name: "Test Skill".to_string(),
            description: "A test skill".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Test Author".to_string()),
            dependencies: vec!["dep1".to_string(), "dep2".to_string()],
            tags: vec!["test".to_string(), "example".to_string()],
        };

        assert_eq!(metadata.id, "test-skill");
        assert_eq!(metadata.name, "Test Skill");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert_eq!(metadata.dependencies.len(), 2);
        assert_eq!(metadata.tags.len(), 2);
    }

    #[test]
    fn test_skill_resources_default() {
        let resources = SkillResources::default();
        assert!(resources.folders.is_empty());
        assert!(resources.tools.is_empty());
        assert!(resources.tests.is_empty());
    }

    #[test]
    fn test_skill_resources_add_folder() {
        let mut resources = SkillResources::default();
        resources.add_folder("./test");
        assert_eq!(resources.folders.len(), 1);

        // Test duplicate prevention
        resources.add_folder("./test");
        assert_eq!(resources.folders.len(), 1);
    }

    #[test]
    fn test_skill_resources_add_tool() {
        let mut resources = SkillResources::default();
        resources.add_tool("search".to_string());
        assert_eq!(resources.tools.len(), 1);

        // Test duplicate prevention
        resources.add_tool("search".to_string());
        assert_eq!(resources.tools.len(), 1);
    }

    #[test]
    fn test_skill_resources_add_test() {
        let mut resources = SkillResources::default();
        resources.add_test("test_basic".to_string());
        assert_eq!(resources.tests.len(), 1);

        // Test duplicate prevention
        resources.add_test("test_basic".to_string());
        assert_eq!(resources.tests.len(), 1);
    }

    #[test]
    fn test_skill_package_creation() {
        let package = SkillPackage {
            metadata: SkillMetadata {
                id: "test-skill".to_string(),
                name: "Test Skill".to_string(),
                description: "A test skill".to_string(),
                version: "1.0.0".to_string(),
                author: Some("Test Author".to_string()),
                dependencies: vec![],
                tags: vec![],
            },
            instructions: "Test instructions".to_string(),
            scripts: vec![],
            resources: SkillResources::default(),
        };

        assert_eq!(package.metadata.id, "test-skill");
        assert_eq!(package.instructions, "Test instructions");
        assert!(package.scripts.is_empty());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_skill_package_yaml_serialization() {
        let package = SkillPackage {
            metadata: SkillMetadata {
                id: "test-skill".to_string(),
                name: "Test Skill".to_string(),
                description: "A test skill for YAML serialization".to_string(),
                version: "1.0.0".to_string(),
                author: Some("Test Author".to_string()),
                dependencies: vec!["dep1".to_string()],
                tags: vec!["test".to_string(), "yaml".to_string()],
            },
            instructions: "Test instructions for YAML".to_string(),
            scripts: vec!["script1.sh".to_string()],
            resources: SkillResources {
                folders: vec!["./resources".into()],
                tools: vec!["search".to_string()],
                tests: vec!["test_basic".to_string()],
            },
        };

        // Test serialization
        let yaml = serde_yaml::to_string(&package).unwrap();
        assert!(yaml.contains("test-skill"));
        assert!(yaml.contains("Test Skill"));
        assert!(yaml.contains("1.0.0"));

        // Test deserialization
        let deserialized: SkillPackage = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.metadata.id, package.metadata.id);
        assert_eq!(deserialized.metadata.name, package.metadata.name);
        assert_eq!(deserialized.metadata.version, package.metadata.version);
        assert_eq!(deserialized.metadata.author, package.metadata.author);
        assert_eq!(
            deserialized.metadata.dependencies,
            package.metadata.dependencies
        );
        assert_eq!(deserialized.metadata.tags, package.metadata.tags);
        assert_eq!(deserialized.instructions, package.instructions);
        assert_eq!(deserialized.scripts, package.scripts);
        assert_eq!(deserialized.resources.folders, package.resources.folders);
        assert_eq!(deserialized.resources.tools, package.resources.tools);
        assert_eq!(deserialized.resources.tests, package.resources.tests);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_skill_package_yaml_save_and_load() {
        let temp_dir = std::env::temp_dir();
        let yaml_path = temp_dir.join("test_skill.yaml");

        let original_package = SkillPackage {
            metadata: SkillMetadata {
                id: "yaml-test-skill".to_string(),
                name: "YAML Test Skill".to_string(),
                description: "Testing YAML save and load".to_string(),
                version: "2.0.0".to_string(),
                author: Some("YAML Test Author".to_string()),
                dependencies: vec!["yaml-dep".to_string()],
                tags: vec!["yaml-test".to_string()],
            },
            instructions: "YAML test instructions".to_string(),
            scripts: vec!["yaml_script.sh".to_string()],
            resources: SkillResources {
                folders: vec![temp_dir.join("yaml_resources")],
                tools: vec!["yaml-tool".to_string()],
                tests: vec!["yaml_test".to_string()],
            },
        };

        // Save to YAML file
        original_package.save_to_yaml(&yaml_path).unwrap();
        assert!(yaml_path.exists());

        // Load from YAML file
        let loaded_package = SkillPackage::load_from_yaml(&yaml_path).unwrap();

        // Verify all fields match
        assert_eq!(loaded_package.metadata.id, original_package.metadata.id);
        assert_eq!(loaded_package.metadata.name, original_package.metadata.name);
        assert_eq!(
            loaded_package.metadata.description,
            original_package.metadata.description
        );
        assert_eq!(
            loaded_package.metadata.version,
            original_package.metadata.version
        );
        assert_eq!(
            loaded_package.metadata.author,
            original_package.metadata.author
        );
        assert_eq!(
            loaded_package.metadata.dependencies,
            original_package.metadata.dependencies
        );
        assert_eq!(loaded_package.metadata.tags, original_package.metadata.tags);
        assert_eq!(loaded_package.instructions, original_package.instructions);
        assert_eq!(loaded_package.scripts, original_package.scripts);

        // Clean up
        std::fs::remove_file(&yaml_path).unwrap();
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_skill_package_yaml_with_optional_fields() {
        let package = SkillPackage {
            metadata: SkillMetadata {
                id: "minimal-skill".to_string(),
                name: "Minimal Skill".to_string(),
                description: "Minimal test skill".to_string(),
                version: "1.0.0".to_string(),
                author: None,
                dependencies: vec![],
                tags: vec![],
            },
            instructions: "Minimal instructions".to_string(),
            scripts: vec![],
            resources: SkillResources::default(),
        };

        let yaml = serde_yaml::to_string(&package).unwrap();
        let deserialized: SkillPackage = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.metadata.author, None);
        assert!(deserialized.metadata.dependencies.is_empty());
        assert!(deserialized.metadata.tags.is_empty());
        assert!(deserialized.scripts.is_empty());
    }

    #[test]
    fn test_skill_input_default() {
        let input = SkillInput::default();
        assert!(input.params.is_null() || input.params.as_object().map_or(true, |m| m.is_empty()));
    }
}
