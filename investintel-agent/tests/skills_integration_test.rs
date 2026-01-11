//! Skills系统集成测试
//!
//! 测试InvestmentAssistant与Claude Skills系统的集成

use investintel_agent::agents::InvestmentAssistant;

#[tokio::test]
async fn test_load_skills_from_project_dir() {
    // 创建投资助手并加载Skills
    let assistant = InvestmentAssistant::with_skills().await.unwrap();

    // 检查Skills是否加载成功
    let skills = assistant.list_skills();
    println!("已加载的Skills: {:?}", skills);

    // 应该至少有5个Skills
    assert!(skills.len() >= 5, "应该至少加载5个Skills");

    // 验证特定Skills存在
    let skill_names: Vec<String> = skills.iter().map(|s| s.to_lowercase()).collect();
    assert!(skill_names.iter().any(|n| n.contains("graham")), "应该有Graham Skill");
    assert!(skill_names.iter().any(|n| n.contains("kelly")), "应该有Kelly Skill");
    assert!(skill_names.iter().any(|n| n.contains("munger")), "应该有Munger Skill");
    assert!(skill_names.iter().any(|n| n.contains("dividend")), "应该有Dividend Skill");
    assert!(skill_names.iter().any(|n| n.contains("mcp")), "应该有MCP Skill");
}

#[tokio::test]
async fn test_find_graham_skill() {
    let assistant = InvestmentAssistant::with_skills().await.unwrap();

    // 查找Graham Skill
    let graham_skill = assistant.find_skill("graham-value-investing");

    assert!(graham_skill.is_some(), "应该找到Graham Value Investing Skill");

    let skill = graham_skill.unwrap();
    println!("Graham Skill: {}", skill.metadata.name);
    println!("Description: {}", skill.metadata.description);
    println!("Version: {}", skill.metadata.version);

    assert_eq!(skill.metadata.name, "Graham Value Investing");
    assert!(skill.metadata.description.contains("Graham"));
    assert!(skill.metadata.tags.contains(&"value-investing".to_string()));
}

#[tokio::test]
async fn test_find_kelly_skill() {
    let assistant = InvestmentAssistant::with_skills().await.unwrap();

    // 查找Kelly Skill
    let kelly_skill = assistant.find_skill("kelly-position");

    assert!(kelly_skill.is_some(), "应该找到Kelly Position Skill");

    let skill = kelly_skill.unwrap();
    println!("Kelly Skill: {}", skill.metadata.name);
    println!("Description: {}", skill.metadata.description);

    assert_eq!(skill.metadata.name, "Kelly Position Sizing");
    assert!(skill.metadata.description.contains("Kelly"));
}

#[tokio::test]
async fn test_find_munger_skill() {
    let assistant = InvestmentAssistant::with_skills().await.unwrap();

    // 查找Munger Skill
    let munger_skill = assistant.find_skill("munger-mental-models");

    assert!(munger_skill.is_some(), "应该找到Munger Mental Models Skill");

    let skill = munger_skill.unwrap();
    println!("Munger Skill: {}", skill.metadata.name);

    assert_eq!(skill.metadata.name, "Munger Mental Models");
    assert!(skill.metadata.tags.contains(&"lollapalooza".to_string()));
}

#[tokio::test]
async fn test_find_dividend_skill() {
    let assistant = InvestmentAssistant::with_skills().await.unwrap();

    // 查找Dividend Skill
    let dividend_skill = assistant.find_skill("dividend-investing");

    assert!(dividend_skill.is_some(), "应该找到Dividend Investing Skill");

    let skill = dividend_skill.unwrap();
    println!("Dividend Skill: {}", skill.metadata.name);

    assert_eq!(skill.metadata.name, "Dividend Investing");
    assert!(skill.metadata.tags.contains(&"dividend".to_string()));
}

