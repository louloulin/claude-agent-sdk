#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct TestSkill;

    #[async_trait]
    impl Skill for TestSkill {
        fn name(&self) -> String {
            "test".to_string()
        }

        fn description(&self) -> String {
            "Test skill".to_string()
        }

        async fn execute(&self, _input: SkillInput) -> SkillResult {
            Ok(SkillOutput::ok("test result"))
        }

        fn validate(&self) -> Result<(), SkillError> {
            Ok(())
        }
    }

    #[test]
    fn test_registry() {
        let mut registry = SkillRegistry::new();
        let skill = Box::new(TestSkill);
        registry.register(skill).unwrap();
        assert_eq!(registry.list(), vec!["test".to_string()]);
    }

    #[test]
    fn test_skill_metadata() {
        let meta = SkillMetadata {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test skill".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            dependencies: vec![],
            tags: vec![],
        };
        assert_eq!(meta.id, "test");
    }
}
