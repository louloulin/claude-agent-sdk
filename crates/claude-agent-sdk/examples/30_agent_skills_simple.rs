//! Simple example of Agent Skills system

use async_trait::async_trait;
use claude_agent_sdk::skills::*;

#[derive(Debug)]
struct HelloSkill;

#[async_trait]
impl Skill for HelloSkill {
    fn name(&self) -> String {
        "hello".to_string()
    }

    fn description(&self) -> String {
        "Says hello".to_string()
    }

    async fn execute(&self, _input: SkillInput) -> SkillResult {
        Ok(SkillOutput::ok("Hello from Agent Skills!"))
    }

    fn validate(&self) -> Result<(), SkillError> {
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = SkillRegistry::new();
    registry.register(Box::new(HelloSkill))?;

    println!("✅ Registered skills: {:?}", registry.list());

    if let Some(skill) = registry.get("hello") {
        println!("✅ Found skill: {}", skill.name());
        println!("📝 Description: {}", skill.description());
    }

    Ok(())
}
