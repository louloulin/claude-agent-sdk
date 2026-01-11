//! InvestIntel AI - CLI工具
//!
//! 投资智能助手命令行界面

use investintel_agent::InvestmentAssistant;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🤖 InvestIntel AI - 投资智能助手");
    println!("基于Graham-Buffett-Munger价值投资理念\n");
    println!("输入'quit'或'exit'退出\n");

    let assistant = InvestmentAssistant::new();

    loop {
        print!("💬 您: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!("👋 再见！");
            break;
        }

        if input.eq_ignore_ascii_case("help") {
            print_help();
            continue;
        }

        // 调用投资助手
        match assistant.chat(input).await {
            Ok(response) => {
                println!("\n🤖 助手:\n{}\n", response);
            }
            Err(e) => {
                eprintln!("❌ 错误: {}", e);
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("\n📖 帮助信息\n");
    println!("可用命令:");
    println!("  analyze <股票代码>   - 分析股票投资价值");
    println!("  help                - 显示此帮助信息");
    println!("  quit/exit           - 退出程序\n");
    println!("示例:");
    println!("  analyze AAPL       - 分析苹果公司");
    println!("  分析 MSFT           - 分析微软公司");
    println!("  GOOGL怎么样        - 分析谷歌公司\n");
}
