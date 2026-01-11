//! AI Team Management
//!
//! Organizes agents into professional subagent teams for Plan6.

use anyhow::Result;
use async_trait::async_trait;
use claude_agent_sdk_rs::orchestration::{Agent, AgentInput, AgentOutput, agent::AgentError};
use serde_json::json;

/// AI Investment Team
pub struct AITeam {
    pub chief_investment_agent: Box<dyn Agent>,
    pub research_team: ResearchTeam,
    pub analysis_team: AnalysisTeam,
    pub trading_team: TradingTeam,
    pub risk_team: RiskTeam,
}

impl AITeam {
    pub fn new() -> Self {
        Self {
            chief_investment_agent: Box::new(ChiefInvestmentAgent::new()),
            research_team: ResearchTeam::new(),
            analysis_team: AnalysisTeam::new(),
            trading_team: TradingTeam::new(),
            risk_team: RiskTeam::new(),
        }
    }

    pub async fn analyze_investment(&self, symbol: &str) -> Result<serde_json::Value> {
        let research_results = self.research_team.analyze_parallel(symbol).await?;
        let analysis_results = self.analysis_team.analyze_hierarchical(symbol, &research_results).await?;
        let debate_results = self.execute_debate(symbol, &analysis_results).await?;

        let context = json!({
            "research": research_results,
            "analysis": analysis_results,
            "debate": debate_results,
        });

        let decision = self.chief_investment_agent
            .execute(AgentInput::new(symbol.to_string()).with_context(context))
            .await?;

        Ok(serde_json::from_str(&decision.content)?)
    }

    async fn execute_debate(&self, symbol: &str, _analysis: &serde_json::Value) -> Result<serde_json::Value> {
        let trading_result = self.trading_team.execution_agent
            .execute(AgentInput::new(symbol.to_string()))
            .await?;

        let risk_result = self.risk_team.portfolio_monitor
            .execute(AgentInput::new(symbol.to_string()))
            .await?;

        Ok(json!({
            "trading": serde_json::from_str::<serde_json::Value>(&trading_result.content)?,
            "risk": serde_json::from_str::<serde_json::Value>(&risk_result.content)?,
        }))
    }
}

pub struct ResearchTeam {
    pub fundamental_researcher: Box<dyn Agent>,
    pub technical_analyst: Box<dyn Agent>,
    pub sentiment_analyst: Box<dyn Agent>,
    pub macro_analyst: Box<dyn Agent>,
}

impl ResearchTeam {
    pub fn new() -> Self {
        Self {
            fundamental_researcher: Box::new(SimpleAgent::new("Fundamental Researcher", "Fundamental analysis")),
            technical_analyst: Box::new(SimpleAgent::new("Technical Analyst", "Technical analysis")),
            sentiment_analyst: Box::new(SimpleAgent::new("Sentiment Analyst", "Sentiment analysis")),
            macro_analyst: Box::new(SimpleAgent::new("Macro Analyst", "Macro analysis")),
        }
    }

    pub async fn analyze_parallel(&self, symbol: &str) -> Result<serde_json::Value> {
        let input = AgentInput::new(symbol.to_string());
        let (fundamental, technical, sentiment, macro_result) = tokio::try_join!(
            self.fundamental_researcher.execute(input.clone()),
            self.technical_analyst.execute(input.clone()),
            self.sentiment_analyst.execute(input.clone()),
            self.macro_analyst.execute(input)
        )?;

        Ok(json!({
            "fundamental": serde_json::from_str::<serde_json::Value>(&fundamental.content)?,
            "technical": serde_json::from_str::<serde_json::Value>(&technical.content)?,
            "sentiment": serde_json::from_str::<serde_json::Value>(&sentiment.content)?,
            "macro": serde_json::from_str::<serde_json::Value>(&macro_result.content)?,
        }))
    }
}

impl std::fmt::Debug for ResearchTeam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResearchTeam")
            .field("fundamental_researcher", &"Agent")
            .field("technical_analyst", &"Agent")
            .field("sentiment_analyst", &"Agent")
            .field("macro_analyst", &"Agent")
            .finish()
    }
}

pub struct AnalysisTeam {
    pub valuation_analyst: Box<dyn Agent>,
    pub quality_analyst: Box<dyn Agent>,
    pub risk_analyst: Box<dyn Agent>,
    pub moat_analyst: Box<dyn Agent>,
}

impl AnalysisTeam {
    pub fn new() -> Self {
        Self {
            valuation_analyst: Box::new(SimpleAgent::new("Valuation Analyst", "Valuation analysis")),
            quality_analyst: Box::new(SimpleAgent::new("Quality Analyst", "Quality analysis")),
            risk_analyst: Box::new(SimpleAgent::new("Risk Analyst", "Risk analysis")),
            moat_analyst: Box::new(SimpleAgent::new("Moat Analyst", "Moat analysis")),
        }
    }

