//! Comprehensive SKILL.md Implementation Test Suite
//!
//! This example performs comprehensive testing of all SKILL.md functionality
//! including parsing, resource discovery, conversion, and edge cases.

use claude_agent_sdk::skills::*;
use std::path::PathBuf;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║     Comprehensive SKILL.md Implementation Test Suite            ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    let mut total_tests = 0;
    let mut passed_tests = 0;

    // Test Suite 1: Basic Parsing
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test Suite 1: Basic Parsing");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let skills_dir = PathBuf::from("examples/.claude/skills");

    if !skills_dir.exists() {
        println!("❌ Skills directory not found: {:?}", skills_dir);
        println!("   Please run this example from the repository root\n");
        return Ok(());
    }

    let scanner = SkillsDirScanner::new(&skills_dir);
    let all_skills = scanner.scan()?;

    println!("✅ Found {} SKILL.md files\n", all_skills.len());

    // Test Suite 2: Metadata Validation
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test Suite 2: Metadata Validation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    total_tests += 1;
    if validate_metadata(&all_skills) {
        println!("✅ Test 2.1: Metadata validation passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 2.1: Metadata validation failed");
    }

    total_tests += 1;
    if validate_semver(&all_skills) {
        println!("✅ Test 2.2: Semantic versioning validation passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 2.2: Semantic versioning validation failed");
    }

    total_tests += 1;
    if validate_tags(&all_skills) {
        println!("✅ Test 2.3: Tags validation passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 2.3: Tags validation failed");
    }

    println!();

    // Test Suite 3: Content Validation
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test Suite 3: Content Validation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    total_tests += 1;
    if validate_content_structure(&all_skills) {
        println!("✅ Test 3.1: Content structure validation passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 3.1: Content structure validation failed");
    }

    total_tests += 1;
    if validate_content_length(&all_skills) {
        println!("✅ Test 3.2: Content length validation passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 3.2: Content length validation failed");
    }

    total_tests += 1;
    if validate_markdown_format(&all_skills) {
        println!("✅ Test 3.3: Markdown format validation passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 3.3: Markdown format validation failed");
    }

    println!();

    // Test Suite 4: Resource Discovery
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test Suite 4: Resource Discovery");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    total_tests += 1;
    if validate_resource_discovery(&all_skills) {
        println!("✅ Test 4.1: Resource discovery validation passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 4.1: Resource discovery validation failed");
    }

    total_tests += 1;
    if test_deployment_automation_resources() {
        println!("✅ Test 4.2: Deployment-automation resources validation passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 4.2: Deployment-automation resources validation failed");
    }

    println!();

    // Test Suite 5: SkillPackage Conversion
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test Suite 5: SkillPackage Conversion");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    total_tests += 1;
    if validate_package_conversion(&all_skills) {
        println!("✅ Test 5.1: Package conversion validation passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 5.1: Package conversion validation failed");
    }

    total_tests += 1;
    if validate_unique_ids(&all_skills) {
        println!("✅ Test 5.2: Unique ID generation validation passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 5.2: Unique ID generation validation failed");
    }

    println!();

    // Test Suite 6: SkillRegistry Integration
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test Suite 6: SkillRegistry Integration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    total_tests += 1;
    if test_registry_discovery(&skills_dir) {
        println!("✅ Test 6.1: Registry discovery validation passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 6.1: Registry discovery validation failed");
    }

    total_tests += 1;
    if test_multiple_directory_discovery() {
        println!("✅ Test 6.2: Multiple directory discovery validation passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 6.2: Multiple directory discovery validation failed");
    }

    println!();

    // Test Suite 7: Edge Cases
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test Suite 7: Edge Cases");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    total_tests += 1;
    if test_empty_directory() {
        println!("✅ Test 7.1: Empty directory handling passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 7.1: Empty directory handling failed");
    }

    total_tests += 1;
    if test_nonexistent_directory() {
        println!("✅ Test 7.2: Nonexistent directory handling passed");
        passed_tests += 1;
    } else {
        println!("❌ Test 7.2: Nonexistent directory handling failed");
    }

    println!();

    // Test Suite 8: Statistics
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test Suite 8: Statistics & Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    print_statistics(&all_skills);

    println!();

    // Final Results
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Final Test Results");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let pass_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
    println!("📊 Tests Passed: {}/{} ({:.1}%)", passed_tests, total_tests, pass_rate);

    if passed_tests == total_tests {
        println!("\n╔═══════════════════════════════════════════════════════════════╗");
        println!("║  🎉 All Tests Passed! SKILL.md Implementation Verified ✅       ║");
        println!("╚═══════════════════════════════════════════════════════════════╝\n");

        println!("Implementation verified:");
        println!("  ✅ YAML frontmatter parsing");
        println!("  ✅ Markdown content extraction");
        println!("  ✅ Resource discovery");
        println!("  ✅ SkillPackage conversion");
        println!("  ✅ SkillRegistry integration");
        println!("  ✅ Error handling");
        println!("  ✅ Edge case handling");
        println!();

    } else {
        println!("\n⚠️  Some tests failed. Please review the implementation.\n");
    }

    Ok(())
}

