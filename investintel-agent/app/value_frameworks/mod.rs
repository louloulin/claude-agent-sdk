//! Value Investment Frameworks
//!
//! 基于Claude Agent SDK的价值投资框架实现
//! 包括Graham、Buffett、Munger三位一体的完整价值投资体系

pub mod graham;
pub mod buffett;
pub mod munger;
pub mod integrated;

pub use graham::{
    GrahamFormula, GrahamFramework, GrahamAnalysis, NetNetScreener,
};
pub use buffett::{
    BuffettFramework, BuffettAnalysis, MoatAnalyzer, MoatScore,
    ManagementEvaluator, ManagementScore,
};
pub use munger::{
    MungerFramework, MungerAnalysis, MentalModel, LollapaloozaDetector,
    CircleOfCompetence,
};
pub use integrated::{
    ValueInvestingFramework, ComprehensiveDecision, InvestmentAction,
};
