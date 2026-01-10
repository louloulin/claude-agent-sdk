//! 高级投资策略引擎
//!
//! 基于Claude Agent SDK的多层次策略系统
//! 使用query_stream实现实时策略分析
//! 使用Agent trait实现策略Agent
//! 使用Orchestrator实现策略组合

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use claude_agent_sdk_rs::{
    orchestration::{
        Agent, AgentInput, AgentOutput, Orchestrator, OrchestratorInput, OrchestratorOutput,
        ParallelOrchestrator, SequentialOrchestrator,
    },
    query_stream, ClaudeAgentOptions, ContentBlock, Message, PermissionMode,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 策略类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrategyType {
    /// 趋势跟踪
    TrendFollowing,
    /// 均值回归
    MeanReversion,
    /// 动量策略
    Momentum,
    /// 价值投资
    ValueInvesting,
    /// 成长投资
    GrowthInvesting,
    /// 分红策略
    DividendStrategy,
    /// 事件驱动
    EventDriven,
    /// 量化套利
    QuantitativeArbitrage,
    /// 自定义策略
    Custom(String),
}

/// 策略信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySignal {
    /// 策略ID
    pub strategy_id: String,
    /// 股票代码
    pub ticker: String,
    /// 信号类型
    pub signal_type: SignalType,
    /// 信号强度 (0-100)
    pub strength: f64,
    /// 信号方向
    pub direction: SignalDirection,
    /// 入场价格
    pub entry_price: Option<f64>,
    /// 目标价格
    pub target_price: Option<f64>,
    /// 止损价格
    pub stop_loss: Option<f64>,
    /// 持有期建议
    pub holding_period: Option<Duration>,
    /// 仓位大小建议 (0-1, 表示占总资金比例)
    pub position_size: f64,
    /// 置信度 (0-1)
    pub confidence: f64,
    /// 生成时间
    pub timestamp: DateTime<Utc>,
    /// 信号原因
    pub reasoning: String,
    /// 风险提示
    pub risk_warnings: Vec<String>,
}

/// 信号类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalType {
    /// 入场信号
    Entry,
    /// 出场信号
    Exit,
    /// 加仓信号
    AddPosition,
    /// 减仓信号
    ReducePosition,
    /// 观望
    Hold,
    /// 观察名单
    Watch,
}

/// 信号方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalDirection {
    /// 做多
    Long,
    /// 做空
    Short,
    /// 中性
    Neutral,
}

/// 策略回测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    /// 策略ID
    pub strategy_id: String,
    /// 策略名称
    pub strategy_name: String,
    /// 回测期间
    pub period: (DateTime<Utc>, DateTime<Utc>),
    /// 初始资金
    pub initial_capital: f64,
    /// 最终资金
    pub final_capital: f64,
    /// 总收益率
    pub total_return: f64,
    /// 年化收益率
    pub annualized_return: f64,
    /// 夏普比率
    pub sharpe_ratio: f64,
    /// 最大回撤
    pub max_drawdown: f64,
    /// 胜率
    pub win_rate: f64,
    /// 盈亏比
    pub profit_loss_ratio: f64,
    /// 总交易次数
    pub total_trades: usize,
    /// 盈利交易次数
    pub winning_trades: usize,
    /// 亏损交易次数
    pub losing_trades: usize,
    /// 平均盈利
    pub avg_win: f64,
    /// 平均亏损
    pub avg_loss: f64,
    /// 最大盈利
    pub max_win: f64,
    /// 最大亏损
    pub max_loss: f64,
    /// 波动率
    pub volatility: f64,
    /// 索提诺比率
    pub sortino_ratio: f64,
    /// 卡玛比率
    pub calmar_ratio: f64,
}

/// 策略参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyParameters {
    /// 参数名称到值的映射
    pub params: HashMap<String, f64>,
    /// 参数描述
    pub descriptions: HashMap<String, String>,
}

