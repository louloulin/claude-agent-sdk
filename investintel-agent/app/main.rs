//! InvestIntel AI - 智能投资助手
//! 基于Claude Agent SDK

use anyhow::Result;
use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ContentBlock, Message, McpServers, ToolResult, create_sdk_mcp_server, query,
    tool,
};
use serde_json::json;

// MCP Tools
async fn technical_analysis(args: serde_json::Value) -> Result<ToolResult> {
    let symbol = args["symbol"].as_str().unwrap_or("UNKNOWN");
    Ok(ToolResult {
        content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
            text: format!("Technical analysis for {}: trend is bullish", symbol),
        }],
        is_error: false,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("InvestIntel AI - 智能投资助手");
    println!("基于Claude Agent SDK\n");

    let tools = create_sdk_mcp_server(
        "investment-tools",
        vec![tool! {
            name: "technical_analysis",
            description: "Technical analysis",
            handler: technical_analysis
        }],
    )?;

    let options = ClaudeAgentOptions::builder()
        .permission_mode(claude_agent_sdk_rs::PermissionMode::BypassPermissions)
        .mcp_servers(McpServers::new().add_server(tools))
        .build();

    println!("✅ 开始分析...\n");

    let messages = query("分析AAPL股票", Some(options)).await?;

    for msg in &messages {
        if let Message::Assistant(assistant_msg) = msg {
            for block in &assistant_msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
    }

    Ok(())
}