    pub async fn analyze_hierarchical(&self, symbol: &str, _research: &serde_json::Value) -> Result<serde_json::Value> {
        let (valuation, quality, risk, moat) = tokio::try_join!(
            self.valuation_analyst.execute(AgentInput::new(symbol.to_string())),
            self.quality_analyst.execute(AgentInput::new(symbol.to_string())),
            self.risk_analyst.execute(AgentInput::new(symbol.to_string())),
            self.moat_analyst.execute(AgentInput::new(symbol.to_string()))
        )?;

        Ok(json!({
            "valuation": serde_json::from_str::<serde_json::Value>(&valuation.content)?,
            "quality": serde_json::from_str::<serde_json::Value>(&quality.content)?,
            "risk": serde_json::from_str::<serde_json::Value>(&risk.content)?,
            "moat": serde_json::from_str::<serde_json::Value>(&moat.content)?,
        }))
    }
}

impl std::fmt::Debug for AnalysisTeam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalysisTeam")
            .field("valuation_analyst", &"Agent")
            .field("quality_analyst", &"Agent")
            .field("risk_analyst", &"Agent")
            .field("moat_analyst", &"Agent")
            .finish()
    }
}

pub struct TradingTeam {
    pub execution_agent: Box<dyn Agent>,
    pub position_sizer: Box<dyn Agent>,
    pub order_router: Box<dyn Agent>,
}

impl TradingTeam {
    pub fn new() -> Self {
        Self {
            execution_agent: Box::new(SimpleAgent::new("Execution Agent", "Trade execution")),
            position_sizer: Box::new(SimpleAgent::new("Position Sizer", "Position sizing")),
            order_router: Box::new(SimpleAgent::new("Order Router", "Order routing")),
        }
    }
}

impl std::fmt::Debug for TradingTeam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TradingTeam")
            .field("execution_agent", &"Agent")
            .field("position_sizer", &"Agent")
            .field("order_router", &"Agent")
            .finish()
    }
}

pub struct RiskTeam {
    pub portfolio_monitor: Box<dyn Agent>,
    pub risk_manager: Box<dyn Agent>,
    pub compliance_agent: Box<dyn Agent>,
}

impl RiskTeam {
    pub fn new() -> Self {
        Self {
            portfolio_monitor: Box::new(SimpleAgent::new("Portfolio Monitor", "Portfolio monitoring")),
            risk_manager: Box::new(SimpleAgent::new("Risk Manager", "Risk management")),
            compliance_agent: Box::new(SimpleAgent::new("Compliance Agent", "Compliance")),
        }
    }
}

impl std::fmt::Debug for RiskTeam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RiskTeam")
            .field("portfolio_monitor", &"Agent")
            .field("risk_manager", &"Agent")
            .field("compliance_agent", &"Agent")
            .finish()
    }
}

impl std::fmt::Debug for AITeam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AITeam")
            .field("chief_investment_agent", &"Box<dyn Agent>")
            .field("research_team", &self.research_team)
            .field("analysis_team", &self.analysis_team)
            .field("trading_team", &self.trading_team)
            .field("risk_team", &self.risk_team)
            .finish()
    }
}

// Simple Agent implementation
#[derive(Debug)]
pub struct SimpleAgent {
    name: String,
    description: String,
}

impl SimpleAgent {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
        }
    }
}

#[async_trait]
impl Agent for SimpleAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, _input: AgentInput) -> std::result::Result<AgentOutput, AgentError> {
        let result = json!({
            "agent": self.name,
            "symbol": "test",
            "score": 75,
            "analysis": format!("Analysis by {}", self.name),
        });

        Ok(AgentOutput::new(serde_json::to_string_pretty(&result).unwrap())
            .with_confidence(0.80))
    }
}

// Chief Investment Agent
#[derive(Debug)]
pub struct ChiefInvestmentAgent {
    name: String,
    description: String,
}

impl ChiefInvestmentAgent {
    pub fn new() -> Self {
        Self {
            name: "Chief Investment Agent".to_string(),
            description: "AI Buffett - Final decision maker".to_string(),
        }
    }
}

#[async_trait]
impl Agent for ChiefInvestmentAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, _input: AgentInput) -> std::result::Result<AgentOutput, AgentError> {
        let decision = json!({
            "action": "buy",
            "confidence": 0.85,
            "position_size": 0.20,
            "reasoning": "Based on comprehensive team analysis",
        });

        Ok(AgentOutput::new(serde_json::to_string_pretty(&decision).unwrap())
            .with_confidence(0.90))
    }
}
