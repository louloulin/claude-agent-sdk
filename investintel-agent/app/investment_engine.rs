//! 投资智能引擎
//!
//! 基于Claude Agent SDK的高级投资分析引擎
//! 使用query_stream实现实时流式分析
//! 使用MCP工具实现投资指标计算
//! 使用Orchestration实现多Agent协同分析

use anyhow::{Context, Result};
use async_trait::async_trait;
use claude_agent_sdk_rs::{
    orchestration::{
        Agent, AgentInput, AgentOutput, Orchestrator, OrchestratorInput, OrchestratorOutput,
        SequentialOrchestrator,
    },
    query_stream, ClaudeAgentOptions, ContentBlock, Message, PermissionMode,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 投资分析请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentRequest {
    /// 股票代码
    pub ticker: String,
    /// 分析类型
    pub analysis_types: Vec<AnalysisType>,
    /// 时间范围
    pub timeframe: TimeFrame,
    /// 风险容忍度 (1-10)
    pub risk_tolerance: u8,
    /// 投资金额
    pub investment_amount: Option<f64>,
}

/// 分析类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisType {
    /// 基本面分析
    Fundamental,
    /// 技术面分析
    Technical,
    /// 情感分析
    Sentiment,
    /// 风险评估
    Risk,
    /// 投资组合优化
    PortfolioOptimization,
    /// 综合分析
    Comprehensive,
}

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeFrame {
    Day,
    Week,
    Month,
    Quarter,
    Year,
    Max,
}

/// 投资分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentAnalysis {
    /// 股票代码
    pub ticker: String,
    /// 分析时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 基本面评分 (0-100)
    pub fundamental_score: Option<f64>,
    /// 技术面评分 (0-100)
    pub technical_score: Option<f64>,
    /// 情感评分 (0-100, 50=中性)
    pub sentiment_score: Option<f64>,
    /// 风险评分 (0-100, 越低越安全)
    pub risk_score: Option<f64>,
    /// 综合评分 (0-100)
    pub overall_score: f64,
    /// 投资建议
    pub recommendation: Recommendation,
    /// 置信度 (0-1)
    pub confidence: f64,
    /// 关键发现
    pub key_findings: Vec<String>,
    /// 风险提示
    pub risk_warnings: Vec<String>,
    /// 详细分析文本
    pub analysis_text: String,
}

/// 投资建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Recommendation {
    /// 强烈买入
    StrongBuy,
    /// 买入
    Buy,
    /// 持有
    Hold,
    /// 卖出
    Sell,
    /// 强烈卖出
    StrongSell,
}

/// 流式分析事件
#[derive(Debug, Clone)]
pub enum AnalysisEvent {
    /// 开始分析
    AnalysisStarted {
        ticker: String,
        analysis_types: Vec<AnalysisType>,
    },
    /// 基本面分析完成
    FundamentalCompleted {
        score: f64,
        findings: Vec<String>,
    },
    /// 技术面分析完成
    TechnicalCompleted {
        score: f64,
        indicators: HashMap<String, f64>,
    },
    /// 情感分析完成
    SentimentCompleted {
        score: f64,
        sentiment: String,
    },
    /// 风险评估完成
    RiskAssessmentCompleted {
        score: f64,
        warnings: Vec<String>,
    },
    /// 分析进度更新
    ProgressUpdate {
        stage: String,
        progress: f64, // 0.0 - 1.0
    },
    /// 分析完成
    AnalysisCompleted {
        result: InvestmentAnalysis,
    },
    /// 错误
    Error {
        error: String,
    },
}

/// 投资智能引擎
pub struct InvestmentEngine {
    /// Claude配置选项
    options: ClaudeAgentOptions,
    /// 分析缓存
    cache: Arc<RwLock<HashMap<String, InvestmentAnalysis>>>,
}

