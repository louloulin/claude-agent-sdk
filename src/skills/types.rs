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
    /// use claude_agent_sdk_rs::skills::SkillResources;
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
    /// use claude_agent_sdk_rs::skills::SkillResources;
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
    /// use claude_agent_sdk_rs::skills::SkillResources;
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
    /// use claude_agent_sdk_rs::skills::SkillResources;
    ///
    /// let mut resources = SkillResources::default();
    /// resources.add_tool("search");
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
    /// use claude_agent_sdk_rs::skills::SkillResources;
    ///
    /// let mut resources = SkillResources::default();
    /// resources.add_test("test_basic_functionality");
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
