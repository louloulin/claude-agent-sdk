//! Skill discovery module for loading skills from filesystem
//!
//! This module provides functions to discover and load skill packages from
//! directories containing either JSON files or SKILL.md files.

use std::collections::HashSet;
use std::path::Path;

use super::{SkillError, SkillPackage, SkillsDirScanner};

/// Discover and load skill packages from a directory
///
/// This function searches for `.json` files in the given directory,
/// attempts to load them as SkillPackages, and returns the loaded packages.
///
/// # Arguments
/// * `dir` - Path to the directory containing skill package files
///
/// # Returns
/// A vector of successfully loaded SkillPackages
///
/// # Errors
/// Returns `SkillError::Io` if the directory doesn't exist or isn't a directory.
///
/// # Examples
/// ```no_run
/// use claude_agent_sdk::skills::discovery::discover_from_dir;
///
/// let packages = discover_from_dir("/path/to/skills")?;
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

/// Discover and load SKILL.md files from a skills directory
///
/// This function searches for subdirectories containing `SKILL.md` files,
/// parses them with full YAML frontmatter support, and returns the loaded
/// packages.
///
/// # Arguments
/// * `dir` - Path to the skills directory (e.g., `.claude/skills/`)
///
/// # Returns
/// A vector of successfully loaded SkillPackages. Returns empty vec if
/// directory doesn't exist.
///
/// # Errors
/// Returns `SkillError::Io` if the path exists but isn't a directory.
///
/// # Examples
/// ```no_run
/// use claude_agent_sdk::skills::discovery::discover_skill_md_from_dir;
///
/// let packages = discover_skill_md_from_dir(".claude/skills")?;
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
    let scanner = SkillsDirScanner::new(dir);
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
/// A vector of successfully loaded SkillPackages with duplicates removed
/// (first occurrence wins).
///
/// # Examples
/// ```no_run
/// use claude_agent_sdk::skills::discovery::discover_from_multiple_dirs;
///
/// let packages = discover_from_multiple_dirs(vec![
///     ".claude/skills",
///     "~/.config/claude/skills",
/// ])?;
/// # Ok::<(), claude_agent_sdk::skills::SkillError>(())
/// ```
pub fn discover_from_multiple_dirs<P: AsRef<Path>>(dirs: Vec<P>) -> Result<Vec<SkillPackage>, SkillError> {
    let mut all_packages = Vec::new();
    let mut seen_ids = HashSet::new();

    for dir in dirs {
        let dir = dir.as_ref();

        // Try SKILL.md discovery first (modern format)
        if let Ok(mut packages) = discover_skill_md_from_dir(dir) {
            // Filter out duplicates (keep first occurrence)
            packages.retain(|p| seen_ids.insert(p.metadata.id.clone()));
            all_packages.extend(packages);
        }

        // Fall back to JSON discovery (legacy format)
        if let Ok(mut packages) = discover_from_dir(dir) {
            // Filter out duplicates (keep first occurrence)
            packages.retain(|p| seen_ids.insert(p.metadata.id.clone()));
            all_packages.extend(packages);
        }
    }

    Ok(all_packages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_discover_from_dir() {
        let temp_dir = std::env::temp_dir().join("skills_test_discover_module");
        fs::create_dir_all(&temp_dir).unwrap();

        let package1 = SkillPackage {
            metadata: crate::skills::SkillMetadata {
                id: "test1".to_string(),
                name: "Test Skill 1".to_string(),
                description: "First test skill".to_string(),
                version: "1.0.0".to_string(),
                author: None,
                dependencies: vec![],
                tags: vec![],
            },
            instructions: "Test instructions 1".to_string(),
            scripts: vec![],
            resources: crate::skills::SkillResources::default(),
        };

        let package2 = SkillPackage {
            metadata: crate::skills::SkillMetadata {
                id: "test2".to_string(),
                name: "Test Skill 2".to_string(),
                description: "Second test skill".to_string(),
                version: "1.0.0".to_string(),
                author: None,
                dependencies: vec![],
                tags: vec![],
            },
            instructions: "Test instructions 2".to_string(),
            scripts: vec![],
            resources: crate::skills::SkillResources::default(),
        };

        let file1 = temp_dir.join("skill1.json");
        let file2 = temp_dir.join("skill2.json");

        package1.save_to_file(&file1).unwrap();
        package2.save_to_file(&file2).unwrap();

        let packages = discover_from_dir(&temp_dir).unwrap();
        assert_eq!(packages.len(), 2);

        fs::remove_file(&file1).unwrap();
        fs::remove_file(&file2).unwrap();
        fs::remove_dir(&temp_dir).unwrap();
    }

    #[test]
    fn test_discover_from_nonexistent_dir() {
        let result = discover_from_dir("/nonexistent/path/that/does/not/exist");
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_skill_md_from_nonexistent_dir() {
        let result = discover_skill_md_from_dir("/nonexistent/path/that/does/not/exist");
        // Should return Ok with empty vec, not error
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
