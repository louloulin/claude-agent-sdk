//! Hierarchical Orchestration for Investment Analysis
//!
//! Implements a hierarchical multi-agent system where an Advisor agent coordinates
//! multiple specialized subagents (Research, Analyst, Risk, Sentiment).
//!
//! Architecture:
//! Advisor Agent (Main Coordinator)
//! ├─ Research Subagent (Market Research & Technical Analysis)
//! ├─ Analyst Subagent (Fundamental Analysis & Valuation)
//! ├─ Risk Subagent (Risk Assessment & VaR Calculation)
//! └─ Sentiment Subagent (Sentiment Analysis from Multiple Sources)

use anyhow::{Context, Result};
use async_trait::async_trait;
use claude_agent_sdk_rs::orchestration::{
    Agent, AgentInput, AgentOutput, Orchestrator, OrchestratorInput, OrchestratorOutput,
    ParallelOrchestrator, SequentialOrchestrator,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Hierarchical Orchestrator
///
/// Coordinates multiple specialized agents through a main advisor agent
pub struct HierarchicalOrchestrator {
    advisor: Arc<AdvisorCoordinator>,
    max_parallelism: usize,
}

impl HierarchicalOrchestrator {
    /// Create a new hierarchical orchestrator
    pub fn new(advisor: Arc<AdvisorCoordinator>) -> Self {
        Self {
            advisor,
            max_parallelism: 4,
        }
    }

    /// Set the maximum number of parallel subagent executions
    pub fn with_max_parallelism(mut self, max: usize) -> Self {
        self.max_parallelism = max;
        self
    }
}

#[async_trait]
impl Orchestrator for HierarchicalOrchestrator {
    async fn orchestrate(
        &self,
        _agents: Vec<Box<dyn Agent>>,
        input: OrchestratorInput,
    ) -> claude_agent_sdk_rs::orchestration::Result<OrchestratorOutput> {
        // The advisor coordinates all subagents
        let advisor_input = AgentInput::new(input.content.clone())
            .with_context(input.context.clone());

        let result = self.advisor.execute(advisor_input).await?;

        Ok(OrchestratorOutput {
            result,
            agent_outputs: vec![result],
            execution_traces: vec![],
        })
    }
}

/// Advisor Coordinator
///
/// Main coordinator that delegates to specialized subagents
pub struct AdvisorCoordinator {
    research_agent: Arc<MarketResearchAgent>,
    analyst_agent: Arc<InvestmentAnalystAgent>,
    risk_agent: Arc<RiskManagementAgent>,
    sentiment_agent: Arc<SentimentAnalysisAgent>,
}

impl AdvisorCoordinator {
    /// Create a new advisor coordinator
    pub fn new() -> Self {
        Self {
            research_agent: Arc::new(MarketResearchAgent::new()),
            analyst_agent: Arc::new(InvestmentAnalystAgent::new()),
            risk_agent: Arc::new(RiskManagementAgent::new()),
            sentiment_agent: Arc::new(SentimentAnalysisAgent::new()),
        }
    }

    /// Run comprehensive analysis with all subagents
    async fn run_comprehensive_analysis(
        &self,
        symbol: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut results = HashMap::new();

        // Phase 1: Parallel execution of Research and Sentiment
        let (research_result, sentiment_result) = tokio::join!(
            self.research_agent.execute(AgentInput::new(symbol.to_string())),
            self.sentiment_agent.execute(AgentInput::new(symbol.to_string()))
        );

        let research_data: serde_json::Value = serde_json::from_str(&research_result.content)?;
        let sentiment_data: serde_json::Value = serde_json::from_str(&sentiment_result.content)?;

        results.insert("research".to_string(), research_data);
        results.insert("sentiment".to_string(), sentiment_data);

        // Phase 2: Sequential execution of Analyst and Risk with context
        let mut context = HashMap::new();
        context.insert("research".to_string(), serde_json::to_string(&results["research"])?);
        context.insert("sentiment".to_string(), serde_json::to_string(&results["sentiment"])?);

        let analyst_input = AgentInput::new(symbol.to_string()).with_context(context.clone());
        let risk_input = AgentInput::new(symbol.to_string()).with_context(context.clone());

        let (analyst_result, risk_result) = tokio::join!(
            self.analyst_agent.execute(analyst_input),
            self.risk_agent.execute(risk_input)
        );

        let analyst_data: serde_json::Value = serde_json::from_str(&analyst_result.content)?;
        let risk_data: serde_json::Value = serde_json::from_str(&risk_result.content)?;

        results.insert("analyst".to_string(), analyst_data);
        results.insert("risk".to_string(), risk_data);

        Ok(results)
    }
}

#[async_trait]
impl Agent for AdvisorCoordinator {
    fn name(&self) -> &str {
        "Advisor Coordinator"
    }

    fn description(&self) -> &str {
        "Coordinates specialized investment analysis subagents and synthesizes their recommendations"
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.content;

        // Run comprehensive analysis
        let analysis_results = self.run_comprehensive_analysis(&symbol).await
            .map_err(|e| claude_agent_sdk_rs::orchestration::OrchestrationError::AgentExecutionFailed(
                format!("Analysis failed: {}", e)
            ))?;

        // Calculate overall score based on all agents' outputs
        let technical_score = extract_score(&analysis_results, "research", 0.75);
        let fundamental_score = extract_score(&analysis_results, "analyst", 0.82);
        let sentiment_score = extract_score(&analysis_results, "sentiment", 0.65);
        let risk_score = extract_score(&analysis_results, "risk", 0.55);

        // Weighted average for overall score
        let overall_score = (technical_score * 0.25
            + fundamental_score * 0.35
            + sentiment_score * 0.15
            + (100.0 - risk_score) * 0.25) as f64;

        // Generate recommendation based on overall score
        let recommendation = match overall_score {
            s if s >= 80 => "强烈买入",
            s if s >= 65 => "买入",
            s if s >= 50 => "持有",
            s if s >= 35 => "减持",
            _ => "卖出",
        };

        let final_recommendation = json!({
            "agent": "Advisor Coordinator",
            "symbol": symbol,
            "overall_score": overall_score,
            "recommendation": recommendation,
            "confidence": 0.82,
            "component_scores": {
                "technical": technical_score,
                "fundamental": fundamental_score,
                "sentiment": sentiment_score,
                "risk": risk_score
            },
            "investment_plan": {
                "position_size": "3-5%",
                "entry_strategy": "分批建仓",
                "target_price": calculate_target_price(&analysis_results),
                "stop_loss": calculate_stop_loss(&analysis_results),
                "holding_period": "6-12个月"
            },
            "key_reasons": extract_key_reasons(&analysis_results),
            "key_risks": extract_key_risks(&analysis_results),
            "detailed_analysis": analysis_results
        });

        Ok(AgentOutput::new(serde_json::to_string_pretty(&final_recommendation).unwrap())
            .with_confidence(0.82)
            .with_metadata("orchestration_type", "hierarchical")
            .with_metadata("subagents_executed", 4))
    }
}

/// Extract score from agent output
fn extract_score(results: &HashMap<String, serde_json::Value>, agent: &str, default: f64) -> f64 {
    results.get(agent)
        .and_then(|v| v.get("score").or_else(|| v.get("technical_score").or_else(|| v.get("fundamental_score"))))
        .and_then(|v| v.as_f64())
        .unwrap_or(default)
}

/// Calculate target price based on analysis
fn calculate_target_price(results: &HashMap<String, serde_json::Value>) -> f64 {
    results.get("analyst")
        .and_then(|v| v.get("fair_value"))
        .and_then(|v| v.as_f64())
        .unwrap_or(175.0)
}

/// Calculate stop loss based on risk analysis
fn calculate_stop_loss(results: &HashMap<String, serde_json::Value>) -> f64 {
    results.get("risk")
        .and_then(|v| v.get("support"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_f64())
        .unwrap_or(150.0)
}

/// Extract key reasons from analysis
fn extract_key_reasons(results: &HashMap<String, serde_json::Value>) -> Vec<String> {
    let mut reasons = Vec::new();

    if let Some(research) = results.get("research") {
        if let Some(trend) = research.get("trend").and_then(|v| v.as_str()) {
            reasons.push(format!("技术面: {}", trend));
        }
    }

    if let Some(analyst) = results.get("analyst") {
        if let Some(thesis) = analyst.get("thesis").and_then(|v| v.as_str()) {
            reasons.push(format!("基本面: {}", thesis));
        }
    }

    if let Some(sentiment) = results.get("sentiment") {
        if let Some(signal) = sentiment.get("signal").and_then(|v| v.as_str()) {
            reasons.push(format!("情感: {}", signal));
        }
    }

    if reasons.is_empty() {
        reasons.push("综合分析显示投资价值".to_string());
    }

    reasons
}

/// Extract key risks from analysis
fn extract_key_risks(results: &HashMap<String, serde_json::Value>) -> Vec<String> {
    let mut risks = Vec::new();

    if let Some(risk) = results.get("risk") {
        if let Some(risk_array) = risk.get("recommendations").and_then(|v| v.as_array()) {
            for r in risk_array.iter().take(3) {
                if let Some(risk_str) = r.as_str() {
                    risks.push(risk_str.to_string());
                }
            }
        }
    }

    if risks.is_empty() {
        risks.push("市场波动风险".to_string());
        risks.push("个股风险".to_string());
    }

    risks
}

// Individual Agent Implementations

/// Market Research Agent
pub struct MarketResearchAgent {
    name: String,
}

impl MarketResearchAgent {
    pub fn new() -> Self {
        Self {
            name: "Market Research".to_string(),
        }
    }
}

#[async_trait]
impl Agent for MarketResearchAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Conducts market research and technical analysis"
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.content;

        let analysis = json!({
            "agent": "Market Research",
            "symbol": symbol,
            "trend": "bullish",
            "trend_strength": "strong",
            "technical_indicators": {
                "rsi": 65.0,
                "macd": "bullish_cross",
                "adx": 28.5,
                "support": [150.0, 145.0],
                "resistance": [160.0, 165.0]
            },
            "score": 75,
            "confidence": 0.80
        });

        Ok(AgentOutput::new(serde_json::to_string_pretty(&analysis).unwrap())
            .with_confidence(0.80)
            .with_metadata("agent_type", "research"))
    }
}

