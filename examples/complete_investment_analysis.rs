//! InvestIntel AI - Complete Advanced Example
//!
//! This example demonstrates the full power of Claude Agent SDK integration:
//! - Real Agent implementations
//! - Real Orchestrator implementations
//! - ClaudeClient bidirectional communication
//! - Hooks system for security and monitoring
//! - Interactive multi-turn analysis
//!
//! Run with:
//! ```bash
//! cargo run --example complete_investment_analysis
//! ```

use anyhow::Result;
use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ContentBlock, Hooks, Message, McpServers,
    PermissionMode, create_sdk_mcp_server, query,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

// Import our implementations
use investintel_agent::agents::*;
use investintel_agent::orchestrators::*;
use investintel_agent::investment_hooks::*;
use investintel_agent::interactive_advisor::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     InvestIntel AI - Complete Advanced Example             ║");
    println!("║     Full Claude Agent SDK Integration                       ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // ========================================================================
    // Demo 1: Real Agent Implementation
    // ========================================================================
    println!("📊 Demo 1: Real Agent Implementation");
    println!("{}\n", "─".repeat(60));

    demo_real_agents().await?;
    sleep(Duration::from_secs(1)).await;

    // ========================================================================
    // Demo 2: Real Orchestrator Implementation
    // ========================================================================
    println!("\n🔄 Demo 2: Real Orchestrator Implementation");
    println!("{}\n", "─".repeat(60));

    demo_real_orchestrators().await?;
    sleep(Duration::from_secs(1)).await;

    // ========================================================================
    // Demo 3: ClaudeClient Interactive Analysis
    // ========================================================================
    println!("\n💬 Demo 3: ClaudeClient Interactive Analysis");
    println!("{}\n", "─".repeat(60));

    demo_claude_client().await?;
    sleep(Duration::from_secs(1)).await;

    // ========================================================================
    // Demo 4: Hooks System
    // ========================================================================
    println!("\n🔒 Demo 4: Investment Analysis Hooks");
    println!("{}\n", "─".repeat(60));

    demo_hooks().await?;
    sleep(Duration::from_secs(1)).await;

    // ========================================================================
    // Demo 5: Complete Workflow with All Features
    // ========================================================================
    println!("\n🎯 Demo 5: Complete Workflow");
    println!("{}\n", "─".repeat(60));

    demo_complete_workflow().await?;

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║     All Demos Complete!                                    ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    Ok(())
}

/// Demo 1: Real Agent Implementation
async fn demo_real_agents() -> Result<()> {
    println!("Creating real Agent implementations...");

    // Create agents
    let market_agent = create_market_research_agent();
    let analyst_agent = create_investment_analyst_agent();
    let risk_agent = create_risk_management_agent();

    println!("✅ Created 3 real Agents:");
    println!("   - {}", market_agent.name());
    println!("   - {}", analyst_agent.name());
    println!("   - {}", risk_agent.name());

    // Execute market research agent
    println!("\n📈 Executing Market Research Agent...");
    let input = investintel_agent::orchestration::AgentInput::new("AAPL".to_string());

    match market_agent.execute(input).await {
        Ok(output) => {
            println!("✅ Agent executed successfully");
            println!("   Confidence: {:.2}", output.confidence);

            // Parse and display result
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output.content) {
                if let Some(score) = json["technical_score"].as_i64() {
                    println!("   Technical Score: {}/100", score);
                }
            }
        }
        Err(e) => {
            println!("❌ Agent execution failed: {}", e);
        }
    }

    Ok(())
}

/// Demo 2: Real Orchestrator Implementation
async fn demo_real_orchestrators() -> Result<()> {
    println!("Creating real Orchestrator implementations...");

    // Create investment orchestrator
    let orchestrator = create_investment_orchestrator();

    println!("✅ Created Investment Analysis Orchestrator");
    println!("   Orchestrator type: InvestmentAnalysisOrchestrator");
    println!("   Capabilities: Sequential + Parallel + Hybrid");

    // Run comprehensive analysis
    println!("\n🔍 Running comprehensive analysis for AAPL...");

    match orchestrator.run_comprehensive_analysis("AAPL").await {
        Ok(report) => {
            println!("✅ Analysis completed successfully");
            println!("\n📊 Analysis Results:");
            println!("   Symbol: {}", report.symbol);
            println!("   Technical Score: {}/100", report.scores.technical);
            println!("   Fundamental Score: {}/100", report.scores.fundamental);
            println!("   Risk Score: {}/100", report.scores.risk);
            println!("   Sentiment Score: {}/100", report.scores.sentiment);
            println!("   Composite Score: {}/100", report.scores.composite);
            println!("   Recommendation: {:?}", report.recommendation);
        }
        Err(e) => {
            println!("❌ Analysis failed: {}", e);
        }
    }

    Ok(())
}

