//! AI Investment Partnership Module
//!
//! This module implements the AI investment partnership model based on
//! Warren Buffett's partnership structure (1956-1969), AI-ified for the modern era.
//!
//! # Core Concept
//!
//! - **AI as Partner**: Not just a tool, but a true investment partner
//! - **Buffett Model**: 6% hurdle rate, 20-25% profit share on excess returns
//! - **Subagent Teams**: Professional AI team working collaboratively
//! - **MCP Architecture**: Unified connection to all data sources and trading APIs
//!
//! # Example
//!
//! ```no_run
//! use claude_agent_sdk::partnership::{PartnershipBuilder, Partner, InvestmentStrategy};
//! use chrono::Local;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create partners
//! let partners = vec![
//!     Partner {
//!         id: "partner-1".to_string(),
//!         name: "Alice".to_string(),
//!         capital_contribution: 100_000.0,
//!         profit_share: 0.8,
//!         voting_rights: true,
//!         risk_profile: RiskProfile::Moderate,
//!         investment_goals: InvestmentGoals {
//!             target_return: 0.15,
//!             time_horizon: Duration::days(365 * 5),
//!             liquidity_needs: LiquidityNeeds::Low,
//!         },
//!     },
//! ];
//!
//! // Create partnership
//! let partnership = PartnershipBuilder::create_partnership(
//!     "AI Investment Partnership".to_string(),
//!     partners,
//!     InvestmentStrategy::ValueInvesting,
//! ).await?;
//!
//! # Ok(())
//! # }
//! ```

pub mod types;
pub mod agreement;
pub mod portfolio;
pub mod builder;

pub use types::{
    Partner, RiskProfile, InvestmentGoals, LiquidityNeeds, InvestmentStrategy,
};
pub use agreement::{
    PartnershipAgreement, ConcentrationLimits, RedemptionPolicy, ProfitDistribution,
};
pub use portfolio::{
    Portfolio, Position, PerformanceHistory, PerformanceMetrics,
};
pub use builder::PartnershipBuilder;
