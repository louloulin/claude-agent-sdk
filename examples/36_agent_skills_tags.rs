//! Agent Skills - 标签系统示例
//!
//! 这个示例展示了如何使用标签系统进行技能过滤、查询和管理
//!
//! 运行: cargo run --example 36_agent_skills_tags

use claude_agent_sdk_rs::skills::{TagFilter, TagQueryBuilder, TagUtils};
use std::collections::HashSet;

/// 示例技能结构
struct Skill {
    id: String,
    name: String,
    tags: Vec<String>,
}

impl Skill {
    fn new(id: impl Into<String>, name: impl Into<String>, tags: Vec<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            tags,
        }
    }
}

fn main() {
    println!("🏷️  Agent Skills - 标签系统示例\n");

    // 1. 标签规范化
    println!("1️⃣  标签规范化");
    let raw_tags = vec![
        "  Rust SDK  ",
        "Data-Processing",
        "web_API",
        "Machine Learning",
    ];
    println!("   原始标签: {:?}", raw_tags);
    let normalized: Vec<String> = raw_tags
        .iter()
        .map(|tag| TagUtils::normalize_tag(tag))
        .collect();
    println!("   规范化后: {:?}", normalized);
    println!();

    // 2. 标签验证
    println!("2️⃣  标签验证");
    let test_tags = vec!["rust", "rust-sdk", "rust_sdk", "rust@sdk", "", "a"];
    println!("   测试标签: {:?}", test_tags);
    for tag in test_tags {
        let valid = TagUtils::is_valid_tag(tag);
        let status = if valid { "✅" } else { "❌" };
        println!("   {} '{}' is valid: {}", status, tag, valid);
    }
    println!();

    // 3. 解析标签字符串
    println!("3️⃣  解析标签字符串");
    let tag_string = "rust, python, data-processing, web-api";
    let parsed = TagUtils::parse_tags(tag_string);
    println!("   输入: '{}'", tag_string);
    println!("   解析结果: {:?}", parsed);
    println!();

    // 4. 创建示例技能集合
    println!("4️⃣  创建技能集合");
    let skills = vec![
        Skill::new(
            "skill1",
            "Data Processor",
            vec!["rust".to_string(), "data".to_string(), "etl".to_string()],
        ),
        Skill::new(
            "skill2",
            "Web Scraper",
            vec![
                "python".to_string(),
                "web".to_string(),
                "scraper".to_string(),
            ],
        ),
        Skill::new(
            "skill3",
            "API Client",
            vec!["rust".to_string(), "web".to_string(), "api".to_string()],
        ),
        Skill::new(
            "skill4",
            "ML Model Trainer",
            vec![
                "python".to_string(),
                "ml".to_string(),
                "training".to_string(),
            ],
        ),
        Skill::new(
            "skill5",
            "Data Validator",
            vec![
                "rust".to_string(),
                "data".to_string(),
                "validation".to_string(),
            ],
        ),
    ];

    for skill in &skills {
        println!("   📦 {} ({:?})", skill.name, skill.tags);
    }
    println!();

    // 5. 基础标签过滤
    println!("5️⃣  基础标签过滤");
    let filter = TagFilter::new().has("rust");
    println!("   过滤条件: 包含标签 'rust'");
    for skill in &skills {
        let tags: HashSet<String> = skill.tags.iter().cloned().collect();
        let matches = filter.matches(&tags);
        let status = if matches { "✅" } else { "❌" };
        println!("   {} {}", status, skill.name);
    }
    println!();

    // 6. 复杂标签过滤
    println!("6️⃣  复杂标签过滤 (AND 逻辑)");
    let complex_filter = TagFilter::new().has("rust").has("data").not_has("web");
    println!("   过滤条件: 包含 'rust' AND 'data' AND NOT 'web'");
    for skill in &skills {
        let tags: HashSet<String> = skill.tags.iter().cloned().collect();
        let matches = complex_filter.matches(&tags);
        let status = if matches { "✅" } else { "❌" };
        println!("   {} {}", status, skill.name);
    }
    println!();

    // 7. AnyOf 和 AllOf 过滤
    println!("7️⃣  AnyOf 和 AllOf 过滤");
    let any_filter = TagFilter::new().any_of(vec!["rust".to_string(), "python".to_string()]);
    println!("   过滤条件: 包含 'rust' OR 'python'");
    let mut count = 0;
    for skill in &skills {
        let tags: HashSet<String> = skill.tags.iter().cloned().collect();
        if any_filter.matches(&tags) {
            println!("      ✅ {}", skill.name);
            count += 1;
        }
    }
    println!("   匹配数: {}", count);
    println!();

    // 8. NoneOf 过滤
    println!("8️⃣  NoneOf 过滤");
    let none_filter = TagFilter::new().none_of(vec!["web".to_string(), "ml".to_string()]);
    println!("   过滤条件: 不包含 'web' AND 不包含 'ml'");
    for skill in &skills {
        let tags: HashSet<String> = skill.tags.iter().cloned().collect();
        let matches = none_filter.matches(&tags);
        let status = if matches { "✅" } else { "❌" };
        println!("   {} {}", status, skill.name);
    }
    println!();

    // 9. 查询构建器 - 查询
    println!("9️⃣  查询构建器 - 查询技能");
    let builder = TagQueryBuilder::new();
    let rust_skills = builder.with_any_tag(&skills, &["rust".to_string()], |skill| &skill.tags);
    println!("   查询: 包含标签 'rust' 的技能");
    for skill in rust_skills {
        println!("      📦 {}", skill.name);
    }
    println!();

    // 10. 查询构建器 - 统计
    println!("🔟 查询构建器 - 标签统计");
    let stats = builder.tag_statistics(&skills, |skill| &skill.tags);
    println!("   标签使用统计:");
    let mut sorted_stats: Vec<_> = stats.iter().collect();
    sorted_stats.sort_by(|a, b| b.1.cmp(a.1));
    for (tag, count) in sorted_stats.iter().take(5) {
        println!("      🏷️  '{}': {} 次", tag, count);
    }
    println!();

    // 11. 查询构建器 - 热门标签
    println!("1️⃣1️⃣  查询构建器 - 热门标签 TOP 3");
    let popular = builder.popular_tags(&skills, |skill| &skill.tags, 3);
    for (i, (tag, count)) in popular.iter().enumerate() {
        println!("      🏆 #{}: '{}' ({} 次)", i + 1, tag, count);
    }
    println!();

    // 12. 查询构建器 - 多标签查询
    println!("1️⃣2️⃣  查询构建器 - 多标签查询 (AND)");
    let multi_tag_skills = builder.with_all_tags(
        &skills,
        &["rust".to_string(), "data".to_string()],
        |skill| &skill.tags,
    );
    println!("   查询: 同时包含 'rust' AND 'data' 的技能");
    for skill in multi_tag_skills {
        println!("      📦 {} ({:?})", skill.name, skill.tags);
    }
    println!();

    // 13. 标签工具 - 合并标签
    println!("1️⃣3️⃣  标签工具 - 合并标签");
    let tags1 = vec!["rust".to_string(), "sdk".to_string()];
    let tags2 = vec!["rust".to_string(), "agent".to_string(), "sdk".to_string()];
    let merged = TagUtils::merge_tags(&tags1, &tags2);
    println!("   标签组1: {:?}", tags1);
    println!("   标签组2: {:?}", tags2);
    println!("   合并后: {:?}", merged);
    println!();

    // 14. 标签工具 - 公共标签
    println!("1️⃣4️⃣  标签工具 - 公共标签");
    let tags3 = vec!["rust".to_string(), "data".to_string(), "etl".to_string()];
    let tags4 = vec!["rust".to_string(), "data".to_string(), "web".to_string()];
    let common = TagUtils::common_tags(&tags3, &tags4);
    println!("   标签组1: {:?}", tags3);
    println!("   标签组2: {:?}", tags4);
    println!("   公共标签: {:?}", common);
    println!();

    // 15. 标签工具 - 相似度计算
    println!("1️⃣5️⃣  标签工具 - 标签相似度 (Jaccard Index)");
    let comparisons = vec![
        (
            vec!["rust".to_string(), "sdk".to_string()],
            vec!["rust".to_string(), "sdk".to_string()],
        ),
        (
            vec!["rust".to_string(), "sdk".to_string()],
            vec!["rust".to_string(), "agent".to_string()],
        ),
        (
            vec!["rust".to_string(), "data".to_string()],
            vec!["python".to_string(), "ml".to_string()],
        ),
        (vec!["rust".to_string()], vec![]),
    ];

    for (tags_a, tags_b) in comparisons {
        let similarity = TagUtils::tag_similarity(&tags_a, &tags_b);
        println!("   {:?} vs {:?}", tags_a, tags_b);
        println!("      相似度: {:.2}%", similarity * 100.0);
    }
    println!();

    // 16. 实际应用场景 - 技能发现
    println!("1️⃣6️⃣  实际应用场景 - 技能发现");
    println!("   场景: 用户需要 Rust 数据处理技能");

    let user_requirements = vec!["rust".to_string(), "data".to_string()];
    println!("   用户需求标签: {:?}", user_requirements);

    let recommended = builder.with_all_tags(&skills, &user_requirements, |skill| &skill.tags);
    println!("   推荐技能:");
    if recommended.is_empty() {
        println!("      ❌ 没有找到匹配的技能");
    } else {
        for skill in recommended {
            println!("      ✅ {} - {:?}", skill.name, skill.tags);
        }
    }
    println!();

    // 17. 实际应用场景 - 技能推荐
    println!("1️⃣7️⃣  实际应用场景 - 技能推荐 (基于相似度)");
    let user_profile = vec!["rust".to_string(), "web".to_string()];
    println!("   用户兴趣标签: {:?}", user_profile);
    println!("   推荐相关技能:");

    let mut recommendations: Vec<_> = skills
        .iter()
        .map(|skill| {
            let similarity = TagUtils::tag_similarity(&user_profile, &skill.tags);
            (skill, similarity)
        })
        .filter(|(_, sim)| *sim > 0.0)
        .collect();

    recommendations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (skill, similarity) in recommendations.iter().take(3) {
        println!(
            "      📦 {} - 相似度: {:.2}%",
            skill.name,
            similarity * 100.0
        );
    }
    println!();

    println!("✨ 示例运行完成!");
    println!("\n💡 标签系统的关键优势:");
    println!("   1. 灵活的标签过滤 (has, not_has, any_of, all_of, none_of)");
    println!("   2. 强大的查询构建器 (支持复杂查询条件)");
    println!("   3. 标签规范化 (统一格式，提高匹配率)");
    println!("   4. 标签验证 (确保数据质量)");
    println!("   5. 标签统计和分析 (热门标签、使用频率)");
    println!("   6. 标签相似度计算 (Jaccard Index)");
    println!("   7. 实用的标签工具 (合并、求交集、解析)");
    println!("   8. 高性能查询 (基于 HashSet 的 O(1) 查找)");
}
