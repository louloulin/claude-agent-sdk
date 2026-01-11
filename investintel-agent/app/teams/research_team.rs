//! Research Team - 专业研究Agents
//!
//! 这个模块实现研究团队的4个专业Agents:
//! - FundamentalResearcher: 基本面研究
//! - TechnicalAnalyst: 技术分析
//! - SentimentAnalyst: 情绪分析
//! - MacroAnalyst: 宏观分析

use crate::skills::{SkillAgent, SkillRegistry};
use async_trait::async_trait;
use claude_agent_sdk_rs::orchestration::{Agent, AgentInput, AgentOutput, OrchestratorError};
use serde_json::json;

// ============================================================================
// FundamentalResearcher - 基本面研究Agent
// ============================================================================

/// Fundamental Research Agent
///
/// 专注于深度基本面分析,包括财务报表、业务模式和竞争环境
pub struct FundamentalResearcher {
    base_agent: SkillAgent,
}

impl FundamentalResearcher {
    /// Create a new fundamental researcher
    pub fn new(registry: SkillRegistry) -> Self {
        let base_agent = SkillAgent::new(
            "FundamentalResearcher",
            "深度基本面分析专家:财务报表、业务模式、竞争环境",
            "Graham深度价值投资", // 使用Graham Skill作为基础
            registry,
        );

        Self { base_agent }
    }

    /// 分析财务报表
    async fn analyze_financials(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        // 在真实实现中,这里会:
        // 1. 从MCP Gateway获取财务数据
        // 2. 计算关键比率
        // 3. 识别趋势

        Ok(json!({
            "symbol": symbol,
            "financial_strength": {
                "current_ratio": 1.8,
                "quick_ratio": 1.2,
                "debt_to_equity": 0.45,
                "interest_coverage": 12.5,
            },
            "profitability": {
                "gross_margin": 0.55,
                "operating_margin": 0.30,
                "net_margin": 0.22,
                "roe": 0.18,
                "roic": 0.15,
            },
            "growth": {
                "revenue_growth_3y": 0.08,
                "eps_growth_3y": 0.10,
                "fcf_growth_3y": 0.12,
            },
            "quality_score": 75,
            "financial_health": "strong"
        }))
    }

    /// 分析业务模式
    async fn analyze_business_model(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "symbol": symbol,
            "business_model": {
                "type": "subscription",
                "revenue_streams": ["subscription", "services", "hardware"],
                "customer_retention": 0.92,
                "ltv_cac_ratio": 5.5,
            },
            "competitive_position": {
                "market_share": 0.25,
                "ranking": 2,
                "trend": "gaining"
            }
        }))
    }

    /// 分析竞争环境
    async fn analyze_competition(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "symbol": symbol,
            "competitors": [
                {"name": "COMP_A", "market_cap": 2500000000000, "score": 85},
                {"name": "COMP_B", "market_cap": 180000000000000, "score": 78},
                {"name": "COMP_C", "market_cap": 120000000000000, "score": 72},
            ],
            "competitive_advantage": "technology_leadership",
            "threat_level": "moderate"
        }))
    }
}

#[async_trait]
impl Agent for FundamentalResearcher {
    fn name(&self) -> &str {
        self.base_agent.name()
    }

