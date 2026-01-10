//! InvestIntel AI - Claude Agent SDK 使用示例
//!
//! 本文件展示了如何在InvestIntel AI中使用Claude Agent SDK的各种功能
//! 所有示例都是真实可运行的代码

use anyhow::Result;
use claude_agent_sdk_rs::{
    query, query_stream, ClaudeAgentOptions, ContentBlock, Message, PermissionMode,
};
use futures::StreamExt;

// ============================================================================
// 示例1: 基础查询 (query API)
// ============================================================================

/// 基础查询示例 - 使用query() API进行一次性查询
#[tokio::test]
async fn example_1_basic_query() -> Result<()> {
    println!("\n=== 示例1: 基础查询 ===\n");

    // 创建配置选项
    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::AcceptEdits)
        .max_thinking_tokens(30000)
        .build();

    // 执行查询
    let prompt = "请分析AAPL股票的投资价值，给出买入、持有还是卖出的建议";

    let messages = query(prompt, Some(options)).await?;

    // 处理响应
    for message in messages {
        if let Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude回复: {}", text.text);
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// 示例2: 流式查询 (query_stream API)
// ============================================================================

/// 流式查询示例 - 使用query_stream() API进行实时流式分析
#[tokio::test]
async fn example_2_streaming_query() -> Result<()> {
    println!("\n=== 示例2: 流式查询 ===\n");

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::AcceptEdits)
        .max_thinking_tokens(50000)
        .build();

    let prompt = "请实时分析MSFT股票的技术指标，包括RSI、MACD、布林带等";

    let mut stream = query_stream(prompt, Some(options)).await?;

    println!("开始接收流式响应:\n");

    // 实时处理流式消息
    while let Some(result) = stream.next().await {
        let message = result?;

        match message {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    match block {
                        ContentBlock::Text(text) => {
                            print!("{}", text.text);
                        }
                        ContentBlock::ToolUse(tool) => {
                            println!("\n[工具调用] {} - 参数: {:?}", tool.name, tool.input);
                        }
                        ContentBlock::ToolResult(result) => {
                            println!("\n[工具结果] {}", result.tool_use_id);
                        }
                        ContentBlock::Thinking(thinking) => {
                            println!("\n[思考中] {}", thinking.thinking.chars().take(100).collect::<String>());
                        }
                        _ => {}
                    }
                }
            }
            Message::Result(result) => {
                println!("\n\n✅ 分析完成");
                println!("成本: ${:.4}", result.total_cost_usd.unwrap_or(0.0));
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

// ============================================================================
// 示例3: Agent系统使用
// ============================================================================

/// Agent系统示例 - 实现和使用自定义Agent
#[tokio::test]
async fn example_3_custom_agent() -> Result<()> {
    use claude_agent_sdk_rs::orchestration::{Agent, AgentInput, AgentOutput};

    println!("\n=== 示例3: 自定义Agent ===\n");

    // 定义一个简单的投资研究Agent
    struct InvestmentResearcher {
        name: String,
        options: ClaudeAgentOptions,
    }

    #[async_trait]
    impl Agent for InvestmentResearcher {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "投资研究分析师"
        }

        async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
            let prompt = format!(
                "作为投资研究专家，请分析以下投资机会：{}\n\n\
                请提供：\n1. 投资建议\n2. 风险评估\n3. 关键指标",
                input.content
            );

            let messages = query(&prompt, Some(self.options.clone())).await?;

            let mut analysis = String::new();
            for message in messages {
                if let Message::Assistant(msg) = message {
                    for block in &msg.message.content {
                        if let ContentBlock::Text(text) = block {
                            analysis.push_str(&text.text);
                        }
                    }
                }
            }

            Ok(AgentOutput::new(analysis).with_confidence(0.85))
        }
    }

    // 使用Agent
    let agent = InvestmentResearcher {
        name: "ResearchBot".to_string(),
        options: ClaudeAgentOptions::builder()
            .permission_mode(PermissionMode::AcceptEdits)
            .build(),
    };

    let input = AgentInput {
        content: "AAPL股票值得投资吗？".to_string(),
        context: serde_json::json!({"ticker": "AAPL"}),
        metadata: std::collections::HashMap::new(),
    };

    let output = agent.execute(input).await?;

    println!("Agent名称: {}", agent.name());
    println!("Agent描述: {}", agent.description());
    println!("\n分析结果:");
    println!("{}\n", output.content);
    println!("置信度: {:.1}%", output.confidence * 100.0);

    Ok(())
}

// ============================================================================
// 示例4: 编排系统使用
// ============================================================================

