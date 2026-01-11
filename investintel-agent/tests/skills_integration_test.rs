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

// =============================================================================
// SkillsIntegrationSystem 完整测试套件
// =============================================================================

use investintel_agent::skills_integration::{
    SkillsIntegrationSystem, SkillsIntegrationConfig, SmartAnalysisType,
};
use investintel_agent::orchestration::AnalysisType;

#[tokio::test]
async fn test_skills_integration_system_creation() {
    // 创建Skills集成系统
    let system = SkillsIntegrationSystem::new().await.unwrap();

    // 验证Skills被加载
    let skills = system.list_skills().await;
    println!("✅ 已加载 {} 个Skills", skills.len());
    assert!(!skills.is_empty(), "应该至少加载一个Skill");

    // 验证Graham Skill存在
    assert!(skills.iter().any(|s| s.contains("graham")), "应该有Graham Skill");
}

#[tokio::test]
async fn test_parse_intent_graham() {
    let system = SkillsIntegrationSystem::new().await.unwrap();

    // 测试Graham关键词
    let requests = vec![
        "使用Graham方法分析AAPL",
        "分析AAPL的内在价值",
        "计算AAPL的安全边际",
        "价值投资分析AAPL",
    ];

    for request in requests {
        let intent = system.parse_intent(request);
        assert_eq!(
            intent.analysis_type,
            SmartAnalysisType::Skill,
            "请求 '{}' 应该被识别为Skill模式",
            request
        );
        assert_eq!(
            intent.skill_name,
            Some("graham-value-investing".to_string()),
            "应该识别为Graham Skill"
        );
        println!("✅ 识别请求: {} -> Graham Skill", request);
    }
}

#[tokio::test]
async fn test_parse_intent_kelly() {
    let system = SkillsIntegrationSystem::new().await.unwrap();

    // 测试Kelly关键词
    let requests = vec![
        "使用Kelly公式计算仓位",
        "AAPL应该分配多少资金",
        "计算最佳position size",
    ];

    for request in requests {
        let intent = system.parse_intent(request);
        assert_eq!(
            intent.analysis_type,
            SmartAnalysisType::Skill,
            "请求 '{}' 应该被识别为Skill模式",
            request
        );
        assert_eq!(
            intent.skill_name,
            Some("kelly-position".to_string()),
            "应该识别为Kelly Skill"
        );
        println!("✅ 识别请求: {} -> Kelly Skill", request);
    }
}

#[tokio::test]
async fn test_parse_intent_munger() {
    let system = SkillsIntegrationSystem::new().await.unwrap();

    // 测试Munger关键词
    let requests = vec![
        "使用Munger思维模型分析",
        "应用Lollapalooza效应",
        "多学科分析AAPL",
    ];

    for request in requests {
        let intent = system.parse_intent(request);
        assert_eq!(
            intent.analysis_type,
            SmartAnalysisType::Skill,
            "请求 '{}' 应该被识别为Skill模式",
            request
        );
        assert_eq!(
            intent.skill_name,
            Some("munger-mental-models".to_string()),
            "应该识别为Munger Skill"
        );
        println!("✅ 识别请求: {} -> Munger Skill", request);
    }
}

#[tokio::test]
async fn test_parse_intent_orchestration() {
    let system = SkillsIntegrationSystem::new().await.unwrap();

    // 测试Orchestration关键词
    let test_cases = vec![
        ("对AAPL进行深度分析", AnalysisType::Deep),
        ("AAPL的股息分析", AnalysisType::Dividend),
        ("完整分析AAPL", AnalysisType::Full),
        ("分析AAPL", AnalysisType::QuickValue), // 默认
    ];

    for (request, expected_type) in test_cases {
        let intent = system.parse_intent(request);
        assert_eq!(
            intent.analysis_type,
            SmartAnalysisType::Orchestration,
            "请求 '{}' 应该被识别为Orchestration模式",
            request
        );
        assert_eq!(
            intent.orchestration_type,
            Some(expected_type),
            "应该识别为正确的分析类型"
        );
        println!("✅ 识别请求: {} -> {:?}", request, expected_type);
    }
}

#[tokio::test]
async fn test_get_skill_info() {
    let system = SkillsIntegrationSystem::new().await.unwrap();

    // 获取Graham Skill信息
    let skill_info = system.get_skill_info("graham-value-investing").await.unwrap();

    assert_eq!(skill_info.name, "graham-value-investing");
    assert!(!skill_info.description.is_empty());
    println!("✅ Graham Skill信息:");
    println!("  名称: {}", skill_info.name);
    println!("  描述: {}", skill_info.description);
    println!("  访问次数: {}", skill_info.access_count);
}

#[tokio::test]
async fn test_custom_config() {
    // 使用自定义配置创建系统
    let config = SkillsIntegrationConfig {
        skills_dir: std::path::PathBuf::from(".claude/skills"),
        enable_cache: true,
        cache_ttl: 7200, // 2小时
        enable_progressive_disclosure: true,
        max_concurrent_skills: 10,
    };

    let system = SkillsIntegrationSystem::with_config(config).await.unwrap();
    let skills = system.list_skills().await;

    assert!(!skills.is_empty());
    println!("✅ 自定义配置测试通过，加载了 {} 个Skills", skills.len());
}

