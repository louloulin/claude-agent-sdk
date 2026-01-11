//! Tests for Partnership Agreement Module
//!
//! Validates Buffett-style profit distribution and agreement terms

use super::agreement::*;
use super::types::InvestmentStrategy;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffett_agreement_default() {
        let agreement = PartnershipAgreement::default();
        
        assert_eq!(agreement.hurdle_rate, 0.06);
        assert_eq!(agreement.ai_profit_share, 0.20);
        assert_eq!(agreement.lockup_period, chrono::Duration::days(365));
        assert_eq!(agreement.minimum_investment, 100_000.0);
    }

    #[test]
    fn test_profit_distribution_with_profit() {
        let agreement = PartnershipAgreement::default();
        let initial_capital = 1_000_000.0;
        let final_capital = 1_200_000.0; // 20% gain
        
        let distribution = agreement.calculate_profit_distribution(initial_capital, final_capital);
        
        // Total return: 200,000
        // Hurdle (6%): 60,000 goes to partners
        // Excess: 140,000, AI takes 20% = 28,000
        // Partners get: 60,000 + 112,000 = 172,000
        
        assert_eq!(distribution.initial_capital, 1_000_000.0);
        assert_eq!(distribution.final_capital, 1_200_000.0);
        assert_eq!(distribution.total_return, 200_000.0);
        assert_eq!(distribution.total_return_pct, 0.20);
        assert_eq!(distribution.hurdle_return, 60_000.0);
        assert_eq!(distribution.excess_return, 140_000.0);
        assert_eq!(distribution.ai_share, 28_000.0);
        assert_eq!(distribution.partner_share, 172_000.0);
    }

    #[test]
    fn test_profit_distribution_below_hurdle() {
        let agreement = PartnershipAgreement::default();
        let initial_capital = 1_000_000.0;
        let final_capital = 1_040_000.0; // 4% gain (below 6% hurdle)
        
        let distribution = agreement.calculate_profit_distribution(initial_capital, final_capital);
        
        // Total return: 40,000 (below hurdle)
        // All goes to partners, AI gets nothing
        
        assert_eq!(distribution.total_return, 40_000.0);
        assert_eq!(distribution.hurdle_return, 60_000.0);
        assert_eq!(distribution.excess_return, 0.0); // No excess
        assert_eq!(distribution.ai_share, 0.0); // AI gets nothing
        assert_eq!(distribution.partner_share, 40_000.0);
    }

    #[test]
    fn test_profit_distribution_with_loss() {
        let agreement = PartnershipAgreement::default();
        let initial_capital = 1_000_000.0;
        let final_capital = 900_000.0; // 10% loss
        
        let distribution = agreement.calculate_profit_distribution(initial_capital, final_capital);
        
        assert_eq!(distribution.total_return, -100_000.0);
        assert_eq!(distribution.hurdle_return, 60_000.0);
        assert_eq!(distribution.excess_return, 0.0);
        assert_eq!(distribution.ai_share, 0.0);
        // Loss case - partners get 0 (they share the loss)
        assert_eq!(distribution.partner_share, 0.0);
        assert!((distribution.total_return_pct - -0.10).abs() < 0.001);
    }

    #[test]
    fn test_profit_distribution_break_even() {
        let agreement = PartnershipAgreement::default();
        let initial_capital = 1_000_000.0;
        let final_capital = 1_000_000.0;
        
        let distribution = agreement.calculate_profit_distribution(initial_capital, final_capital);
        
        assert_eq!(distribution.total_return, 0.0);
        assert_eq!(distribution.hurdle_return, 60_000.0);
        assert_eq!(distribution.excess_return, 0.0);
        assert_eq!(distribution.ai_share, 0.0);
        // Break even - no profit, partners get 0
        assert_eq!(distribution.partner_share, 0.0);
    }

    #[test]
    fn test_concentration_limits_default() {
        let limits = ConcentrationLimits::default();
        
        assert_eq!(limits.max_single_position, 0.40);
        assert_eq!(limits.max_top_5_concentration, 0.70);
        assert_eq!(limits.min_positions, 3);
    }

    #[test]
    fn test_agreement_for_strategy() {
        let value_agreement = PartnershipAgreement::for_strategy(InvestmentStrategy::ValueInvesting);
        let deep_value_agreement = PartnershipAgreement::for_strategy(InvestmentStrategy::DeepValue);
        
        // Both should have 6% hurdle
        assert_eq!(value_agreement.hurdle_rate, 0.06);
        assert_eq!(deep_value_agreement.hurdle_rate, 0.06);
        
        // Deep value should have same or different concentration limits depending on strategy
        // ValueInvesting combines all three approaches
        assert_eq!(deep_value_agreement.concentration_limits.min_positions, 3);
    }

    #[test]
    fn test_fee_structure_default() {
        let agreement = PartnershipAgreement::default();
        
        assert_eq!(agreement.fee_structure.management_fee, 0.0);
    }
}
