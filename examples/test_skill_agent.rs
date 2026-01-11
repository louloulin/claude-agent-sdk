use std::path::PathBuf;
use claude_agent_sdk_rs::orchestration::{Agent, AgentInput};
use investintel_agent::skills::{SkillAgent, SkillRegistry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Testing Skill Agent Integration\n");

    // Create skill registry
    let skills_dir = PathBuf::from(".claude/skills");
    println!("📁 Loading skills from: {:?}\n", skills_dir);

    let registry: SkillRegistry = SkillRegistry::from_dir(skills_dir).await?;
    let skills: Vec<String> = registry.list_skills().await;

    println!("✅ Loaded {} skills:", skills.len());
    for skill in &skills {
        println!("   - {}", skill);
    }
    println!();

    // Test 1: Graham Agent
    println!("🔬 Test 1: Graham Value Investing Agent");
    println!("=========================================\n");

    let graham_agent = SkillAgent::new(
        "GrahamAgent",
        "Applies Graham value investing analysis",
        "Graham深度价值投资",
        registry.clone(),
    );

    let input_data = serde_json::json!({
        "symbol": "AAPL",
        "eps": 5.50,
        "growth_rate": 0.08,
        "price": 165.0
    });

    let input = AgentInput::new("Analyze AAPL using Graham method")
        .with_context(input_data);

    let output = graham_agent.execute(input).await?;

    println!("{}\n", output.content);
    println!("Confidence: {:.2}\n", output.confidence);

    // Test 2: Kelly Agent
    println!("💰 Test 2: Kelly Position Sizing Agent");
    println!("========================================\n");

    let kelly_agent = SkillAgent::new(
        "KellyAgent",
        "Calculates optimal position size using Kelly criterion",
        "Kelly准则仓位管理",
        registry.clone(),
    );

    let input_data = serde_json::json!({
        "symbol": "AAPL",
        "expected_return": 0.15,
        "variance": 0.0625
    });

    let input = AgentInput::new("Calculate Kelly position for AAPL")
        .with_context(input_data);

    let output = kelly_agent.execute(input).await?;

    println!("{}\n", output.content);
    println!("Confidence: {:.2}\n", output.confidence);

    // Test 3: Lollapalooza Agent
    println!("🌟 Test 3: Lollapalooza Detection Agent");
    println!("=========================================\n");

    let lollapalooza_agent = SkillAgent::new(
        "LollapaloozaAgent",
        "Detects Lollapalooza investment opportunities",
        "Lollapalooza效应检测",
        registry.clone(),
    );

    let input_data = serde_json::json!({
        "symbol": "TSLA",
        "valuation_score": 0.18,
        "quality_score": 0.22,
        "moat_score": 0.15,
        "catalyst_score": 0.20
    });

    let input = AgentInput::new("Detect Lollapalooza for TSLA")
        .with_context(input_data);

    let output = lollapalooza_agent.execute(input).await?;

    println!("{}\n", output.content);
    println!("Confidence: {:.2}\n", output.confidence);

    // Test 4: Buffett Agent
    println!("🏆 Test 4: Buffett Quality Value Agent");
    println!("========================================\n");

    let buffett_agent = SkillAgent::new(
        "BuffettAgent",
        "Applies Buffett quality value investing",
        "Buffett质量价值投资",
        registry.clone(),
    );

    let input_data = serde_json::json!({
        "symbol": "BRK.A",
        "roic": 0.18,
        "roe": 0.22,
        "pe": 18.0
    });

    let input = AgentInput::new("Analyze BRK.A using Buffett method")
        .with_context(input_data);

    let output = buffett_agent.execute(input).await?;

    println!("{}\n", output.content);
    println!("Confidence: {:.2}\n", output.confidence);

    // Test 5: Munger Agent
    println!("🧠 Test 5: Munger Mental Models Agent");
    println!("========================================\n");

    let munger_agent = SkillAgent::new(
        "MungerAgent",
        "Applies Munger's multidisciplinary mental models",
        "Munger多元思维模型",
        registry.clone(),
    );

    let input_data = serde_json::json!({
        "symbol": "COST",
        "growth_rate": 0.10,
        "scale_advantage": true,
        "brand_strength": "high",
        "network_effects": true
    });

    let input = AgentInput::new("Analyze COST using Munger mental models")
        .with_context(input_data);

    let output = munger_agent.execute(input).await?;

    println!("{}\n", output.content);
    println!("Confidence: {:.2}\n", output.confidence);

    println!("✅ All tests completed successfully!");

    Ok(())
}
