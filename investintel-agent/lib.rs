//! InvestIntel AI - 智能投资助手
//!
//! 基于Claude Agent SDK的完整智能投资分析平台

pub mod data;
pub mod strategies;
pub mod trading;

// Re-export commonly used types
pub use trading::{
    BinanceFuturesClient, OkxClient, OrderManager, RiskEngine,
    EmergencyStopManager, OrderRequest, OrderResponse,
};