impl StrategyParameters {
    /// 创建新的策略参数
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
            descriptions: HashMap::new(),
        }
    }

    /// 添加参数
    pub fn add(mut self, name: impl Into<String>, value: f64, desc: impl Into<String>) -> Self {
        let name = name.into();
        self.descriptions.insert(name.clone(), desc.into());
        self.params.insert(name, value);
        self
    }

    /// 获取参数值
    pub fn get(&self, name: &str) -> Option<f64> {
        self.params.get(name).copied()
    }

    /// 更新参数值
    pub fn set(&mut self, name: impl Into<String>, value: f64) {
        self.params.insert(name.into(), value);
    }
}

impl Default for StrategyParameters {
    fn default() -> Self {
        Self::new()
    }
}

/// 策略Agent trait
#[async_trait]
pub trait StrategyAgent: Send + Sync {
    /// 获取策略ID
    fn id(&self) -> &str;

    /// 获取策略名称
    fn name(&self) -> &str;

    /// 获取策略类型
    fn strategy_type(&self) -> StrategyType;

    /// 获取策略参数
    fn parameters(&self) -> &StrategyParameters;

    /// 设置策略参数
    async fn set_parameters(&mut self, params: StrategyParameters) -> Result<()>;

    /// 生成信号
    async fn generate_signal(&self, ticker: &str, market_data: &MarketData) -> Result<StrategySignal>;

    /// 分析市场
    async fn analyze_market(&self, market_data: &MarketData) -> Result<String>;

    /// 验证信号
    async fn validate_signal(&self, signal: &StrategySignal) -> Result<bool>;
}

/// 市场数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    /// 股票代码
    pub ticker: String,
    /// 当前价格
    pub current_price: f64,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 成交量
    pub volume: u64,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 历史价格数据
    pub historical_prices: Vec<PriceData>,
    /// 技术指标
    pub indicators: HashMap<String, f64>,
    /// 基本面数据
    pub fundamentals: Option<FundamentalData>,
}

/// 价格数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 收盘价
    pub close: f64,
    /// 成交量
    pub volume: u64,
}

/// 基本面数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalData {
    /// 市盈率
    pub pe_ratio: Option<f64>,
    /// 市净率
    pub pb_ratio: Option<f64>,
    /// 营收
    pub revenue: Option<f64>,
    /// 净利润
    pub net_income: Option<f64>,
    /// 每股收益
    pub eps: Option<f64>,
    /// 股息率
    pub dividend_yield: Option<f64>,
    /// ROE
    pub roe: Option<f64>,
    /// 债务股本比
    pub debt_to_equity: Option<f64>,
}

/// 趋势跟踪策略
pub struct TrendFollowingStrategy {
    id: String,
    name: String,
    parameters: StrategyParameters,
    options: ClaudeAgentOptions,
}

impl TrendFollowingStrategy {
    /// 创建新的趋势跟踪策略
    pub fn new() -> Self {
        let parameters = StrategyParameters::new()
            .add("short_ma", 20.0, "短期均线周期")
            .add("long_ma", 50.0, "长期均线周期")
            .add("signal_threshold", 0.02, "信号阈值（百分比）")
            .add("position_size", 0.1, "默认仓位大小");

        let options = ClaudeAgentOptions::builder()
            .permission_mode(PermissionMode::AcceptEdits)
            .max_thinking_tokens(30000)
            .build();

        Self {
            id: "trend_following_001".to_string(),
            name: "双均线趋势跟踪策略".to_string(),
            parameters,
            options,
        }
    }

    /// 使用自定义参数创建
    pub fn with_parameters(parameters: StrategyParameters) -> Self {
        let options = ClaudeAgentOptions::builder()
            .permission_mode(PermissionMode::AcceptEdits)
            .max_thinking_tokens(30000)
            .build();

        Self {
            id: "trend_following_001".to_string(),
            name: "双均线趋势跟踪策略".to_string(),
            parameters,
            options,
        }
    }
}

#[async_trait]
impl StrategyAgent for TrendFollowingStrategy {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn strategy_type(&self) -> StrategyType {
        StrategyType::TrendFollowing
    }