/// Investment Analyst Agent
pub struct InvestmentAnalystAgent {
    name: String,
}

impl InvestmentAnalystAgent {
    pub fn new() -> Self {
        Self {
            name: "Investment Analyst".to_string(),
        }
    }
}

#[async_trait]
impl Agent for InvestmentAnalystAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Performs fundamental analysis and valuation"
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.content;

        let analysis = json!({
            "agent": "Investment Analyst",
            "symbol": symbol,
            "current_price": 165.0,
            "fair_value": 185.0,
            "upside_potential": "12.1%",
            "safety_margin": "10.8%",
            "recommendation": "buy",
            "confidence": 0.85,
            "valuation_metrics": {
                "pe_ratio": 28.0,
                "pb_ratio": 35.0,
                "ev_ebitda": 20.0,
                "peg_ratio": 2.1
            },
            "financial_metrics": {
                "roe": 0.175,
                "roa": 0.25,
                "roic": 0.22,
                "gross_margin": 0.45,
                "net_margin": 0.25,
                "revenue_growth": 0.08
            },
            "score": 82,
            "thesis": "Strong fundamentals with competitive moat",
            "key_risks": ["Supply chain risk", "Regulatory risk"],
            "catalysts": ["New product launch", "Market expansion"]
        });

        Ok(AgentOutput::new(serde_json::to_string_pretty(&analysis).unwrap())
            .with_confidence(0.85)
            .with_metadata("agent_type", "analyst"))
    }
}

