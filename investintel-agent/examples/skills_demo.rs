//! Plan6 Skills使用示例
//!
//! 展示如何使用Claude Skills系统进行投资分析

use investintel_agent::{
    InvestmentAssistant, SkillsExecutor,
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Plan6 - AI投资合伙公司 Skills集成演示                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // 创建投资助手和Skills执行器
    let assistant = InvestmentAssistant::new();
    let mut executor = SkillsExecutor::new(assistant);

    // 加载Skills
    println!("📚 加载Claude Skills...\n");
    let count = executor.load_skills_from_project().await?;
    println!("✅ 成功加载{}个Skills\n", count);

    // 列出所有可用Skills
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 可用Skills列表:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    for skill_desc in executor.list_skills() {
        println!("• {}", skill_desc);
    }

    // Skills统计信息
    let stats = executor.skills_stats();
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Skills统计:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("总数: {}", stats.total);
    println!("带参考文档: {}", stats.with_reference);
    println!("总指令长度: {} 字符\n", stats.total_instructions);

    // 示例1: 使用Graham Skill分析股票
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔍 示例1: 使用Graham Skill分析AAPL");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    match executor.execute_skill("Graham", "分析AAPL的价值").await {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("⚠️  分析失败: {}\n", e),
    }

    // 示例2: 使用Kelly Skill计算仓位
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 示例2: 使用Kelly Skill计算MSFT仓位");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    match executor.execute_skill("Kelly", "MSFT的Kelly仓位建议").await {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("⚠️  计算失败: {}\n", e),
    }

    // 示例3: 使用Munger Skill进行思维模型分析
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧠 示例3: 使用Munger Skill分析GOOGL");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    match executor.execute_skill("Munger", "GOOGL的思维模型分析").await {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("⚠️  分析失败: {}\n", e),
    }

    // 示例4: 使用Dividend Skill分析股息
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💰 示例4: 使用Dividend Skill分析KO股息");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    match executor.execute_skill("Dividend", "KO的股息投资价值").await {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("⚠️  分析失败: {}\n", e),
    }

    // 示例5: 执行策略回测
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⏱️  示例5: 执行Graham策略回测");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    match executor.execute_backtest(
        "graham",
        "AAPL",
        "2020-01-01",
        "2023-12-31"
    ).await {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("⚠️  回测失败: {}\n", e),
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Skills集成演示完成！");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}
