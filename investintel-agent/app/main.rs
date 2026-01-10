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
mod hierarchical_orchestration;
mod data;

use tools::*;
use orchestration::run_comprehensive_analysis;
use hierarchical_orchestration::{create_hierarchical_orchestrator, AdvisorCoordinator};
use data::yahoo;
use data::alpha_vantage;
use data::websocket;
use data::fusion;

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
            // Yahoo Finance Data Source Tools
            tool! {
                name: "yahoo_finance_quote",
                description: "Get real-time quote from Yahoo Finance (symbol: stock ticker)",
                handler: yahoo::yahoo_finance_quote
            },
            tool! {
                name: "yahoo_finance_historical",
                description: "Get historical OHLCV data from Yahoo Finance (symbol, interval, range)",
                handler: yahoo::yahoo_finance_historical
            },
            tool! {
                name: "yahoo_finance_search",
                description: "Search for stock symbols on Yahoo Finance (query: company name or ticker)",
                handler: yahoo::yahoo_finance_search
            },
            // Alpha Vantage Data Source Tools
            tool! {
                name: "alpha_vantage_quote",
                description: "Get real-time quote from Alpha Vantage (symbol: stock ticker, requires ALPHA_VANTAGE_API_KEY env var)",
                handler: alpha_vantage::alpha_vantage_quote
            },
            tool! {
                name: "alpha_vantage_technical",
                description: "Get technical indicators from Alpha Vantage (symbol, function: RSI/MACD/SMA/etc, interval, time_period)",
                handler: alpha_vantage::alpha_vantage_technical
            },
            tool! {
                name: "alpha_vantage_news_sentiment",
                description: "Get news sentiment from Alpha Vantage (tickers: comma-separated symbols, optional time_from/time_to)",
                handler: alpha_vantage::alpha_vantage_news_sentiment
            },
            tool! {
                name: "alpha_vantage_overview",
                description: "Get company overview/fundamentals from Alpha Vantage (symbol: stock ticker)",
                handler: alpha_vantage::alpha_vantage_overview
            },
            // WebSocket Real-time Data Tools
            tool! {
                name: "websocket_start_polygon",
                description: "Start WebSocket connection to Polygon.io for real-time market data (api_key: optional, uses POLYGON_API_KEY env var)",
                handler: websocket::websocket_start_polygon
            },
            tool! {
                name: "websocket_subscribe_ticks",
                description: "Subscribe to real-time tick data for a symbol via WebSocket (symbol: stock ticker or * for all)",
                handler: websocket::websocket_subscribe_ticks
            },
            tool! {
                name: "websocket_stats",
                description: "Get WebSocket connection statistics and subscriber information",
                handler: websocket::websocket_stats
            },
            // Data Fusion Engine Tools
            tool! {
                name: "fusion_initialize",
                description: "Initialize multi-source data fusion engine (alpha_vantage_key: optional)",
                handler: fusion::fusion_initialize
            },
            tool! {
                name: "fusion_get_quote",
                description: "Get quote using smart multi-source fusion with caching (symbol: stock ticker)",
                handler: fusion::fusion_get_quote
            },
            tool! {
                name: "fusion_stats",
                description: "Get fusion engine statistics (cache, sources, latency)",
                handler: fusion::fusion_stats
            },
            tool! {
                name: "fusion_clear_cache",
                description: "Clear fusion engine cache",
                handler: fusion::fusion_clear_cache
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

    // 示例5: 层次化Agent编排 (Hierarchical Orchestration)
    println!("📌 示例5: 层次化Agent编排 - Advisor协调所有Subagents");
    println!("{}\n", "─".repeat(60));

    println!("🤖 运行层次化多Agent系统...\n");

    let orchestrator = create_hierarchical_orchestrator();
    let input = claude_agent_sdk_rs::orchestration::OrchestratorInput::new("AAPL".to_string());

    match orchestrator.orchestrate(vec![], input).await {
        Ok(output) => {
            println!("✅ 层次化分析完成\n");
            println!("置信度: {:.1}%", output.result.confidence * 100.0);

            if let Ok(result_json) = serde_json::from_str::<serde_json::Value>(&output.result.content) {
                println!("综合评分: {:.1}/100", result_json["overall_score"].as_f64().unwrap_or(0.0));
                println!("投资建议: {}\n", result_json["recommendation"].as_str().unwrap_or("N/A"));

                if let Some(component_scores) = result_json["component_scores"].as_object() {
                    println!("各维度评分:");
                    for (key, value) in component_scores {
                        println!("  - {}: {:.1}", key, value.as_f64().unwrap_or(0.0));
                    }
                }

                if let Some(investment_plan) = result_json["investment_plan"].as_object() {
                    println!("\n投资计划:");
                    for (key, value) in investment_plan {
                        if let Some(str_val) = value.as_str() {
                            println!("  - {}: {}", key, str_val);
                        } else if let Some(num_val) = value.as_f64() {
                            println!("  - {}: ${:.2}", key, num_val);
                        }
                    }
                }

                println!("\n详细分析:\n{}", serde_json::to_string_pretty(&result_json).unwrap_or_default());
            }

            if !output.result.metadata.is_empty() {
                println!("\n元数据:");
                for (key, value) in &output.result.metadata {
                    println!("  - {}: {}", key, value);
                }
            }
        }
        Err(e) => {
            println!("❌ 层次化分析失败: {}", e);
        }
    }

    println!("{}\n", "─".repeat(60));

    // 示例6: 组合压力测试
    println!("📌 示例6: 投资组合压力测试");
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
