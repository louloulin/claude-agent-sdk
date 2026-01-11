//! InvestIntel AI - 智能投资助手
//!
//! 基于Claude Agent SDK的完整智能投资分析平台
//!
//! ## 核心功能
//!
//! 1. **价值投资分析** - Graham-Buffett-Munger三位一体价值分析
//! 2. **投资组合管理** - 资产配置、再平衡、绩效评估
//! 3. **交易建议** - 时机选择、仓位管理、风险控制
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use investintel_agent::agents::InvestmentAssistant;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let assistant = InvestmentAssistant::new();
//!
//! // 分析股票
//! let analysis = assistant.analyze_stock("AAPL").await?;
//! println!("{}", analysis.value_analysis);
//!
//! // 交互式咨询
//! let response = assistant.chat("分析MSFT的投资价值").await?;
//! println!("{}", response);
//! # Ok(())
//! # }
//! ```

pub mod agents;
pub mod data;
pub mod trading;
pub mod mcp;

#[cfg(feature = "ml")]
pub mod strategies;

// Re-export commonly used types
pub use agents::{
    InvestmentAssistant, InvestmentAction, InvestmentRecommendation,
    PortfolioManagerAgent, TradingAdvisorAgent, ValueInvestmentAgent,
    DividendInvestorAgent, KellyPositionAgent, MungerFrameworkAgent, MungerAnalysis,
    MarketDataProvider, StockQuote, FundamentalData, DividendData,
};
pub use mcp::{MCPGateway, GatewayConfig, DataQuery, Data, OrderRequest, OrderResponse};
pub use trading::{
    BinanceFuturesClient, OkxClient, OrderManager, RiskEngine,
    EmergencyStopManager, OrderRequest as TradingOrderRequest, OrderResponse as TradingOrderResponse,
};