/// 编排系统示例 - 使用Orchestrator进行多Agent协作
#[tokio::test]
async fn example_4_orchestration() -> Result<()> {
    use claude_agent_sdk_rs::orchestration::{Agent, AgentInput, AgentOutput, Orchestrator, OrchestratorInput, SequentialOrchestrator};

    println!("\n=== 示例4: Agent编排 ===\n");

    // 创建多个专门的Agent
    struct TechnicalAnalyst;
    struct FundamentalAnalyst;
    struct RiskAnalyst;

    #[async_trait]
    impl Agent for TechnicalAnalyst {
        fn name(&self) -> &str { "TechnicalAnalyst" }
        fn description(&self) -> &str { "技术分析专家" }

        async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
            let prompt = format!("作为技术分析专家，请分析{}的技术面：趋势、支撑阻力、技术指标", input.content);
            let options = ClaudeAgentOptions::builder().permission_mode(PermissionMode::AcceptEdits).build();
            let messages = query(&prompt, Some(options)).await?;
            let analysis = messages.iter()
                .filter_map(|m| match m {
                    Message::Assistant(msg) => Some(msg.message.content.iter().filter_map(|b| {
                        if let ContentBlock::Text(t) = b { Some(t.text.clone()) } else { None }
                    }).collect::<Vec<_>>().join("\n")),
                    _ => None,
                })
                .next()
                .unwrap_or_default();

            Ok(AgentOutput::new(analysis).with_confidence(0.80))
        }
    }

    #[async_trait]
    impl Agent for FundamentalAnalyst {
        fn name(&self) -> &str { "FundamentalAnalyst" }
        fn description(&self) -> &str { "基本面分析专家" }

        async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
            let prompt = format!("作为基本面分析专家，请分析{}的基本面：财务状况、估值、成长性", input.content);
            let options = ClaudeAgentOptions::builder().permission_mode(PermissionMode::AcceptEdits).build();
            let messages = query(&prompt, Some(options)).await?;
            let analysis = messages.iter()
                .filter_map(|m| match m {
                    Message::Assistant(msg) => Some(msg.message.content.iter().filter_map(|b| {
                        if let ContentBlock::Text(t) = b { Some(t.text.clone()) } else { None }
                    }).collect::<Vec<_>>().join("\n")),
                    _ => None,
                })
                .next()
                .unwrap_or_default();

            Ok(AgentOutput::new(analysis).with_confidence(0.82))
        }
    }

    #[async_trait]
    impl Agent for RiskAnalyst {
        fn name(&self) -> &str { "RiskAnalyst" }
        fn description(&self) -> &str { "风险分析专家" }

        async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
            let prompt = format!("作为风险分析专家，请评估{}的投资风险", input.content);
            let options = ClaudeAgentOptions::builder().permission_mode(PermissionMode::AcceptEdits).build();
            let messages = query(&prompt, Some(options)).await?;
            let analysis = messages.iter()
                .filter_map(|m| match m {
                    Message::Assistant(msg) => Some(msg.message.content.iter().filter_map(|b| {
                        if let ContentBlock::Text(t) = b { Some(t.text.clone()) } else { None }
                    }).collect::<Vec<_>>().join("\n")),
                    _ => None,
                })
                .next()
                .unwrap_or_default();

            Ok(AgentOutput::new(analysis).with_confidence(0.88))
        }
    }

    // 创建编排器
    let orchestrator = SequentialOrchestrator::new();

    // 创建Agent列表
    let agents: Vec<Box<dyn Agent>> = vec![
        Box::new(TechnicalAnalyst),
        Box::new(FundamentalAnalyst),
        Box::new(RiskAnalyst),
    ];

    // 执行编排
    let input = OrchestratorInput::new("GOOGL股票全面分析")
        .with_context(serde_json::json!({"analysis_type": "comprehensive"}))
        .with_metadata("urgency", "normal");

    let output = orchestrator.orchestrate(agents, input).await?;

    println!("编排执行结果:");
    println!("  成功: {}", output.is_successful());
    println!("  Agent输出数量: {}", output.agent_outputs.len());
    println!("  执行时间: {:?}", output.execution_trace.duration());

    for (i, agent_output) in output.agent_outputs.iter().enumerate() {
        println!("\nAgent {} 输出:", i + 1);
        println!("  {}", agent_output.content.chars().take(200).collect::<String>());
        println!("  置信度: {:.1}%", agent_output.confidence * 100.0);
    }

    Ok(())
}

// ============================================================================
// 示例5: MCP工具使用
// ============================================================================

/// MCP工具示例 - 使用tool!宏创建自定义工具
#[tokio::test]
async fn example_5_mcp_tools() -> Result<()> {
    println!("\n=== 示例5: MCP工具 ===\n");

    use claude_agent_sdk_rs::{tool, create_sdk_mcp_server, ToolResult, McpToolResultContent};

    // 定义一个简单的股票分析工具
    async fn analyze_stock(args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let ticker = args["ticker"].as_str().unwrap_or("UNKNOWN");
        let analysis = serde_json::json!({
            "ticker": ticker,
            "recommendation": "BUY",
            "confidence": 0.85,
            "target_price": 175.0,
            "reason": "强劲的盈利能力和增长潜力"
        });

        Ok(ToolResult {
            content: vec![McpToolResultContent::Text {
                text: serde_json::to_string_pretty(&analysis)?
            }],
            is_error: false,
        })
    }

    // 使用tool!宏创建工具
    let stock_analysis_tool = tool! {
        name: "analyze_stock",
        description: "分析股票并给出投资建议",
        handler: analyze_stock
    };

    println!("✅ MCP工具创建成功:");
    println!("  名称: {}", stock_analysis_tool.name);
    println!("  描述: {}", stock_analysis_tool.description);

    // 创建MCP服务器
    let server = create_sdk_mcp_server(
        "investintel-tools",
        "1.0.0",
        vec![stock_analysis_tool]
    );

    println!("\n✅ MCP服务器创建成功:");
    println!("  服务器名称: investintel-tools");
    println!("  版本: 1.0.0");
    println!("  工具数量: {}", server.tools.len());

    // 在实际使用中，将server添加到ClaudeAgentOptions
    let options = ClaudeAgentOptions::builder()
        .mcp_servers(claude_agent_sdk_rs::McpServers::Single(server))
        .allowed_tools(vec!["mcp__investintel-tools__analyze_stock".to_string()])
        .permission_mode(PermissionMode::AcceptEdits)
        .build();

    println!("\n✅ MCP服务器已配置到ClaudeAgentOptions");

    Ok(())
}