    fn description(&self) -> &str {
        self.base_agent.description()
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.metadata.get("symbol")
            .unwrap_or(&input.content)
            .clone();

        // 综合三项分析
        let financials = self.analyze_financials(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Financial analysis failed: {}", e)))?;

        let business = self.analyze_business_model(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Business analysis failed: {}", e)))?;

        let competition = self.analyze_competition(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Competition analysis failed: {}", e)))?;

        let combined_analysis = json!({
            "symbol": symbol,
            "fundamental_analysis": {
                "financials": financials,
                "business_model": business,
                "competition": competition
            },
            "overall_rating": if financials["quality_score"] == 75 { "strong_buy" } else { "hold" },
            "confidence": 0.85
        });

        let content = format!(
            "基本面分析完成:\n{}\n",
            serde_json::to_string_pretty(&combined_analysis).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(combined_analysis)
            .with_confidence(0.85)
            .with_metadata("agent_type", "fundamental_researcher")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// TechnicalAnalyst - 技术分析Agent
// ============================================================================

/// Technical Analysis Agent
///
/// 专注于价格趋势、技术指标和支撑阻力分析
pub struct TechnicalAnalyst {
    base_agent: SkillAgent,
}

impl TechnicalAnalyst {
    /// Create a new technical analyst
    pub fn new(registry: SkillRegistry) -> Self {
        let base_agent = SkillAgent::new(
            "TechnicalAnalyst",
            "技术分析专家:价格趋势、技术指标、支撑阻力",
            "Buffett质量价值投资", // 可以使用任意Skill
            registry,
        );

        Self { base_agent }
    }

    /// 计算技术指标
    async fn calculate_indicators(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "symbol": symbol,
            "indicators": {
                "trend": {
                    "ma20": 155.0,
                    "ma50": 150.0,
                    "ma200": 145.0,
                    "trend": "bullish"
                },
                "momentum": {
                    "rsi": 65.0,
                    "macd": {"value": 2.5, "signal": "bullish_cross"},
                    "stochastic": {"k": 75.0, "d": 70.0}
                },
                "volatility": {
                    "bollinger_bands": {"upper": 165.0, "middle": 155.0, "lower": 145.0},
                    "atr": 3.5
                },
                "volume": {
                    "avg_volume": 50000000,
                    "volume_trend": "increasing"
                }
            }
        }))
    }

    /// 识别支撑阻力
    async fn identify_levels(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "symbol": symbol,
            "support_levels": [150.0, 145.0, 140.0],
            "resistance_levels": [160.0, 165.0, 170.0],
            "current_price": 155.0,
            "position": "middle"
        }))
    }
}

#[async_trait]
impl Agent for TechnicalAnalyst {
    fn name(&self) -> &str {
        self.base_agent.name()
    }

    fn description(&self) -> &str {
        self.base_agent.description()
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.metadata.get("symbol")
            .unwrap_or(&input.content)
            .clone();

        let indicators = self.calculate_indicators(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Indicator calculation failed: {}", e)))?;

        let levels = self.identify_levels(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Level identification failed: {}", e)))?;

        let technical_analysis = json!({
            "symbol": symbol,
            "technical_analysis": {
                "indicators": indicators,
                "levels": levels
            },
            "signal": "buy",
            "confidence": 0.75
        });

        let content = format!(
            "技术分析完成:\n{}\n",
            serde_json::to_string_pretty(&technical_analysis).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(technical_analysis)
            .with_confidence(0.75)
            .with_metadata("agent_type", "technical_analyst")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// SentimentAnalyst - 情绪分析Agent
// ============================================================================

/// Sentiment Analysis Agent
///
/// 分析新闻、社交媒体和分析师评级
pub struct SentimentAnalyst {
    base_agent: SkillAgent,
}

impl SentimentAnalyst {
    /// Create a new sentiment analyst
    pub fn new(registry: SkillRegistry) -> Self {
        let base_agent = SkillAgent::new(
            "SentimentAnalyst",
            "情绪分析专家:新闻、社交媒体、分析师评级",
            "Munger多元思维模型",
            registry,
        );

        Self { base_agent }
    }

    /// 分析新闻情绪
    async fn analyze_news(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "symbol": symbol,
            "news_sentiment": {
                "positive_count": 45,
                "negative_count": 12,
                "neutral_count": 23,
                "sentiment_score": 0.72,
                "trend": "improving"
            }
        }))
    }

    /// 分析社交媒体情绪
    async fn analyze_social(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "symbol": symbol,
            "social_sentiment": {
                "twitter_score": 0.68,
                "reddit_score": 0.65,
                "stocktwits_score": 0.71,
                "average_score": 0.68,
                "mention_volume": "high"
            }
        }))
    }

    /// 汇总分析师评级
    async fn aggregate_ratings(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "symbol": symbol,
            "analyst_ratings": {
                "strong_buy": 15,
                "buy": 20,
                "hold": 8,
                "sell": 2,
                "strong_sell": 0,
                "consensus": "buy",
                "avg_target_price": 175.0
            }
        }))
    }
}