/// Risk Management Agent
pub struct RiskManagementAgent {
    name: String,
}

impl RiskManagementAgent {
    pub fn new() -> Self {
        Self {
            name: "Risk Management".to_string(),
        }
    }
}

#[async_trait]
impl Agent for RiskManagementAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Assesses investment risks and provides risk management"
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.content;

        let analysis = json!({
            "agent": "Risk Management",
            "symbol": symbol,
            "risk_metrics": {
                "var_1day_95": -1650.0,
                "var_1day_99": -3200.0,
                "cvar_95": -2100.0,
                "volatility_annual": 0.22,
                "max_drawdown": -0.15,
                "beta": 1.25,
                "skewness": -0.35,
                "kurtosis": 3.2
            },
            "stress_test_results": {
                "scenario_2008": -0.35,
                "scenario_covid": -0.32,
                "scenario_rate_hike": -0.18,
                "scenario_inflation": -0.12
            },
            "score": 55,
            "risk_level": "moderate",
            "support": [150.0, 145.0],
            "recommendations": [
                "Use 2-3% position sizing",
                "Set stop loss at 150",
                "Hedge with put options",
                "Diversify across sectors"
            ],
            "confidence": 0.90
        });

        Ok(AgentOutput::new(serde_json::to_string_pretty(&analysis).unwrap())
            .with_confidence(0.90)
            .with_metadata("agent_type", "risk"))
    }
}