#[tokio::test]
async fn test_find_mcp_gateway_skill() {
    let assistant = InvestmentAssistant::with_skills().await.unwrap();

    // 查找MCP Gateway Skill
    let mcp_skill = assistant.find_skill("mcp-data-gateway");

    assert!(mcp_skill.is_some(), "应该找到MCP Data Gateway Skill");

    let skill = mcp_skill.unwrap();
    println!("MCP Gateway Skill: {}", skill.metadata.name);

    assert_eq!(skill.metadata.name, "MCP Data Gateway");
    assert!(skill.metadata.tags.contains(&"market-data".to_string()));
}

#[tokio::test]
async fn test_skills_metadata() {
    let assistant = InvestmentAssistant::with_skills().await.unwrap();

    // 检查所有Skills的元数据
    for skill_name in assistant.list_skills() {
        if let Some(skill) = assistant.find_skill(&skill_name) {
            println!("\n========================================");
            println!("Skill: {}", skill.metadata.name);
            println!("========================================");
            println!("ID: {}", skill.metadata.id);
            println!("Version: {}", skill.metadata.version);
            println!("Author: {:?}", skill.metadata.author);
            println!("Description: {}", skill.metadata.description);
            println!("Tags: {:?}", skill.metadata.tags);
            println!("Allowed Tools: {:?}", skill.metadata.allowed_tools);

            // 验证必需字段
            assert!(!skill.metadata.name.is_empty(), "Skill名称不能为空");
            assert!(!skill.metadata.description.is_empty(), "Skill描述不能为空");
            assert!(!skill.metadata.version.is_empty(), "Skill版本不能为空");
        }
    }
}

#[tokio::test]
async fn test_skill_instructions_content() {
    let assistant = InvestmentAssistant::with_skills().await.unwrap();

    // 检查Graham Skill的instructions内容
    if let Some(skill) = assistant.find_skill("graham-value-investing") {
        let instructions = &skill.instructions;

        // 验证instructions包含关键内容
        assert!(instructions.contains("Graham"), "应该包含Graham相关内容");
        assert!(instructions.contains("内在价值"), "应该包含内在价值公式");
        assert!(instructions.contains("安全边际"), "应该包含安全边际概念");

        println!("\nGraham Skill Instructions 长度: {} 字符", instructions.len());
    }
}

#[tokio::test]
async fn test_skill_tags() {
    let assistant = InvestmentAssistant::with_skills().await.unwrap();

    // 检查Skills的标签分类
    let mut value_investing_count = 0;
    let mut risk_management_count = 0;
    let mut data_count = 0;

    for skill_name in assistant.list_skills() {
        if let Some(skill) = assistant.find_skill(&skill_name) {
            if skill.metadata.tags.contains(&"value-investing".to_string()) {
                value_investing_count += 1;
            }
            if skill.metadata.tags.contains(&"risk-management".to_string()) ||
               skill.metadata.tags.contains(&"position-sizing".to_string()) {
                risk_management_count += 1;
            }
            if skill.metadata.tags.contains(&"market-data".to_string()) {
                data_count += 1;
            }
        }
    }

    println!("价值投资相关Skills: {}", value_investing_count);
    println!("风险管理相关Skills: {}", risk_management_count);
    println!("数据相关Skills: {}", data_count);

    assert!(value_investing_count >= 2, "应该至少有2个价值投资相关Skills");
    assert!(risk_management_count >= 1, "应该至少有1个风险管理相关Skill");
    assert!(data_count >= 1, "应该至少有1个数据相关Skill");
}

#[test]
fn test_skill_files_exist() {
    use std::path::Path;

    // 检查Skills目录是否存在
    let skills_dir = Path::new(".claude/skills");
    assert!(skills_dir.exists(), ".claude/skills目录应该存在");

    // 检查各个Skill目录
    let expected_skills = vec![
        "graham-value-investing",
        "kelly-position",
        "munger-mental-models",
        "dividend-investing",
        "mcp-data-gateway",
    ];

    for skill_name in expected_skills {
        let skill_dir = skills_dir.join(skill_name);
        assert!(skill_dir.exists(), "{}目录应该存在", skill_name);

        let skill_md = skill_dir.join("SKILL.md");
        assert!(skill_md.exists(), "{}/SKILL.md应该存在", skill_name);

        println!("✅ {} Skill文件存在", skill_name);
    }
}
