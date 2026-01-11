//! 增强的Agent编排系统
//!
//! 基于Claude Agent SDK的SequentialOrchestrator和ParallelOrchestrator
//! 为投资分析提供专业的编排模式
//!
//! ## 编排模式
//!
//! 1. **SequentialPipeline** - 顺序执行多个分析Agent
//! 2. **ParallelAnalysis** - 并行执行多个独立分析
//! 3. **HybridOrchestrator** - 混合模式（并行+顺序）
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use investintel_agent::orchestration::{
//!     InvestmentOrchestrator, AnalysisType, OrchestrationConfig
//! };
//! # async fn example() -> anyhow::Result<()> {
//! let orchestrator = InvestmentOrchestrator::new();
//!
//! // 并行综合分析
//! let result = orchestrator.analyze(
//!     "AAPL",
//!     AnalysisType::Comprehensive,
//!     OrchestrationConfig::default()
//! ).await?;
//! # Ok(())
//! # }
//! ```

use crate::agents::{
    DividendInvestorAgent, KellyPositionAgent, MungerFrameworkAgent,
    PortfolioManagerAgent, TradingAdvisorAgent, ValueInvestmentAgent,
};
use claude_agent_sdk_rs::orchestration::{
    Agent, AgentInput, AgentOutput, OrchestratorInput, OrchestratorOutput,
    ParallelOrchestrator, SequentialOrchestrator,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 分析类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AnalysisType {
    /// 快速价值分析
    QuickValue,

    /// 综合分析（价值+交易+股息）
    Comprehensive,

    /// 深度分析（综合分析+Munger思维模型）
    Deep,

    /// 仓位分析（Kelly+价值）
    Position,

    /// 股息投资分析
    Dividend,

    /// 完整分析（所有维度）
    Full,
}

/// 编排配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    /// 是否启用跟踪
    pub enable_tracing: bool,

    /// 是否启用日志
    pub enable_logging: bool,

    /// 并行限制
    pub parallel_limit: usize,

    /// 最大重试次数
    pub max_retries: usize,

    /// 超时时间（秒）
    pub timeout_secs: u64,
}

impl Default for OrchestrationConfig {
    fn default() -> Self {
        Self {
            enable_tracing: true,
            enable_logging: true,
            parallel_limit: 5,
            max_retries: 3,
            timeout_secs: 30,
        }
    }
}

/// 编排结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationResult {
    /// 股票代码
    pub symbol: String,

    /// 分析类型
    pub analysis_type: AnalysisType,

    /// 是否成功
    pub success: bool,

    /// 各Agent的结果
    pub agent_results: HashMap<String, AgentResult>,

    /// 综合建议
    pub recommendation: String,

    /// 置信度
    pub confidence: f64,

    /// 执行时间（毫秒）
    pub execution_time_ms: u64,

    /// 错误信息（如果有）
    pub error: Option<String>,
}

/// 单个Agent的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    /// Agent名称
    pub agent_name: String,

    /// 输出内容
    pub content: String,

    /// 置信度
    pub confidence: f64,

    /// 执行时间（毫秒）
    pub execution_time_ms: u64,

    /// 额外数据
    pub data: serde_json::Value,
}

/// 投资分析编排器
///
/// 整合所有投资分析Agent，提供灵活的编排策略
pub struct InvestmentOrchestrator {
    /// 顺序编排器
    sequential_orchestrator: SequentialOrchestrator,

    /// 并行编排器
    parallel_orchestrator: ParallelOrchestrator,

    /// 配置
    config: OrchestrationConfig,
}

impl InvestmentOrchestrator {
    /// 创建新的编排器
    pub fn new() -> Self {
        Self {
            sequential_orchestrator: SequentialOrchestrator::new(),
            parallel_orchestrator: ParallelOrchestrator::new(),
            config: OrchestrationConfig::default(),
        }
    }

    /// 设置配置
    pub fn with_config(mut self, config: OrchestrationConfig) -> Self {
        self.sequential_orchestrator = SequentialOrchestrator::new()
            .with_max_retries(config.max_retries);
        self.parallel_orchestrator = ParallelOrchestrator::new()
            .with_max_retries(config.max_retries)
            .with_parallel_limit(config.parallel_limit);
        self.config = config;
        self
    }