// Validation Functions

fn validate_metadata(skills: &[SkillMdFile]) -> bool {
    for skill in skills {
        if skill.metadata.name.is_empty() {
            println!("   ❌ Empty name in {:?}", skill.skill_dir);
            return false;
        }
        if skill.metadata.description.is_empty() {
            println!("   ❌ Empty description in {}", skill.metadata.name);
            return false;
        }
        if skill.metadata.version.is_empty() {
            println!("   ❌ Empty version in {}", skill.metadata.name);
            return false;
        }
    }
    true
}

fn validate_semver(skills: &[SkillMdFile]) -> bool {
    for skill in skills {
        if skill.metadata.version.parse::<semver::Version>().is_err() {
            println!("   ❌ Invalid semver version '{}' in {}",
                skill.metadata.version, skill.metadata.name);
            return false;
        }
    }
    true
}

fn validate_tags(skills: &[SkillMdFile]) -> bool {
    for skill in skills {
        for tag in &skill.metadata.tags {
            if tag.is_empty() || tag.contains(' ') {
                println!("   ❌ Invalid tag '{}' in {}", tag, skill.metadata.name);
                return false;
            }
        }
    }
    true
}

fn validate_content_structure(skills: &[SkillMdFile]) -> bool {
    for skill in skills {
        if skill.content.is_empty() {
            println!("   ❌ Empty content in {}", skill.metadata.name);
            return false;
        }
        if !skill.content.contains('#') {
            println!("   ⚠️  No headers found in {}", skill.metadata.name);
        }
    }
    true
}

fn validate_content_length(skills: &[SkillMdFile]) -> bool {
    for skill in skills {
        if skill.content.len() < 50 {
            println!("   ❌ Content too short (< 50 chars) in {}", skill.metadata.name);
            return false;
        }
    }
    true
}

fn validate_markdown_format(skills: &[SkillMdFile]) -> bool {
    for skill in skills {
        // Check for balanced code blocks
        let code_blocks = skill.content.matches("```").count();
        if code_blocks % 2 != 0 {
            println!("   ❌ Unbalanced code blocks in {}", skill.metadata.name);
            return false;
        }
    }
    true
}

fn validate_resource_discovery(skills: &[SkillMdFile]) -> bool {
    for skill in skills {
        // Validate that resource paths exist if they're supposed to
        for script in &skill.scripts {
            if !script.exists() {
                println!("   ❌ Script path doesn't exist: {:?}", script);
                return false;
            }
        }
        for resource in &skill.resources {
            if !resource.exists() {
                println!("   ❌ Resource path doesn't exist: {:?}", resource);
                return false;
            }
        }
        if let Some(ref reference) = skill.reference {
            if !reference.exists() {
                println!("   ❌ Reference path doesn't exist: {:?}", reference);
                return false;
            }
        }
        if let Some(ref forms) = skill.forms {
            if !forms.exists() {
                println!("   ❌ Forms path doesn't exist: {:?}", forms);
                return false;
            }
        }
    }
    true
}

fn test_deployment_automation_resources() -> bool {
    let deploy_skill_dir = PathBuf::from("examples/.claude/skills/deployment-automation");
    let deploy_skill = match SkillMdFile::parse(deploy_skill_dir.join("SKILL.md")) {
        Ok(skill) => skill,
        Err(e) => {
            println!("   ❌ Failed to parse deployment-automation SKILL.md: {}", e);
            return false;
        }
    };

    // Should have 2 scripts
    if deploy_skill.scripts.len() != 2 {
        println!("   ❌ Expected 2 scripts, found {}", deploy_skill.scripts.len());
        return false;
    }

    // Should have 1 resource
    if deploy_skill.resources.len() != 1 {
        println!("   ❌ Expected 1 resource, found {}", deploy_skill.resources.len());
        return false;
    }

    // Should have reference.md
    if deploy_skill.reference.is_none() {
        println!("   ❌ Expected reference.md to be found");
        return false;
    }

    true
}

fn validate_package_conversion(skills: &[SkillMdFile]) -> bool {
    for skill in skills {
        let package = skill.to_skill_package();

        if package.metadata.id.is_empty() {
            println!("   ❌ Empty package ID for {}", skill.metadata.name);
            return false;
        }

        if package.metadata.name != skill.metadata.name {
            println!("   ❌ Name mismatch in {}", skill.metadata.name);
            return false;
        }

        if package.instructions.is_empty() {
            println!("   ❌ Empty instructions in {}", skill.metadata.name);
            return false;
        }

        if package.instructions != skill.content {
            println!("   ❌ Content mismatch in {}", skill.metadata.name);
            return false;
        }
    }
    true
}