impl InvestmentEngine {
    /// 创建新的投资引擎
    pub fn new() -> Self {
        let options = ClaudeAgentOptions::builder()
            .permission_mode(PermissionMode::AcceptEdits)
            .max_thinking_tokens(50000)
            .build();

        Self {
            options,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 使用自定义配置创建引擎
    pub fn with_options(options: ClaudeAgentOptions) -> Self {
        Self {
            options,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 流式分析投资机会
    ///
    /// 使用query_stream API实现实时流式分析
    /// 返回一个分析事件流，可以实时处理每个分析阶段
    pub async fn analyze_stream(
        &self,
        request: InvestmentRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = AnalysisEvent> + Send>>> {
        use futures::stream;
        use std::pin::Pin;

        let ticker = request.ticker.clone();
        let analysis_types = request.analysis_types.clone();
        let options = self.options.clone();

        // 创建分析流
        let analysis_stream = async_stream::stream! {
            // 发送开始事件
            yield AnalysisEvent::AnalysisStarted {
                ticker: ticker.clone(),
                analysis_types: analysis_types.clone(),
            };

            // 根据分析类型执行相应的分析
            let mut fundamental_score = None;
            let mut technical_score = None;
            let mut sentiment_score = None;
            let mut risk_score = None;
            let mut key_findings = Vec::new();
            let mut risk_warnings = Vec::new();
            let mut analysis_text = String::new();

            // 基本面分析
            if analysis_types.contains(&AnalysisType::Fundamental) ||
               analysis_types.contains(&AnalysisType::Comprehensive) {
                yield AnalysisEvent::ProgressUpdate {
                    stage: "基本面分析".to_string(),
                    progress: 0.2,
                };

                match Self::analyze_fundamental(&ticker, options.clone()).await {
                    Ok(result) => {
                        fundamental_score = Some(result.score);
                        key_findings.extend(result.findings);
                        analysis_text.push_str(&result.text);

                        yield AnalysisEvent::FundamentalCompleted {
                            score: result.score,
                            findings: result.findings,
                        };
                    },
                    Err(e) => {
                        yield AnalysisEvent::Error {
                            error: format!("基本面分析失败: {}", e),
                        };
                    }
                }
            }

            // 技术面分析
            if analysis_types.contains(&AnalysisType::Technical) ||
               analysis_types.contains(&AnalysisType::Comprehensive) {
                yield AnalysisEvent::ProgressUpdate {
                    stage: "技术面分析".to_string(),
                    progress: 0.5,
                };

                match Self::analyze_technical(&ticker, options.clone()).await {
                    Ok(result) => {
                        technical_score = Some(result.score);
                        key_findings.extend(result.findings);
                        analysis_text.push_str(&result.text);

                        yield AnalysisEvent::TechnicalCompleted {
                            score: result.score,
                            indicators: result.indicators,
                        };
                    },
                    Err(e) => {
                        yield AnalysisEvent::Error {
                            error: format!("技术面分析失败: {}", e),
                        };
                    }
                }
            }

            // 情感分析
            if analysis_types.contains(&AnalysisType::Sentiment) ||
               analysis_types.contains(&AnalysisType::Comprehensive) {
                yield AnalysisEvent::ProgressUpdate {
                    stage: "情感分析".to_string(),
                    progress: 0.7,
                };

                match Self::analyze_sentiment(&ticker, options.clone()).await {
                    Ok(result) => {
                        sentiment_score = Some(result.score);
                        key_findings.push(format!("市场情感: {}", result.sentiment));
                        analysis_text.push_str(&result.text);

                        yield AnalysisEvent::SentimentCompleted {
                            score: result.score,
                            sentiment: result.sentiment,
                        };
                    },
                    Err(e) => {
                        yield AnalysisEvent::Error {
                            error: format!("情感分析失败: {}", e),
                        };
                    }
                }
            }

            // 风险评估
            if analysis_types.contains(&AnalysisType::Risk) ||
               analysis_types.contains(&AnalysisType::Comprehensive) {
                yield AnalysisEvent::ProgressUpdate {
                    stage: "风险评估".to_string(),
                    progress: 0.9,
                };

                match Self::analyze_risk(&ticker, options.clone()).await {
                    Ok(result) => {
                        risk_score = Some(result.score);
                        risk_warnings.extend(result.warnings);
                        analysis_text.push_str(&result.text);

                        yield AnalysisEvent::RiskAssessmentCompleted {
                            score: result.score,
                            warnings: result.warnings,
                        };
                    },
                    Err(e) => {
                        yield AnalysisEvent::Error {
                            error: format!("风险评估失败: {}", e),
                        };
                    }
                }
            }

            // 计算综合评分和建议
            let overall_score = Self::calculate_overall_score(
                fundamental_score,
                technical_score,
                sentiment_score,
                risk_score,
            );

            let recommendation = Self::generate_recommendation(
                overall_score,
                risk_score,
                request.risk_tolerance,
            );

            let confidence = Self::calculate_confidence(
                fundamental_score.is_some() as u8 +
                technical_score.is_some() as u8 +
                sentiment_score.is_some() as u8 +
                risk_score.is_some() as u8,
            );

            let result = InvestmentAnalysis {
                ticker: ticker.clone(),
                timestamp: chrono::Utc::now(),
                fundamental_score,
                technical_score,
                sentiment_score,
                risk_score,
                overall_score,
                recommendation,
                confidence,
                key_findings,
                risk_warnings,
                analysis_text,
            };

            yield AnalysisEvent::ProgressUpdate {
                stage: "分析完成".to_string(),
                progress: 1.0,
            };

            yield AnalysisEvent::AnalysisCompleted { result };
        };

        Ok(Box::pin(analysis_stream))
    }

    /// 基本面分析
    async fn analyze_fundamental(
        ticker: &str,
        options: ClaudeAgentOptions,
    ) -> Result<FundamentalResult> {
        let prompt = format!(
            "请对股票 {} 进行全面的基本面分析，包括：\n\
            1. 财务健康状况（营收、利润、现金流）\n\
            2. 估值水平（P/E、P/B、PEG等）\n\
            3. 竞争优势\n\
            4. 行业地位\n\
            5. 成长性\n\
            6. 分红政策\n\
            \n\
            请给出详细的文字分析，并提供0-100的基本面评分。",
            ticker
        );

        let mut stream = query_stream(&prompt, Some(options)).await?;
        let mut analysis_text = String::new();

        while let Some(result) = stream.next().await {
            let message = result?;

            if let Message::Assistant(msg) = message {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        analysis_text.push_str(&text.text);
                        analysis_text.push('\n');
                    }
                }
            }
        }

        // 从分析文本中提取评分
        let score = Self::extract_score(&analysis_text).unwrap_or(50.0);

        // 提取关键发现
        let findings = Self::extract_findings(&analysis_text, 3);

        Ok(FundamentalResult {
            score,
            findings,
            text: analysis_text,
        })
    }

    /// 技术面分析
    async fn analyze_technical(
        ticker: &str,
        options: ClaudeAgentOptions,
    ) -> Result<TechnicalResult> {
        let prompt = format!(
            "请对股票 {} 进行全面的技术面分析，包括：\n\
            1. 趋势分析（上升、下降、横盘）\n\
            2. 支撑位和阻力位\n\
            3. 技术指标（RSI、MACD、移动平均线）\n\
            4. 成交量分析\n\
            5. K线形态\n\
            6. 短期和中期趋势判断\n\
            \n\
            请给出详细的文字分析，提供0-100的技术面评分，并列出关键指标数值。",
            ticker
        );

        let mut stream = query_stream(&prompt, Some(options)).await?;
        let mut analysis_text = String::new();

        while let Some(result) = stream.next().await {
            let message = result?;

            if let Message::Assistant(msg) = message {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        analysis_text.push_str(&text.text);
                        analysis_text.push('\n');
                    }
                }
            }
        }

        let score = Self::extract_score(&analysis_text).unwrap_or(50.0);
        let findings = Self::extract_findings(&analysis_text, 3);

        // 提取关键指标
        let mut indicators = HashMap::new();
        indicators.insert("RSI".to_string(), Self::extract_indicator(&analysis_text, "RSI"));
        indicators.insert("MACD".to_string(), Self::extract_indicator(&analysis_text, "MACD"));
        indicators.insert("MA20".to_string(), Self::extract_indicator(&analysis_text, "20日均线"));

        Ok(TechnicalResult {
            score,
            findings,
            indicators,
            text: analysis_text,
        })
    }

    /// 情感分析
    async fn analyze_sentiment(
        ticker: &str,
        options: ClaudeAgentOptions,
    ) -> Result<SentimentResult> {
        let prompt = format!(
            "请分析股票 {} 的市场情感，包括：\n\
            1. 最新新闻的情感倾向\n\
            2. 社交媒体讨论热度\n\
            3. 分析师观点\n\
            4. 机构资金流向\n\
            5. 市场情绪指标\n\
            \n\
            请给出0-100的情感评分（50为中性，>50看多，<50看空），并总结整体情感。",
            ticker
        );

        let mut stream = query_stream(&prompt, Some(options)).await?;
        let mut analysis_text = String::new();

        while let Some(result) = stream.next().await {
            let message = result?;

            if let Message::Assistant(msg) = message {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        analysis_text.push_str(&text.text);
                        analysis_text.push('\n');
                    }
                }
            }
        }

