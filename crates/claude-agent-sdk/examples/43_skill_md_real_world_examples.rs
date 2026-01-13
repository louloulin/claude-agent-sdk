//! Real-World SKILL.md Examples Verification
//!
//! This example verifies the implementation with real-world SKILL.md files
//! covering various domains and use cases.

use claude_agent_sdk::skills::*;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║     Real-World SKILL.md Examples Verification                 ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Find the examples skills directory
    let skills_dir = PathBuf::from("examples/.claude/skills");

    if !skills_dir.exists() {
        println!("❌ Skills directory not found: {:?}", skills_dir);
        println!("   Please run this example from the repository root\n");
        return Ok(());
    }

    println!("📁 Scanning directory: {:?}\n", skills_dir);

    // Discover all SKILL.md files
    let scanner = SkillsDirScanner::new(&skills_dir);
    let skills = scanner.scan()?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Found {} SKILL.md files\n", skills.len());

    // Verify each skill
    for (i, skill) in skills.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Skill #{}", i + 1);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        // Display metadata
        println!("📋 Metadata:");
        println!("   Name:        {}", skill.metadata.name);
        println!("   Description: {}", skill.metadata.description);
        println!("   Version:     {}", skill.metadata.version);
        println!("   Author:      {:?}", skill.metadata.author);
        println!("   Tags:        {:?}", skill.metadata.tags);
        println!("   Dependencies:{:?}", skill.metadata.dependencies);

        // Display resources
        println!("\n📁 Resources:");
        println!("   Skill Dir:  {:?}", skill.skill_dir);
        println!("   Scripts:    {} found", skill.scripts.len());
        println!("   Resources:  {} found", skill.resources.len());
        println!("   Reference:  {}", if skill.reference.is_some() { "✅" } else { "❌" });
        println!("   Forms:      {}", if skill.forms.is_some() { "✅" } else { "❌" });

        // Display content preview
        println!("\n📄 Content Preview:");
        let preview: String = skill.content.chars().take(200).collect();
        println!("   {}...\n", preview);

        // Convert to SkillPackage
        let package = skill.to_skill_package();
        println!("✅ Successfully converted to SkillPackage");
        println!("   Package ID: {}", package.metadata.id);
        println!("   Instructions: {} bytes\n", package.instructions.len());
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Summary Statistics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Calculate statistics
    let total_tags: usize = skills.iter().map(|s| s.metadata.tags.len()).sum();
    let total_deps: usize = skills.iter().map(|s| s.metadata.dependencies.len()).sum();
    let skills_with_scripts = skills.iter().filter(|s| !s.scripts.is_empty()).count();
    let skills_with_resources = skills.iter().filter(|s| !s.resources.is_empty()).count();
    let skills_with_reference = skills.iter().filter(|s| s.reference.is_some()).count();
    let skills_with_forms = skills.iter().filter(|s| s.forms.is_some()).count();

    println!("📊 General:");
    println!("   Total Skills:        {}", skills.len());
    println!("   Total Tags:          {}", total_tags);
    println!("   Total Dependencies:  {}", total_deps);

    println!("\n📁 Resource Distribution:");
    println!("   With Scripts:        {} / {}", skills_with_scripts, skills.len());
    println!("   With Resources:      {} / {}", skills_with_resources, skills.len());
    println!("   With Reference:      {} / {}", skills_with_reference, skills.len());
    println!("   With Forms:          {} / {}", skills_with_forms, skills.len());

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Domain Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Group by tags
    let mut tag_counts = std::collections::HashMap::new();
    for skill in &skills {
        for tag in &skill.metadata.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    println!("🏷️  Tag Distribution:");
    let mut sorted_tags: Vec<_> = tag_counts.into_iter().collect();
    sorted_tags.sort_by(|a, b| b.1.cmp(&a.1));

    for (tag, count) in sorted_tags {
        println!("   {:20} : {} skill(s)", tag, count);
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Verification Tests");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Run verification tests
    let mut tests_passed = 0;
    let mut tests_total = 0;

    // Test 1: All skills have required metadata
    tests_total += 1;
    let all_have_metadata = skills.iter().all(|s| {
        !s.metadata.name.is_empty()
            && !s.metadata.description.is_empty()
            && !s.metadata.version.is_empty()
    });
    if all_have_metadata {
        println!("✅ Test 1: All skills have required metadata");
        tests_passed += 1;
    } else {
        println!("❌ Test 1: Some skills missing required metadata");
    }

    // Test 2: All skills have content
    tests_total += 1;
    let all_have_content = skills.iter().all(|s| !s.content.is_empty());
    if all_have_content {
        println!("✅ Test 2: All skills have content");
        tests_passed += 1;
    } else {
        println!("❌ Test 2: Some skills have empty content");
    }

    // Test 3: All skills can be converted to SkillPackage
    tests_total += 1;
    let all_convertible = skills.iter().all(|s| {
        let package = s.to_skill_package();
        !package.metadata.id.is_empty()
            && !package.metadata.name.is_empty()
            && !package.instructions.is_empty()
    });
    if all_convertible {
        println!("✅ Test 3: All skills convertible to SkillPackage");
        tests_passed += 1;
    } else {
        println!("❌ Test 3: Some skills failed conversion");
    }

    // Test 4: Version format consistency
    tests_total += 1;
    let valid_versions = skills.iter().all(|s| {
        s.metadata.version.parse::<semver::Version>().is_ok()
    });
    if valid_versions {
        println!("✅ Test 4: All versions follow semantic versioning");
        tests_passed += 1;
    } else {
        println!("❌ Test 4: Some versions don't follow semver");
    }

    // Test 5: Unique skill IDs
    tests_total += 1;
    let packages: Vec<_> = skills.iter().map(|s| s.to_skill_package()).collect();
    let mut ids = std::collections::HashSet::new();
    let all_unique = packages.iter().all(|p| ids.insert(p.metadata.id.clone()));
    if all_unique {
        println!("✅ Test 5: All skill IDs are unique");
        tests_passed += 1;
    } else {
        println!("❌ Test 5: Duplicate skill IDs found");
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test Results: {} / {} passed", tests_passed, tests_total);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    if tests_passed == tests_total {
        println!("🎉 All verification tests passed!\n");

        println!("╔═══════════════════════════════════════════════════════════════╗");
        println!("║  ✅ Real-World SKILL.md Implementation Verified              ║");
        println!("╚═══════════════════════════════════════════════════════════════╝\n");

        println!("The following real-world skills have been successfully loaded:");
        for skill in &skills {
            println!("  • {}", skill.metadata.name);
        }
        println!();

    } else {
        println!("⚠️  Some tests failed. Please review the implementation.\n");
    }

    Ok(())
}