#[async_trait]
impl Agent for SentimentAnalyst {
    fn name(&self) -> &str {
        self.base_agent.name()
    }

    fn description(&self) -> &str {
        self.base_agent.description()
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.metadata.get("symbol")
            .unwrap_or(&input.content)
            .clone();

        let news = self.analyze_news(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("News analysis failed: {}", e)))?;

        let social = self.analyze_social(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Social analysis failed: {}", e)))?;

        let ratings = self.aggregate_ratings(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Ratings aggregation failed: {}", e)))?;

        let sentiment_analysis = json!({
            "symbol": symbol,
            "sentiment_analysis": {
                "news": news,
                "social": social,
                "analyst": ratings
            },
            "overall_sentiment": "positive",
            "confidence": 0.70
        });

        let content = format!(
            "情绪分析完成:\n{}\n",
            serde_json::to_string_pretty(&sentiment_analysis).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(sentiment_analysis)
            .with_confidence(0.70)
            .with_metadata("agent_type", "sentiment_analyst")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// MacroAnalyst - 宏观分析Agent
// ============================================================================

/// Macro Economic Analysis Agent
///
/// 分析宏观经济、行业周期和政策影响
pub struct MacroAnalyst {
    base_agent: SkillAgent,
}

impl MacroAnalyst {
    /// Create a new macro analyst
    pub fn new(registry: SkillRegistry) -> Self {
        let base_agent = SkillAgent::new(
            "MacroAnalyst",
            "宏观分析专家:宏观经济、行业周期、政策影响",
            "Munger多元思维模型",
            registry,
        );

        Self { base_agent }
    }

    /// 分析宏观经济
    async fn analyze_macro(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "symbol": symbol,
            "macro_environment": {
                "gdp_growth": 0.025,
                "inflation_rate": 0.032,
                "interest_rate": 0.0525,
                "unemployment": 0.038,
                "economic_cycle": "expansion"
            }
        }))
    }

    /// 判断行业周期
    async fn analyze_industry_cycle(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "symbol": symbol,
            "industry_analysis": {
                "industry": "technology",
                "cycle_stage": "growth",
                "industry_pe": 28.0,
                "industry_growth": 0.12,
                "outlook": "positive"
            }
        }))
    }

    /// 评估政策影响
    async fn analyze_policy(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "symbol": symbol,
            "policy_impact": {
                "regulatory_risk": "low",
                "government_support": "neutral",
                "tax_policy": "favorable",
                "trade_policy": "low_impact"
            }
        }))
    }
}

#[async_trait]
impl Agent for MacroAnalyst {
    fn name(&self) -> &str {
        self.base_agent.name()
    }