    fn parameters(&self) -> &StrategyParameters {
        &self.parameters
    }

    async fn set_parameters(&mut self, params: StrategyParameters) -> Result<()> {
        self.parameters = params;
        Ok(())
    }

    async fn generate_signal(&self, ticker: &str, market_data: &MarketData) -> Result<StrategySignal> {
        let short_period = self.parameters.get("short_ma").unwrap_or(20.0) as usize;
        let long_period = self.parameters.get("long_ma").unwrap_or(50.0) as usize;
        let threshold = self.parameters.get("signal_threshold").unwrap_or(0.02);
        let default_position = self.parameters.get("position_size").unwrap_or(0.1);

        // 计算均线
        let short_ma = self.calculate_ma(&market_data.historical_prices, short_period)?;
        let long_ma = self.calculate_ma(&market_data.historical_prices, long_period)?;

        // 计算信号强度和方向
        let (signal_type, direction, strength) = if short_ma > long_ma * (1.0 + threshold) {
            // 强烈买入信号
            (
                SignalType::Entry,
                SignalDirection::Long,
                (short_ma / long_ma - 1.0) * 500.0, // 转换为0-100
            )
        } else if short_ma < long_ma * (1.0 - threshold) {
            // 强烈卖出信号
            (
                SignalType::Exit,
                SignalDirection::Short,
                (1.0 - short_ma / long_ma) * 500.0,
            )
        } else {
            // 观望
            (SignalType::Hold, SignalDirection::Neutral, 50.0)
        };

        let strength = strength.clamp(0.0, 100.0);

        // 计算目标价和止损价
        let (target_price, stop_loss) = match direction {
            SignalDirection::Long => {
                let target = market_data.current_price * 1.1; // +10%
                let stop = market_data.current_price * 0.95; // -5%
                (Some(target), Some(stop))
            }
            SignalDirection::Short => {
                let target = market_data.current_price * 0.9; // -10%
                let stop = market_data.current_price * 1.05; // +5%
                (Some(target), Some(stop))
            }
            SignalDirection::Neutral => (None, None),
        };

        let signal = StrategySignal {
            strategy_id: self.id.clone(),
            ticker: ticker.to_string(),
            signal_type,
            strength,
            direction,
            entry_price: Some(market_data.current_price),
            target_price,
            stop_loss,
            holding_period: Some(Duration::days(30)), // 默认30天
            position_size: default_position,
            confidence: (strength / 100.0).clamp(0.5, 0.95),
            timestamp: Utc::now(),
            reasoning: format!(
                "短期均线({:.2}) vs 长期均线({:.2}): {}",
                short_ma,
                long_ma,
                if short_ma > long_ma { "金叉，看多" } else { "死叉，看空" }
            ),
            risk_warnings: if strength < 60.0 {
                vec!["信号强度较弱，建议小仓位试探".to_string()]
            } else {
                vec![]
            },
        };

        Ok(signal)
    }

    async fn analyze_market(&self, market_data: &MarketData) -> Result<String> {
        let prompt = format!(
            "请分析股票 {} 的市场趋势：\n\
            当前价格: {:.2}\n\
            历史数据点数: {}\n\
            技术指标: {:?}\n\n\
            请提供详细的技术分析，包括趋势判断、支撑阻力位、以及交易建议。",
            market_data.ticker,
            market_data.current_price,
            market_data.historical_prices.len(),
            market_data.indicators
        );

        let mut stream = query_stream(&prompt, Some(self.options.clone())).await?;
        let mut analysis = String::new();

        while let Some(result) = stream.next().await {
            let message = result?;
            if let Message::Assistant(msg) = message {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        analysis.push_str(&text.text);
                        analysis.push('\n');
                    }
                }
            }
        }

        Ok(analysis)
    }

    async fn validate_signal(&self, signal: &StrategySignal) -> Result<bool> {
        // 基本验证
        if signal.confidence < 0.5 {
            return Ok(false);
        }

        if signal.strength < 50.0 {
            return Ok(false);
        }

        // 风险收益比验证
        if let (Some(target), Some(entry), Some(stop)) =
            (signal.target_price, signal.entry_price, signal.stop_loss)
        {
            let risk_reward = match signal.direction {
                SignalDirection::Long => (target - entry) / (entry - stop),
                SignalDirection::Short => (entry - target) / (stop - entry),
                SignalDirection::Neutral => 0.0,
            };

            // 风险收益比应该大于1.5
            return Ok(risk_reward > 1.5);
        }

        Ok(true)
    }
}

