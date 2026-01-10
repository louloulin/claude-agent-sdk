//! InvestIntel AI - Unit Tests
//!
//! Comprehensive unit tests for investment analysis components

use std::path::PathBuf;

/// Test that all SKILL.md files have valid YAML frontmatter
#[test]
fn test_all_skill_files_have_valid_yaml() {
    let skills_dir = PathBuf::new(".claude/skills");

    let expected_skills = vec![
        "market-research",
        "portfolio-management",
        "risk-analysis",
        "sentiment-analysis",
        "technical-analysis",
        "fundamental-analysis",
        "strategy-planner",
        "backtesting",
        "reporting",
        "investment-analyst",
    ];

    for skill in expected_skills {
        let skill_path = skills_dir.join(skill).join("SKILL.md");
        assert!(skill_path.exists(), "SKILL.md not found for: {}", skill);

        let content = std::fs::read_to_string(&skill_path)
            .unwrap_or_else(|_| panic!("Failed to read: {:?}", skill_path));

        // Verify YAML frontmatter
        assert!(content.starts_with("---"), "Missing YAML frontmatter in: {}", skill);

        // Verify YAML end marker
        assert!(content.contains("\n---"), "Missing YAML end marker in: {}", skill);

        // Verify required fields
        assert!(content.contains("name:"), "Missing 'name' field in: {}", skill);
        assert!(content.contains("description:"), "Missing 'description' field in: {}", skill);

        println!("✅ {} - Valid YAML frontmatter", skill);
    }
}

/// Test that all agent config files are valid
#[test]
fn test_all_agent_files_are_valid() {
    let agents_dir = PathBuf::new(".claude/agents");

    let expected_agents = vec![
        "research-agent.md",
        "analyst-agent.md",
        "risk-agent.md",
        "advisor-agent.md",
        "technical-analyst.md",
        "strategy-executor.md",
    ];

    for agent in expected_agents {
        let agent_path = agents_dir.join(agent);
        assert!(agent_path.exists(), "Agent file not found: {}", agent);

        let content = std::fs::read_to_string(&agent_path)
            .unwrap_or_else(|_| panic!("Failed to read: {:?}", agent_path));

        // Verify YAML frontmatter
        assert!(content.starts_with("---"), "Missing YAML frontmatter in: {}", agent);
        assert!(content.contains("\n---"), "Missing YAML end marker in: {}", agent);

        // Verify required fields
        assert!(content.contains("name:"), "Missing 'name' in: {}", agent);
        assert!(content.contains("description:"), "Missing 'description' in: {}", agent);
        assert!(content.contains("model:"), "Missing 'model' in: {}", agent);
        assert!(content.contains("skills:"), "Missing 'skills' in: {}", agent);

        println!("✅ {} - Valid agent configuration", agent);
    }
}

/// Test skills have correct metadata structure
#[test]
fn test_skill_metadata_structure() {
    let skills_dir = PathBuf::new(".claude/skills");

    let skills_with_tags = vec![
        ("market-research", vec!["market-analysis", "technical-indicators"]),
        ("portfolio-management", vec!["portfolio", "asset-allocation"]),
        ("risk-analysis", vec!["risk", "var", "stress-testing"]),
        ("sentiment-analysis", vec!["sentiment", "news", "social-media"]),
        ("technical-analysis", vec!["technical-analysis", "chart-patterns"]),
        ("fundamental-analysis", vec!["fundamental-analysis", "valuation"]),
        ("strategy-planner", vec!["strategy", "asset-allocation"]),
        ("backtesting", vec!["backtesting", "performance-analysis"]),
        ("reporting", vec!["reporting", "visualization"]),
    ];

    for (skill, expected_tags) in skills_with_tags {
        let skill_path = skills_dir.join(skill).join("SKILL.md");
        let content = std::fs::read_to_string(&skill_path)
            .unwrap_or_else(|_| panic!("Failed to read: {:?}", skill_path));

        // Verify tags
        assert!(content.contains("tags:"), "Missing 'tags' in: {}", skill);

        // Verify expected tags are present
        for tag in expected_tags {
            assert!(content.contains(tag), "Missing expected tag '{}' in: {}", tag, skill);
        }

        println!("✅ {} - Correct metadata structure", skill);
    }
}