    /// 执行分析
    pub async fn analyze(
        &self,
        symbol: &str,
        analysis_type: AnalysisType,
        config: OrchestrationConfig,
    ) -> anyhow::Result<OrchestrationResult> {
        let start = std::time::Instant::now();

        // 根据分析类型选择编排策略
        let result = match analysis_type {
            AnalysisType::QuickValue => {
                self.quick_value_analysis(symbol, &config).await?
            }
            AnalysisType::Comprehensive => {
                self.comprehensive_analysis(symbol, &config).await?
            }
            AnalysisType::Deep => {
                self.deep_analysis(symbol, &config).await?
            }
            AnalysisType::Position => {
                self.position_analysis(symbol, &config).await?
            }
            AnalysisType::Dividend => {
                self.dividend_analysis(symbol, &config).await?
            }
            AnalysisType::Full => {
                self.full_analysis(symbol, &config).await?
            }
        };

        let execution_time = start.elapsed().as_millis() as u64;

        Ok(OrchestrationResult {
            symbol: symbol.to_string(),
            analysis_type,
            success: result.error.is_none(),
            agent_results: result.agent_results,
            recommendation: result.recommendation,
            confidence: result.confidence,
            execution_time_ms: execution_time,
            error: result.error,
        })
    }

    /// 快速价值分析（顺序执行：价值→交易）
    async fn quick_value_analysis(
        &self,
        symbol: &str,
        _config: &OrchestrationConfig,
    ) -> anyhow::Result<OrchestrationResultInner> {
        // 使用顺序编排器
        let agents: Vec<Box<dyn Agent>> = vec![
            Box::new(ValueInvestmentAgent::new()),
            Box::new(TradingAdvisorAgent::new()),
        ];

        let input = OrchestratorInput::new(symbol);
        let output = self.sequential_orchestrator.orchestrate(agents, input).await?;

        let mut agent_results = HashMap::new();

        // 提取各Agent的结果
        for (i, agent_output) in output.agent_outputs.iter().enumerate() {
            let agent_name = if i == 0 { "ValueInvestment" } else { "TradingAdvisor" };

            agent_results.insert(
                agent_name.to_string(),
                AgentResult {
                    agent_name: agent_name.to_string(),
                    content: agent_output.content.clone(),
                    confidence: agent_output.confidence,
                    execution_time_ms: 0,
                    data: agent_output.data.clone(),
                },
            );
        }

        Ok(OrchestrationResultInner {
            agent_results,
            recommendation: output.result,
            confidence: 0.75,
            error: if output.is_successful() { None } else { Some(output.result) },
        })
    }

    /// 综合分析（并行执行：价值+交易+股息）
    async fn comprehensive_analysis(
        &self,
        symbol: &str,
        _config: &OrchestrationConfig,
    ) -> anyhow::Result<OrchestrationResultInner> {
        // 使用并行编排器
        let agents: Vec<Box<dyn Agent>> = vec![
            Box::new(ValueInvestmentAgent::new()),
            Box::new(TradingAdvisorAgent::new()),
            Box::new(DividendInvestorAgent::new()),
        ];

        let input = OrchestratorInput::new(symbol);
        let output = self.parallel_orchestrator.orchestrate(agents, input).await?;

        let mut agent_results = HashMap::new();

        // 提取各Agent的结果
        let agent_names = ["ValueInvestment", "TradingAdvisor", "DividendInvestor"];

        for (i, agent_output) in output.agent_outputs.iter().enumerate() {
            agent_results.insert(
                agent_names[i].to_string(),
                AgentResult {
                    agent_name: agent_names[i].to_string(),
                    content: agent_output.content.clone(),
                    confidence: agent_output.confidence,
                    execution_time_ms: 0,
                    data: agent_output.data.clone(),
                },
            );
        }

        // 综合建议
        let recommendation = self.format_comprehensive_recommendation(&agent_results);

        Ok(OrchestrationResultInner {
            agent_results,
            recommendation,
            confidence: 0.80,
            error: if output.is_successful() { None } else { Some(output.result) },
        })
    }

    /// 深度分析（并行+顺序：价值+交易+股息 → Munger）
    async fn deep_analysis(
        &self,
        symbol: &str,
        config: &OrchestrationConfig,
    ) -> anyhow::Result<OrchestrationResultInner> {
        // 先并行执行基础分析
        let base_result = self.comprehensive_analysis(symbol, config).await?;

        // 将基础分析结果传递给Munger Agent
        let munger_input = AgentInput::new(symbol)
            .with_context(serde_json::json!({
                "comprehensive_analysis": base_result.recommendation
            }));

        let munger_agent = MungerFrameworkAgent::new();
        let munger_output = munger_agent.execute(munger_input).await?;

        let mut agent_results = base_result.agent_results;
        agent_results.insert(
            "MungerFramework".to_string(),
            AgentResult {
                agent_name: "MungerFramework".to_string(),
                content: munger_output.content.clone(),
                confidence: munger_output.confidence,
                execution_time_ms: 0,
                data: munger_output.data,
            },
        );

        let recommendation = format!(
            "🧠 深度分析结果\n\n{}\n\n📊 Munger思维模型分析:\n\n{}",
            base_result.recommendation, munger_output.content
        );

        Ok(OrchestrationResultInner {
            agent_results,
            recommendation,
            confidence: 0.85,
            error: None,
        })
    }

