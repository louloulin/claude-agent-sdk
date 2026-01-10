//! Investment Agent Orchestration
//!
//! Multi-agent orchestration for investment analysis using Claude Agent SDK

use anyhow::Result;
use claude_agent_sdk_rs::orchestration::{
    Agent, AgentInput, AgentOutput, Orchestrator, OrchestratorInput,
    ParallelOrchestrator, SequentialOrchestrator,
};
use async_trait::async_trait;
use std::collections::HashMap;

/// Market Research Agent
///
/// Conducts technical analysis and market research
pub struct MarketResearchAgent {
    pub name: String,
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
        // Simulate market research
        let symbol = input.content.clone();

        let analysis = serde_json::json!({
            "agent": "Market Research",
            "symbol": symbol,
            "trend": "bullish",
            "rsi": 65.0,
            "macd": "bullish_cross",
            "support": [150.0, 145.0],
            "resistance": [160.0, 165.0],
            "recommendation": "buy",
            "confidence": 0.80
        });

        Ok(AgentOutput::new(serde_json::to_string_pretty(&analysis).unwrap())
            .with_confidence(0.80)
            .with_metadata("agent_type", "research"))
    }
}

/// Investment Analyst Agent
///
/// Performs fundamental analysis and valuation
pub struct InvestmentAnalystAgent {
    pub name: String,
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
        // Simulate fundamental analysis
        let symbol = input.content.clone();

        let analysis = serde_json::json!({
            "agent": "Investment Analyst",
            "symbol": symbol,
            "fair_value": 175.0,
            "current_price": 165.0,
            "upside": "6.1%",
            "pe_ratio": 28.0,
            "roe": 0.175,
            "recommendation": "buy",
            "thesis": "Strong fundamentals and growth potential"
        });

        Ok(AgentOutput::new(serde_json::to_string_pretty(&analysis).unwrap())
            .with_confidence(0.85)
            .with_metadata("agent_type", "analyst"))
    }
}

/// Risk Management Agent
///
/// Assesses portfolio risks and calculates risk metrics
pub struct RiskManagementAgent {
    pub name: String,
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
        // Simulate risk analysis
        let symbol = input.content.clone();

        let analysis = serde_json::json!({
            "agent": "Risk Management",
            "symbol": symbol,
            "var_1day_95": -2000.0,
            "volatility": 0.20,
            "max_drawdown": -0.08,
            "beta": 1.2,
            "risk_level": "moderate",
            "recommendations": [
                "Use position sizing",
                "Set stop loss at 150",
                "Diversify holdings"
            ]
        });

        Ok(AgentOutput::new(serde_json::to_string_pretty(&analysis).unwrap())
            .with_confidence(0.90)
            .with_metadata("agent_type", "risk"))
    }
}

/// Sentiment Analysis Agent
///
/// Analyzes market sentiment from news and social media
pub struct SentimentAnalysisAgent {
    pub name: String,
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
        // Simulate sentiment analysis
        let symbol = input.content.clone();

        let analysis = serde_json::json!({
            "agent": "Sentiment Analysis",
            "symbol": symbol,
            "news_sentiment": 0.65,
            "social_sentiment": 0.58,
            "composite_sentiment": 0.62,
            "signal": "bullish",
            "trend": "improving",
            "sources_analyzed": 150
        });

        Ok(AgentOutput::new(serde_json::to_string_pretty(&analysis).unwrap())
            .with_confidence(0.75)
            .with_metadata("agent_type", "sentiment"))
    }
}

/// Investment Advisor Agent
///
/// Synthesizes all analysis and provides final investment recommendation
pub struct InvestmentAdvisorAgent {
    pub name: String,
}

#[async_trait]
impl Agent for InvestmentAdvisorAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Synthesizes analysis and provides investment recommendations"
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        // Synthesize all previous agent outputs
        let mut analyses = Vec::new();

        for (key, value) in &input.context {
            if let Some(json_str) = value.as_str() {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                    analyses.push(parsed);
                }
            }
        }

        let recommendation = serde_json::json!({
            "agent": "Investment Advisor",
            "action": "buy",
            "confidence": 0.82,
            "target_price": 175.0,
            "stop_loss": 150.0,
            "position_size": "5%",
            "reasoning": {
                "technical": "Bullish trend with good momentum",
                "fundamental": "Reasonable valuation with strong growth",
                "risk": "Manageable with proper risk controls",
                "sentiment": "Positive market sentiment"
            },
            "analyses": analyses
        });

        Ok(AgentOutput::new(serde_json::to_string_pretty(&recommendation).unwrap())
            .with_confidence(0.82)
            .with_metadata("agent_type", "advisor"))
    }
}

/// Comprehensive Investment Analysis Orchestrator
///
/// Runs multiple agents in parallel then synthesizes results
pub async fn run_comprehensive_analysis(symbol: &str) -> Result<AgentOutput> {
    // Run research and sentiment agents in parallel
    let parallel_agents: Vec<Box<dyn Agent>> = vec![
        Box::new(MarketResearchAgent {
            name: "Market Research".to_string(),
        }),
        Box::new(SentimentAnalysisAgent {
            name: "Sentiment Analysis".to_string(),
        }),
    ];

    let parallel_orchestrator = ParallelOrchestrator::new().with_parallel_limit(2);
    let parallel_input = OrchestratorInput::new(symbol);
    let parallel_output = parallel_orchestrator.orchestrate(parallel_agents, parallel_input).await?;

    // Create context from parallel results
    let mut context = HashMap::new();
    for agent_output in &parallel_output.agent_outputs {
        context.insert(
            agent_output.metadata.get("agent_type").unwrap_or(&"unknown".to_string()).clone(),
            serde_json::json!(agent_output.content),
        );
    }

    // Run analyst and risk agents with the context
    let sequential_agents: Vec<Box<dyn Agent>> = vec![
        Box::new(InvestmentAnalystAgent {
            name: "Investment Analyst".to_string(),
        }),
        Box::new(RiskManagementAgent {
            name: "Risk Management".to_string(),
        }),
        Box::new(InvestmentAdvisorAgent {
            name: "Investment Advisor".to_string(),
        }),
    ];

    let sequential_orchestrator = SequentialOrchestrator::new().with_max_retries(1);

    let mut sequential_input = OrchestratorInput::new(symbol);
    sequential_input.context = context;

    let final_output = sequential_orchestrator.orchestrate(sequential_agents, sequential_input).await?;

    Ok(final_output.result)
}