impl TrendFollowingStrategy {
    /// 计算移动平均线
    fn calculate_ma(&self, prices: &[PriceData], period: usize) -> Result<f64> {
        if prices.len() < period {
            return Ok(prices.last().map(|p| p.close).unwrap_or(0.0));
        }

        let sum: f64 = prices
            .iter()
            .rev()
            .take(period)
            .map(|p| p.close)
            .sum();

        Ok(sum / period as f64)
    }
}

impl Default for TrendFollowingStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// 策略引擎
pub struct StrategyEngine {
    strategies: Arc<RwLock<Vec<Box<dyn StrategyAgent>>>>,
    options: ClaudeAgentOptions,
}

impl StrategyEngine {
    /// 创建新的策略引擎
    pub fn new() -> Self {
        let options = ClaudeAgentOptions::builder()
            .permission_mode(PermissionMode::AcceptEdits)
            .max_thinking_tokens(50000)
            .build();

        Self {
            strategies: Arc::new(RwLock::new(Vec::new())),
            options,
        }
    }

    /// 添加策略
    pub async fn add_strategy(&self, strategy: Box<dyn StrategyAgent>) {
        let mut strategies = self.strategies.write().await;
        strategies.push(strategy);
    }

    /// 移除策略
    pub async fn remove_strategy(&self, strategy_id: &str) {
        let mut strategies = self.strategies.write().await;
        strategies.retain(|s| s.id() != strategy_id);
    }

    /// 获取所有策略
    pub async fn get_strategies(&self) -> Vec<String> {
        let strategies = self.strategies.read().await;
        strategies.iter().map(|s| s.id().to_string()).collect()
    }

    /// 为单个股票生成信号（使用所有策略）
    pub async fn generate_signals(
        &self,
        ticker: &str,
        market_data: &MarketData,
    ) -> Result<Vec<StrategySignal>> {
        let strategies = self.strategies.read().await;
        let mut signals = Vec::new();

        for strategy in strategies.iter() {
            match strategy.generate_signal(ticker, market_data).await {
                Ok(signal) => {
                    // 验证信号
                    if strategy.validate_signal(&signal).await.unwrap_or(false) {
                        signals.push(signal);
                    }
                }
                Err(e) => {
                    eprintln!("策略 {} 生成信号失败: {}", strategy.id(), e);
                }
            }
        }

        Ok(signals)
    }