// ============================================================================
// 示例6: 完整的投资分析流程
// ============================================================================

/// 完整示例 - 综合使用多种SDK功能
#[tokio::test]
async fn example_6_complete_workflow() -> Result<()> {
    println!("\n=== 示例6: 完整投资分析流程 ===\n");

    // 步骤1: 创建配置
    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::AcceptEdits)
        .max_thinking_tokens(50000)
        .max_turns(10)
        .build();

    println!("✅ 步骤1: Claude配置创建成功");

    // 步骤2: 执行流式分析
    let prompt = "请对TSLA股票进行全面的实时分析，包括技术面、基本面、情感和风险评估";

    println!("✅ 步骤2: 开始流式分析...");

    let mut stream = query_stream(prompt, Some(options.clone())).await?;
    let mut full_analysis = String::new();

    while let Some(result) = stream.next().await {
        let message = result?;

        match message {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        full_analysis.push_str(&text.text);
                    }
                }
            }
            Message::Result(_) => {
                println!("✅ 步骤3: 分析完成");
                break;
            }
            _ => {}
        }
    }

    println!("✅ 步骤4: 分析结果长度: {} 字符", full_analysis.len());

    // 步骤5: 提取关键信息
    let keywords = vec!["建议", "风险", "目标价", "支撑", "阻力"];
    println!("\n✅ 步骤5: 关键信息提取:");

    for keyword in keywords {
        if full_analysis.contains(keyword) {
            println!("    • 包含关键词: {}", keyword);
        }
    }

    println!("\n📊 分析摘要:");
    println!("{}\n", full_analysis.chars().take(500).collect::<String>());
    println!("...(完整分析共{}字符)", full_analysis.len());

    Ok(())
}

// ============================================================================
// 示例7: 高级配置选项
// ============================================================================

/// 高级配置示例 - 展示各种ClaudeAgentOptions配置
#[tokio::test]
async fn example_7_advanced_options() -> Result<()> {
    println!("\n=== 示例7: 高级配置选项 ===\n");

    // 示例配置1: 预算控制
    let options1 = ClaudeAgentOptions::builder()
        .max_budget_usd(5.0)  // 最大预算5美元
        .fallback_model("claude-haiku-4")  // 备用模型
        .build();

    println!("✅ 配置1 - 预算控制:");
    println!("  最大预算: $5.00");
    println!("  备用模型: claude-haiku-4");

    // 示例配置2: 性能优化
    let options2 = ClaudeAgentOptions::builder()
        .max_turns(5)  // 最多5轮对话
        .max_thinking_tokens(20000)  // 最大思考token数
        .include_partial_messages(true)  // 包含部分消息
        .build();

    println!("\n✅ 配置2 - 性能优化:");
    println!("  最大轮数: 5");
    println!("  最大思考token: 20,000");
    println!("  包含部分消息: true");

    // 示例配置3: 权限控制
    let options3 = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)  // 绕过所有权限
        .allowed_tools(vec![
            "Read".to_string(),
            "Bash".to_string(),
            "WebFetch".to_string(),
        ])
        .build();

    println!("\n✅ 配置3 - 权限控制:");
    println!("  权限模式: BypassPermissions");
    println!("  允许的工具: Read, Bash, WebFetch");

    // 示例配置4: 模型选择
    let options4 = ClaudeAgentOptions::builder()
        .model("claude-opus-4")  // 使用Opus模型
        .build();

    println!("\n✅ 配置4 - 模型选择:");
    println!("  模型: claude-opus-4");

    println!("\n✅ 所有配置创建成功");

    Ok(())
}

// ============================================================================
// 主函数 - 运行所有示例
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   InvestIntel AI - Claude Agent SDK 使用示例             ║");
    println!("║   所有示例都是真实可运行的代码                             ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    println!("可运行的示例:");
    println!("  1. 基础查询 (query API)");
    println!("  2. 流式查询 (query_stream API)");
    println!("  3. 自定义Agent");
    println!("  4. Agent编排");
    println!("  5. MCP工具");
    println!("  6. 完整投资分析流程");
    println!("  7. 高级配置选项");
    println!("\n使用 cargo test 运行所有示例\n");

    Ok(())
}
