//! InvestIntel AI - 智能投资助手
//!
//! 基于Claude Agent SDK的完整智能投资分析平台
//!
//! ## 功能特性
//!
//! - ✅ Agent Skills系统 - 模块化投资技能
//! - ✅ Multi-Agent Orchestration - 多Agent协同分析
//! - ✅ MCP Tools - 完整的投资分析工具集
//! - ✅ libSQL存储 - 200ns查询延迟的高性能存储
//! - ✅ 全面的投资分析 - 技术、基本面、风险、情感

use anyhow::Result;
use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ContentBlock, Message, McpServers, PermissionMode,
    ToolResult, create_sdk_mcp_server, query, tool,
};

mod tools;
mod orchestration;

use tools::*;
use orchestration::run_comprehensive_analysis;

/// 创建完整的MCP工具服务器
fn create_investment_tools() -> Result<claude_agent_sdk_rs::SdkMcpServer> {
    let tools = create_sdk_mcp_server(
        "investment-tools",
        vec![
            tool! {
                name: "technical_analysis",
                description: "Technical analysis with indicators (RSI, MACD, MA, Support/Resistance)",
                handler: technical_analysis
            },
            tool! {
                name: "var_calculation",
                description: "Calculate Value at Risk (VaR) using parametric method",
                handler: var_calculation
            },
            tool! {
                name: "sentiment_analysis",
                description: "Analyze market sentiment from news, Twitter, Reddit",
                handler: sentiment_analysis
            },
            tool! {
                name: "save_portfolio",
                description: "Save portfolio to libSQL storage",
                handler: save_portfolio
            },
            tool! {
                name: "load_portfolio",
                description: "Load portfolio from storage",
                handler: load_portfolio
            },
            tool! {
                name: "stress_test",
                description: "Run stress test scenarios on portfolio",
                handler: stress_test
            },
            tool! {
                name: "correlation_analysis",
                description: "Calculate correlation matrix for assets",
                handler: correlation_analysis
            },
        ],
    )?;

    Ok(tools)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║           InvestIntel AI - 智能投资助手                    ║");
    println!("║           基于Claude Agent SDK构建                        ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // 创建MCP工具
    println!("🔧 初始化投资分析工具...");
    let tools = create_investment_tools()?;
    println!("✅ 已加载 {} 个工具\n", tools.tools.len());

    // 配置Claude Agent选项
    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .mcp_servers(McpServers::new().add_server(tools))
        .auto_discover_skills(true)
        .project_skills_dir(std::path::PathBuf::from(".claude/skills"))
        .build();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 投资分析演示");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 示例1: 使用Agent Skills进行技术分析
    println!("📌 示例1: 使用Agent Skills分析AAPL股票");
    println!("{}\n", "─".repeat(60));

    let query1 = "使用market-research skill分析AAPL股票的技术面，包括趋势、支撑位和阻力位";
    println!("🤖 Claude查询: {}\n", query1);

    let messages = query(query1, Some(options.clone())).await?;

    for msg in &messages {
        if let Message::Assistant(assistant_msg) = msg {
            for block in &assistant_msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("{}\n", text.text);
                }
            }
        }
    }

    println!("{}\n", "─".repeat(60));

    // 示例2: 使用MCP工具进行风险分析
    println!("📌 示例2: 使用MCP工具计算投资组合VaR");
    println!("{}\n", "─".repeat(60));

    let query2 = "使用var_calculation工具计算一个$100,000投资组合的VaR，波动率20%，95%置信度";
    println!("🤖 Claude查询: {}\n", query2);

    let messages = query(query2, Some(options.clone())).await?;

    for msg in &messages {
        if let Message::Assistant(assistant_msg) = msg {
            for block in &assistant_msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("{}\n", text.text);
                }
            }
        }
    }

    println!("{}\n", "─".repeat(60));

    // 示例3: Multi-Agent Orchestration
    println!("📌 示例3: Multi-Agent综合分析");
    println!("{}\n", "─".repeat(60));

    println!("🤖 运行多Agent协同分析...\n");

    match run_comprehensive_analysis("AAPL").await {
        Ok(output) => {
            println!("✅ 综合分析完成\n");
            println!("置信度: {:.1}%", output.confidence * 100.0);
            println!("分析结果:\n{}\n", output.content);

            if !output.metadata.is_empty() {
                println!("元数据:");
                for (key, value) in &output.metadata {
                    println!("  - {}: {}", key, value);
                }
            }
        }
        Err(e) => {
            println!("❌ 分析失败: {}", e);
        }
    }

    println!("{}\n", "─".repeat(60));

    // 示例4: 情感分析
    println!("📌 示例4: 市场情感分析");
    println!("{}\n", "─".repeat(60));

    let query4 = "使用sentiment_analysis工具分析AAPL的市场情感，包括新闻和社交媒体";
    println!("🤖 Claude查询: {}\n", query4);

    let messages = query(query4, Some(options.clone())).await?;

    for msg in &messages {
        if let Message::Assistant(assistant_msg) = msg {
            for block in &assistant_msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("{}\n", text.text);
                }
            }
        }
    }

    println!("{}\n", "─".repeat(60));

    // 示例5: 组合压力测试
    println!("📌 示例5: 投资组合压力测试");
    println!("{}\n", "─".repeat(60));

    let query5 = "使用stress_test工具对$100,000投资组合进行压力测试，包括2008金融危机和COVID-19场景";
    println!("🤖 Claude查询: {}\n", query5);

    let messages = query(query5, Some(options.clone())).await?;

    for msg in &messages {
        if let Message::Assistant(assistant_msg) = msg {
            for block in &assistant_msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("{}\n", text.text);
                }
            }
        }
    }

    println!("{}\n", "─".repeat(60));

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                    分析完成 ✅                            ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    println!("🎯 核心特性:");
    println!("  ✅ Agent Skills - 模块化技能系统");
    println!("  ✅ MCP Tools - 7个投资分析工具");
    println!("  ✅ Multi-Agent - 并行和顺序编排");
    println!("  ✅ libSQL存储 - 高性能数据持久化");
    println!("  ✅ 全面分析 - 技术、基本面、风险、情感\n");

    println!("📚 可用Skills:");
    println!("  • market-research - 市场研究和技术分析");
    println!("  • portfolio-management - 投资组合管理");
    println!("  • risk-analysis - 风险分析和VaR计算");
    println!("  • sentiment-analysis - 情感分析\n");

    println!("⚠️  免责声明: 本工具仅供参考，不构成投资建议。");

    Ok(())
}
