//! Teams Module - 专业投资团队
//!
//! 这个模块组织所有的专业投资团队:
//! - ResearchTeam: 研究团队(4个Agents)
//! - AnalysisTeam: 分析团队(4个Agents)
//! - TradingTeam: 交易团队(3个Agents)
//! - RiskTeam: 风控团队(3个Agents)

pub mod research_team;
pub mod analysis_team;
pub mod trading_team;
pub mod risk_team;

pub use research_team::{
    ResearchTeam,
    FundamentalResearcher,
    TechnicalAnalyst as ResearchTechnicalAnalyst,
    SentimentAnalyst as ResearchSentimentAnalyst,
    MacroAnalyst,
};

pub use analysis_team::{
    AnalysisTeam,
    ValuationAnalyst,
    QualityAnalyst,
    RiskAnalyst as AnalysisRiskAnalyst,
    MoatAnalyst,
};

pub use trading_team::{
    TradingTeam,
    ExecutionAgent,
    PositionSizer,
    OrderRouter,
};

pub use risk_team::{
    RiskTeam,
    PortfolioMonitor,
    RiskManager,
    ComplianceAgent,
};
