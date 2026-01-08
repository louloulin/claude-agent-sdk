//! Agent Skills - 文件夹资源管理示例
//!
//! 这个示例展示了如何使用 SkillResources 来管理技能的文件夹资源
//!
//! 运行: cargo run --example 33_agent_skills_resources

use claude_agent_sdk_rs::skills::{SkillMetadata, SkillPackage, SkillResources};
use std::fs::{self, File};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Agent Skills - 文件夹资源管理示例\n");

    // 创建临时目录结构用于演示
    let temp_dir = std::env::temp_dir().join("skills_resources_example");
    let resources_dir = temp_dir.join("resources");
    let docs_dir = resources_dir.join("docs");
    let scripts_dir = resources_dir.join("scripts");

    fs::create_dir_all(&docs_dir)?;
    fs::create_dir_all(&scripts_dir)?;

    // 创建一些示例文件
    let readme = docs_dir.join("README.md");
    let guide = docs_dir.join("guide.md");
    let setup_script = scripts_dir.join("setup.sh");
    let run_script = scripts_dir.join("run.sh");

    let mut file = File::create(&readme)?;
    file.write_all(b"# Documentation\n\nThis is a README file.")?;

    let mut file = File::create(&guide)?;
    file.write_all(b"# User Guide\n\nThis is a user guide.")?;

    let mut file = File::create(&setup_script)?;
    file.write_all(b"#!/bin/bash\n# Setup script")?;

    let mut file = File::create(&run_script)?;
    file.write_all(b"#!/bin/bash\n# Run script")?;

    println!("📁 创建了演示目录结构:");
    println!("  resources/");
    println!("    docs/");
    println!("      - README.md");
    println!("      - guide.md");
    println!("    scripts/");
    println!("      - setup.sh");
    println!("      - run.sh");
    println!();

    // 1. 创建 SkillResources 并添加文件夹
    println!("1️⃣  创建 SkillResources");
    let mut resources = SkillResources::default();

    // 添加资源文件夹
    resources.add_folder(&resources_dir);
    println!("   ✅ 添加资源文件夹: {:?}", resources_dir);

    // 添加工具
    resources.add_tool("search".to_string());
    resources.add_tool("analyze".to_string());
    println!("   ✅ 添加工具: search, analyze");

    // 添加测试
    resources.add_test("test_basic".to_string());
    resources.add_test("test_advanced".to_string());
    println!("   ✅ 添加测试: test_basic, test_advanced");
    println!();

    // 2. 验证文件夹
    println!("2️⃣  验证文件夹");
    match resources.validate_folders() {
        Ok(_) => println!("   ✅ 所有文件夹都有效"),
        Err(e) => println!("   ❌ 文件夹验证失败: {}", e),
    }
    println!();

    // 3. 扫描文件夹
    println!("3️⃣  扫描文件夹中的文件");
    let files = resources.scan_folders()?;
    println!("   ✅ 发现 {} 个文件:", files.len());
    for file in &files {
        println!("      - {}", file.display());
    }
    println!();

    // 4. 创建包含资源的 SkillPackage
    println!("4️⃣  创建包含资源的 SkillPackage");
    let skill_package = SkillPackage {
        metadata: SkillMetadata {
            id: "data-processor".to_string(),
            name: "Data Processor Skill".to_string(),
            description: "处理和分析数据的技能".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Claude SDK Team".to_string()),
            dependencies: vec!["serde".to_string(), "tokio".to_string()],
            tags: vec!["data".to_string(), "processing".to_string()],
        },
        instructions: "你是一个专业的数据处理助手。".to_string(),
        scripts: vec!["setup.sh".to_string(), "run.sh".to_string()],
        resources,
    };

    println!("   ✅ SkillPackage 创建完成");
    println!("      ID: {}", skill_package.metadata.id);
    println!("      资源文件夹: {} 个", skill_package.resources.folders.len());
    println!("      工具: {} 个", skill_package.resources.tools.len());
    println!("      测试: {} 个", skill_package.resources.tests.len());
    println!();

    // 5. 保存 SkillPackage
    let package_file = temp_dir.join("data_processor_skill.json");
    skill_package.save_to_file(&package_file)?;
    println!("5️⃣  保存 SkillPackage 到文件:");
    println!("   ✅ {}", package_file.display());
    println!();

    // 6. 加载并验证
    println!("6️⃣  从文件加载并验证");
    let loaded_package = SkillPackage::load_from_file(&package_file)?;
    println!("   ✅ SkillPackage 加载成功");
    println!("      名称: {}", loaded_package.metadata.name);
    println!("      版本: {}", loaded_package.metadata.version);
    println!("      资源文件: {} 个", {
        let files = loaded_package.resources.scan_folders()?;
        files.len()
    });
    println!();

    // 7. 演示重复添加防护
    println!("7️⃣  演示重复添加防护");
    let mut resources_test = SkillResources::default();
    resources_test.add_folder("test");
    resources_test.add_folder("test");
    println!("   ✅ 尝试添加同一文件夹两次");
    println!("      实际文件夹数量: {} (防止了重复)", resources_test.folders.len());
    println!();

    // 清理
    println!("🧹 清理临时文件...");
    fs::remove_file(&package_file)?;
    fs::remove_file(&readme)?;
    fs::remove_file(&guide)?;
    fs::remove_file(&setup_script)?;
    fs::remove_file(&run_script)?;
    fs::remove_dir(&docs_dir)?;
    fs::remove_dir(&scripts_dir)?;
    fs::remove_dir(&resources_dir)?;
    fs::remove_dir(&temp_dir)?;
    println!("   ✅ 清理完成");

    println!("\n✨ 示例运行完成!");
    Ok(())
}
