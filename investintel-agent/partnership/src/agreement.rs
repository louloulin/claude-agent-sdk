//! Partnership Agreement - Buffett Style
//!
//! Implements Warren Buffett's partnership agreement structure (1956-1969):
//! - 6% hurdle rate (partners get 100% of first 6% returns)
//! - 20-25% profit sharing on excess returns (AI gets this)
//! - Concentration limits (Munger-style: 40% max single position)
//! - Lockup periods and redemption policies

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::InvestmentStrategy;

/// Partnership agreement based on Buffett's model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnershipAgreement {
    /// Hurdle rate (Buffett: 6%) - partners get 100% of returns up to this
    pub hurdle_rate: f64,

    /// AI profit share on excess returns (Buffett: 20-25%)
    pub ai_profit_share: f64,

    /// Minimum investment amount
    pub minimum_investment: f64,

    /// Lockup period (partners cannot withdraw during this time)
    pub lockup_period: Duration,

    /// Investment strategy
    pub investment_strategy: InvestmentStrategy,

    /// Concentration limits
    pub concentration_limits: ConcentrationLimits,

    /// Redemption policy
    pub redemption_policy: RedemptionPolicy,

    /// Established date
    pub established_date: NaiveDate,

    /// Fee structure
    pub fee_structure: FeeStructure,
}

impl Default for PartnershipAgreement {
    fn default() -> Self {
        Self {
            hurdle_rate: 0.06,      // 6% - Buffett's hurdle rate
            ai_profit_share: 0.20,  // 20% - Buffett's take on excess
            minimum_investment: 100_000.0,
            lockup_period: Duration::days(365), // 1 year lockup
            investment_strategy: InvestmentStrategy::ValueInvesting,
            concentration_limits: ConcentrationLimits::default(),
            redemption_policy: RedemptionPolicy::default(),
            established_date: chrono::Utc::now().date_naive(),
            fee_structure: FeeStructure::default(),
        }
    }
}

impl PartnershipAgreement {
    /// Calculate profit distribution
    ///
    /// # Example
    ///
    /// ```text
    /// Initial capital: $100,000
    /// Final capital: $120,000
    /// Total return: $20,000 (20%)
    ///
    /// Calculation:
    /// 1. Hurdle return: $100,000 × 6% = $6,000 (100% to partners)
    /// 2. Excess return: $20,000 - $6,000 = $14,000
    /// 3. AI share: $14,000 × 20% = $2,800
    /// 4. Partner share: $14,000 × 80% = $11,200
    ///
    /// Final distribution:
    /// - Partners: $6,000 + $11,200 = $17,200 (17.2%)
    /// - AI: $2,800 (2.8%)
    /// ```
    pub fn calculate_profit_distribution(
        &self,
        initial_capital: f64,
        final_capital: f64,
    ) -> ProfitDistribution {
        let total_return = final_capital - initial_capital;
        let total_return_pct = total_return / initial_capital;

        // Calculate hurdle return
        let hurdle_return = initial_capital * self.hurdle_rate;

        // Calculate excess return
        let excess_return = if total_return > hurdle_return {
            total_return - hurdle_return
        } else {
            0.0
        };

        // Distribute excess
        let ai_share = excess_return * self.ai_profit_share;
        let partner_excess_share = excess_return * (1.0 - self.ai_profit_share);

        // Partner gets hurdle (if positive) + share of excess
        // If total return is below hurdle, partners get all of it
        let partner_total = if total_return > 0.0 && total_return < hurdle_return {
            total_return  // Partners keep all returns below hurdle
        } else if total_return >= hurdle_return {
            hurdle_return + partner_excess_share  // Buffett model
        } else {
            0.0  // Loss - partners get nothing (they share in the loss)
        };

        ProfitDistribution {
            initial_capital,
            final_capital,
            total_return,
            total_return_pct,
            hurdle_return,
            excess_return,
            partner_share: partner_total,
            ai_share,
            partner_return_pct: partner_total / initial_capital,
            ai_return_pct: ai_share / initial_capital,
        }
    }

