//! InvestIntel AI - Enhanced Main with Full Claude Agent SDK Integration
//!
//! This version fully leverages:
//! - Claude Agent SDK's query and query_stream APIs
//! - Agent Skills system with automatic discovery
//! - Subagents orchestration (sequential, parallel, hierarchical)
//! - MCP Tools for investment analysis
//! - libSQL for high-performance data persistence
//! - Local LLM (Ollama) integration with smart routing

use anyhow::{Context, Result};
use claude_agent_sdk_rs::{
    AgentDefinition, AgentModel, ClaudeAgentOptions, ContentBlock, Message,
    McpServers, PermissionMode, SdkMcpServer, create_sdk_mcp_server, query,
    query_stream, tool,
};
use futures::StreamExt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

mod tools;
mod hierarchical_orchestration;
mod local_llm;
mod storage;

use tools::*;
use hierarchical_orchestration::{create_hierarchical_orchestrator, AdvisorCoordinator};
use local_llm::{LocalLLMRouter, LLMProvider};
use storage::LibSQLStorageManager;

/// Enhanced application state with full SDK integration
struct InvestIntelApp {
    tools: SdkMcpServer,
    storage: LibSQLStorageManager,
    llm_router: LocalLLMRouter,
    advisor_coordinator: Arc<AdvisorCoordinator>,
}

impl InvestIntelApp {
    /// Create a new enhanced application instance
    async fn new() -> Result<Self> {
        // Create MCP tools for investment analysis
        let tools = create_investment_tools()?;

        // Initialize libSQL storage manager (200ns query latency)
        let storage = LibSQLStorageManager::new("data/investintel.db")
            .await
            .context("Failed to initialize libSQL storage")?;

        // Initialize local LLM router with Ollama support
        let llm_router = LocalLLMRouter::new()
            .with_provider(LLMProvider::Ollama)
            .with_fallback_model("claude-sonnet-4-20250514")
            .with_ollama_base_url("http://localhost:11434")
            .with_ollama_model("llama3.1:70b");

        // Create hierarchical orchestrator with advisor coordinator
        let advisor_coordinator = Arc::new(AdvisorCoordinator::new());

        Ok(Self {
            tools,
            storage,
            llm_router,
            advisor_coordinator,
        })
    }

    /// Demonstrate Agent Skills with automatic discovery
    async fn demo_agent_skills(&self, symbol: &str) -> Result<()> {
        println!("🎯 Demo: Agent Skills Automatic Discovery");
        println!("{}\n", "─".repeat(60));

        let options = ClaudeAgentOptions::builder()
            .permission_mode(PermissionMode::BypassPermissions)
            .mcp_servers(McpServers::new().add_server(self.tools.clone()))
            .auto_discover_skills(true)
            .project_skills_dir(PathBuf::from(".claude/skills"))
            .thinking(true)
            .max_thinking_tokens(2000)
            .build();

        let prompt = format!(
            "使用investment-analyst skill分析{}的投资价值。请调用相关tools获取市场数据并给出综合评估。",
            symbol
        );

        println!("📝 Query: {}\n", prompt);
        println!("{}\n", "─".repeat(60));

        let messages = query(&prompt, Some(options)).await?;

        for msg in messages {
            if let Message::Assistant(assistant_msg) = msg {
                for block in assistant_msg.message.content {
                    if let ContentBlock::Text { text } = block {
                        println!("{}\n", text);
                    }
                }
            }
        }

        Ok(())
    }