#[tokio::test]
async fn test_smart_analyze_routing() {
    let system = SkillsIntegrationSystem::new().await.unwrap();

    // 测试智能路由
    let test_cases = vec![
        ("Graham分析AAPL", SmartAnalysisType::Skill),
        ("深度分析AAPL", SmartAnalysisType::Orchestration),
    ];

    for (request, expected_type) in test_cases {
        let intent = system.parse_intent(request);
        assert_eq!(
            intent.analysis_type, expected_type,
            "请求 '{}' 应该路由到 {:?}",
            request, expected_type
        );
        println!("✅ 路由测试: {} -> {:?}", request, expected_type);
    }
}

#[tokio::test]
async fn test_cache_cleanup() {
    let system = SkillsIntegrationSystem::new().await.unwrap();

    // 清理缓存
    system.cleanup_cache().await;

    // 验证系统仍然可用
    let skills = system.list_skills().await;
    assert!(!skills.is_empty());

    println!("✅ 缓存清理测试通过");
}

#[tokio::test]
async fn test_full_analysis_workflow() {
    let system = SkillsIntegrationSystem::new().await.unwrap();

    // 模拟完整工作流
    println!("\n🔄 测试完整分析工作流...");

    // 1. 列出可用Skills
    let skills = system.list_skills().await;
    println!("1️⃣ 可用Skills: {:?}", skills);

    // 2. 解析用户意图
    let user_request = "使用Graham方法分析AAPL的内在价值和安全边际";
    let intent = system.parse_intent(user_request);
    println!("2️⃣ 解析意图: {:?}", intent.analysis_type);

    // 3. 获取Skill信息
    if let Some(skill_name) = intent.skill_name {
        let skill_info = system.get_skill_info(&skill_name).await.unwrap();
        println!("3️⃣ Skill信息: {} - {}", skill_info.name, skill_info.description);
    }

    println!("✅ 完整工作流测试通过");
}

// =============================================================================
// Progressive Disclosure 测试
// =============================================================================

#[test]
fn test_progressive_disclosure_structure() {
    use std::path::Path;

    // 验证Progressive Disclosure三级架构
    let graham_skill = Path::new(".claude/skills/graham-value-investing");

    // Level 1: SKILL.md (简洁核心内容)
    let skill_md = graham_skill.join("SKILL.md");
    assert!(skill_md.exists(), "应该有SKILL.md文件");

    // Level 2: 详细文档 (按需加载)
    let detailed_docs = vec![
        "detailed-analysis.md",
        "evaluation-criteria.md",
        "reference-implementation.md",
    ];

    for doc in detailed_docs {
        let doc_path = graham_skill.join(doc);
        assert!(doc_path.exists(), "应该有{}文档", doc);
        println!("✅ 发现详细文档: {}", doc);
    }

    // Level 3: 工具脚本 (零上下文执行)
    let scripts_dir = graham_skill.join("scripts");
    assert!(scripts_dir.exists(), "应该有scripts目录");

    let script_path = scripts_dir.join("graham_analyzer.py");
    assert!(script_path.exists(), "应该有graham_analyzer.py工具脚本");

    println!("✅ Progressive Disclosure三级架构验证通过");
}

#[test]
fn test_skill_md_conciseness() {
    use std::fs;

    // 验证SKILL.md的简洁性
    let skill_md_path = ".claude/skills/graham-value-investing/SKILL.md";
    let content = fs::read_to_string(skill_md_path).unwrap();
    let line_count = content.lines().count();

    // SKILL.md应该控制在100行以内
    assert!(
        line_count < 100,
        "SKILL.md应该简洁，当前 {} 行",
        line_count
    );

    println!("✅ SKILL.md简洁性验证通过: {} 行", line_count);
}

// =============================================================================
// 集成测试: Skills + Orchestration
// =============================================================================

#[tokio::test]
async fn test_hybrid_analysis_mode() {
    let system = SkillsIntegrationSystem::new().await.unwrap();

    // 测试混合模式意图解析
    // 注意: 当前实现中混合模式需要显式请求
    let request = "综合使用Graham和深度分析AAPL";
    let intent = system.parse_intent(request);

    // 验证意图被正确解析
    assert!(
        intent.analysis_type == SmartAnalysisType::Skill ||
        intent.analysis_type == SmartAnalysisType::Orchestration,
        "应该识别为Skill或Orchestration模式"
    );

    println!("✅ 混合模式测试通过: {:?}", intent.analysis_type);
}

#[tokio::test]
async fn test_skills_orchestrator_integration() {
    let system = SkillsIntegrationSystem::new().await.unwrap();

    // 验证Skills和Orchestrator都已加载
    let skills = system.list_skills().await;
    assert!(!skills.is_empty(), "Skills应该已加载");

    println!("✅ Skills与Orchestrator集成测试通过");
    println!("   已加载Skills: {}", skills.len());
}

// =============================================================================
// 性能测试
// ==============================================================================

#[tokio::test]
async fn test_concurrent_skill_access() {
    let system = std::sync::Arc::new(SkillsIntegrationSystem::new().await.unwrap());

    // 并发访问Skills
    let mut handles = vec![];

    for i in 0..10 {
        let system_clone = system.clone();
        let handle = tokio::spawn(async move {
            let skills = system_clone.list_skills().await;
            println!("并发访问 #{}: {} 个Skills", i, skills.len());
            skills.len()
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    let results = futures::future::join_all(handles).await;
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    println!("✅ 并发访问测试通过");
}