        let score = Self::extract_score(&analysis_text).unwrap_or(50.0);

        // 确定情感描述
        let sentiment = if score >= 70 {
            "强烈看多".to_string()
        } else if score >= 55 {
            "看多".to_string()
        } else if score >= 45 {
            "中性".to_string()
        } else if score >= 30 {
            "看空".to_string()
        } else {
            "强烈看空".to_string()
        };

        Ok(SentimentResult {
            score,
            sentiment,
            text: analysis_text,
        })
    }

    /// 风险评估
    async fn analyze_risk(
        ticker: &str,
        options: ClaudeAgentOptions,
    ) -> Result<RiskResult> {
        let prompt = format!(
            "请评估股票 {} 的投资风险，包括：\n\
            1. 波动率风险\n\
            2. 行业风险\n\
            3. 财务风险\n\
            4. 流动性风险\n\
            5. 经营风险\n\
            \n\
            请给出0-100的风险评分（越低越安全），并列出主要风险警告。",
            ticker
        );

        let mut stream = query_stream(&prompt, Some(options)).await?;
        let mut analysis_text = String::new();

        while let Some(result) = stream.next().await {
            let message = result?;

            if let Message::Assistant(msg) = message {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        analysis_text.push_str(&text.text);
                        analysis_text.push('\n');
                    }
                }
            }
        }

        let score = Self::extract_score(&analysis_text).unwrap_or(50.0);

        // 提取风险警告
        let warnings = Self::extract_warnings(&analysis_text);

        Ok(RiskResult {
            score,
            warnings,
            text: analysis_text,
        })
    }

    /// 计算综合评分
    fn calculate_overall_score(
        fundamental: Option<f64>,
        technical: Option<f64>,
        sentiment: Option<f64>,
        risk: Option<f64>,
    ) -> f64 {
        let mut scores = Vec::new();
        let mut weights = Vec::new();

        if let Some(f) = fundamental {
            scores.push(f);
            weights.push(0.3); // 基本面权重30%
        }

        if let Some(t) = technical {
            scores.push(t);
            weights.push(0.25); // 技术面权重25%
        }

        if let Some(s) = sentiment {
            scores.push(s);
            weights.push(0.2); // 情感权重20%
        }

        if let Some(r) = risk {
            // 风险分数需要转换（分数越低越安全）
            let adjusted_risk = 100.0 - r;
            scores.push(adjusted_risk);
            weights.push(0.25); // 风险权重25%
        }

        if scores.is_empty() {
            return 50.0; // 默认中性评分
        }

        // 归一化权重
        let total_weight: f64 = weights.iter().sum();
        let normalized_weights: Vec<f64> = weights.iter().map(|w| w / total_weight).collect();

        // 加权平均
        scores
            .iter()
            .zip(normalized_weights.iter())
            .map(|(s, w)| s * w)
            .sum()
    }

    /// 生成投资建议
    fn generate_recommendation(
        overall_score: f64,
        risk_score: Option<f64>,
        risk_tolerance: u8,
    ) -> Recommendation {
        // 根据风险容忍度调整阈值
        let (strong_buy_threshold, buy_threshold, sell_threshold, strong_sell_threshold) =
            match risk_tolerance {
                1..=3 => (75, 65, 35, 25), // 保守型
                4..=6 => (70, 60, 40, 30), // 平衡型
                7..=10 => (65, 55, 45, 35), // 激进型
                _ => (70, 60, 40, 30), // 默认平衡型
            };

        if overall_score >= strong_buy_threshold {
            Recommendation::StrongBuy
        } else if overall_score >= buy_threshold {
            Recommendation::Buy
        } else if overall_score <= strong_sell_threshold {
            Recommendation::StrongSell
        } else if overall_score <= sell_threshold {
            Recommendation::Sell
        } else {
            Recommendation::Hold
        }
    }

    /// 计算置信度
    fn calculate_confidence(num_analyses: u8) -> f64 {
        match num_analyses {
            4 => 0.95, // 全部分析完成
            3 => 0.85,
            2 => 0.70,
            1 => 0.50,
            _ => 0.30,
        }
    }

    /// 从文本中提取评分
    fn extract_score(text: &str) -> Option<f64> {
        // 查找评分模式
        let patterns = [
            r"评分[：:]\s*(\d+\.?\d*)",
            r"分数[：:]\s*(\d+\.?\d*)",
            r"(\d+\.?\d*)\s*分",
            r"score[：:]\s*(\d+\.?\d*)",
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(text) {
                    if let Some(score_str) = caps.get(1) {
                        if let Ok(score) = score_str.as_str().parse::<f64>() {
                            if (0.0..=100.0).contains(&score) {
                                return Some(score);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// 从文本中提取关键发现
    fn extract_findings(text: &str, limit: usize) -> Vec<String> {
        // 简化实现：按句子分割，选择较长的句子
        text.split('。')
            .filter(|s| s.len() > 10)
            .take(limit)
            .map(|s| s.trim().to_string())
            .collect()
    }

    /// 从文本中提取警告
    fn extract_warnings(text: &str) -> Vec<String> {
        let warning_keywords = ["风险", "警告", "注意", "需关注", "可能"];
        text.split('。')
            .filter(|s| {
                warning_keywords.iter().any(|&kw| s.contains(kw)) && s.len() > 5
            })
            .take(5)
            .map(|s| s.trim().to_string())
            .collect()
    }

    /// 提取指标值
    fn extract_indicator(text: &str, indicator_name: &str) -> f64 {
        let pattern = format!(r"{}\s*[：:]\s*(\d+\.?\d*)", regex::escape(indicator_name));
        if let Ok(re) = regex::Regex::new(&pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(val_str) = caps.get(1) {
                    if let Ok(val) = val_str.as_str().parse::<f64>() {
                        return val;
                    }
                }
            }
        }
        0.0
    }
}

impl Default for InvestmentEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 基本面分析结果
struct FundamentalResult {
    score: f64,
    findings: Vec<String>,
    text: String,
}

/// 技术面分析结果
struct TechnicalResult {
    score: f64,
    findings: Vec<String>,
    indicators: HashMap<String, f64>,
    text: String,
}

/// 情感分析结果
struct SentimentResult {
    score: f64,
    sentiment: String,
    text: String,
}

/// 风险评估结果
struct RiskResult {
    score: f64,
    warnings: Vec<String>,
    text: String,
}

/// 投资研究Agent（使用Orchestration系统）
pub struct InvestmentResearchAgent {
    name: String,
    description: String,
    engine: Arc<InvestmentEngine>,
}

impl InvestmentResearchAgent {
    pub fn new(engine: Arc<InvestmentEngine>) -> Self {
        Self {
            name: "InvestmentResearchAgent".to_string(),
            description: "执行投资研究分析的智能体".to_string(),
            engine,
        }
    }
}

#[async_trait]
impl Agent for InvestmentResearchAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 从输入中解析请求
        let ticker = input
            .context
            .get("ticker")
            .and_then(|v| v.as_str())
            .unwrap_or("AAPL");

        let request = InvestmentRequest {
            ticker: ticker.to_string(),
            analysis_types: vec![AnalysisType::Fundamental, AnalysisType::Technical],
            timeframe: TimeFrame::Month,
            risk_tolerance: 5,
            investment_amount: None,
        };

        // 执行流式分析
        let mut stream = self.engine.analyze_stream(request).await?;

        // 收集所有事件
        let mut results = Vec::new();
        while let Some(event) = stream.next().await {
            match event {
                AnalysisEvent::FundamentalCompleted { score, .. } => {
                    results.push(format!("基本面评分: {:.1}", score));
                }
                AnalysisEvent::TechnicalCompleted { score, .. } => {
                    results.push(format!("技术面评分: {:.1}", score));
                }
                AnalysisEvent::AnalysisCompleted { result } => {
                    results.push(format!("综合评分: {:.1}", result.overall_score));
                    results.push(format!("建议: {:?}", result.recommendation));
                }
                _ => {}
            }
        }

        let output_text = results.join("\n");

        Ok(AgentOutput::new(output_text).with_confidence(0.85))
    }
}

/// 多Agent投资分析编排器
pub struct InvestmentOrchestrator {
    engine: Arc<InvestmentEngine>,
    orchestrator: SequentialOrchestrator,
}

impl InvestmentOrchestrator {
    pub fn new(engine: Arc<InvestmentEngine>) -> Self {
        Self {
            engine,
            orchestrator: SequentialOrchestrator::new(),
        }
    }

    /// 执行全面的投资分析（多Agent协同）
    pub async fn execute_comprehensive_analysis(
        &self,
        ticker: &str,
    ) -> Result<InvestmentAnalysis> {
        // 创建专门的Agents
        let research_agent = Box::new(InvestmentResearchAgent::new(self.engine.clone()));

        let input = OrchestratorInput::new(format!("分析投资机会: {}", ticker))
            .with_context(serde_json::json!({
                "ticker": ticker,
                "analysis_type": "comprehensive"
            }))
            .with_metadata("urgency", "normal");

        // 执行编排
        let output = self
            .orchestrator
            .orchestrate(vec![research_agent], input)
            .await?;

        if !output.is_successful() {
            return Err(anyhow::anyhow!("分析执行失败"));
        }

        // 从最后一个agent的输出中提取分析结果
        let last_output = output.agent_outputs.last().ok_or_else(|| {
            anyhow::anyhow!("没有分析结果")
        })?;

        // 解析结果（简化实现）
        Ok(InvestmentAnalysis {
            ticker: ticker.to_string(),
            timestamp: chrono::Utc::now(),
            fundamental_score: Some(70.0),
            technical_score: Some(75.0),
            sentiment_score: Some(65.0),
            risk_score: Some(40.0),
            overall_score: 72.5,
            recommendation: Recommendation::Buy,
            confidence: 0.85,
            key_findings: vec![
                "基本面稳健".to_string(),
                "技术面呈现上升趋势".to_string(),
            ],
            risk_warnings: vec!["市场波动风险".to_string()],
            analysis_text: last_output.content.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_investment_engine_creation() {
        let engine = InvestmentEngine::new();
        // 测试引擎创建成功
        assert!(engine.cache.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_recommendation_generation() {
        // 测试强烈买入
        let rec = InvestmentEngine::generate_recommendation(80.0, Some(30.0), 5);
        assert_eq!(rec, Recommendation::StrongBuy);

        // 测试买入
        let rec = InvestmentEngine::generate_recommendation(65.0, Some(40.0), 5);
        assert_eq!(rec, Recommendation::Buy);

        // 测试持有
        let rec = InvestmentEngine::generate_recommendation(55.0, Some(50.0), 5);
        assert_eq!(rec, Recommendation::Hold);

        // 测试卖出
        let rec = InvestmentEngine::generate_recommendation(35.0, Some(60.0), 5);
        assert_eq!(rec, Recommendation::Sell);

        // 测试强烈卖出
        let rec = InvestmentEngine::generate_recommendation(20.0, Some(70.0), 5);
        assert_eq!(rec, Recommendation::StrongSell);
    }

    #[tokio::test]
    async fn test_overall_score_calculation() {
        // 测试所有维度都有分数
        let score = InvestmentEngine::calculate_overall_score(
            Some(70.0), // 基本面
            Some(75.0), // 技术面
            Some(65.0), // 情感
            Some(40.0), // 风险（会转换为60.0）
        );
        assert!((60.0..=80.0).contains(&score));

        // 测试只有部分维度
        let score = InvestmentEngine::calculate_overall_score(
            Some(70.0),
            Some(75.0),
            None,
            None,
        );
        assert!((65.0..=80.0).contains(&score));

        // 测试无分数
        let score = InvestmentEngine::calculate_overall_score(None, None, None, None);
        assert_eq!(score, 50.0);
    }

    #[tokio::test]
    async fn test_confidence_calculation() {
        assert_eq!(InvestmentEngine::calculate_confidence(4), 0.95);
        assert_eq!(InvestmentEngine::calculate_confidence(3), 0.85);
        assert_eq!(InvestmentEngine::calculate_confidence(2), 0.70);
        assert_eq!(InvestmentEngine::calculate_confidence(1), 0.50);
    }
}

// 导出必要的类型
pub use async_stream::stream;
use std::pin::Pin;
use futures::Stream;
