//! Agent编排系统演示
//!
//! 展示基于SDK最佳实践的Agent编排功能

use investintel_agent::orchestration::{
    InvestmentOrchestrator, AnalysisType, OrchestrationConfig,
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Plan7 - 增强Agent编排系统演示                             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // 创建编排器
    let orchestrator = InvestmentOrchestrator::new();

    // 示例1: 快速价值分析
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔍 示例1: 快速价值分析 - AAPL");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let config = OrchestrationConfig::default();

    match orchestrator.analyze("AAPL", AnalysisType::QuickValue, config.clone()).await {
        Ok(result) => {
            println!("{}\n", result.recommendation);
            println!("✅ 成功 | 置信度: {:.1}% | 耗时: {}ms\n",
                result.confidence * 100.0,
                result.execution_time_ms
            );
        },
        Err(e) => println!("⚠️  分析失败: {}\n", e),
    }

    // 示例2: 综合分析（并行执行）
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 示例2: 综合分析（并行） - MSFT");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    match orchestrator.analyze("MSFT", AnalysisType::Comprehensive, config.clone()).await {
        Ok(result) => {
            println!("{}\n", result.recommendation);
            println!("✅ 成功 | 置信度: {:.1}% | 耗时: {}ms",
                result.confidence * 100.0,
                result.execution_time_ms
            );
            println!("📈 参与Agents: {}\n", result.agent_results.len());
        },
        Err(e) => println!("⚠️  分析失败: {}\n", e),
    }

    // 示例3: 深度分析（并行+顺序）
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧠 示例3: 深度分析（含Munger思维模型） - GOOGL");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    match orchestrator.analyze("GOOGL", AnalysisType::Deep, config.clone()).await {
        Ok(result) => {
            println!("{}\n", result.recommendation);
            println!("✅ 成功 | 置信度: {:.1}% | 耗时: {}ms",
                result.confidence * 100.0,
                result.execution_time_ms
            );
            println!("🧠 思维模型分析已整合\n");
        },
        Err(e) => println!("⚠️  分析失败: {}\n", e),
    }

    // 示例4: 仓位分析
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 示例4: Kelly仓位分析 - TSLA");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    match orchestrator.analyze("TSLA", AnalysisType::Position, config.clone()).await {
        Ok(result) => {
            println!("{}\n", result.recommendation);
            println!("✅ 成功 | 置信度: {:.1}% | 耗时: {}ms\n",
                result.confidence * 100.0,
                result.execution_time_ms
            );
        },
        Err(e) => println!("⚠️  分析失败: {}\n", e),
    }

    // 示例5: 股息投资分析
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💰 示例5: 股息投资分析 - KO");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    match orchestrator.analyze("KO", AnalysisType::Dividend, config.clone()).await {
        Ok(result) => {
            println!("{}\n", result.recommendation);
            println!("✅ 成功 | 置信度: {:.1}% | 耗时: {}ms\n",
                result.confidence * 100.0,
                result.execution_time_ms
            );
        },
        Err(e) => println!("⚠️  分析失败: {}\n", e),
    }

    // 示例6: 完整分析（所有维度）
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎯 示例6: 完整投资分析（所有维度） - JNJ");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    match orchestrator.analyze("JNJ", AnalysisType::Full, config.clone()).await {
        Ok(result) => {
            println!("{}\n", result.recommendation);
            println!("✅ 成功 | 置信度: {:.1}% | 耗时: {}ms",
                result.confidence * 100.0,
                result.execution_time_ms
            );
            println!("🔍 分析维度: {}个Agents参与\n", result.agent_results.len());
        },
        Err(e) => println!("⚠️  分析失败: {}\n", e),
    }

    // 示例7: 自定义配置
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⚙️  示例7: 自定义配置分析");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let custom_config = OrchestrationConfig {
        enable_tracing: false,
        enable_logging: false,
        parallel_limit: 3,
        max_retries: 2,
        timeout_secs: 60,
    };

    let custom_orchestrator = orchestrator.with_config(custom_config);

    match custom_orchestrator.analyze("AAPL", AnalysisType::Comprehensive, custom_config).await {
        Ok(result) => {
            println!("{}\n", result.recommendation);
            println!("✅ 成功（自定义配置） | 耗时: {}ms\n",
                result.execution_time_ms
            );
        },
        Err(e) => println!("⚠️  分析失败: {}\n", e),
    }

    // 编排模式说明
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📚 编排模式说明:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("1️⃣  **QuickValue** - 顺序执行");
    println!("   价值投资Agent → 交易建议Agent");
    println!("   适用于: 快速决策\n");

    println!("2️⃣  **Comprehensive** - 并行执行");
    println!("   价值投资 + 交易建议 + 股息投资");
    println!("   适用于: 全面了解投资标的\n");

    println!("3️⃣  **Deep** - 混合模式");
    println!("   (并行: 价值+交易+股息) → Munger思维模型");
    println!("   适用于: 深度研究和长期投资决策\n");

    println!("4️⃣  **Position** - 并行执行");
    println!("   价值投资 + Kelly仓位");
    println!("   适用于: 仓位管理\n");

    println!("5️⃣  **Dividend** - 顺序执行");
    println!("   股息投资Agent → 价值投资Agent");
    println!("   适用于: 股息收入投资\n");

    println!("6️⃣  **Full** - 完整编排");
    println!("   (并行: 所有专业Agent) → 组合管理Agent");
    println!("   适用于: 最全面的投资分析\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Agent编排系统演示完成！");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}