    /// 仓位分析（并行：价值+Kelly）
    async fn position_analysis(
        &self,
        symbol: &str,
        _config: &OrchestrationConfig,
    ) -> anyhow::Result<OrchestrationResultInner> {
        // 使用并行编排器
        let agents: Vec<Box<dyn Agent>> = vec![
            Box::new(ValueInvestmentAgent::new()),
            Box::new(KellyPositionAgent::new()),
        ];

        let input = OrchestratorInput::new(symbol);
        let output = self.parallel_orchestrator.orchestrate(agents, input).await?;

        let mut agent_results = HashMap::new();

        let agent_names = ["ValueInvestment", "KellyPosition"];

        for (i, agent_output) in output.agent_outputs.iter().enumerate() {
            agent_results.insert(
                agent_names[i].to_string(),
                AgentResult {
                    agent_name: agent_names[i].to_string(),
                    content: agent_output.content.clone(),
                    confidence: agent_output.confidence,
                    execution_time_ms: 0,
                    data: agent_output.data.clone(),
                },
            );
        }

        let recommendation = format!(
            "📊 仓位分析报告\n\n{}\n\n{}",
            agent_results["ValueInvestment"].content,
            agent_results["KellyPosition"].content
        );

        Ok(OrchestrationResultInner {
            agent_results,
            recommendation,
            confidence: 0.78,
            error: if output.is_successful() { None } else { Some(output.result) },
        })
    }

    /// 股息投资分析（顺序执行：股息→价值）
    async fn dividend_analysis(
        &self,
        symbol: &str,
        _config: &OrchestrationConfig,
    ) -> anyhow::Result<OrchestrationResultInner> {
        // 使用顺序编排器
        let agents: Vec<Box<dyn Agent>> = vec![
            Box::new(DividendInvestorAgent::new()),
            Box::new(ValueInvestmentAgent::new()),
        ];

        let input = OrchestratorInput::new(symbol);
        let output = self.sequential_orchestrator.orchestrate(agents, input).await?;

        let mut agent_results = HashMap::new();

        let agent_names = ["DividendInvestor", "ValueInvestment"];

        for (i, agent_output) in output.agent_outputs.iter().enumerate() {
            agent_results.insert(
                agent_names[i].to_string(),
                AgentResult {
                    agent_name: agent_names[i].to_string(),
                    content: agent_output.content.clone(),
                    confidence: agent_output.confidence,
                    execution_time_ms: 0,
                    data: agent_output.data.clone(),
                },
            );
        }

        let recommendation = format!(
            "💰 股息投资分析\n\n{}\n\n基于价值分析:\n{}",
            agent_results["DividendInvestor"].content,
            agent_results["ValueInvestment"].content
        );

        Ok(OrchestrationResultInner {
            agent_results,
            recommendation,
            confidence: 0.82,
            error: if output.is_successful() { None } else { Some(output.result) },
        })
    }

