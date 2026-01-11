//! Integration test for Skill Agent functionality
//!
//! This test demonstrates that agents can successfully load and use skills
//! from the .claude/skills directory.

use investintel_agent::skills::{SkillAgent, SkillRegistry};
use claude_agent_sdk_rs::orchestration::{Agent, AgentInput};
use std::path::PathBuf;

#[tokio::test]
async fn test_skill_registry() {
    // Load skills from .claude/skills directory
    let skills_dir = PathBuf::from(".claude/skills");
    let registry = SkillRegistry::from_dir(skills_dir).await;

    assert!(registry.is_ok(), "Failed to load skill registry");

    let registry = registry.unwrap();
    let skills = registry.list_skills().await;

    println!("Loaded {} skills:", skills.len());
    for skill in &skills {
        println!("  - {}", skill);
    }

    // Should have loaded at least 5 skills
    assert!(skills.len() >= 5, "Expected at least 5 skills, got {}", skills.len());
    assert!(skills.contains(&"Graham深度价值投资".to_string()));
    assert!(skills.contains(&"Buffett质量价值投资".to_string()));
    assert!(skills.contains(&"Munger多元思维模型".to_string()));
    assert!(skills.contains(&"Kelly准则仓位管理".to_string()));
    assert!(skills.contains(&"Lollapalooza效应检测".to_string()));
}

#[tokio::test]
async fn test_graham_agent() {
    let registry = SkillRegistry::from_dir(PathBuf::from(".claude/skills"))
        .await
        .unwrap();

    let agent = SkillAgent::new(
        "GrahamAgent",
        "Applies Graham value investing analysis",
        "Graham深度价值投资",
        registry,
    );

    // Test with sample data
    let input_data = serde_json::json!({
        "symbol": "TEST",
        "eps": 5.0,
        "growth_rate": 0.06,
        "price": 50.0
    });

    let input = AgentInput::new("Analyze TEST")
        .with_context(input_data);

    let output = agent.execute(input).await.unwrap();

    assert!(output.is_successful());
    assert!(output.confidence > 0.5);
    assert!(output.content.contains("Graham"));

    println!("Graham Agent Output:\n{}", output.content);
}

#[tokio::test]
async fn test_kelly_agent() {
    let registry = SkillRegistry::from_dir(PathBuf::from(".claude/skills"))
        .await
        .unwrap();

    let agent = SkillAgent::new(
        "KellyAgent",
        "Applies Kelly position sizing",
        "Kelly准则仓位管理",
        registry,
    );

    let input_data = serde_json::json!({
        "symbol": "TEST",
        "expected_return": 0.15,
        "variance": 0.0625
    });

    let input = AgentInput::new("Calculate position")
        .with_context(input_data);

    let output = agent.execute(input).await.unwrap();

    assert!(output.is_successful());
    assert!(output.data.is_object());

    println!("Kelly Agent Output:\n{}", output.content);
}

#[tokio::test]
async fn test_lollapalooza_agent() {
    let registry = SkillRegistry::from_dir(PathBuf::from(".claude/skills"))
        .await
        .unwrap();

    let agent = SkillAgent::new(
        "LollapaloozaAgent",
        "Detects Lollapalooza opportunities",
        "Lollapalooza效应检测",
        registry,
    );

    // High scores example - should be Strong Lollapalooza
    let input_data = serde_json::json!({
        "symbol": "SUPER",
        "valuation_score": 0.20,
        "quality_score": 0.25,
        "moat_score": 0.25,
        "catalyst_score": 0.18
    });

    let input = AgentInput::new("Detect Lollapalooza")
        .with_context(input_data);

    let output = agent.execute(input).await.unwrap();

    assert!(output.is_successful());

    // Should detect Strong Lollapalooza
    let data = &output.data;
    if let Some(analysis) = data.get("analysis") {
        if let Some(level) = analysis.get("level") {
            println!("Lollapalooza Level: {}", level);
        }
    }

    println!("Lollapalooza Agent Output:\n{}", output.content);
}
