//! Example: Using Agent Skills System
//!
//! This example demonstrates how to use the Agent Skills system
//! (Anthropic's open standard announced 2025-12-18) to create
//! modular, reusable AI capabilities.

use async_trait::async_trait;
use claude_agent_sdk::skills::*;

/// A simple skill that calculates Fibonacci numbers
#[derive(Debug)]
struct FibonacciSkill;

#[async_trait]
impl Skill for FibonacciSkill {
    fn name(&self) -> String {
        "fibonacci".to_string()
    }

    fn description(&self) -> String {
        "Calculates Fibonacci numbers".to_string()
    }

    async fn execute(&self, _input: SkillInput) -> SkillResult {
        // For simplicity, return Fibonacci(10)
        let result = fibonacci(10);
        Ok(SkillOutput::ok(serde_json::json!({
            "result": result,
            "n": 10
        })))
    }

    fn validate(&self) -> Result<(), SkillError> {
        Ok(())
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
        },
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a skill registry
    let mut registry = SkillRegistry::new();

    // Register the Fibonacci skill
    registry.register(Box::new(FibonacciSkill))?;

    println!("✅ Registered Fibonacci skill");
    println!("📋 Available skills: {:?}", registry.list());

    // Execute the skill
    if let Some(skill) = registry.get("fibonacci") {
        println!("\n🔢 Calculating Fibonacci(10)...");

        let input = SkillInput::default();

        // Since execute is async, we need a runtime
        let rt = tokio::runtime::Runtime::new()?;
        match rt.block_on(skill.execute(input)) {
            Ok(output) => {
                if output.success {
                    println!("✅ Result: {}", output.data);
                } else {
                    println!("❌ Error: {:?}", output.error);
                }
            },
            Err(e) => {
                println!("❌ Execution error: {}", e);
            },
        }
    }

    Ok(())
}