/// Demo 3: ClaudeClient Interactive Analysis
async fn demo_claude_client() -> Result<()> {
    println!("Initializing ClaudeClient for interactive analysis...");

    let mut advisor = InteractiveInvestmentAdvisor::new().await?;

    println!("✅ ClaudeClient initialized");
    println!("   Capabilities:");
    println!("   - Multi-turn conversations");
    println!("   - Context awareness");
    println!("   - Real-time interaction");

    advisor.start_session().await?;

    // Single query demo
    println!("\n💬 Sending query: 分析AAPL的投资价值");
    let advice = advisor.analyze_interactive("请简要分析AAPL的投资价值,包括技术面和基本面。").await?;

    println!("\n✅ Received advice:");
    println!("   Recommendation: {:?}", advice.recommendation);
    println!("   Confidence: {:.2}", advice.confidence);

    advisor.end_session().await?;

    Ok(())
}

/// Demo 4: Hooks System
async fn demo_hooks() -> Result<()> {
    println!("Initializing Investment Analysis Hooks...");

    // Create investment hooks
    let investment_hooks = create_investment_hooks();
    let budget_hooks = create_budget_hooks(10.0);
    let compliance_hooks = create_compliance_hooks();

    println!("✅ Created hooks:");
    println!("   - Investment Hooks: Tool usage monitoring");
    println!("   - Budget Hooks: Budget control ($10 limit)");
    println!("   - Compliance Hooks: Regulatory compliance");

    // Convert to SDK hooks
    let hooks = investment_hooks.into_hooks();

    println!("\n🔧 Using hooks with query API...");

    // Create MCP tools
    let tools = create_sdk_mcp_server(
        "demo-tools",
        vec![
            tool! {
                name: "demo_tool",
                description: "Demo tool",
                handler: |args| async {
                    Ok(claude_agent_sdk_rs::ToolResult {
                        content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                            text: "Demo tool executed".to_string()
                        }],
                        is_error: false,
                    })
                }
            }
        ],
    )?;

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .mcp_servers(McpServers::new().add_server(tools))
        .hooks(hooks)
        .build();

    println!("✅ Hooks configured and ready");

    Ok(())
}

/// Demo 5: Complete Workflow
async fn demo_complete_workflow() -> Result<()> {
    println!("Running complete investment analysis workflow...");

    // Step 1: Create agents
    println!("\n📝 Step 1: Create specialized agents");
    let agents = create_all_agents();
    println!("   Created {} specialized agents", agents.len());

    // Step 2: Create orchestrator
    println!("\n🔄 Step 2: Create orchestrator");
    let orchestrator = create_investment_orchestrator();
    println!("   Orchestrator ready");

    // Step 3: Run analysis
    println!("\n🔍 Step 3: Run comprehensive analysis");
    let report = orchestrator.run_comprehensive_analysis("MSFT").await?;

    println!("\n📊 Analysis Report:");
    println!("   Symbol: {}", report.symbol);
    println!("   Technical Score: {}/100", report.scores.technical);
    println!("   Fundamental Score: {}/100", report.scores.fundamental);
    println!("   Risk Score: {}/100", report.scores.risk);
    println!("   Sentiment Score: {}/100", report.scores.sentiment);
    println!("   Composite Score: {}/100", report.scores.composite);
    println!("   Recommendation: {:?}", report.recommendation);

    // Step 4: Generate recommendation
    println!("\n💡 Step 4: Investment Recommendation");
    match report.recommendation {
        investintel_agent::orchestrators::Recommendation::StrongBuy => {
            println!("   🟢 Strong Buy - Strongly recommended to buy");
        }
        investintel_agent::orchestrators::Recommendation::Buy => {
            println!("   🟢 Buy - Recommended to buy");
        }
        investintel_agent::orchestrators::Recommendation::Hold => {
            println!("   🟡 Hold - Hold current position");
        }
        investintel_agent::orchestrators::Recommendation::Reduce => {
            println!("   🟡 Reduce - Consider reducing position");
        }
        investintel_agent::orchestrators::Recommendation::Sell => {
            println!("   🔴 Sell - Recommended to sell");
        }
    }

    println!("\n✅ Complete workflow finished successfully");

    Ok(())
}
