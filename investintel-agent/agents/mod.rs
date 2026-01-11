//! InvestIntel AI Investment Agents
//!
//! 投资智能助手 - 基于Claude Agent SDK的最小化改造实现
//!
//! ## 核心设计原则
//!
//! 1. **充分复用**: 复用现有25+ Skills、Orchestration系统、数据层
//! 2. **最小改造**: 只添加必要的协调Agent，不重写现有代码
//! 3. **高内聚低耦合**: 每个Agent职责单一，通过trait松耦合
//! 4. **实用主义**: 先实现核心价值，后续可扩展
//!
//! ## 架构设计
//!
//! ```text
//! 用户请求
//!    ↓
//! InvestmentAssistant (协调层 - 新增)
//!    ├─ ValueInvestmentAgent (价值分析 - 新增)
//!    ├─ PortfolioManagerAgent (组合管理 - 新增)
//!    └─ TradingAdvisorAgent (交易建议 - 新增)
//!        ↓
//! 现有Skills系统 (复用)
//!    ├─ fundamental-analysis
//!    ├─ portfolio-management
//!    ├─ market-research
//!    ├─ risk-analysis
//!    └─ ... (25+ skills)
//!        ↓
//! 现有数据层 (复用)
//!    ├─ Yahoo Finance
//!    ├─ Alpha Vantage
//!    └─ WebSocket实时数据
//! ```

pub mod value_investment;
pub mod portfolio_manager;
pub mod trading_advisor;
pub mod dividend_investor;
pub mod kelly_position;
pub mod munger_framework;
pub mod assistant;
pub mod market_data;
pub mod parallel_data;

// Re-export commonly used types
pub use assistant::InvestmentAssistant;
pub use value_investment::ValueInvestmentAgent;
pub use portfolio_manager::PortfolioManagerAgent;
pub use trading_advisor::TradingAdvisorAgent;
pub use dividend_investor::DividendInvestorAgent;
pub use kelly_position::KellyPositionAgent;
pub use munger_framework::{MungerFrameworkAgent, MungerAnalysis};
pub use market_data::{
    MarketDataProvider, StockQuote, FundamentalData, DividendData,
    get_realtime_quote, get_fundamental_data, get_dividend_info,
};
pub use parallel_data::{ParallelDataFetcher, FetchStats};

use claude_agent_sdk_rs::orchestration::{
    Agent, AgentInput, AgentOutput,
};
use claude_agent_sdk_rs::orchestration::agent::{AgentError, Result as AgentResult};
use claude_agent_sdk_rs::orchestration::OrchestrationError;
use async_trait::async_trait;

/// 从InvestError转换到AgentError
impl From<InvestError> for AgentError {
    fn from(err: InvestError) -> Self {
        AgentError::Other(err.into())
    }
}

/// 从InvestError转换到OrchestrationError (用于orchestrator)
impl From<InvestError> for OrchestrationError {
    fn from(err: InvestError) -> Self {
        OrchestrationError::AgentError(err.into())
    }
}

// 重新导出Agent的Result类型，以便其他模块使用
pub use claude_agent_sdk_rs::orchestration::agent::Result;

/// InvestIntel Agent错误类型
#[derive(Debug, thiserror::Error)]
pub enum InvestError {
    #[error("Data error: {0}")]
    DataError(String),

    #[error("Analysis error: {0}")]
    AnalysisError(String),

    #[error("Trading error: {0}")]
    TradingError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// 投资建议
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InvestmentRecommendation {
    /// 标的代码
    pub symbol: String,

    /// 建议: buy/sell/hold
    pub action: InvestmentAction,

    /// 置信度 0-1
    pub confidence: f64,

    /// 建议理由
    pub reasoning: String,

    /// 目标价位
    pub target_price: Option<f64>,

    /// 当前价格
    pub current_price: Option<f64>,

    /// 安全边际 (%)
    pub margin_of_safety: Option<f64>,

    /// 风险等级 0-3
    pub risk_level: u8,

    /// 额外信息
    pub metadata: serde_json::Value,
}

/// 投资行动
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InvestmentAction {
    /// 强烈买入
    StrongBuy,

    /// 买入
    Buy,

    /// 持有
    Hold,

    /// 卖出
    Sell,

    /// 强烈卖出
    StrongSell,
}

impl InvestmentAction {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.8 {
            InvestmentAction::StrongBuy
        } else if score >= 0.6 {
            InvestmentAction::Buy
        } else if score >= 0.4 {
            InvestmentAction::Hold
        } else if score >= 0.2 {
            InvestmentAction::Sell
        } else {
            InvestmentAction::StrongSell
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            InvestmentAction::StrongBuy => "强烈买入",
            InvestmentAction::Buy => "买入",
            InvestmentAction::Hold => "持有",
            InvestmentAction::Sell => "卖出",
            InvestmentAction::StrongSell => "强烈卖出",
        }
    }
}

/// Graham价值分析结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrahamAnalysis {
    /// 内在价值
    pub intrinsic_value: f64,

    /// 当前价格
    pub current_price: f64,

    /// 安全边际
    pub margin_of_safety: f64,

    /// 每股收益
    pub eps: f64,

    /// 预期增长率
    pub growth_rate: f64,

    /// 是否符合Graham标准
    pub meets_criteria: bool,
}

/// Buffett价值分析结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuffettAnalysis {
    /// ROIC
    pub roic: f64,

    /// ROE
    pub roe: f64,

    /// 护城河评分
    pub moat_score: u8, // 0-3

    /// 内在价值 (DCF)
    pub intrinsic_value: f64,

    /// 公允价格
    pub fair_price: f64,

    /// 是否符合Buffett标准
    pub meets_criteria: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_investment_action_from_score() {
        assert!(matches!(
            InvestmentAction::from_score(0.9),
            InvestmentAction::StrongBuy
        ));
        assert!(matches!(
            InvestmentAction::from_score(0.7),
            InvestmentAction::Buy
        ));
        assert!(matches!(
            InvestmentAction::from_score(0.5),
            InvestmentAction::Hold
        ));
        assert!(matches!(
            InvestmentAction::from_score(0.3),
            InvestmentAction::Sell
        ));
        assert!(matches!(
            InvestmentAction::from_score(0.1),
            InvestmentAction::StrongSell
        ));
    }

    #[test]
    fn test_investment_action_display_name() {
        assert_eq!(InvestmentAction::StrongBuy.display_name(), "强烈买入");
        assert_eq!(InvestmentAction::Buy.display_name(), "买入");
        assert_eq!(InvestmentAction::Hold.display_name(), "持有");
        assert_eq!(InvestmentAction::Sell.display_name(), "卖出");
        assert_eq!(InvestmentAction::StrongSell.display_name(), "强烈卖出");
    }
}