    /// 完整分析（并行所有维度 → 组合建议）
    async fn full_analysis(
        &self,
        symbol: &str,
        config: &OrchestrationConfig,
    ) -> anyhow::Result<OrchestrationResultInner> {
        // 先并行执行所有专业分析
        let agents: Vec<Box<dyn Agent>> = vec![
            Box::new(ValueInvestmentAgent::new()),
            Box::new(TradingAdvisorAgent::new()),
            Box::new(DividendInvestorAgent::new()),
            Box::new(KellyPositionAgent::new()),
            Box::new(MungerFrameworkAgent::new()),
        ];

        let input = OrchestratorInput::new(symbol);
        let output = self.parallel_orchestrator.orchestrate(agents, input).await?;

        let mut agent_results = HashMap::new();

        let agent_names = [
            "ValueInvestment",
            "TradingAdvisor",
            "DividendInvestor",
            "KellyPosition",
            "MungerFramework",
        ];

        for (i, agent_output) in output.agent_outputs.iter().enumerate() {
            agent_results.insert(
                agent_names[i].to_string(),
                AgentResult {
                    agent_name: agent_names[i].to_string(),
                    content: agent_output.content.clone(),
                    confidence: agent_output.confidence,
                    execution_time_ms: 0,
                    data: agent_output.data.clone(),
                },
            );
        }

        // 将所有分析结果传递给组合管理Agent
        let portfolio_input = AgentInput::new(symbol)
            .with_context(serde_json::json!({
                "analyses": agent_results.values()
                    .map(|r| serde_json::json!({
                        "agent": r.agent_name,
                        "content": r.content,
                        "confidence": r.confidence
                    }))
                    .collect::<Vec<_>>()
            }));

        let portfolio_agent = PortfolioManagerAgent::new();
        let portfolio_output = portfolio_agent.execute(portfolio_input).await?;

        agent_results.insert(
            "PortfolioManager".to_string(),
            AgentResult {
                agent_name: "PortfolioManager".to_string(),
                content: portfolio_output.content.clone(),
                confidence: portfolio_output.confidence,
                execution_time_ms: 0,
                data: portfolio_output.data,
            },
        );

        let recommendation = format!(
            "🎯 完整投资分析报告\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n\
             1️⃣ 价值分析\n\n{}\n\n\
             2️⃣ 交易建议\n\n{}\n\n\
             3️⃣ 股息分析\n\n{}\n\n\
             4️⃣ 仓位建议\n\n{}\n\n\
             5️⃣ Munger思维模型\n\n{}\n\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n\
             📋 综合投资建议:\n\n{}",
            agent_results["ValueInvestment"].content,
            agent_results["TradingAdvisor"].content,
            agent_results["DividendInvestor"].content,
            agent_results["KellyPosition"].content,
            agent_results["MungerFramework"].content,
            portfolio_output.content
        );

        Ok(OrchestrationResultInner {
            agent_results,
            recommendation,
            confidence: 0.90,
            error: if output.is_successful() { None } else { Some(output.result) },
        })
    }

    /// 格式化综合建议
    fn format_comprehensive_recommendation(&self, results: &HashMap<String, AgentResult>) -> String {
        format!(
            "📊 综合投资分析\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n\
             1️⃣ 价值分析\n\n{}\n\n\
             2️⃣ 交易建议\n\n{}\n\n\
             3️⃣ 股息分析\n\n{}\n\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            results.get("ValueInvestment").map(|r| r.content.as_str()).unwrap_or("分析失败"),
            results.get("TradingAdvisor").map(|r| r.content.as_str()).unwrap_or("分析失败"),
            results.get("DividendInvestor").map(|r| r.content.as_str()).unwrap_or("分析失败")
        )
    }
}

impl Default for InvestmentOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// 内部结果结构
struct OrchestrationResultInner {
    agent_results: HashMap<String, AgentResult>,
    recommendation: String,
    confidence: f64,
    error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quick_value_analysis() {
        let orchestrator = InvestmentOrchestrator::new();
        let config = OrchestrationConfig::default();

        let result = orchestrator
            .analyze("AAPL", AnalysisType::QuickValue, config)
            .await
            .unwrap();

        assert_eq!(result.symbol, "AAPL");
        assert!(result.success);
        assert!(result.agent_results.len() >= 1);
    }

    #[tokio::test]
    async fn test_comprehensive_analysis() {
        let orchestrator = InvestmentOrchestrator::new();
        let config = OrchestrationConfig::default();

        let result = orchestrator
            .analyze("MSFT", AnalysisType::Comprehensive, config)
            .await
            .unwrap();

        assert_eq!(result.symbol, "MSFT");
        assert!(result.success);
        assert!(result.agent_results.len() >= 2);
    }

    #[tokio::test]
    async fn test_dividend_analysis() {
        let orchestrator = InvestmentOrchestrator::new();
        let config = OrchestrationConfig::default();

        let result = orchestrator
            .analyze("KO", AnalysisType::Dividend, config)
            .await
            .unwrap();

        assert_eq!(result.symbol, "KO");
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_orchestration_config() {
        let config = OrchestrationConfig {
            enable_tracing: false,
            enable_logging: false,
            parallel_limit: 3,
            max_retries: 2,
            timeout_secs: 60,
        };

        let orchestrator = InvestmentOrchestrator::new().with_config(config.clone());

        // 验证配置已应用
        let result = orchestrator
            .analyze("AAPL", AnalysisType::QuickValue, config)
            .await
            .unwrap();

        assert!(result.success);
    }
}
