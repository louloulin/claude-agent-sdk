//! Agent Skills - YAML 配置支持示例
//!
//! 这个示例展示了如何使用 YAML 格式保存和加载 Agent Skills 配置
//!
//! 运行: cargo run --example 37_agent_skills_yaml --features yaml

use claude_agent_sdk_rs::skills::{SkillMetadata, SkillPackage, SkillResources};
use std::path::PathBuf;

fn main() {
    println!("📝 Agent Skills - YAML 配置支持示例\n");

    // 1. 创建完整的技能包
    println!("1️⃣  创建技能包");
    let skill = SkillPackage {
        metadata: SkillMetadata {
            id: "code-reviewer".to_string(),
            name: "Code Reviewer".to_string(),
            description: "AI-powered code review assistant that analyzes code quality, security vulnerabilities, and best practices".to_string(),
            version: "1.2.0".to_string(),
            author: Some("Claude SDK Team".to_string()),
            dependencies: vec![
                "git-parser >= 1.0".to_string(),
                "linter-engine >= 2.0".to_string(),
            ],
            tags: vec![
                "code-review".to_string(),
                "security".to_string(),
                "quality".to_string(),
                "development".to_string(),
            ],
        },

        instructions: r#"# Code Review Instructions

You are an expert code reviewer with deep knowledge of:
- Security best practices and vulnerability detection
- Code quality metrics and maintainability
- Performance optimization techniques
- Industry-standard coding patterns

## Review Process

1. **Security Analysis**: Check for common vulnerabilities (SQL injection, XSS, etc.)
2. **Code Quality**: Assess readability, maintainability, and adherence to standards
3. **Performance**: Identify potential bottlenecks and optimization opportunities
4. **Best Practices**: Verify compliance with language-specific best practices

## Output Format

Provide structured feedback with:
- Critical issues (must fix)
- Warnings (should fix)
- Suggestions (nice to have)
- Positive feedback (what's done well)"#
            .to_string(),

        scripts: vec![
            "scripts/setup_env.sh".to_string(),
            "scripts/analyze_code.py".to_string(),
            "scripts/generate_report.sh".to_string(),
        ],

        resources: SkillResources {
            folders: vec![
                PathBuf::from("./rules"),
                PathBuf::from("./templates"),
                PathBuf::from("./examples"),
            ],
            tools: vec![
                "git".to_string(),
                "eslint".to_string(),
                "pylint".to_string(),
                "security-scanner".to_string(),
            ],
            tests: vec![
                "test_basic_review".to_string(),
                "test_security_scan".to_string(),
                "test_performance_check".to_string(),
            ],
        },
    };

    println!(
        "   ✅ 创建技能: {} (v{})",
        skill.metadata.name, skill.metadata.version
    );
    println!("   📦 标签: {:?}", skill.metadata.tags);
    println!("   🔧 依赖: {:?}", skill.metadata.dependencies);
    println!();

    // 2. 保存为 YAML 格式
    println!("2️⃣  保存为 YAML 格式");
    let yaml_path = PathBuf::from("example_skill.yaml");
    skill.save_to_file(&yaml_path).unwrap();
    println!("   ✅ 已保存到: {:?}", yaml_path);
    println!();

    // 3. 读取并显示 YAML 内容
    println!("3️⃣  YAML 文件内容");
    let yaml_content = std::fs::read_to_string(&yaml_path).unwrap();
    println!("   ┌─ {} ─────────────────", yaml_path.display());
    for line in yaml_content.lines().take(30) {
        println!("   │ {}", line);
    }
    if yaml_content.lines().count() > 30 {
        println!("   │ ... (省略部分内容)");
    }
    println!("   └────────────────────────────");
    println!();

    // 4. 从 YAML 文件加载
    println!("4️⃣  从 YAML 文件加载");
    let loaded_skill = SkillPackage::load_from_file(&yaml_path).unwrap();
    println!("   ✅ 成功加载技能: {}", loaded_skill.metadata.name);
    println!("   📝 描述: {}", loaded_skill.metadata.description);
    println!();

    // 5. 验证数据完整性
    println!("5️⃣  验证数据完整性");
    assert_eq!(skill.metadata.id, loaded_skill.metadata.id);
    assert_eq!(skill.metadata.name, loaded_skill.metadata.name);
    assert_eq!(skill.metadata.version, loaded_skill.metadata.version);
    assert_eq!(skill.metadata.author, loaded_skill.metadata.author);
    assert_eq!(
        skill.metadata.dependencies,
        loaded_skill.metadata.dependencies
    );
    assert_eq!(skill.metadata.tags, loaded_skill.metadata.tags);
    assert_eq!(skill.instructions, loaded_skill.instructions);
    assert_eq!(skill.scripts, loaded_skill.scripts);
    assert_eq!(skill.resources.folders, loaded_skill.resources.folders);
    assert_eq!(skill.resources.tools, loaded_skill.resources.tools);
    assert_eq!(skill.resources.tests, loaded_skill.resources.tests);
    println!("   ✅ 所有字段验证通过");
    println!();

    // 6. 创建简化版技能包
    println!("6️⃣  创建简化版技能包（最小化配置）");
    let minimal_skill = SkillPackage {
        metadata: SkillMetadata {
            id: "hello-world".to_string(),
            name: "Hello World".to_string(),
            description: "A minimal skill for demonstration".to_string(),
            version: "0.1.0".to_string(),
            author: None,
            dependencies: vec![],
            tags: vec![],
        },

        instructions: "Say hello to the world!".to_string(),

        scripts: vec![],
        resources: Default::default(),
    };

    let minimal_path = PathBuf::from("minimal_skill.yaml");
    minimal_skill.save_to_file(&minimal_path).unwrap();
    println!("   ✅ 已保存简化版技能到: {:?}", minimal_path);

    let minimal_yaml = std::fs::read_to_string(&minimal_path).unwrap();
    println!("   ┌─ {} ────", minimal_path.display());
    for line in minimal_yaml.lines() {
        println!("   │ {}", line);
    }
    println!("   └─────────────────");
    println!();

    // 7. YAML vs JSON 对比
    println!("7️⃣  YAML vs JSON 格式对比");
    let json_path = PathBuf::from("example_skill.json");
    skill.save_to_file(&json_path).unwrap();

    let yaml_size = yaml_content.len();
    let json_content = std::fs::read_to_string(&json_path).unwrap();
    let json_size = json_content.len();

    println!("   📄 YAML 大小: {} bytes", yaml_size);
    println!("   📄 JSON 大小: {} bytes", json_size);
    println!(
        "   📊 大小差异: {:+} bytes ({:.1}%)",
        yaml_size as i64 - json_size as i64,
        ((yaml_size as f64 - json_size as f64) / json_size as f64) * 100.0
    );
    println!();

    // 8. YAML 特性展示
    println!("8️⃣  YAML 格式的优势");
    println!("   ✨ 更易读 - 类似自然语言的语法");
    println!("   ✨ 更简洁 - 减少重复的引号和括号");
    println!("   ✨ 注释支持 - 可以在配置中添加说明");
    println!("   ✨ 多行文本 - 保留格式的长文本更清晰");
    println!();

    // 9. 清理临时文件
    println!("9️⃣  清理临时文件");
    std::fs::remove_file(&yaml_path).unwrap();
    std::fs::remove_file(&json_path).unwrap();
    std::fs::remove_file(&minimal_path).unwrap();
    println!("   ✅ 已清理所有临时文件");
    println!();

    println!("✨ 示例运行完成!");
    println!("\n💡 YAML 支持的关键特性:");
    println!("   1. 完整的序列化/反序列化支持");
    println!("   2. 保留所有元数据和配置信息");
    println!("   3. 支持 SkillPackage 的所有字段");
    println!("   4. 可选 feature flag (--features yaml)");
    println!("   5. 类型安全的配置加载");
    println!("   6. 与 JSON 格式完全兼容的数据结构");
    println!("   7. 使用安全的 serde_norway crate");
    println!("\n📚 使用方法:");
    println!("   - 添加依赖: cargo add --features yaml");
    println!("   - 保存配置: skill.save_to_yaml(path)");
    println!("   - 加载配置: SkillPackage::load_from_yaml(path)");
}
