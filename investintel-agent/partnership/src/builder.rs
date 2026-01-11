//! Partnership Builder
//!
//! Builder pattern for creating investment partnerships

use anyhow::Result;
use chrono::Utc;
use serde_json::json;

use super::types::*;
use super::agreement::*;
use super::portfolio::*;
use super::ai_team::*;

/// Investment Partnership
///
/// Main structure that combines partners, AI team, and portfolio
#[derive(Debug)]
pub struct InvestmentPartnership {
    /// Partnership name
    pub name: String,

    /// Partners
    pub partners: Vec<Partner>,

    /// AI investment team
    pub ai_team: AITeam,

    /// Portfolio
    pub portfolio: Portfolio,

    /// Partnership agreement
    pub agreement: PartnershipAgreement,

    /// Creation date
    pub created_at: chrono::DateTime<Utc>,
}

impl InvestmentPartnership {
    /// Analyze investment opportunity
    pub async fn analyze(&self, symbol: &str) -> Result<InvestmentDecision> {
        // Use AI team to analyze
        let analysis_result = self.ai_team.analyze_investment(symbol).await?;

        // Parse results
        let action = match analysis_result["action"].as_str() {
            Some("heavy_buy") => InvestmentAction::HeavyBuy,
            Some("buy") => InvestmentAction::Buy,
            Some("small_buy") => InvestmentAction::SmallBuy,
            Some("hold") => InvestmentAction::Hold,
            Some("reduce") => InvestmentAction::Reduce,
            Some("sell") | _ => InvestmentAction::Sell,
        };

        let confidence = analysis_result["confidence"].as_f64().unwrap_or(0.5);
        let position_size = analysis_result["position_size"].as_f64().unwrap_or(0.1);
        let reasoning = analysis_result["reasoning"].as_str().unwrap_or("AI analysis").to_string();

        Ok(InvestmentDecision {
            symbol: symbol.to_string(),
            action,
            confidence,
            position_size,
            reasoning,
            expected_return: None,
            time_horizon: chrono::Duration::days(365 * 3),
            team_inputs: TeamInputs {
                research: serde_json::from_value(analysis_result["team_inputs"]["research"].clone()).ok(),
                analysis: serde_json::from_value(analysis_result["team_inputs"]["analysis"].clone()).ok(),
                trading: serde_json::from_value(analysis_result["team_inputs"]["trading"].clone()).ok(),
                risk: serde_json::from_value(analysis_result["team_inputs"]["risk"].clone()).ok(),
            },
            risk_considerations: vec![],
            timestamp: Utc::now(),
        })
    }

    /// Get partnership status
    pub fn status(&self) -> PartnershipStatus {
        let total_capital: f64 = self.partners.iter().map(|p| p.capital_contribution).sum();
        let metrics = self.portfolio.calculate_metrics();

        PartnershipStatus {
            name: self.name.clone(),
            partner_count: self.partners.len(),
            total_capital,
            current_value: self.portfolio.total_value,
            total_return: metrics.total_return,
            position_count: self.portfolio.positions.len(),
            cash: self.portfolio.cash,
            strategy: self.agreement.investment_strategy.clone(),
            created_at: self.created_at,
        }
    }
}

/// Partnership status
#[derive(Debug, Clone, serde::Serialize)]
pub struct PartnershipStatus {
    pub name: String,
    pub partner_count: usize,
    pub total_capital: f64,
    pub current_value: f64,
    pub total_return: f64,
    pub position_count: usize,
    pub cash: f64,
    pub strategy: InvestmentStrategy,
    pub created_at: chrono::DateTime<Utc>,
}

/// Partnership Builder
pub struct PartnershipBuilder {
    name: Option<String>,
    partners: Vec<Partner>,
    strategy: InvestmentStrategy,
    custom_agreement: Option<PartnershipAgreement>,
}

impl PartnershipBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            name: None,
            partners: Vec::new(),
            strategy: InvestmentStrategy::ValueInvesting,
            custom_agreement: None,
        }
    }

    /// Set partnership name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add partners
    pub fn partners(mut self, partners: Vec<Partner>) -> Self {
        self.partners = partners;
        self
    }

    /// Add single partner
    pub fn add_partner(mut self, partner: Partner) -> Self {
        self.partners.push(partner);
        self
    }

    /// Set investment strategy
    pub fn strategy(mut self, strategy: InvestmentStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set custom agreement
    pub fn agreement(mut self, agreement: PartnershipAgreement) -> Self {
        self.custom_agreement = Some(agreement);
        self
    }

    /// Build the partnership
    pub async fn build(self) -> Result<InvestmentPartnership> {
        // Validate
        if let Some(ref name) = self.name {
            anyhow::ensure!(!name.is_empty(), "Partnership name cannot be empty");
        } else {
            anyhow::bail!("Partnership name is required");
        }

        anyhow::ensure!(!self.partners.is_empty(), "At least one partner is required");

        // Calculate total capital
        let total_capital: f64 = self.partners.iter().map(|p| p.capital_contribution).sum();

        // Create agreement
        let agreement = self.custom_agreement
            .unwrap_or_else(|| PartnershipAgreement::for_strategy(self.strategy.clone()));

        // Validate minimum investment
        for partner in &self.partners {
            if partner.capital_contribution < agreement.minimum_investment {
                anyhow::bail!(
                    "Partner {} investment {} below minimum {}",
                    partner.name,
                    partner.capital_contribution,
                    agreement.minimum_investment
                );
            }
        }

        // Create portfolio
        let portfolio = Portfolio::new(total_capital);

        // Create AI team
        let ai_team = AITeam::new();

        Ok(InvestmentPartnership {
            name: self.name.unwrap(),
            partners: self.partners,
            ai_team,
            portfolio,
            agreement,
            created_at: Utc::now(),
        })
    }

    /// Reuse existing agents from investintel-agent
    pub fn reuse_existing_agents(self) -> Self {
        // This is the default behavior - ai_team will be created with existing agents
        self
    }
}

impl Default for PartnershipBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_partnership_creation() {
        let partners = vec![
            Partner::new("Alice", 100_000.0, 0.8),
            Partner::new("Bob", 150_000.0, 0.8),
        ];

        let partnership = PartnershipBuilder::new()
            .name("Test Partnership".to_string())
            .partners(partners)
            .strategy(InvestmentStrategy::ValueInvesting)
            .build()
            .await
            .unwrap();

        assert_eq!(partnership.name, "Test Partnership");
        assert_eq!(partnership.partners.len(), 2);
        assert_eq!(partnership.portfolio.total_value, 250_000.0);
    }

    #[test]
    fn test_partnership_status() {
        let partners = vec![Partner::new("Alice", 100_000.0, 0.8)];

        let partnership = InvestmentPartnership {
            name: "Test".to_string(),
            partners: partners.clone(),
            ai_team: AITeam::new(),
            portfolio: Portfolio::new(100_000.0),
            agreement: PartnershipAgreement::default(),
            created_at: Utc::now(),
        };

        let status = partnership.status();
        assert_eq!(status.name, "Test");
        assert_eq!(status.partner_count, 1);
        assert_eq!(status.total_capital, 100_000.0);
    }
}
