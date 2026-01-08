//! Agent Skills Discovery Example
//!
//! This example demonstrates how to discover and load skill packages from a directory

use claude_agent_sdk_rs::skills::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory with skill packages
    let skills_dir = std::env::temp_dir().join("demo_skills");
    std::fs::create_dir_all(&skills_dir)?;

    println!("📁 Creating demo skills in: {:?}", skills_dir);

    // Create first skill package
    let calculator_skill = SkillPackage {
        metadata: SkillMetadata {
            id: "calculator".to_string(),
            name: "Calculator".to_string(),
            description: "Performs mathematical calculations".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Math Team".to_string()),
            dependencies: vec![],
            tags: vec!["math".to_string(), "utility".to_string()],
        },
        instructions: r#"You are a calculator assistant.
When given mathematical expressions, evaluate them and provide the result.
Support basic operations: addition, subtraction, multiplication, division."#
            .to_string(),
        scripts: vec![
            "function add(a, b) { return a + b; }".to_string(),
            "function multiply(a, b) { return a * b; }".to_string(),
        ],
        resources: SkillResources {
            folders: vec![],
            tools: vec!["calculate".to_string()],
            tests: vec!["calc_test.json".to_string()],
        },
    };

    let calc_path = skills_dir.join("calculator.json");
    calculator_skill.save_to_file(&calc_path)?;
    println!("✅ Created: calculator.json");

    // Create second skill package
    let translator_skill = SkillPackage {
        metadata: SkillMetadata {
            id: "translator".to_string(),
            name: "Translator".to_string(),
            description: "Translates text between languages".to_string(),
            version: "1.2.0".to_string(),
            author: Some("I18n Team".to_string()),
            dependencies: vec![],
            tags: vec!["translation".to_string(), "text".to_string()],
        },
        instructions: r#"You are a translation assistant.
Translate the given text to the target language while preserving meaning and tone."#
            .to_string(),
        scripts: vec!["function translate(text, targetLang) { ... }".to_string()],
        resources: SkillResources::default(),
    };

    let trans_path = skills_dir.join("translator.json");
    translator_skill.save_to_file(&trans_path)?;
    println!("✅ Created: translator.json");

    // Discover all skills from directory
    println!("\n🔍 Discovering skills from directory...");
    let discovered_skills = SkillRegistry::discover_from_dir(&skills_dir)?;

    println!("\n📦 Found {} skill package(s):", discovered_skills.len());

    for (i, skill) in discovered_skills.iter().enumerate() {
        println!("\n{}. {}", i + 1, skill.metadata.name);
        println!("   ID: {}", skill.metadata.id);
        println!("   Version: {}", skill.metadata.version);
        println!("   Author: {:?}", skill.metadata.author);
        println!("   Description: {}", skill.metadata.description);
        println!("   Tags: {:?}", skill.metadata.tags);
        println!("   Scripts: {} script(s)", skill.scripts.len());
        println!("   Tools: {:?}", skill.resources.tools);
    }

    // Verify integrity
    assert_eq!(discovered_skills.len(), 2);
    assert_eq!(discovered_skills[0].metadata.id, "calculator");
    assert_eq!(discovered_skills[1].metadata.id, "translator");

    println!("\n✅ Discovery successful!");

    // Cleanup
    std::fs::remove_file(&calc_path)?;
    std::fs::remove_file(&trans_path)?;
    std::fs::remove_dir(&skills_dir)?;
    println!("\n🧹 Cleaned up demo files");

    Ok(())
}