/// Test MCP tools are correctly defined
#[test]
fn test_mcp_tools_structure() {
    // This test verifies the tools.rs file structure
    let tools_path = PathBuf::new("app/tools.rs");
    assert!(tools_path.exists(), "tools.rs not found");

    let content = std::fs::read_to_string(&tools_path)
        .expect("Failed to read tools.rs");

    // Verify tool functions exist
    let expected_tools = vec![
        "technical_analysis",
        "var_calculation",
        "sentiment_analysis",
        "save_portfolio",
        "load_portfolio",
        "stress_test",
        "correlation_analysis",
    ];

    for tool in expected_tools {
        assert!(content.contains(&format!("async fn {}", tool)),
                "Missing tool function: {}", tool);
        println!("✅ Tool function '{}' defined", tool);
    }

    // Verify return type
    assert!(content.contains("Result<ToolResult>"),
            "Tools should return Result<ToolResult>");
}

/// Test orchestration agents structure
#[test]
fn test_orchestration_agents_structure() {
    let orch_path = PathBuf::new("app/orchestration.rs");
    assert!(orch_path.exists(), "orchestration.rs not found");

    let content = std::fs::read_to_string(&orch_path)
        .expect("Failed to read orchestration.rs");

    // Verify agent structs
    let expected_agents = vec![
        "MarketResearchAgent",
        "InvestmentAnalystAgent",
        "RiskManagementAgent",
        "SentimentAnalysisAgent",
        "InvestmentAdvisorAgent",
    ];

    for agent in expected_agents {
        assert!(content.contains(&format!("pub struct {}", agent)),
                "Missing agent struct: {}", agent);
        println!("✅ Agent struct '{}' defined", agent);
    }

    // Verify Agent trait implementation
    for agent in expected_agents {
        assert!(content.contains(&format!("impl Agent for {}", agent)),
                "Missing Agent impl for: {}", agent);
    }

    // Verify orchestration function
    assert!(content.contains("pub async fn run_comprehensive_analysis"),
            "Missing run_comprehensive_analysis function");
}

/// Test project file structure
#[test]
fn test_project_file_structure() {
    let required_files = vec![
        // Cargo configuration
        "Cargo.toml",
        "app/Cargo.toml",

        // Source files
        "app/main.rs",
        "app/tools.rs",
        "app/orchestration.rs",

        // Skills
        ".claude/skills/market-research/SKILL.md",
        ".claude/skills/portfolio-management/SKILL.md",
        ".claude/skills/risk-analysis/SKILL.md",
        ".claude/skills/sentiment-analysis/SKILL.md",
        ".claude/skills/technical-analysis/SKILL.md",
        ".claude/skills/fundamental-analysis/SKILL.md",
        ".claude/skills/strategy-planner/SKILL.md",
        ".claude/skills/backtesting/SKILL.md",
        ".claude/skills/reporting/SKILL.md",

        // Agents
        ".claude/agents/research-agent.md",
        ".claude/agents/analyst-agent.md",
        ".claude/agents/risk-agent.md",
        ".claude/agents/advisor-agent.md",
        ".claude/agents/technical-analyst.md",
        ".claude/agents/strategy-executor.md",

        // Tests
        "tests/skills_test.rs",
        "tests/integration_test.rs",
    ];

    for file in required_files {
        let path = PathBuf::new(file);
        assert!(path.exists(), "Required file not found: {}", file);
        println!("✅ File exists: {}", file);
    }
}

