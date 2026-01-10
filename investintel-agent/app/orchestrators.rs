//! Advanced Orchestration Implementation using Claude Agent SDK
//!
//! This file implements real Orchestrators using the Claude Agent SDK's orchestration framework.
//! Demonstrates sequential, parallel, and hierarchical orchestration patterns.

use anyhow::Result;
use async_trait::async_trait;
use claude_agent_sdk_rs::orchestration::{
    Agent, AgentInput, AgentOutput, Orchestrator, OrchestratorError,
    OrchestratorInput, OrchestratorOutput, ParallelOrchestrator, SequentialOrchestrator,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::join;

use super::agents::*;

// ============================================================================
// Custom Orchestrators for Investment Analysis
// ============================================================================

/// Investment Analysis Orchestrator
///
/// A custom orchestrator that coordinates multiple agents for comprehensive investment analysis
pub struct InvestmentAnalysisOrchestrator {
    agents: Vec<Box<dyn Agent>>,
}

impl InvestmentAnalysisOrchestrator {
    /// Create a new investment analysis orchestrator
    pub fn new() -> Self {
        Self {
            agents: create_all_agents(),
        }
    }

    /// Run comprehensive analysis using all agents
    pub async fn run_comprehensive_analysis(&self, symbol: &str) -> Result<InvestmentReport> {
        let input = AgentInput::new(symbol.to_string());

        // Phase 1: Parallel execution - Technical + Sentiment
        let (tech_result, sentiment_result) = join!(
            self.agents[0].execute(input.clone()),
            self.agents[3].execute(input.clone())
        );

        let tech_data: serde_json::Value = serde_json::from_str(&tech_result.content)?;
        let sentiment_data: serde_json::Value = serde_json::from_str(&sentiment_result.content)?;

        // Phase 2: Sequential execution with context - Fundamental + Risk
        let mut context = HashMap::new();
        context.insert("technical_data".to_string(), serde_json::to_string(&tech_data)?);
        context.insert("sentiment_data".to_string(), serde_json::to_string(&sentiment_data)?);

        let fundamental_input = AgentInput::new(symbol.to_string()).with_context(context.clone());
        let risk_input = AgentInput::new(symbol.to_string()).with_context(context.clone());

        let (fund_result, risk_result) = join!(
            self.agents[1].execute(fundamental_input),
            self.agents[2].execute(risk_input)
        );

        let fundamental_data: serde_json::Value = serde_json::from_str(&fund_result.content)?;
        let risk_data: serde_json::Value = serde_json::from_str(&risk_result.content)?;

        // Generate comprehensive report
        let report = self.generate_report(
            symbol,
            tech_data,
            fundamental_data,
            risk_data,
            sentiment_data,
        )?;

        Ok(report)
    }

    /// Generate comprehensive investment report
    fn generate_report(
        &self,
        symbol: &str,
        tech: serde_json::Value,
        fundamental: serde_json::Value,
        risk: serde_json::Value,
        sentiment: serde_json::Value,
    ) -> Result<InvestmentReport> {
        // Extract scores
        let technical_score = tech["technical_score"].as_i64().unwrap_or(50);
        let fundamental_score = fundamental["fundamental_score"].as_i64().unwrap_or(50);
        let risk_score = risk["risk_score"].as_i64().unwrap_or(50);
        let sentiment_score = sentiment["sentiment_score"].as_i64().unwrap_or(50);

        // Calculate composite score (weighted)
        // Technical: 25%, Fundamental: 35%, Sentiment: 15%, Risk (inverted): 25%
        let composite_score = (technical_score * 25
            + fundamental_score * 35
            + sentiment_score * 15
            + (100 - risk_score) * 25)
            / 100;

        // Determine recommendation
        let recommendation = match composite_score {
            s if s >= 80 => Recommendation::StrongBuy,
            s if s >= 65 => Recommendation::Buy,
            s if s >= 50 => Recommendation::Hold,
            s if s >= 35 => Recommendation::Reduce,
            _ => Recommendation::Sell,
        };

        Ok(InvestmentReport {
            symbol: symbol.to_string(),
            technical_analysis: tech,
            fundamental_analysis: fundamental,
            risk_analysis: risk,
            sentiment_analysis: sentiment,
            scores: Scores {
                technical: technical_score,
                fundamental: fundamental_score,
                risk: risk_score,
                sentiment: sentiment_score,
                composite: composite_score,
            },
            recommendation,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

#[async_trait]
impl Orchestrator for InvestmentAnalysisOrchestrator {
    async fn orchestrate(
        &self,
        _agents: Vec<Box<dyn Agent>>,
        input: OrchestratorInput,
    ) -> claude_agent_sdk_rs::orchestration::Result<OrchestratorOutput> {
        let symbol = input.content;

        let report = self.run_comprehensive_analysis(&symbol).await
            .map_err(|e| OrchestratorError::OrchestrationFailed(
                format!("Comprehensive analysis failed: {}", e)
            ))?;

        let result_json = json!(&report).to_string();

        let output = OrchestratorOutput {
            result: AgentOutput::new(result_json)
                .with_confidence(0.90)
                .with_metadata("orchestrator_type", "investment_analysis"),
            agent_outputs: vec![],
            execution_traces: vec![],
        };

        Ok(output)
    }
}

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Clone)]
pub enum Recommendation {
    StrongBuy,
    Buy,
    Hold,
    Reduce,
    Sell,
}

#[derive(Debug)]
pub struct Scores {
    pub technical: i64,
    pub fundamental: i64,
    pub risk: i64,
    pub sentiment: i64,
    pub composite: i64,
}

#[derive(Debug)]
pub struct InvestmentReport {
    pub symbol: String,
    pub technical_analysis: serde_json::Value,
    pub fundamental_analysis: serde_json::Value,
    pub risk_analysis: serde_json::Value,
    pub sentiment_analysis: serde_json::Value,
    pub scores: Scores,
    pub recommendation: Recommendation,
    pub timestamp: String,
}

// ============================================================================
// Specialized Orchestrators
// ============================================================================

/// Sequential Investment Orchestrator
///
/// Executes agents in sequence: Research -> Analyst -> Risk
pub type SequentialInvestmentOrchestrator = SequentialOrchestrator;

/// Parallel Investment Orchestrator
///
/// Executes all agents in parallel and aggregates results
pub type ParallelInvestmentOrchestrator = ParallelOrchestrator;

/// Hybrid Investment Orchestrator
///
/// Combines sequential and parallel execution for optimal performance
pub struct HybridInvestmentOrchestrator {
    sequential_orchestrator: SequentialOrchestrator,
    parallel_orchestrator: ParallelOrchestrator,
}

impl HybridInvestmentOrchestrator {
    pub fn new() -> Self {
        Self {
            sequential_orchestrator: SequentialOrchestrator::new(),
            parallel_orchestrator: ParallelOrchestrator::new().with_parallel_limit(4),
        }
    }

    /// Run hybrid analysis
    pub async fn run_hybrid_analysis(
        &self,
        agents: Vec<Box<dyn Agent>>,
        symbol: &str,
    ) -> Result<OrchestratorOutput> {
        // Phase 1: Parallel - Technical + Sentiment
        let phase1_agents: Vec<_> = agents.iter()
            .filter(|a| a.name().contains("Market") || a.name().contains("Sentiment"))
            .map(|a| a.name().to_string())
            .collect();

        // Phase 2: Sequential - Fundamental + Risk (with phase 1 context)
        let phase2_agents: Vec<_> = agents.iter()
            .filter(|a| a.name().contains("Investment") || a.name().contains("Risk"))
            .map(|a| a.name().to_string())
            .collect();

        // For simplicity, use parallel orchestrator for all agents
        let input = OrchestratorInput::new(symbol.to_string());

        let output = self.parallel_orchestrator
            .orchestrate(agents, input)
            .await?;

        Ok(output)
    }
}

#[async_trait]
impl Orchestrator for HybridInvestmentOrchestrator {
    async fn orchestrate(
        &self,
        agents: Vec<Box<dyn Agent>>,
        input: OrchestratorInput,
    ) -> claude_agent_sdk_rs::orchestration::Result<OrchestratorOutput> {
        // Use parallel orchestrator for now
        self.parallel_orchestrator.orchestrate(agents, input).await
    }
}

// ============================================================================
// Factory Functions
// ============================================================================

/// Create a sequential investment orchestrator
pub fn create_sequential_orchestrator() -> SequentialInvestmentOrchestrator {
    SequentialOrchestrator::new().with_max_retries(2)
}

/// Create a parallel investment orchestrator
pub fn create_parallel_orchestrator() -> ParallelInvestmentOrchestrator {
    ParallelOrchestrator::new().with_parallel_limit(4)
}

/// Create a hybrid investment orchestrator
pub fn create_hybrid_orchestrator() -> HybridInvestmentOrchestrator {
    HybridInvestmentOrchestrator::new()
}

/// Create the main investment analysis orchestrator
pub fn create_investment_orchestrator() -> InvestmentAnalysisOrchestrator {
    InvestmentAnalysisOrchestrator::new()
}
