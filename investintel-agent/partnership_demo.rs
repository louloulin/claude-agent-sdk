//! AI Investment Partnership Demo
//!
//! Demonstrates the Plan6 Partnership functionality:
//! - Creates an investment partnership based on Buffett's model
//! - Uses AI team for investment analysis
//! - Shows profit distribution and partnership status

use anyhow::Result;
use partnership::{
    PartnershipBuilder, Partner, InvestmentStrategy, ProfitDistribution,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     AI Investment Partnership - Plan6 Demo                 ║");
    println!("║     Buffett Partnership Model (1956-1969) AI-ified          ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // ========================================================================
    // Demo 1: Create Partnership
    // ========================================================================
    println!("📊 Demo 1: Creating AI Investment Partnership");
    println!("{}\n", "─".repeat(60));

    let partners = vec![
        Partner::new("Alice", 100_000.0, 0.8)
            .with_risk_profile(partnership::RiskProfile::Moderate),
        Partner::new("Bob", 150_000.0, 0.8),
        Partner::new("Charlie", 200_000.0, 0.75),
    ];

    let partnership = PartnershipBuilder::new()
        .name("AI Value Investment Partnership".to_string())
        .partners(partners)
        .strategy(InvestmentStrategy::ValueInvesting)
        .build()
        .await?;

    println!("✅ Partnership created successfully!");
    let status = partnership.status();
    println!("   Name: {}", status.name);
    println!("   Partners: {}", status.partner_count);
    println!("   Total Capital: ${:,.2}", status.total_capital);
    println!("   Strategy: {:?}", status.strategy);

    // ========================================================================
    // Demo 2: Buffett Profit Distribution
    // ========================================================================
    println!("\n💰 Demo 2: Buffett-Style Profit Distribution");
    println!("{}\n", "─".repeat(60));

    let agreement = &partnership.agreement;
    println!("   Hurdle Rate: {:.1}%", agreement.hurdle_rate * 100.0);
    println!("   AI Profit Share: {:.1}%", agreement.ai_profit_share * 100.0);

    // Example: 20% return
    let initial = 100_000.0;
    let final_val = 120_000.0;
    let distribution = agreement.calculate_profit_distribution(initial, final_val);

    println!("\n   Example: 20% Return ($100K → $120K)");
    println!("   ┌─────────────────────────────────┐");
    println!("   │ Total Return:        ${:>10.2} │", distribution.total_return);
    println!("   │ Hurdle (6%):         ${:>10.2} │", distribution.hurdle_return);
    println!("   │ Excess:              ${:>10.2} │", distribution.excess_return);
    println!("   ├─────────────────────────────────┤");
    println!("   │ Partner Share:       ${:>10.2} │", distribution.partner_share);
    println!("   │ AI Share:            ${:>10.2} │", distribution.ai_share);
    println!("   ├─────────────────────────────────┤");
    println!("   │ Partner Return:      {:>10.1}% │", distribution.partner_return_pct * 100.0);
    println!("   │ AI Return:           {:>10.1}% │", distribution.ai_return_pct * 100.0);
    println!("   └─────────────────────────────────┘");

    // ========================================================================
    // Demo 3: Concentration Limits (Munger-Style)
    // ========================================================================
    println!("\n🎯 Demo 3: Munger-Style Concentration Limits");
    println!("{}\n", "─".repeat(60));

    let limits = &agreement.concentration_limits;
    println!("   Max Single Position: {:.1}%", limits.max_single_position * 100.0);
    println!("   Max Top 5: {:.1}%", limits.max_top_5_concentration * 100.0);
    println!("   Max Single Industry: {:.1}%", limits.max_industry_concentration * 100.0);
    println!("   Min Positions: {}", limits.min_positions);

    println!("\n   Munger Philosophy:");
    println!("   \"If it's a once-in-a-lifetime opportunity,");
    println!("    put 40% in it. Otherwise, keep diversifying.\"");

    // ========================================================================
    // Demo 4: AI Team Structure
    // ========================================================================
    println!("\n🤖 Demo 4: AI Team Structure");
    println!("{}\n", "─".repeat(60));

    println!("   Chief Investment Agent (AI Buffett)");
    println!("   ├─ Research Team (Parallel)");
    println!("   │  ├─ Fundamental Researcher");
    println!("   │  ├─ Technical Analyst");
    println!("   │  ├─ Sentiment Analyst");
    println!("   │  └─ Macro Analyst");
    println!("   ├─ Analysis Team (Hierarchical)");
    println!("   │  ├─ Valuation Analyst (Planner)");
    println!("   │  ├─ Quality Analyst");
    println!("   │  ├─ Risk Analyst");
    println!("   │  └─ Moat Analyst");
    println!("   ├─ Trading Team (Sequential)");
    println!("   │  ├─ Execution Agent");
    println!("   │  ├─ Position Sizer");
    println!("   │  └─ Order Router");
    println!("   └─ Risk Team (Parallel)");
    println!("      ├─ Portfolio Monitor");
    println!("      ├─ Risk Manager");
    println!("      └─ Compliance Agent");

    // ========================================================================
    // Demo 5: Investment Analysis
    // ========================================================================
    println!("\n📈 Demo 5: AI Investment Analysis");
    println!("{}\n", "─".repeat(60));

    println!("   Analyzing AAPL...");
    let decision = partnership.analyze("AAPL").await?;

    println!("\n   Investment Decision:");
    println!("   ┌─────────────────────────────────┐");
    println!("   │ Symbol:              {:>15} │", decision.symbol);
    println!("   │ Action:              {:>15} │", format!("{:?}", decision.action));
    println!("   │ Confidence:          {:>14.1}% │", decision.confidence * 100.0);
    println!("   │ Position Size:       {:>14.1}% │", decision.position_size * 100.0);
    println!("   │ Time Horizon:        {:>15} │", format!("{} years", decision.time_horizon.num_days() / 365));
    println!("   ├─────────────────────────────────┤");
    println!("   │ Reasoning:                    │");
    for line in decision.reasoning.lines() {
        println!("   │ {}{}", line, " ".repeat(30 - line.len().min(30)));
    }
    println!("   └─────────────────────────────────┘");

    // ========================================================================
    // Demo 6: Team Inputs
    // ========================================================================
    println!("\n🔍 Demo 6: Team Analysis Details");
    println!("{}\n", "─".repeat(60));

    if let Some(research) = &decision.team_inputs.research {
        println!("   Research Team:");
        if let Some(fundamental) = research.get("fundamental") {
            println!("     Fundamental Score: {}/100",
                fundamental.get("fundamental_score").and_then(|v| v.as_i64()).unwrap_or(0));
        }
        if let Some(technical) = research.get("technical") {
            println!("     Technical Score: {}/100",
                technical.get("technical_score").and_then(|v| v.as_i64()).unwrap_or(0));
        }
    }

    if let Some(analysis) = &decision.team_inputs.analysis {
        println!("\n   Analysis Team:");
        if let Some(quality) = analysis.get("quality") {
            println!("     Quality Score: {}/100",
                quality.get("quality_score").and_then(|v| v.as_i64()).unwrap_or(0));
        }
        if let Some(moat) = analysis.get("moat") {
            println!("     Moat: {:?}",
                moat.get("moat_score").and_then(|v| v.as_str()).unwrap_or("unknown"));
        }
    }

    // ========================================================================
    // Summary
    // ========================================================================
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║     Demo Complete!                                        ║");
    println!("║                                                            ║");
    println!("║  Key Features Demonstrated:                               ║");
    println!("║  ✓ Buffett Partnership Model                              ║");
    println!("║  ✓ Munger Concentration Limits                            ║");
    println!("║  ✓ AI Team Coordination                                   ║");
    println!("║  ✓ Investment Analysis                                    ║");
    println!("║  ✓ Profit Distribution                                    ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