    /// 聚合多个策略的信号
    pub async fn aggregate_signals(&self, signals: &[StrategySignal]) -> Result<AggregatedSignal> {
        if signals.is_empty() {
            return Ok(AggregatedSignal {
                ticker: "UNKNOWN".to_string(),
                timestamp: Utc::now(),
                consensus_direction: SignalDirection::Neutral,
                consensus_strength: 0.0,
                consensus_signal_type: SignalType::Hold,
                num_strategies: 0,
                num_long: 0,
                num_short: 0,
                num_hold: 0,
                avg_confidence: 0.0,
                recommended_position_size: 0.0,
                reasoning: "没有有效的策略信号".to_string(),
            });
        }

        // 统计各方向数量
        let num_long = signals
            .iter()
            .filter(|s| s.direction == SignalDirection::Long)
            .count();

        let num_short = signals
            .iter()
            .filter(|s| s.direction == SignalDirection::Short)
            .count();

        let num_hold = signals
            .iter()
            .filter(|s| s.direction == SignalDirection::Neutral)
            .count();

        // 确定共识方向
        let consensus_direction = if num_long > num_short && num_long > num_hold {
            SignalDirection::Long
        } else if num_short > num_long && num_short > num_hold {
            SignalDirection::Short
        } else {
            SignalDirection::Neutral
        };

        // 计算共识强度
        let consensus_strength = match consensus_direction {
            SignalDirection::Long => {
                signals
                    .iter()
                    .filter(|s| s.direction == SignalDirection::Long)
                    .map(|s| s.strength)
                    .sum::<f64>()
                    / num_long.max(1) as f64
            }
            SignalDirection::Short => {
                signals
                    .iter()
                    .filter(|s| s.direction == SignalDirection::Short)
                    .map(|s| s.strength)
                    .sum::<f64>()
                    / num_short.max(1) as f64
            }
            SignalDirection::Neutral => 50.0,
        };

        // 确定信号类型
        let consensus_signal_type = if consensus_strength >= 70.0 {
            SignalType::Entry
        } else if consensus_strength <= 30.0 {
            SignalType::Exit
        } else {
            SignalType::Hold
        };

        // 计算平均置信度
        let avg_confidence = signals.iter().map(|s| s.confidence).sum::<f64>() / signals.len() as f64;

        // 推荐仓位大小（基于共识强度和置信度）
        let recommended_position_size = (consensus_strength / 100.0) * avg_confidence;

        // 生成推理说明
        let reasoning = format!(
            "基于 {} 个策略的共识：{} ({}, {:.1}个策略)",
            signals.len(),
            match consensus_direction {
                SignalDirection::Long => "看多",
                SignalDirection::Short => "看空",
                SignalDirection::Neutral => "中性",
            },
            match consensus_signal_type {
                SignalType::Entry => "建议入场",
                SignalType::Exit => "建议出场",
                SignalType::Hold => "建议持有",
                _ => "观察",
            },
            match consensus_direction {
                SignalDirection::Long => num_long as f64,
                SignalDirection::Short => num_short as f64,
                SignalDirection::Neutral => num_hold as f64,
            }
        );

        Ok(AggregatedSignal {
            ticker: signals[0].ticker.clone(),
            timestamp: Utc::now(),
            consensus_direction,
            consensus_strength,
            consensus_signal_type,
            num_strategies: signals.len(),
            num_long,
            num_short,
            num_hold,
            avg_confidence,
            recommended_position_size,
            reasoning,
        })
    }

    /// 优化策略参数（使用Claude进行分析）
    pub async fn optimize_strategy(
        &self,
        strategy_id: &str,
        backtest_results: &[BacktestResult],
    ) -> Result<StrategyParameters> {
        let strategies = self.strategies.read().await;
        let strategy = strategies
            .iter()
            .find(|s| s.id() == strategy_id)
            .context("策略未找到")?;

        let prompt = format!(
            "请优化以下投资策略的参数：\n\
            策略名称: {}\n\
            策略类型: {:?}\n\n\
            当前参数:\n{:?}\n\n\
            回测结果:\n{:?}\n\n\
            请基于回测结果，提供优化建议和新的参数值。",
            strategy.name(),
            strategy.strategy_type(),
            strategy.parameters(),
            backtest_results
        );

        let mut stream = query_stream(&prompt, Some(self.options.clone())).await?;
        let mut response = String::new();

        while let Some(result) = stream.next().await {
            let message = result?;
            if let Message::Assistant(msg) = message {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        response.push_str(&text.text);
                        response.push('\n');
                    }
                }
            }
        }

        // 从响应中提取优化参数（简化实现）
        // 实际应用中应该使用结构化提取或JSON Schema
        let optimized_params = strategy.parameters().clone();

        Ok(optimized_params)
    }

    /// 创建策略组合Agent
    pub async fn create_strategy_portfolio_agent(&self) -> Result<StrategyPortfolioAgent> {
        let strategy_ids = self.get_strategies().await;
        Ok(StrategyPortfolioAgent::new(
            self.strategies.clone(),
            self.options.clone(),
        ))
    }
}

