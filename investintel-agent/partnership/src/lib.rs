//! AI Investment Partnership Module
//!
//! Based on Warren Buffett's partnership model (1956-1969), AI-ified for the modern era.
//!
//! This module extends the existing investintel-agent with partnership capabilities:
//! - Partnership structure (partners, AI team, portfolio)
//! - Buffett-style profit sharing (6% hurdle, 20-25% of excess)
//! - Subagent teams (Research, Analysis, Trading, Risk)
//! - MCP gateway integration
//!
//! ## Design Philosophy
//!
//! **Minimal Change, Maximum Reuse**:
//! - Reuses existing Agents from investintel-agent/app/agents.rs
//! - Reuses existing Orchestrators from investintel-agent/app/orchestrators.rs
//! - Reuses existing MCP tools from investintel-agent/app/tools.rs
//! - Only adds partnership layer on top
//!
//! ## Example
//!
//! ```no_run
//! use partnership::{PartnershipBuilder, Partner, InvestmentStrategy};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create partners
//! let partners = vec![
//!     Partner::new("Alice", 100_000.0, 0.8),
//! ];
//!
//! // Create partnership with existing agents
//! let partnership = PartnershipBuilder::new()
//!     .name("AI Investment Partnership".to_string())
//!     .partners(partners)
//!     .strategy(InvestmentStrategy::ValueInvesting)
//!     .build()
//!     .await?;
//!
//! // Analyze investment
//! let decision = partnership.analyze("AAPL").await?;
//! println!("Recommendation: {:?}", decision.action);
//! # Ok(())
//! # }
//! ```

pub mod types;
pub mod agreement;
pub mod portfolio;
pub mod builder;
pub mod ai_team;
pub mod mcp_gateway;

#[cfg(test)]
mod types_test;
#[cfg(test)]
mod agreement_test;
#[cfg(test)]
mod portfolio_test;
#[cfg(test)]
mod ai_team_test;
#[cfg(test)]
mod builder_test;

pub use types::*;
pub use agreement::*;
pub use portfolio::*;
pub use builder::*;
pub use ai_team::*;
pub use mcp_gateway::*;
