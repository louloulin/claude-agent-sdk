//! 自动安装 Claude Code CLI 示例
//!
//! 此示例展示如何启用 SDK 的自动 CLI 安装功能

use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Claude Agent SDK - Auto-Install Example\n");

    // 方式 2: 通过代码配置启用
    let options = ClaudeAgentOptions::builder()
        .auto_install_cli(true)
        .build();

    println!("📦 Creating client with auto-install enabled...");
    println!("   If Claude CLI is not found, it will be downloaded automatically.\n");

    // 创建客户端（会触发自动安装检查）
    let mut client = ClaudeClient::try_new(options)?;

    println!("✅ Client created successfully! (CLI found or installed)\n");
    
    // 连接
    client.connect().await?;

    // 使用客户端进行查询
    println!("💬 Sending query to Claude...");
    client.query("Hello, Claude!").await?;
    
    // 接收响应
    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
         match message {
             Ok(msg) => println!("Received: {:?}", msg),
             Err(e) => eprintln!("Error: {}", e),
         }
    }

    Ok(())
}
