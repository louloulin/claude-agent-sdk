//! Example: Using Agent Skills System
//!
//! This example demonstrates how to use the Agent Skills system
//! (Anthropic's open standard announced 2025-12-18) to create
//! modular, reusable AI capabilities.

use claude_agent_sdk_rs::skills::*;
use async_trait::async_trait;

/// A simple skill that calculates Fibonacci numbers
struct FibonacciSkill;

#[async_trait]
impl Skill for FibonacciSkill {
    fn name(&self) -> String {
        "fibonacci".to_string()
    }

    fn description(&self) -> String {
        "Calculates Fibonacci numbers".to_string()
    }

    async fn execute(&self, input: SkillInput) -> SkillResult {
        let n = input.params["n"]
            .as_u64()
            .ok_or_else(|| SkillError::Validation("Missing 'n' parameter".to_string()))?;

        if n > 93 {
            return Err(SkillError::Validation("n must be <= 93 to avoid overflow".to_string()));
        }

        let result = fibonacci(n);
        Ok(SkillOutput::ok(serde_json::json!({
            "result": result,
            "n": n
        })))
    }

    fn validate(&self) -> Result<()> {
        Ok(())
    }

    fn version(&self) -> String {
        "1.0.0".to_string()
    }

    fn tags(&self) -> Vec<String> {
        vec!["math".to_string(), "calculation".to_string()]
    }
}

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0;
            let mut b = 1;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a skill registry
    let registry = SkillRegistry::new();

    // Register the Fibonacci skill
    let skill = SkillBox::new(FibonacciSkill);
    registry.register_skill(skill).await?;

    println!("✅ Registered Fibonacci skill");
    println!("📋 Available skills: {:?}", registry.list_skills().await);

    // Execute the skill
    if let Some(skill) = registry.get_skill("fibonacci").await {
        println!("\n🔢 Calculating Fibonacci(10)...");

        let input = SkillInput {
            params: serde_json::json!({"n": 10}),
            ..Default::default()
        };

        match skill.execute(input).await {
            Ok(output) => {
                if output.success {
                    println!("✅ Result: {}", output.data);
                } else {
                    println!("❌ Error: {:?}", output.error);
                }
            }
            Err(e) => {
                println!("❌ Execution error: {}", e);
            }
        }
    }

    // Find skills by tag
    println!("\n🔍 Skills with 'math' tag:");
    for skill in registry.find_by_tag("math").await {
        println!("  - {} ({})", skill.name(), skill.description());
    }

    Ok(())
}
