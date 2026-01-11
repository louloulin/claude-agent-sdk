//! Core types for AI Investment Partnership
//!
//! Defines all the fundamental data structures for the partnership model.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Partner in the investment partnership
///
/// Represents a human partner who contributes capital and shares in profits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partner {
    /// Unique partner ID
    pub id: Uuid,

    /// Partner name
    pub name: String,

    /// Capital contribution amount
    pub capital_contribution: f64,

    /// Profit share ratio (0.0 - 1.0) of excess returns above hurdle rate
    /// E.g., 0.8 means partner keeps 80% of excess, AI gets 20%
    pub profit_share: f64,

    /// Voting rights (true if can vote on major decisions)
    pub voting_rights: bool,

    /// Risk profile
    pub risk_profile: RiskProfile,

    /// Investment goals
    pub investment_goals: InvestmentGoals,

    /// Date partner joined
    pub joined_at: DateTime<Utc>,
}

impl Partner {
    /// Create a new partner
    ///
    /// # Arguments
    ///
    /// * `name` - Partner name
    /// * `capital_contribution` - Amount invested
    /// * `profit_share` - Share of excess profits (typically 0.75-0.85 for partners)
    pub fn new(name: impl Into<String>, capital_contribution: f64, profit_share: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            capital_contribution,
            profit_share,
            voting_rights: true,
            risk_profile: RiskProfile::Moderate,
            investment_goals: InvestmentGoals::default(),
            joined_at: Utc::now(),
        }
    }

    /// Create partner with custom risk profile
    pub fn with_risk_profile(mut self, risk_profile: RiskProfile) -> Self {
        self.risk_profile = risk_profile;
        self
    }

    /// Create partner with custom investment goals
    pub fn with_investment_goals(mut self, goals: InvestmentGoals) -> Self {
        self.investment_goals = goals;
        self
    }
}

/// Risk tolerance level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskProfile {
    Conservative,
    Moderate,
    Aggressive,
}

/// Investment goals and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentGoals {
    /// Target annual return (e.g., 0.15 for 15%)
    pub target_return: f64,

    /// Investment time horizon
    pub time_horizon: Duration,

    /// Liquidity needs
    pub liquidity_needs: LiquidityNeeds,

    /// ESG preference
    pub esg_preference: ESGLPreference,
}

impl Default for InvestmentGoals {
    fn default() -> Self {
        Self {
            target_return: 0.15,
            time_horizon: Duration::days(365 * 5), // 5 years
            liquidity_needs: LiquidityNeeds::Low,
            esg_preference: ESGLPreference::Neutral,
        }
    }
}

/// Liquidity needs
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LiquidityNeeds {
    Low,
    Medium,
    High,
}

/// ESG (Environmental, Social, Governance) preference
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ESGLPreference {
    /// Prefer high ESG scores
    Positive,

    /// Neutral on ESG
    Neutral,

    /// Willing to invest in lower ESG scores if returns are good
    Negative,
}

/// Investment strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvestmentStrategy {
    /// Graham-style deep value investing
    DeepValue,

    /// Buffett-style quality value investing
    QualityValue,

    /// Munger-style concentrated value investing
    ConcentratedValue,

    /// Combination of all three (Graham-Buffett-Munger)
    ValueInvesting,

    /// Growth at reasonable price
    GARP,

    /// Diversified index-like approach
    Diversified,
}

impl InvestmentStrategy {
    /// Get strategy description
    pub fn description(&self) -> &str {
        match self {
            Self::DeepValue => "Deep value investing with 30-40% margin of safety",
            Self::QualityValue => "Quality value investing with wide moat and high ROIC",
            Self::ConcentratedValue => "Concentrated positions in best opportunities",
            Self::ValueInvesting => "Comprehensive value investing (Graham-Buffett-Munger)",
            Self::GARP => "Growth at reasonable price",
            Self::Diversified => "Diversified portfolio approach",
        }
    }
}

/// Investment decision action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvestmentAction {
    HeavyBuy,   // 30-50% position
    Buy,        // 15-25% position
    SmallBuy,   // 5-10% position
    Hold,
    Reduce,
    Sell,
}

impl InvestmentAction {
    /// Get position size range
    pub fn position_range(&self) -> (f64, f64) {
        match self {
            Self::HeavyBuy => (0.30, 0.50),
            Self::Buy => (0.15, 0.25),
            Self::SmallBuy => (0.05, 0.10),
            Self::Hold => (0.0, 0.0),
            Self::Reduce => (-0.10, -0.05),
            Self::Sell => (-0.25, -0.10),
        }
    }
}

/// Comprehensive investment decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentDecision {
    /// Stock symbol
    pub symbol: String,

    /// Investment action
    pub action: InvestmentAction,

    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,

    /// Recommended position size (0.0 - 1.0)
    pub position_size: f64,

    /// Reasoning
    pub reasoning: String,

    /// Expected annual return
    pub expected_return: Option<f64>,

    /// Time horizon
    pub time_horizon: Duration,

    /// Team inputs
    pub team_inputs: TeamInputs,

    /// Risk considerations
    pub risk_considerations: Vec<String>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Inputs from different agent teams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInputs {
    /// Research team findings
    pub research: Option<serde_json::Value>,

    /// Analysis team findings
    pub analysis: Option<serde_json::Value>,

    /// Trading team inputs
    pub trading: Option<serde_json::Value>,

    /// Risk team inputs
    pub risk: Option<serde_json::Value>,
}

/// Investment opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentOpportunity {
    /// Stock symbol
    pub symbol: String,

    /// Current price
    pub current_price: f64,

    /// Intrinsic value estimate
    pub intrinsic_value: f64,

    /// Margin of safety (0.0 - 1.0)
    pub margin_of_safety: f64,

    /// Expected return
    pub expected_return: f64,

    /// Risk (variance)
    pub variance: f64,

    /// Risk-free rate
    pub risk_free_rate: f64,

    /// Lollapalooza score (0.0 - 1.0)
    pub lollapalooza_score: f64,

    /// Is in circle of competence
    pub in_circle_of_competence: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partner_creation() {
        let partner = Partner::new("Alice", 100_000.0, 0.8);
        assert_eq!(partner.name, "Alice");
        assert_eq!(partner.capital_contribution, 100_000.0);
        assert_eq!(partner.profit_share, 0.8);
    }

    #[test]
    fn test_investment_action_ranges() {
        assert_eq!(InvestmentAction::HeavyBuy.position_range(), (0.30, 0.50));
        assert_eq!(InvestmentAction::Buy.position_range(), (0.15, 0.25));
        assert_eq!(InvestmentAction::SmallBuy.position_range(), (0.05, 0.10));
    }
}