impl Default for StrategyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 聚合信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedSignal {
    /// 股票代码
    pub ticker: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 共识方向
    pub consensus_direction: SignalDirection,
    /// 共识强度 (0-100)
    pub consensus_strength: f64,
    /// 共识信号类型
    pub consensus_signal_type: SignalType,
    /// 参与策略数量
    pub num_strategies: usize,
    /// 看多策略数量
    pub num_long: usize,
    /// 看空策略数量
    pub num_short: usize,
    /// 中性策略数量
    pub num_hold: usize,
    /// 平均置信度
    pub avg_confidence: f64,
    /// 推荐仓位大小
    pub recommended_position_size: f64,
    /// 推理说明
    pub reasoning: String,
}

/// 策略组合Agent
pub struct StrategyPortfolioAgent {
    strategies: Arc<RwLock<Vec<Box<dyn StrategyAgent>>>>,
    options: ClaudeAgentOptions,
}

impl StrategyPortfolioAgent {
    pub fn new(
        strategies: Arc<RwLock<Vec<Box<dyn StrategyAgent>>>>,
        options: ClaudeAgentOptions,
    ) -> Self {
        Self { strategies, options }
    }

    /// 分析整个投资组合
    pub async fn analyze_portfolio(&self, tickers: &[String]) -> Result<PortfolioAnalysis> {
        let mut signals_by_ticker = HashMap::new();

        // 为每个股票生成信号
        for ticker in tickers {
            // 这里简化了market_data的获取
            // 实际应用中应该从数据库或API获取
            let market_data = MarketData {
                ticker: ticker.clone(),
                current_price: 100.0,
                open: 99.0,
                high: 101.0,
                low: 98.0,
                volume: 1000000,
                timestamp: Utc::now(),
                historical_prices: vec![],
                indicators: HashMap::new(),
                fundamentals: None,
            };

            let strategies = self.strategies.read().await;
            let mut ticker_signals = Vec::new();

            for strategy in strategies.iter() {
                if let Ok(signal) = strategy.generate_signal(ticker, &market_data).await {
                    ticker_signals.push(signal);
                }
            }

            signals_by_ticker.insert(ticker.clone(), ticker_signals);
        }

        // 生成组合分析
        let analysis = PortfolioAnalysis {
            timestamp: Utc::now(),
            total_tickers: tickers.len(),
            signals_by_ticker,
            portfolio_recommendations: vec![],
            overall_sentiment: SignalDirection::Neutral,
            risk_assessment: "中等风险".to_string(),
        };

        Ok(analysis)
    }
}

#[async_trait]
impl Agent for StrategyPortfolioAgent {
    fn name(&self) -> &str {
        "StrategyPortfolioAgent"
    }

    fn description(&self) -> &str {
        "分析和管理多个策略的投资组合"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 从输入中解析股票列表
        let tickers: Vec<String> = input
            .context
            .get("tickers")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        if tickers.is_empty() {
            return Ok(AgentOutput::new("没有提供股票列表".to_string()));
        }

        // 分析投资组合
        let analysis = self.analyze_portfolio(&tickers).await?;

        // 格式化输出
        let output_text = format!(
            "投资组合分析完成:\n\
            分析股票数: {}\n\
            整体情绪: {:?}\n\
            风险评估: {}\n\
            时间戳: {}",
            analysis.total_tickers,
            analysis.overall_sentiment,
            analysis.risk_assessment,
            analysis.timestamp
        );

        Ok(AgentOutput::new(output_text))
    }
}

