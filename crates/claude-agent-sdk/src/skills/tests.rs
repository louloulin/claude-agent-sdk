//! Tests for the Skills system

use super::*;
use async_trait::async_trait;

#[derive(Debug)]
struct TestSkill {
    name: String,
    description: String,
}

#[async_trait]
impl Skill for TestSkill {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    async fn execute(&self, input: SkillInput) -> error::Result<SkillOutput> {
        Ok(SkillOutput::ok(format!(
            "Executed with params: {:?}",
            input.params
        )))
    }

    fn validate(&self) -> error::Result<()> {
        if self.name.is_empty() {
            return Err(SkillError::Validation("Skill name cannot be empty".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_registry_register() {
        let mut registry = SkillRegistry::new();
        let skill = TestSkill {
            name: "test".to_string(),
            description: "Test skill".to_string(),
        };

        let result = registry.register(Box::new(skill));
        assert!(result.is_ok());
        assert_eq!(registry.list(), vec!["test".to_string()]);
    }

    #[test]
    fn test_skill_registry_get() {
        let mut registry = SkillRegistry::new();
        let skill = TestSkill {
            name: "test".to_string(),
            description: "Test skill".to_string(),
        };

        registry.register(Box::new(skill)).unwrap();

        let retrieved = registry.get("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test");

        let not_found = registry.get("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_skill_output_ok() {
        let output = SkillOutput::ok("test data");
        assert!(output.success);
        assert_eq!(output.data, "test data");
        assert!(output.error.is_none());
    }

    #[test]
    fn test_skill_output_err() {
        let output = SkillOutput::err("test error");
        assert!(!output.success);
        assert_eq!(output.error, Some("test error".to_string()));
    }

    #[test]
    fn test_skill_package_serialization() {
        let package = SkillPackage {
            metadata: SkillMetadata {
                id: "test-id".to_string(),
                name: "Test".to_string(),
                description: "Test skill".to_string(),
                version: "1.0.0".to_string(),
                author: None,
                dependencies: vec![],
                tags: vec![],
            },
            instructions: "Test instructions".to_string(),
            scripts: vec![],
            resources: SkillResources::default(),
        };

        let json = serde_json::to_string(&package);
        assert!(json.is_ok());

        let deserialized: Result<SkillPackage, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
        let pkg = deserialized.unwrap();
        assert_eq!(pkg.metadata.id, "test-id");
    }

    #[test]
    fn test_discover_from_dir() {
        use std::fs;

        let temp_dir = std::env::temp_dir().join("skills_test_discover");
        fs::create_dir_all(&temp_dir).unwrap();

        let package1 = SkillPackage {
            metadata: SkillMetadata {
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
            resources: SkillResources::default(),
        };

        let package2 = SkillPackage {
            metadata: SkillMetadata {
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
            resources: SkillResources::default(),
        };

        let file1 = temp_dir.join("skill1.json");
        let file2 = temp_dir.join("skill2.json");

        package1.save_to_file(&file1).unwrap();
        package2.save_to_file(&file2).unwrap();

        let packages = SkillRegistry::discover_from_dir(&temp_dir).unwrap();
        assert_eq!(packages.len(), 2);

        fs::remove_file(&file1).unwrap();
        fs::remove_file(&file2).unwrap();
        fs::remove_dir(&temp_dir).unwrap();
    }

    #[test]
    fn test_discover_from_nonexistent_dir() {
        let result = SkillRegistry::discover_from_dir("/nonexistent/path/that/does/not/exist");
        assert!(result.is_err());
    }

    #[test]
    fn test_skill_resources_add_folder() {
        let mut resources = SkillResources::default();
        resources.add_folder("./test_folder");
        assert_eq!(resources.folders.len(), 1);
        assert_eq!(
            resources.folders[0],
            std::path::PathBuf::from("./test_folder")
        );

        // Test duplicate prevention
        resources.add_folder("./test_folder");
        assert_eq!(resources.folders.len(), 1);
    }

    #[test]
    fn test_skill_resources_add_tool() {
        let mut resources = SkillResources::default();
        resources.add_tool("search".to_string());
        assert_eq!(resources.tools.len(), 1);
        assert_eq!(resources.tools[0], "search");

        // Test duplicate prevention
        resources.add_tool("search".to_string());
        assert_eq!(resources.tools.len(), 1);
    }

    #[test]
    fn test_skill_resources_add_test() {
        let mut resources = SkillResources::default();
        resources.add_test("test_basic".to_string());
        assert_eq!(resources.tests.len(), 1);
        assert_eq!(resources.tests[0], "test_basic");

        // Test duplicate prevention
        resources.add_test("test_basic".to_string());
        assert_eq!(resources.tests.len(), 1);
    }

    #[test]
    fn test_skill_resources_validate_folders() {
        use std::fs;

        let temp_dir = std::env::temp_dir().join("skills_validate_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let resources = SkillResources {
            folders: vec![temp_dir.clone()],
            ..Default::default()
        };

        assert!(resources.validate_folders().is_ok());

        // Test with non-existent folder
        let resources_invalid = SkillResources {
            folders: vec![std::path::PathBuf::from("/nonexistent/folder")],
            ..Default::default()
        };

        assert!(resources_invalid.validate_folders().is_err());

        fs::remove_dir(&temp_dir).unwrap();
    }

    #[test]
    fn test_skill_resources_scan_folders() {
        use std::fs::{self, File};

        let temp_dir = std::env::temp_dir().join("skills_scan_test");
        fs::create_dir_all(&temp_dir).unwrap();

        // Create nested structure
        let sub_dir = temp_dir.join("subdir");
        fs::create_dir_all(&sub_dir).unwrap();

        // Create test files
        let file1 = temp_dir.join("file1.txt");
        let file2 = temp_dir.join("file2.txt");
        let file3 = sub_dir.join("file3.txt");

        File::create(&file1).unwrap();
        File::create(&file2).unwrap();
        File::create(&file3).unwrap();

        let resources = SkillResources {
            folders: vec![temp_dir.clone()],
            ..Default::default()
        };

        let files = resources.scan_folders().unwrap();
        assert_eq!(files.len(), 3);

        // Clean up
        fs::remove_file(&file1).unwrap();
        fs::remove_file(&file2).unwrap();
        fs::remove_file(&file3).unwrap();
        fs::remove_dir(&sub_dir).unwrap();
        fs::remove_dir(&temp_dir).unwrap();
    }

    #[test]
    fn test_skill_resources_scan_nonexistent_folder() {
        let resources = SkillResources {
            folders: vec![std::path::PathBuf::from("/nonexistent/folder")],
            ..Default::default()
        };

        // Should return empty vec, not error
        let files = resources.scan_folders().unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_skill_resources_multiple_folders() {
        use std::fs::{self, File};

        let temp_dir1 = std::env::temp_dir().join("skills_multi_test1");
        let temp_dir2 = std::env::temp_dir().join("skills_multi_test2");
        fs::create_dir_all(&temp_dir1).unwrap();
        fs::create_dir_all(&temp_dir2).unwrap();

        let file1 = temp_dir1.join("file1.txt");
        let file2 = temp_dir2.join("file2.txt");

        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

        let resources = SkillResources {
            folders: vec![temp_dir1.clone(), temp_dir2.clone()],
            ..Default::default()
        };

        let files = resources.scan_folders().unwrap();
        assert_eq!(files.len(), 2);

        // Clean up
        fs::remove_file(&file1).unwrap();
        fs::remove_file(&file2).unwrap();
        fs::remove_dir(&temp_dir1).unwrap();
        fs::remove_dir(&temp_dir2).unwrap();
    }
}