    /// Create agreement for specific strategy
    pub fn for_strategy(strategy: InvestmentStrategy) -> Self {
        let (hurdle, ai_share, limits) = match strategy {
            InvestmentStrategy::DeepValue => (
                0.06,
                0.20,
                ConcentrationLimits {
                    max_single_position: 0.40,
                    max_top_5_concentration: 0.75,
                    max_industry_concentration: 0.60,
                    min_positions: 3,
                },
            ),
            InvestmentStrategy::QualityValue => (
                0.06,
                0.25,
                ConcentrationLimits {
                    max_single_position: 0.30,
                    max_top_5_concentration: 0.70,
                    max_industry_concentration: 0.50,
                    min_positions: 5,
                },
            ),
            InvestmentStrategy::ConcentratedValue => (
                0.06,
                0.20,
                ConcentrationLimits {
                    max_single_position: 0.50,
                    max_top_5_concentration: 0.90,
                    max_industry_concentration: 0.70,
                    min_positions: 2,
                },
            ),
            InvestmentStrategy::ValueInvesting => (
                0.06,
                0.22,
                ConcentrationLimits::default(),
            ),
            _ => (
                0.06,
                0.20,
                ConcentrationLimits {
                    max_single_position: 0.20,
                    max_top_5_concentration: 0.60,
                    max_industry_concentration: 0.40,
                    min_positions: 10,
                },
            ),
        };

        Self {
            hurdle_rate: hurdle,
            ai_profit_share: ai_share,
            concentration_limits: limits,
            investment_strategy: strategy,
            ..Default::default()
        }
    }
}

/// Concentration limits (Munger-style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentrationLimits {
    /// Maximum single position (Munger: 40%)
    pub max_single_position: f64,

    /// Maximum for top 5 positions
    pub max_top_5_concentration: f64,

    /// Maximum for single industry
    pub max_industry_concentration: f64,

    /// Minimum number of positions
    pub min_positions: usize,
}

impl Default for ConcentrationLimits {
    fn default() -> Self {
        Self {
            max_single_position: 0.40,  // Munger's typical max
            max_top_5_concentration: 0.70,
            max_industry_concentration: 0.50,
            min_positions: 3,
        }
    }
}

/// Redemption policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedemptionPolicy {
    /// Notice period required
    pub notice_period_days: u64,

    /// Redemption frequency (quarterly, annually, etc.)
    pub redemption_frequency: RedemptionFrequency,

    /// Gates during stress periods
    pub gates_enabled: bool,

    /// Maximum redemption per period (% of AUM)
    pub max_redemption_pct: f64,
}

impl Default for RedemptionPolicy {
    fn default() -> Self {
        Self {
            notice_period_days: 30,
            redemption_frequency: RedemptionFrequency::Quarterly,
            gates_enabled: true,
            max_redemption_pct: 0.10, // 10% per quarter
        }
    }
}

/// Redemption frequency
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RedemptionFrequency {
    Monthly,
    Quarterly,
    SemiAnnually,
    Annually,
}

/// Fee structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStructure {
    /// Management fee (if any)
    pub management_fee: f64,

    /// Performance fee (on top of profit share)
    pub performance_fee: f64,

    /// Other fees
    pub other_fees: HashMap<String, f64>,
}

impl Default for FeeStructure {
    fn default() -> Self {
        let mut other_fees = HashMap::new();
        other_fees.insert("admin".to_string(), 0.001); // 0.1% admin fee

        Self {
            management_fee: 0.0,    // No management fee in Buffett model
            performance_fee: 0.0,   // Only profit sharing
            other_fees,
        }
    }
}

/// Profit distribution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitDistribution {
    pub initial_capital: f64,
    pub final_capital: f64,
    pub total_return: f64,
    pub total_return_pct: f64,
    pub hurdle_return: f64,
    pub excess_return: f64,
    pub partner_share: f64,
    pub ai_share: f64,
    pub partner_return_pct: f64,
    pub ai_return_pct: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffett_profit_distribution() {
        let agreement = PartnershipAgreement::default();

        // Test: 20% return
        let dist = agreement.calculate_profit_distribution(100_000.0, 120_000.0);

        assert_eq!(dist.initial_capital, 100_000.0);
        assert_eq!(dist.final_capital, 120_000.0);
        assert_eq!(dist.total_return, 20_000.0);
        assert_eq!(dist.total_return_pct, 0.20);

        // Hurdle: 6% of 100k = 6k
        assert_eq!(dist.hurdle_return, 6_000.0);

        // Excess: 20k - 6k = 14k
        assert_eq!(dist.excess_return, 14_000.0);

        // AI: 20% of 14k = 2.8k
        assert_eq!(dist.ai_share, 2_800.0);

        // Partner: 6k + 80% of 14k = 17.2k
        assert_eq!(dist.partner_share, 17_200.0);
    }

    #[test]
    fn test_below_hurdle() {
        let agreement = PartnershipAgreement::default();

        // Test: 4% return (below hurdle)
        let dist = agreement.calculate_profit_distribution(100_000.0, 104_000.0);

        // AI gets nothing if below hurdle
        assert_eq!(dist.ai_share, 0.0);

        // Partner keeps all
        assert_eq!(dist.partner_share, 4_000.0);
    }

    #[test]
    fn test_concentration_limits() {
        let limits = ConcentrationLimits::default();
        assert_eq!(limits.max_single_position, 0.40);
        assert_eq!(limits.min_positions, 3);
    }
}