/// 投资组合分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAnalysis {
    /// 分析时间戳
    pub timestamp: DateTime<Utc>,
    /// 分析的股票总数
    pub total_tickers: usize,
    /// 每个股票的信号
    pub signals_by_ticker: HashMap<String, Vec<StrategySignal>>,
    /// 投资组合建议
    pub portfolio_recommendations: Vec<String>,
    /// 整体情绪
    pub overall_sentiment: SignalDirection,
    /// 风险评估
    pub risk_assessment: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_market_data() -> MarketData {
        MarketData {
            ticker: "TEST".to_string(),
            current_price: 100.0,
            open: 99.0,
            high: 101.0,
            low: 98.0,
            volume: 1000000,
            timestamp: Utc::now(),
            historical_prices: vec![
                PriceData {
                    timestamp: Utc::now() - Duration::days(1),
                    open: 98.0,
                    high: 100.0,
                    low: 97.0,
                    close: 99.0,
                    volume: 1000000,
                },
                PriceData {
                    timestamp: Utc::now() - Duration::days(2),
                    open: 97.0,
                    high: 99.0,
                    low: 96.0,
                    close: 98.0,
                    volume: 1000000,
                },
            ],
            indicators: HashMap::new(),
            fundamentals: None,
        }
    }

    #[tokio::test]
    async fn test_strategy_creation() {
        let strategy = TrendFollowingStrategy::new();
        assert_eq!(strategy.id(), "trend_following_001");
        assert_eq!(strategy.name(), "双均线趋势跟踪策略");
        assert_eq!(strategy.strategy_type(), StrategyType::TrendFollowing);
        println!("✅ 策略创建成功");
    }

    #[tokio::test]
    async fn test_strategy_parameters() {
        let params = StrategyParameters::new()
            .add("test_param", 42.0, "测试参数")
            .add("another_param", 10.0, "另一个参数");

        assert_eq!(params.get("test_param"), Some(42.0));
        assert_eq!(params.get("another_param"), Some(10.0));
        assert_eq!(params.get("nonexistent"), None);
        println!("✅ 策略参数管理正常");
    }

    #[tokio::test]
    async fn test_signal_generation() {
        let strategy = TrendFollowingStrategy::new();
        let market_data = create_test_market_data();

        let signal = strategy
            .generate_signal("TEST", &market_data)
            .await
            .unwrap();

        assert_eq!(signal.ticker, "TEST");
        assert_eq!(signal.strategy_id, "trend_following_001");
        assert!(signal.strength >= 0.0 && signal.strength <= 100.0);
        println!("✅ 信号生成成功: {:?}", signal);
    }

    #[tokio::test]
    async fn test_strategy_engine() {
        let engine = StrategyEngine::new();
        let strategy = Box::new(TrendFollowingStrategy::new());

        engine.add_strategy(strategy).await;

        let strategies = engine.get_strategies().await;
        assert_eq!(strategies.len(), 1);
        assert_eq!(strategies[0], "trend_following_001");
        println!("✅ 策略引擎正常");
    }

    #[tokio::test]
    async fn test_signal_aggregation() {
        let engine = StrategyEngine::new();
        let strategy = Box::new(TrendFollowingStrategy::new());
        engine.add_strategy(strategy).await;

        let market_data = create_test_market_data();
        let signals = engine.generate_signals("TEST", &market_data).await.unwrap();

        let aggregated = engine.aggregate_signals(&signals).await.unwrap();

        assert_eq!(aggregated.ticker, "TEST");
        assert!(aggregated.num_strategies > 0);
        println!("✅ 信号聚合成功");
        println!("   共识方向: {:?}", aggregated.consensus_direction);
        println!("   共识强度: {:.1}", aggregated.consensus_strength);
    }

    #[tokio::test]
    async fn test_strategy_agent_integration() {
        let engine = StrategyEngine::new();
        let strategy = Box::new(TrendFollowingStrategy::new());
        engine.add_strategy(strategy).await;

        let agent = engine.create_strategy_portfolio_agent().await.unwrap();

        let input = AgentInput {
            content: "分析投资组合".to_string(),
            context: serde_json::json!({
                "tickers": ["AAPL", "MSFT", "GOOGL"]
            }),
            metadata: HashMap::new(),
        };

        let output = agent.execute(input).await.unwrap();
        assert!(!output.content.is_empty());
        println!("✅ Agent集成测试通过");
        println!("   输出: {}", output.content);
    }
}