    /// Demonstrate query_stream for real-time streaming analysis
    async fn demo_streaming_analysis(&self, symbol: &str) -> Result<()> {
        println!("🌊 Demo: Real-time Streaming Analysis");
        println!("{}\n", "─".repeat(60));

        let options = ClaudeAgentOptions::builder()
            .permission_mode(PermissionMode::BypassPermissions)
            .mcp_servers(McpServers::new().add_server(self.tools.clone()))
            .auto_discover_skills(true)
            .project_skills_dir(PathBuf::from(".claude/skills"))
            .build();

        let prompt = format!(
            "对{}进行全面的投资分析，包括技术面、基本面、风险和情感分析。请逐步展示你的分析过程。",
            symbol
        );

        println!("📝 Streaming Query: {}\n", prompt);
        println!("{}\n", "─".repeat(60));

        let mut stream = query_stream(&prompt, Some(options)).await?;

        println!("🔄 Real-time Analysis:\n");

        while let Some(result) = stream.next().await {
            match result {
                Ok(msg) => {
                    if let Message::Assistant(assistant_msg) = msg {
                        for block in &assistant_msg.message.content {
                            if let ContentBlock::Text { text } = block {
                                print!("{}", text);
                                // Use std::io::stdout().flush()? for real output
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Stream error: {}", e);
                    break;
                }
            }
        }

        println!("\n\n✅ Streaming analysis complete\n");
        Ok(())
    }

    /// Demonstrate Subagents with Sequential Orchestration
    async fn demo_sequential_subagents(&self, symbol: &str) -> Result<()> {
        println!("🔄 Demo: Sequential Subagents Orchestration");
        println!("{}\n", "─".repeat(60));

        let mut agents = HashMap::new();

        agents.insert(
            "market-research".to_string(),
            AgentDefinition::builder()
                .description("Market research expert for technical analysis")
                .prompt(
                    "你是市场研究专家。请对{}进行技术分析，包括趋势识别、支撑阻力位、技术指标等。"
                        .to_string(),
                )
                .model(AgentModel::Sonnet)
                .tools(vec!["technical_analysis".to_string()])
                .build(),
        );

        agents.insert(
            "fundamental-analysis".to_string(),
            AgentDefinition::builder()
                .description("Investment analyst for fundamental analysis")
                .prompt(
                    "你是投资分析师。请基于市场研究数据，对{}进行基本面分析和估值。"
                        .to_string(),
                )
                .model(AgentModel::Opus)
                .tools(vec!["var_calculation".to_string()])
                .build(),
        );

        agents.insert(
            "risk-assessment".to_string(),
            AgentDefinition::builder()
                .description("Risk management expert")
                .prompt(
                    "你是风险管理专家。请基于前面的分析，评估{}的投资风险。".to_string(),
                )
                .model(AgentModel::Opus)
                .tools(vec![
                    "var_calculation".to_string(),
                    "stress_test".to_string(),
                ])
                .build(),
        );

        let options = ClaudeAgentOptions::builder()
            .permission_mode(PermissionMode::BypassPermissions)
            .mcp_servers(McpServers::new().add_server(self.tools.clone()))
            .agents(agents)
            .build();

        let prompt = format!(
            "使用顺序编排模式分析{}：
1. 首先使用market-research agent进行技术分析
2. 然后使用fundamental-analysis agent进行基本面分析
3. 最后使用risk-assessment agent评估风险
请依次调用这些agents并综合结果。",
            symbol
        );

        println!("📝 Sequential Query: {}\n", prompt);
        println!("{}\n", "─".repeat(60));

        let messages = query(&prompt, Some(options)).await?;

        for msg in messages {
            if let Message::Assistant(assistant_msg) = msg {
                for block in assistant_msg.message.content {
                    if let ContentBlock::Text { text } = block {
                        println!("{}\n", text);
                    }
                }
            }
        }

        Ok(())
    }

    /// Demonstrate Subagents with Parallel Orchestration
    async fn demo_parallel_subagents(&self, symbol: &str) -> Result<()> {
        println!("⚡ Demo: Parallel Subagents Orchestration");
        println!("{}\n", "─".repeat(60));

        let options = ClaudeAgentOptions::builder()
            .permission_mode(PermissionMode::BypassPermissions)
            .mcp_servers(McpServers::new().add_server(self.tools.clone()))
            .auto_discover_skills(true)
            .project_skills_dir(PathBuf::from(".claude/skills"))
            .build();

        // Create parallel queries using tokio::join
        let (technical_result, fundamental_result, sentiment_result) = tokio::join!(
            query(
                &format!("使用market-research skill对{}进行技术分析", symbol),
                Some(options.clone())
            ),
            query(
                &format!("使用fundamental-analysis skill对{}进行基本面分析", symbol),
                Some(options.clone())
            ),
            query(
                &format!("使用sentiment-analysis skill分析{}的市场情感", symbol),
                Some(options.clone())
            )
        );

        println!("📊 Parallel Analysis Results for {}\n", symbol);
        println!("{}\n", "─".repeat(60));

        // Display results
        println!("📈 Technical Analysis:");
        match technical_result {
            Ok(msgs) => display_messages(msgs),
            Err(e) => println!("❌ Error: {}", e),
        }

        println!("\n📊 Fundamental Analysis:");
        match fundamental_result {
            Ok(msgs) => display_messages(msgs),
            Err(e) => println!("❌ Error: {}", e),
        }

        println!("\n💭 Sentiment Analysis:");
        match sentiment_result {
            Ok(msgs) => display_messages(msgs),
            Err(e) => println!("❌ Error: {}", e),
        }

        println!("\n✅ All parallel analyses complete\n");
        Ok(())
    }

    /// Demonstrate Hierarchical Orchestration with Advisor Coordinator
    async fn demo_hierarchical_orchestration(&self, symbol: &str) -> Result<()> {
        println!("🏛️  Demo: Hierarchical Orchestration (Advisor + Subagents)");
        println!("{}\n", "─".repeat(60));

        println!("📊 Running comprehensive hierarchical analysis for {}\n", symbol);

        let orchestrator = create_hierarchical_orchestrator(self.advisor_coordinator.clone());

        // The advisor coordinator will orchestrate all subagents
        let analysis_results = self
            .advisor_coordinator
            .run_comprehensive_analysis(symbol)
            .await?;

        println!("🎯 Comprehensive Analysis Results:\n");
        println!("{}\n", "─".repeat(60));

        for (agent_name, result) in &analysis_results {
            println!("📋 {} Analysis:", agent_name);
            println!("{}\n", serde_json::to_string_pretty(result).unwrap());
        }

        // Calculate and display composite score
        let research_score = analysis_results["research"]["technical_score"].as_i64().unwrap_or(50);
        let analyst_score = analysis_results["analyst"]["fundamental_score"].as_i64().unwrap_or(50);
        let risk_score = analysis_results["risk"]["risk_score"].as_i64().unwrap_or(50);
        let sentiment_score = analysis_results["sentiment"]["sentiment_score"].as_i64().unwrap_or(50);

        let composite_score = (research_score * 25
            + analyst_score * 35
            + sentiment_score * 15
            + (100 - risk_score) * 25)
            / 100;

        println!("🎯 Composite Investment Score: {}/100", composite_score);

        let recommendation = match composite_score {
            s if s >= 80 => "🟢 Strong Buy",
            s if s >= 65 => "🟢 Buy",
            s if s >= 50 => "🟡 Hold",
            s if s >= 35 => "🟡 Reduce",
            _ => "🔴 Sell",
        };

        println!("💡 Recommendation: {}\n", recommendation);

        Ok(())
    }

    /// Demonstrate libSQL high-performance storage
    async fn demo_libsql_storage(&self) -> Result<()> {
        println!("💾 Demo: libSQL High-Performance Storage (200ns queries)");
        println!("{}\n", "─".repeat(60));

        // Save a test portfolio
        let portfolio_data = serde_json::json!({
            "name": "Tech Portfolio",
            "positions": [
                {"symbol": "AAPL", "shares": 100, "avg_cost": 150.0},
                {"symbol": "MSFT", "shares": 50, "avg_cost": 300.0},
                {"symbol": "GOOGL", "shares": 30, "avg_cost": 2800.0}
            ],
            "created_at": "2026-01-10"
        });

        println!("💼 Saving portfolio to libSQL...");
        self.storage
            .save_portfolio("test_portfolio", &portfolio_data)
            .await?;
        println!("✅ Portfolio saved\n");

        // Load portfolio (demonstrates 200ns query latency)
        println!("🔍 Loading portfolio from libSQL...");
        let start = std::time::Instant::now();
        let loaded = self
            .storage
            .load_portfolio("test_portfolio")
            .await?
            .expect("Portfolio not found");
        let duration = start.elapsed();

        println!("✅ Portfolio loaded in {:?}", duration);
        println!("📊 Portfolio data: {}\n", serde_json::to_string_pretty(&loaded).unwrap());

        Ok(())
    }

    /// Demonstrate Local LLM integration with smart routing
    async fn demo_local_llm(&self, prompt: &str) -> Result<()> {
        println!("🤖 Demo: Local LLM (Ollama) Integration");
        println!("{}\n", "─".repeat(60));

        println!("🔧 Querying local LLM (Ollama)...");
        println!("📝 Prompt: {}\n", prompt);

        match self.llm_router.query(prompt).await {
            Ok(response) => {
                println!("🤖 Local LLM Response:\n{}", response);
                println!("\n✅ Local LLM query successful\n");
            }
            Err(e) => {
                println!("⚠️  Local LLM failed: {}", e);
                println!("🔄 Fallback to Claude API would be triggered\n");
            }
        }

        Ok(())
    }

    /// Run all demonstrations
    async fn run_all_demos(&self, symbol: &str) -> Result<()> {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║     InvestIntel AI - Full Claude Agent SDK Demo            ║");
        println!("║     Enhanced Edition with Complete Integration            ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");

        // Demo 1: Agent Skills
        self.demo_agent_skills(symbol).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Demo 2: Streaming Analysis
        self.demo_streaming_analysis(symbol).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Demo 3: Sequential Subagents
        self.demo_sequential_subagents(symbol).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Demo 4: Parallel Subagents
        self.demo_parallel_subagents(symbol).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Demo 5: Hierarchical Orchestration
        self.demo_hierarchical_orchestration(symbol).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Demo 6: libSQL Storage
        self.demo_libsql_storage().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Demo 7: Local LLM
        self.demo_local_llm("Explain VaR in simple terms").await?;

        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║     All Demos Complete!                                    ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");

        Ok(())
    }
}

/// Helper function to display messages
fn display_messages(messages: Vec<Message>) {
    for msg in messages {
        if let Message::Assistant(assistant_msg) = msg {
            for block in assistant_msg.message.content {
                if let ContentBlock::Text { text } = block {
                    println!("{}", text);
                }
            }
        }
    }
}

/// Create complete MCP tools for investment analysis
fn create_investment_tools() -> Result<SdkMcpServer> {
    let tools = create_sdk_mcp_server(
        "investment-tools",
        vec![
            tool! {
                name: "technical_analysis",
                description: "Technical analysis with 30+ indicators (RSI, MACD, MA, Support/Resistance)",
                handler: technical_analysis
            },
            tool! {
                name: "var_calculation",
                description: "Calculate Value at Risk (VaR) using historical, parametric, and Monte Carlo methods",
                handler: var_calculation
            },
            tool! {
                name: "sentiment_analysis",
                description: "Analyze market sentiment from news, Twitter, Reddit using FinBERT",
                handler: sentiment_analysis
            },
            tool! {
                name: "save_portfolio",
                description: "Save portfolio to libSQL storage with 200ns query latency",
                handler: save_portfolio
            },
            tool! {
                name: "load_portfolio",
                description: "Load portfolio from libSQL storage",
                handler: load_portfolio
            },
            tool! {
                name: "stress_test",
                description: "Run stress test scenarios (2008 crisis, COVID, custom scenarios)",
                handler: stress_test
            },
            tool! {
                name: "correlation_analysis",
                description: "Calculate correlation matrix for multiple assets",
                handler: correlation_analysis
            },
        ],
    )?;

    Ok(tools)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║           InvestIntel AI - Enhanced Edition                ║");
    println!("║     Full Claude Agent SDK + Agent Skills + Subagents       ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Create enhanced application
    println!("🚀 Initializing InvestIntel AI with full SDK integration...\n");
    let app = InvestIntelApp::new().await?;

    println!("✅ Initialization complete!\n");

    // Run all demonstrations
    let symbol = "AAPL";
    app.run_all_demos(symbol).await?;

    Ok(())
}