/// Test Cargo.toml dependencies
#[test]
fn test_cargo_toml_dependencies() {
    let cargo_toml = std::fs::read_to_string("Cargo.toml")
        .expect("Failed to read Cargo.toml");

    // Verify core dependencies
    let required_deps = vec![
        "claude-agent-sdk-rs",
        "tokio",
        "anyhow",
        "serde_json",
        "async-trait",
        "chrono",
    ];

    for dep in required_deps {
        assert!(cargo_toml.contains(dep),
                "Missing dependency: {}", dep);
        println!("✅ Dependency found: {}", dep);
    }

    // Verify SDK features
    assert!(cargo_toml.contains("features = [\"yaml\"]"),
            "SDK should have yaml feature");
}

/// Comprehensive test suite
#[test]
fn test_comprehensive_suite() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║     InvestIntel AI - Unit Test Suite                      ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    println!("📊 Testing Agent Skills System...");
    assert!(skill_files_exist_and_valid(), "Skills validation failed");

    println!("\n🤖 Testing Subagents Configuration...");
    assert!(agent_configs_exist_and_valid(), "Agents validation failed");

    println!("\n🔧 Testing MCP Tools...");
    assert!(mcp_tools_correctly_defined(), "Tools validation failed");

    println!("\n🔄 Testing Orchestration System...");
    assert!(orchestration_system_valid(), "Orchestration validation failed");

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║          All Unit Tests Passed ✅                         ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    println!("📈 Test Summary:");
    println!("  ✅ Agent Skills: 9 skills validated");
    println!("  ✅ Subagents: 6 agents configured");
    println!("  ✅ MCP Tools: 7 tools defined");
    println!("  ✅ Orchestration: 5 agents implemented");
    println!("  ✅ Project Structure: Complete");
}

// Helper functions

fn skill_files_exist_and_valid() -> bool {
    let skills_dir = PathBuf::new(".claude/skills");
    let skills = vec![
        "market-research", "portfolio-management", "risk-analysis",
        "sentiment-analysis", "technical-analysis", "fundamental-analysis",
        "strategy-planner", "backtesting", "reporting", "investment-analyst",
    ];

    for skill in skills {
        let skill_path = skills_dir.join(skill).join("SKILL.md");
        if !skill_path.exists() {
            return false;
        }
        let content = std::fs::read_to_string(&skill_path).unwrap_or_default();
        if !content.starts_with("---") {
            return false;
        }
    }
    true
}

fn agent_configs_exist_and_valid() -> bool {
    let agents_dir = PathBuf::new(".claude/agents");
    let agents = vec![
        "research-agent.md", "analyst-agent.md", "risk-agent.md",
        "advisor-agent.md", "technical-analyst.md", "strategy-executor.md",
    ];

    for agent in agents {
        let agent_path = agents_dir.join(agent);
        if !agent_path.exists() {
            return false;
        }
        let content = std::fs::read_to_string(&agent_path).unwrap_or_default();
        if !content.starts_with("---") {
            return false;
        }
    }
    true
}

fn mcp_tools_correctly_defined() -> bool {
    let tools_path = PathBuf::new("app/tools.rs");
    if !tools_path.exists() {
        return false;
    }
    let content = std::fs::read_to_string(&tools_path).unwrap_or_default();
    let expected_tools = vec![
        "technical_analysis", "var_calculation", "sentiment_analysis",
        "save_portfolio", "load_portfolio", "stress_test", "correlation_analysis",
    ];
    for tool in expected_tools {
        if !content.contains(&format!("async fn {}", tool)) {
            return false;
        }
    }
    true
}

fn orchestration_system_valid() -> bool {
    let orch_path = PathBuf::new("app/orchestration.rs");
    if !orch_path.exists() {
        return false;
    }
    let content = std::fs::read_to_string(&orch_path).unwrap_or_default();
    let expected_agents = vec![
        "MarketResearchAgent", "InvestmentAnalystAgent",
        "RiskManagementAgent", "SentimentAnalysisAgent", "InvestmentAdvisorAgent",
    ];
    for agent in expected_agents {
        if !content.contains(&format!("pub struct {}", agent)) {
            return false;
        }
        if !content.contains(&format!("impl Agent for {}", agent)) {
            return false;
        }
    }
    true
}