fn validate_unique_ids(skills: &[SkillMdFile]) -> bool {
    let mut ids = std::collections::HashSet::new();

    for skill in skills {
        let package = skill.to_skill_package();
        if !ids.insert(package.metadata.id.clone()) {
            println!("   ❌ Duplicate ID: {}", package.metadata.id);
            return false;
        }
    }

    true
}

fn test_registry_discovery(skills_dir: &PathBuf) -> bool {
    match SkillRegistry::discover_skill_md_from_dir(skills_dir) {
        Ok(packages) => {
            if packages.is_empty() {
                println!("   ❌ No packages discovered");
                return false;
            }
            true
        }
        Err(e) => {
            println!("   ❌ Registry discovery failed: {}", e);
            false
        }
    }
}

fn test_multiple_directory_discovery() -> bool {
    let temp_dir = std::env::temp_dir().join("skill_test_multiple");

    // Create two test directories
    let dir1 = temp_dir.join("skills1");
    let dir2 = temp_dir.join("skills2");
    fs::create_dir_all(&dir1).unwrap();
    fs::create_dir_all(&dir2).unwrap();

    // Create test skills
    fs::write(
        dir1.join("test1").join("SKILL.md"),
        "---\nname: \"Test1\"\ndescription: \"Test\"\nversion: \"1.0.0\"\n---\n\n# Test1"
    ).unwrap();
    fs::write(
        dir2.join("test2").join("SKILL.md"),
        "---\nname: \"Test2\"\ndescription: \"Test\"\nversion: \"1.0.0\"\n---\n\n# Test2"
    ).unwrap();

    let result = match SkillRegistry::discover_from_multiple_dirs(vec![dir1, dir2]) {
        Ok(packages) => packages.len() >= 2,
        Err(_) => false,
    };

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();

    result
}

fn test_empty_directory() -> bool {
    let temp_dir = std::env::temp_dir().join("skill_test_empty");
    fs::create_dir_all(&temp_dir).unwrap();

    let scanner = SkillsDirScanner::new(&temp_dir);
    let result = match scanner.scan() {
        Ok(skills) => skills.is_empty(),
        Err(_) => false,
    };

    fs::remove_dir(&temp_dir).ok();
    result
}

fn test_nonexistent_directory() -> bool {
    let temp_dir = std::env::temp_dir().join("skill_test_nonexistent");

    let scanner = SkillsDirScanner::new(&temp_dir);
    match scanner.scan() {
        Ok(skills) => skills.is_empty(),
        Err(_) => false,
    }
}

fn print_statistics(skills: &[SkillMdFile]) {
    println!("📊 Skill Statistics:\n");

    let total_content_len: usize = skills.iter().map(|s| s.content.len()).sum();
    let avg_content_len = total_content_len / skills.len();

    let total_tags: usize = skills.iter().map(|s| s.metadata.tags.len()).sum();
    let total_deps: usize = skills.iter().map(|s| s.metadata.dependencies.len()).sum();

    let skills_with_scripts = skills.iter().filter(|s| !s.scripts.is_empty()).count();
    let skills_with_resources = skills.iter().filter(|s| !s.resources.is_empty()).count();
    let skills_with_reference = skills.iter().filter(|s| s.reference.is_some()).count();

    println!("   Total Skills:            {}", skills.len());
    println!("   Average Content Length:  {} bytes", avg_content_len);
    println!("   Total Tags:              {}", total_tags);
    println!("   Total Dependencies:      {}", total_deps);
    println!("   Skills with Scripts:     {}", skills_with_scripts);
    println!("   Skills with Resources:   {}", skills_with_resources);
    println!("   Skills with Reference:   {}", skills_with_reference);

    println!("\n🏷️  Tag Distribution:");
    let mut tag_counts = std::collections::HashMap::new();
    for skill in skills {
        for tag in &skill.metadata.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    let mut sorted_tags: Vec<_> = tag_counts.into_iter().collect();
    sorted_tags.sort_by_key(|b| std::cmp::Reverse(b.1));

    for (tag, count) in sorted_tags.iter().take(10) {
        println!("   {:20} : {}", tag, count);
    }

    println!("\n📂 Skills with Most Resources:");
    let mut resource_counts: Vec<_> = skills.iter().collect();
    resource_counts.sort_by(|a, b| {
        (b.scripts.len() + b.resources.len()).cmp(&(a.scripts.len() + a.resources.len()))
    });

    for skill in resource_counts.iter().take(5) {
        let total = skill.scripts.len() + skill.resources.len();
        println!("   {:20} : {} resources", skill.metadata.name, total);
    }
}
