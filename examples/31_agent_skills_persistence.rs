//! Agent Skills Persistence Example
//!
//! This example demonstrates how to save and load SkillPackages

use claude_agent_sdk_rs::skills::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a skill package
    let package = SkillPackage {
        metadata: SkillMetadata {
            id: "calculator".to_string(),
            name: "Calculator".to_string(),
            description: "A simple calculator skill".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Claude Agent Team".to_string()),
            dependencies: vec![],
            tags: vec!["math".to_string(), "utility".to_string()],
        },
        instructions: r#"You are a calculator. 
When given mathematical expressions, evaluate them and return the result.
Support basic operations: +, -, *, /"#
            .to_string(),
        scripts: vec![
            "function add(a, b) { return a + b; }".to_string(),
            "function subtract(a, b) { return a - b; }".to_string(),
        ],
        resources: SkillResources {
            folders: vec![],
            tools: vec!["calculate".to_string()],
            tests: vec!["test_calculator.js".to_string()],
        },
    };

    println!("📦 Created skill package: {}", package.metadata.name);

    // Save to JSON file
    let json_path = "/tmp/calculator_skill.json";
    package.save_to_file(json_path)?;
    println!("✅ Saved skill package to: {}", json_path);

    // Load from JSON file
    let loaded_package = SkillPackage::load_from_file(json_path)?;
    println!("📥 Loaded skill package: {}", loaded_package.metadata.name);
    println!("   Version: {}", loaded_package.metadata.version);
    println!("   Author: {:?}", loaded_package.metadata.author);
    println!("   Tags: {:?}", loaded_package.metadata.tags);
    println!(
        "   Instructions: {}",
        loaded_package
            .instructions
            .chars()
            .take(50)
            .collect::<String>()
            + "..."
    );
    println!("   Scripts: {} script(s)", loaded_package.scripts.len());
    println!("   Tools: {:?}", loaded_package.resources.tools);

    // Verify integrity
    assert_eq!(package.metadata.id, loaded_package.metadata.id);
    assert_eq!(package.metadata.name, loaded_package.metadata.name);
    assert_eq!(package.metadata.version, loaded_package.metadata.version);
    println!("\n✅ Integrity check passed!");

    Ok(())
}
