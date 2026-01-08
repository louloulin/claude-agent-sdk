//! Tests for the Skills system

use super::*;
use async_trait::async_trait;

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

    async fn execute(&self, input: SkillInput) -> SkillResult {
        Ok(SkillOutput::ok(format!("Executed with params: {:?}", input.params)))
    }

    fn validate(&self) -> std::result::Result<(), SkillError> {
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
}