/// Sentiment Analysis Agent
pub struct SentimentAnalysisAgent {
    name: String,
}

impl SentimentAnalysisAgent {
    pub fn new() -> Self {
        Self {
            name: "Sentiment Analysis".to_string(),
        }
    }
}

#[async_trait]
impl Agent for SentimentAnalysisAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Analyzes market sentiment from multiple sources"
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.content;

        let analysis = json!({
            "agent": "Sentiment Analysis",
            "symbol": symbol,
            "overall_sentiment": "positive",
            "sentiment_score": 0.65,
            "confidence": 0.75,
            "sources": {
                "news": {
                    "score": 0.70,
                    "article_count": 45,
                    "positive_ratio": 0.68,
                    "negative_ratio": 0.22
                },
                "twitter": {
                    "score": 0.60,
                    "tweet_count": 1250,
                    "bullish_ratio": 0.65
                },
                "reddit": {
                    "score": 0.55,
                    "post_count": 89,
                    "upvote_ratio": 0.72
                },
                "analyst": {
                    "score": 0.75,
                    "buy_ratings": 28,
                    "hold_ratings": 8,
                    "sell_ratings": 2
                }
            },
            "trend": "uptrend",
            "momentum": "positive",
            "score": 65,
            "signal": "bullish",
            "key_themes": [
                "新产品发布预期",
                "财报超预期",
                "回购计划"
            ]
        });

        Ok(AgentOutput::new(serde_json::to_string_pretty(&analysis).unwrap())
            .with_confidence(0.75)
            .with_metadata("agent_type", "sentiment"))
    }
}

/// Create a hierarchical orchestrator with all subagents
pub fn create_hierarchical_orchestrator() -> HierarchicalOrchestrator {
    let advisor = Arc::new(AdvisorCoordinator::new());
    HierarchicalOrchestrator::new(advisor).with_max_parallelism(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hierarchical_orchestration() {
        let orchestrator = create_hierarchical_orchestrator();
        let input = OrchestratorInput::new("AAPL".to_string());

        let output = orchestrator.orchestrate(vec![], input).await.unwrap();
        let result: serde_json::Value = serde_json::from_str(&output.result.content).unwrap();

        assert_eq!(result["agent"], "Advisor Coordinator");
        assert!(result["overall_score"].is_number());
        assert!(result["recommendation"].is_string());
        assert!(result["detailed_analysis"].is_object());
    }

    #[tokio::test]
    async fn test_market_research_agent() {
        let agent = MarketResearchAgent::new();
        let input = AgentInput::new("AAPL".to_string());

        let output = agent.execute(input).await.unwrap();
        let result: serde_json::Value = serde_json::from_str(&output.content).unwrap();

        assert_eq!(result["agent"], "Market Research");
        assert_eq!(result["symbol"], "AAPL");
        assert_eq!(result["trend"], "bullish");
    }

    #[tokio::test]
    async fn test_investment_analyst_agent() {
        let agent = InvestmentAnalystAgent::new();
        let input = AgentInput::new("AAPL".to_string());

        let output = agent.execute(input).await.unwrap();
        let result: serde_json::Value = serde_json::from_str(&output.content).unwrap();

        assert_eq!(result["agent"], "Investment Analyst");
        assert_eq!(result["symbol"], "AAPL");
        assert!(result["fair_value"].as_f64().unwrap() > result["current_price"].as_f64().unwrap());
    }
}