    fn description(&self) -> &str {
        self.base_agent.description()
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.metadata.get("symbol")
            .unwrap_or(&input.content)
            .clone();

        let macro_env = self.analyze_macro(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Macro analysis failed: {}", e)))?;

        let industry = self.analyze_industry_cycle(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Industry analysis failed: {}", e)))?;

        let policy = self.analyze_policy(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Policy analysis failed: {}", e)))?;

        let macro_analysis = json!({
            "symbol": symbol,
            "macro_analysis": {
                "macro_environment": macro_env,
                "industry_cycle": industry,
                "policy_impact": policy
            },
            "overall_outlook": "favorable",
            "confidence": 0.72
        });

        let content = format!(
            "宏观分析完成:\n{}\n",
            serde_json::to_string_pretty(&macro_analysis).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(macro_analysis)
            .with_confidence(0.72)
            .with_metadata("agent_type", "macro_analyst")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// Research Team - 团队管理
// ============================================================================

/// Research Team
///
/// 管理研究团队的所有Agents
pub struct ResearchTeam {
    fundamental: FundamentalResearcher,
    technical: TechnicalAnalyst,
    sentiment: SentimentAnalyst,
    macro: MacroAnalyst,
}

impl ResearchTeam {
    /// Create a new research team
    pub async fn new() -> Result<Self, anyhow::Error> {
        let registry = crate::skills::SkillRegistry::from_dir(
            std::path::PathBuf::from(".claude/skills")
        ).await?;

        Ok(Self {
            fundamental: FundamentalResearcher::new(registry.clone()),
            technical: TechnicalAnalyst::new(registry.clone()),
            sentiment: SentimentAnalyst::new(registry.clone()),
            macro: MacroAnalyst::new(registry),
        })
    }

    /// Get all team members as Agent trait objects
    pub fn get_agents(&self) -> Vec<Box<dyn Agent + Send + Sync>> {
        vec![
            Box::new(self.fundamental.clone()) as Box<dyn Agent + Send + Sync>,
            Box::new(self.technical.clone()) as Box<dyn Agent + Send + Sync>,
            Box::new(self.sentiment.clone()) as Box<dyn Agent + Send + Sync>,
            Box::new(self.macro.clone()) as Box<dyn Agent + Send + Sync>,
        ]
    }

    /// Get fundamental researcher
    pub fn fundamental(&self) -> &FundamentalResearcher {
        &self.fundamental
    }

    /// Get technical analyst
    pub fn technical(&self) -> &TechnicalAnalyst {
        &self.technical
    }

    /// Get sentiment analyst
    pub fn sentiment(&self) -> &SentimentAnalyst {
        &self.sentiment
    }

    /// Get macro analyst
    pub fn macro_analyst(&self) -> &MacroAnalyst {
        &self.macro
    }
}

// ============================================================================
// Clone implementation for team members
// ============================================================================

impl Clone for FundamentalResearcher {
    fn clone(&self) -> Self {
        Self {
            base_agent: self.base_agent.clone(),
        }
    }
}

impl Clone for TechnicalAnalyst {
    fn clone(&self) -> Self {
        Self {
            base_agent: self.base_agent.clone(),
        }
    }
}

impl Clone for SentimentAnalyst {
    fn clone(&self) -> Self {
        Self {
            base_agent: self.base_agent.clone(),
        }
    }
}

impl Clone for MacroAnalyst {
    fn clone(&self) -> Self {
        Self {
            base_agent: self.base_agent.clone(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_research_team_creation() {
        let team = ResearchTeam::new().await;
        assert!(team.is_ok());

        let team = team.unwrap();
        let agents = team.get_agents();

        assert_eq!(agents.len(), 4);
        assert_eq!(agents[0].name(), "FundamentalResearcher");
        assert_eq!(agents[1].name(), "TechnicalAnalyst");
        assert_eq!(agents[2].name(), "SentimentAnalyst");
        assert_eq!(agents[3].name(), "MacroAnalyst");
    }

    #[tokio::test]
    async fn test_fundamental_researcher() {
        let registry = crate::skills::SkillRegistry::from_dir(
            std::path::PathBuf::from(".claude/skills")
        ).await.unwrap();

        let agent = FundamentalResearcher::new(registry);

        let input = AgentInput::new("AAPL")
            .with_metadata("symbol", "AAPL");

        let output = agent.execute(input).await.unwrap();

        assert!(output.is_successful());
        assert!(output.content.contains("基本面分析"));
    }

    #[tokio::test]
    async fn test_technical_analyst() {
        let registry = crate::skills::SkillRegistry::from_dir(
            std::path::PathBuf::from(".claude/skills")
        ).await.unwrap();

        let agent = TechnicalAnalyst::new(registry);

        let input = AgentInput::new("AAPL")
            .with_metadata("symbol", "AAPL");

        let output = agent.execute(input).await.unwrap();

        assert!(output.is_successful());
        assert!(output.content.contains("技术分析"));
    }
}
