//! SKILL.md Integration Example
//!
//! This example demonstrates the complete SKILL.md filesystem integration,
//! including YAML frontmatter parsing, directory scanning, and resource discovery.

use claude_agent_sdk::skills::*;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║        SKILL.md Integration Example                              ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Create a temporary directory structure for demo
    let demo_dir = std::env::temp_dir().join("skill_md_demo");
    let skills_dir = demo_dir.join(".claude").join("skills");
    fs::create_dir_all(&skills_dir)?;

    println!("📁 Created demo directory: {:?}", demo_dir);
    println!("📁 Skills directory: {:?}\n", skills_dir);

    // Create example skill 1: Calculator with complete metadata
    let calc_skill_dir = skills_dir.join("calculator");
    fs::create_dir_all(&calc_skill_dir)?;
    fs::write(
        calc_skill_dir.join("SKILL.md"),
        r#"---
name: "Calculator"
description: "Performs mathematical calculations with support for basic operations"
version: "1.0.0"
author: "Math Team <math@example.com>"
tags:
  - math
  - calculator
  - utility
dependencies:
  - logger
---

# Calculator Skill

You are a calculator assistant. When given mathematical expressions, evaluate them
and provide the result with step-by-step explanations.

## Supported Operations

- Addition (+)
- Subtraction (-)
- Multiplication (*)
- Division (/)
- Modulo (%)

## Usage

Simply provide a mathematical expression, and I'll calculate the result.

Example: "What is 2 + 2 * 3?"

## Notes

- Follow standard order of operations (PEMDAS)
- Handle division by zero gracefully
- Provide clear error messages for invalid input
"#,
    )?;

    // Create scripts directory with example script
    let calc_scripts = calc_skill_dir.join("scripts");
    fs::create_dir_all(&calc_scripts)?;
    fs::write(
        calc_scripts.join("add.js"),
        r#"function add(a, b) {
    return a + b;
}
"#,
    )?;

    // Create resources directory
    let calc_resources = calc_skill_dir.join("resources");
    fs::create_dir_all(&calc_resources)?;
    fs::write(
        calc_resources.join("help.txt"),
        "Calculator Help: Available operations: +, -, *, /, %",
    )?;

    println!("✅ Created Calculator skill");

    // Create example skill 2: Translator with minimal metadata
    let trans_skill_dir = skills_dir.join("translator");
    fs::create_dir_all(&trans_skill_dir)?;
    fs::write(
        trans_skill_dir.join("SKILL.md"),
        r#"---
name: "Translator"
description: "Translates text between multiple languages"
version: "2.1.0"
---

# Translator Skill

You are a translation assistant. Translate the given text to the target language
while preserving meaning, tone, and context.

## Supported Languages

- English
- Spanish
- French
- German
- Chinese
- Japanese

## Best Practices

- Preserve idioms and cultural references
- Maintain formal/informal tone
- Ask for clarification if ambiguous
"#,
    )?;

    // Create reference.md
    fs::write(
        trans_skill_dir.join("reference.md"),
        r#"# Translation Reference

## Common Phrases

- Hello -> Hola (ES), Bonjour (FR)
- Thank you -> Gracias (ES), Merci (FR)
- Goodbye -> Adiós (ES), Au revoir (FR)
"#,
    )?;

    println!("✅ Created Translator skill");

    // Create example skill 3: Git Helper
    let git_skill_dir = skills_dir.join("git-helper");
    fs::create_dir_all(&git_skill_dir)?;
    fs::write(
        git_skill_dir.join("SKILL.md"),
        r#"---
name: "Git Helper"
description: "Assists with Git version control operations"
version: "1.5.0"
author: "DevOps Team"
tags:
  - git
  - version-control
  - devops
dependencies: []
---

# Git Helper

You are a Git assistant. Help users with Git operations, explain commands,
and troubleshoot common issues.

## Common Tasks

- Initialize repositories
- Commit changes
- Create branches
- Merge changes
- Resolve conflicts
"#,
    )?;

    println!("✅ Created Git Helper skill\n");

    // Demonstrate different discovery methods
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Method 1: Parse Individual SKILL.md Files");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let calc_md = SkillMdFile::parse(calc_skill_dir.join("SKILL.md"))?;
    println!("📄 Parsed: {}", calc_md.metadata.name);
    println!("   Description: {}", calc_md.metadata.description);
    println!("   Version: {}", calc_md.metadata.version);
    println!("   Author: {:?}", calc_md.metadata.author);
    println!("   Tags: {:?}", calc_md.metadata.tags);
    println!("   Dependencies: {:?}", calc_md.metadata.dependencies);
    println!("   Scripts: {} found", calc_md.scripts.len());
    println!("   Resources: {} found", calc_md.resources.len());
    println!("   Reference: {:?}", calc_md.reference.is_some());
    println!("   Forms: {:?}", calc_md.forms.is_some());
    println!();

    // Convert to SkillPackage
    let calc_package = calc_md.to_skill_package();
    println!("✅ Converted to SkillPackage");
    println!("   Package ID: {}", calc_package.metadata.id);
    println!("   Instructions: {} chars", calc_package.instructions.len());
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Method 2: Scan Skills Directory");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let scanner = SkillsDirScanner::new(&skills_dir);
    let skills = scanner.scan()?;

    println!("🔍 Scanned directory: {:?}", skills_dir);
    println!("📦 Found {} skill(s):\n", skills.len());

    for (i, skill) in skills.iter().enumerate() {
        println!("{}. {}", i + 1, skill.metadata.name);
        println!("   Version: {}", skill.metadata.version);
        println!("   Description: {}", skill.metadata.description);
        println!("   Scripts: {}", skill.scripts.len());
        println!("   Resources: {}", skill.resources.len());
        println!("   Has Reference: {}", skill.reference.is_some());
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Method 3: Discover via SkillRegistry");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let packages = SkillRegistry::discover_skill_md_from_dir(&skills_dir)?;

    println!("📦 Discovered {} skill package(s):\n", packages.len());

    for (i, package) in packages.iter().enumerate() {
        println!("{}. {}", i + 1, package.metadata.name);
        println!("   ID: {}", package.metadata.id);
        println!("   Version: {}", package.metadata.version);
        println!("   Author: {:?}", package.metadata.author);
        println!("   Tags: {:?}", package.metadata.tags);
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Method 4: Scan from Project Directory");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let project_scanner = SkillsDirScanner::from_project_dir(&demo_dir);
    let project_skills = project_scanner.scan()?;

    println!("🔍 Scanned project: {:?}", demo_dir);
    println!("📦 Found {} skill(s) in .claude/skills/", project_skills.len());
    println!();

    // Verify results
    assert_eq!(skills.len(), 3, "Should find 3 skills");
    assert_eq!(packages.len(), 3, "Should create 3 packages");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ All tests passed!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Cleanup
    fs::remove_dir_all(&demo_dir)?;
    println!("🧹 Cleaned up demo directory\n");

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  Summary: SKILL.md Integration is Fully Functional             ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");

    Ok(())
}
