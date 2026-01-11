//! Tests for Partnership Types Module
//!
//! Validates core type definitions and Partner creation

use super::types::*;
use chrono::Utc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partner_creation() {
        let partner = Partner::new("Alice", 100_000.0, 0.8);
        
        assert_eq!(partner.name, "Alice");
        assert_eq!(partner.capital_contribution, 100_000.0);
        assert_eq!(partner.profit_share, 0.8);
        assert!(partner.voting_rights);
        assert_eq!(partner.risk_profile, RiskProfile::Moderate);
    }

    #[test]
    fn test_partner_with_custom_risk_profile() {
        let partner = Partner::new("Bob", 150_000.0, 0.75)
            .with_risk_profile(RiskProfile::Aggressive);
        
        assert_eq!(partner.risk_profile, RiskProfile::Aggressive);
    }

    #[test]
    fn test_investment_decision() {
        let team_inputs = TeamInputs {
            research: None,
            analysis: None,
            trading: None,
            risk: None,
        };

        let decision = InvestmentDecision {
            symbol: "AAPL".to_string(),
            action: InvestmentAction::Buy,
            confidence: 0.85,
            position_size: 0.15,
            reasoning: "Strong fundamentals".to_string(),
            expected_return: Some(0.20),
            time_horizon: chrono::Duration::days(365 * 3),
            team_inputs,
            risk_considerations: vec!["Market volatility".to_string()],
            timestamp: Utc::now(),
        };
        
        assert_eq!(decision.symbol, "AAPL");
        assert_eq!(decision.action, InvestmentAction::Buy);
        assert_eq!(decision.confidence, 0.85);
        assert_eq!(decision.position_size, 0.15);
    }

    #[test]
    fn test_investment_action_position_range() {
        let (min, max) = InvestmentAction::HeavyBuy.position_range();
        assert_eq!(min, 0.30);
        assert_eq!(max, 0.50);

        let (min, max) = InvestmentAction::Buy.position_range();
        assert_eq!(min, 0.15);
        assert_eq!(max, 0.25);

        let (min, max) = InvestmentAction::SmallBuy.position_range();
        assert_eq!(min, 0.05);
        assert_eq!(max, 0.10);

        let (min, max) = InvestmentAction::Hold.position_range();
        assert_eq!(min, 0.0);
        assert_eq!(max, 0.0);
    }

    #[test]
    fn test_risk_profile_default() {
        let partner = Partner::new("Charlie", 200_000.0, 0.7);
        assert_eq!(partner.risk_profile, RiskProfile::Moderate);
    }

    #[test]
    fn test_investment_strategy_description() {
        assert!(InvestmentStrategy::ValueInvesting.description().contains("Graham-Buffett-Munger"));
        assert!(InvestmentStrategy::DeepValue.description().contains("margin of safety"));
        assert!(InvestmentStrategy::QualityValue.description().contains("wide moat"));
        assert!(InvestmentStrategy::GARP.description().contains("reasonable price"));
    }
}
