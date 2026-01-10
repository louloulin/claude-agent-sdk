//! Bidirectional Investment Analysis using ClaudeClient
//!
//! This module implements real-time interactive investment analysis using
//! Claude Agent SDK's ClaudeClient for bidirectional communication.

use anyhow::Result;
use claude_agent_sdk_rs::{ClaudeAgentOptions, ClaudeClient, ContentBlock, Message};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::agents::*;
use super::orchestrators::*;

/// Interactive Investment Advisor
///
/// Uses ClaudeClient for real-time, context-aware investment advice
pub struct InteractiveInvestmentAdvisor {
    client: ClaudeClient,
    orchestrator: Arc<InvestmentAnalysisOrchestrator>,
    conversation_history: Arc<RwLock<Vec<Message>>>,
}

impl InteractiveInvestmentAdvisor {
    /// Create a new interactive advisor
    pub async fn new() -> Result<Self> {
        let options = ClaudeAgentOptions::builder()
            .max_turns(Some(10))
            .thinking(true)
            .max_thinking_tokens(2000)
            .build();

        let client = ClaudeClient::new(options);
        let orchestrator = Arc::new(InvestmentAnalysisOrchestrator::new());

        Ok(Self {
            client,
            orchestrator,
            conversation_history: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Start interactive session
    pub async fn start_session(&mut self) -> Result<()> {
        println!("🔗 Connecting to Claude...");
        self.client.connect().await?;
        println!("✅ Connected!\n");

        Ok(())
    }

    /// Interactive analysis with full context
    pub async fn analyze_interactive(
        &mut self,
        user_query: &str,
    ) -> Result<InvestmentAdvice> {
        // Send query
        self.client.query(user_query).await?;

        // Receive response
        let mut response = String::new();
        let mut stream = self.client.receive_response();

        while let Some(message) = stream.next().await {
            match message {
                Ok(Message::Assistant(msg)) => {
                    for block in &msg.message.content {
                        if let ContentBlock::Text { text } = block {
                            response.push_str(text);
                            print!("{}", text);
                        }
                    }
                }
                Ok(Message::Result(result)) => {
                    println!("\n[Analysis completed in {}ms]", result.duration_ms);
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }
        drop(stream);

        // Store in conversation history
        let history = self.conversation_history.clone();
        // Note: In real implementation, we'd store the actual messages

        // Parse response and generate advice
        let advice = self.parse_advice(&response).await?;

        Ok(advice)
    }

    /// Follow-up question with context
    pub async fn follow_up(&mut self, question: &str) -> Result<String> {
        // Claude remembers the conversation context!
        self.client.query(question).await?;

        let mut response = String::new();
        let mut stream = self.client.receive_response();

        while let Some(message) = stream.next().await {
            match message {
                Ok(Message::Assistant(msg)) => {
                    for block in &msg.message.content {
                        if let ContentBlock::Text { text } = block {
                            response.push_str(text);
                            println!("{}", text);
                        }
                    }
                }
                Ok(Message::Result(_)) => {
                    println!("\n[Follow-up answered]");
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }

        Ok(response)
    }

    /// Multi-turn comprehensive analysis
    pub async fn comprehensive_analysis_session(
        &mut self,
        symbol: &str,
    ) -> Result<ComprehensiveAnalysisReport> {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║     Comprehensive Investment Analysis Session               ║");
        println!("║     Symbol: {}                                    ║", symbol);
        println!("╚════════════════════════════════════════════════════════════╝\n");

        // Turn 1: Technical Analysis
        println!("🔍 Turn 1: Technical Analysis");
        let tech_query = format!(
            "请对{}进行技术分析,包括趋势、支撑阻力位、技术指标等。",
            symbol
        );
        let tech_response = self.analyze_interactive(&tech_query).await?;

        // Turn 2: Fundamental Analysis (Claude remembers context!)
        println!("\n📊 Turn 2: Fundamental Analysis");
        let fund_query = format!(
            "基于前面的技术分析,现在请对{}进行基本面分析和估值。",
            symbol
        );
        let fund_response = self.analyze_interactive(&fund_query).await?;

        // Turn 3: Risk Assessment
        println!("\n⚠️  Turn 3: Risk Assessment");
        let risk_query = format!(
            "综合前面的分析,请评估{}的投资风险和适当的仓位管理。",
            symbol
        );
        let risk_response = self.analyze_interactive(&risk_query).await?;

        // Turn 4: Investment Recommendation
        println!("\n💡 Turn 4: Final Recommendation");
        let rec_query = "基于以上所有分析,请给出最终的投资建议和行动计划。";
        let rec_response = self.analyze_interactive(rec_query).await?;

        // Generate comprehensive report
        let report = ComprehensiveAnalysisReport {
            symbol: symbol.to_string(),
            technical_analysis: tech_response,
            fundamental_analysis: fund_response,
            risk_assessment: risk_response,
            recommendation: rec_response,
            session_turns: 4,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        Ok(report)
    }

    /// Ask follow-up question
    pub async fn ask_follow_up(&mut self, question: &str) -> Result<String> {
        println!("\n❓ Follow-up: {}", question);
        let answer = self.follow_up(question).await?;
        Ok(answer)
    }

    /// End session
    pub async fn end_session(&mut self) -> Result<()> {
        println!("\n🔚 Ending session...");
        self.client.disconnect().await?;
        println!("✅ Session ended\n");
        Ok(())
    }

    /// Parse advice from Claude response
    async fn parse_advice(&self, response: &str) -> Result<InvestmentAdvice> {
        // In a real implementation, this would use NLP to parse the response
        // For now, return a structured placeholder
        Ok(InvestmentAdvice {
            recommendation: extract_recommendation(response)?,
            confidence: extract_confidence(response)?,
            reasoning: response.to_string(),
            action_items: vec![],
        })
    }
}

/// Extract recommendation from text
fn extract_recommendation(text: &str) -> Result<Recommendation> {
    let text_lower = text.to_lowercase();

    if text_lower.contains("强烈买入") || text_lower.contains("strong buy") {
        Ok(Recommendation::StrongBuy)
    } else if text_lower.contains("买入") || text_lower.contains("buy") {
        Ok(Recommendation::Buy)
    } else if text_lower.contains("持有") || text_lower.contains("hold") {
        Ok(Recommendation::Hold)
    } else if text_lower.contains("减持") || text_lower.contains("reduce") {
        Ok(Recommendation::Reduce)
    } else if text_lower.contains("卖出") || text_lower.contains("sell") {
        Ok(Recommendation::Sell)
    } else {
        Ok(Recommendation::Hold)
    }
}

/// Extract confidence from text
fn extract_confidence(text: &str) -> Result<f64> {
    // Simple heuristic - in real implementation, use more sophisticated parsing
    if text.contains("高确定") || text.contains("高度确定") {
        Ok(0.85)
    } else if text.contains("中等确定") {
        Ok(0.70)
    } else if text.contains("不确定") {
        Ok(0.55)
    } else {
        Ok(0.70)
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
pub struct InvestmentAdvice {
    pub recommendation: Recommendation,
    pub confidence: f64,
    pub reasoning: String,
    pub action_items: Vec<String>,
}

#[derive(Debug)]
pub struct ComprehensiveAnalysisReport {
    pub symbol: String,
    pub technical_analysis: InvestmentAdvice,
    pub fundamental_analysis: InvestmentAdvice,
    pub risk_assessment: InvestmentAdvice,
    pub recommendation: InvestmentAdvice,
    pub session_turns: usize,
    pub timestamp: String,
}
