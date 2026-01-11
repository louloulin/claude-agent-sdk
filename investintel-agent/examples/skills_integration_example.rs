//! Skills集成系统使用示例
//!
//! 展示如何使用SkillsIntegrationSystem进行智能投资分析

use investintel_agent::skills_integration::{
    SkillsIntegrationSystem, SkillsIntegrationConfig, SmartAnalysisType,
};
use investintel_agent::orchestration::AnalysisType;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🎯 Skills集成系统示例\n");

    // ========================================
    // 示例1: 基础使用 - 自动Skills发现和加载
    // ========================================
    println!("【示例1】创建Skills集成系统");
    println!("{}\n", "=".repeat(50));

    let system = SkillsIntegrationSystem::new().await?;

    // 列出已加载的Skills
    let skills = system.list_skills().await;
    println!("✅ 已加载 {} 个Skills:", skills.len());
    for (i, skill) in skills.iter().enumerate() {
        println!("   {}. {}", i + 1, skill);
    }
    println!();

    // ========================================
    // 示例2: 智能意图解析
    // ========================================
    println!("【示例2】智能意图解析");
    println!("{}\n", "=".repeat(50));

    let test_requests = vec![
        "使用Graham方法分析AAPL的内在价值",
        "AAPL应该分配多少仓位?",
        "对MSFT进行深度分析",
        "使用Munger思维模型评估GOOGL",
    ];

    for request in test_requests {
        let intent = system.parse_intent(request);
        println!("📝 用户请求: {}", request);
        println!("   → 分析类型: {:?}", intent.analysis_type);
        if let Some(skill) = intent.skill_name {
            println!("   → Skill: {}", skill);
        }
        if let Some(analysis_type) = intent.orchestration_type {
            println!("   → 分析类型: {:?}", analysis_type);
        }
        println!();
    }

    // ========================================
    // 示例3: 获取Skill详细信息
    // ========================================
    println!("【示例3】Skill详细信息");
    println!("{}\n", "=".repeat(50));

    let skill_name = "graham-value-investing";
    let skill_info = system.get_skill_info(skill_name).await?;

    println!("📚 Skill: {}", skill_info.name);
    println!("   描述: {}", skill_info.description);
    println!("   访问次数: {}", skill_info.access_count);
    println!();

    // ========================================
    // 示例4: 自定义配置
    // ========================================
    println!("【示例4】自定义配置");
    println!("{}\n", "=".repeat(50));

    let custom_config = SkillsIntegrationConfig {
        skills_dir: std::path::PathBuf::from(".claude/skills"),
        enable_cache: true,
        cache_ttl: 7200,  // 2小时
        enable_progressive_disclosure: true,
        max_concurrent_skills: 10,
    };

    let custom_system = SkillsIntegrationSystem::with_config(custom_config).await?;
    println!("✅ 自定义系统创建成功");
    println!();

    // ========================================
    // 示例5: 智能分析路由
    // ========================================
    println!("【示例5】智能分析路由演示");
    println!("{}\n", "=".repeat(50));

    let analysis_examples = vec![
        ("分析AAPL", "QuickValue"),
        ("Graham估值AAPL", "graham-value-investing"),
        ("深度分析MSFT", "Deep"),
        ("计算GOOGL仓位", "kelly-position"),
    ];

    for (request, expected) in analysis_examples {
        let intent = system.parse_intent(request);
        println!("🔍 请求: {}", request);
        println!("   预期: {}", expected);
        println!("   实际: {:?}", intent.analysis_type);
        println!();
    }

    // ========================================
    // 示例6: 缓存管理
    // ========================================
    println!("【示例6】缓存管理");
    println!("{}\n", "=".repeat(50));

    println!("🧹 清理过期缓存...");
    system.cleanup_cache().await;
    println!("✅ 缓存清理完成");
    println!();

    // ========================================
    // 示例7: 完整工作流
    // ========================================
    println!("【示例7】完整分析工作流");
    println!("{}\n", "=".repeat(50));

    // 用户请求
    let user_request = "使用Graham方法分析AAPL";

    // 1. 解析意图
    let intent = system.parse_intent(user_request);
    println!("1️⃣ 解析意图: {:?}", intent.analysis_type);

    // 2. 获取Skill信息
    if let Some(skill_name) = intent.skill_name {
        let skill_info = system.get_skill_info(&skill_name).await?;
        println!("2️⃣ Skill信息:");
        println!("   名称: {}", skill_info.name);
        println!("   描述: {}", skill_info.description);
    }

    // 3. 执行分析(实际场景会调用smart_analyze)
    println!("3️⃣ 执行分析...");
    println!("   ✅ 分析完成");
    println!();

    // ========================================
    // 统计信息
    // ========================================
    println!("📊 系统统计");
    println!("{}\n", "=".repeat(50));
    println!("总Skills数: {}", skills.len());
    println!("缓存TTL: 3600秒");
    println!("最大并发Skills数: 5");
    println!("Progressive Disclosure: 启用");
    println!();

    println!("✅ 所有示例运行完成!");

    Ok(())
}
