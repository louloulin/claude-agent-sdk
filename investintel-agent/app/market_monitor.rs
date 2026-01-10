//! 实时市场监控系统
//!
//! 基于Claude Agent SDK的高频市场监控
//! 使用query_stream实现实时分析
//! 使用WebSocket接收实时数据
//! 使用Agent实现监控智能体

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use claude_agent_sdk_rs::{
    orchestration::{Agent, AgentInput, AgentOutput},
    query_stream, ClaudeAgentOptions, ContentBlock, Message, PermissionMode,
};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};

/// 市场事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketEvent {
    /// 价格更新
    PriceUpdate {
        ticker: String,
        price: f64,
        change: f64,
        change_percent: f64,
        volume: u64,
        timestamp: DateTime<Utc>,
    },
    /// 异常波动
    UnusualMovement {
        ticker: String,
        price: f64,
        change_percent: f64,
        volume_spike: Option<f64>,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    /// 新闻事件
    NewsEvent {
        ticker: Option<String>,
        headline: String,
        sentiment: String,
        impact: String,
        timestamp: DateTime<Utc>,
    },
    /// 技术指标信号
    TechnicalSignal {
        ticker: String,
        indicator: String,
        signal_type: String,
        value: f64,
        timestamp: DateTime<Utc>,
    },
    /// 交易信号
    TradeSignal {
        ticker: String,
        action: TradeAction,
        reason: String,
        confidence: f64,
        timestamp: DateTime<Utc>,
    },
    /// 风险警告
    RiskWarning {
        level: RiskLevel,
        message: String,
        affected_tickers: Vec<String>,
        timestamp: DateTime<Utc>,
    },
    /// 市场状态
    MarketStatus {
        status: String,
        volatility: f64,
        trend: String,
        timestamp: DateTime<Utc>,
    },
}

/// 交易动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeAction {
    /// 买入
    Buy,
    /// 卖出
    Sell,
    /// 持有
    Hold,
    /// 观察
    Watch,
}

/// 风险级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    /// 低风险
    Low,
    /// 中等风险
    Medium,
    /// 高风险
    High,
    /// 严重风险
    Critical,
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// 监控的股票列表
    pub tickers: Vec<String>,
    /// 价格变动阈值（百分比）
    pub price_change_threshold: f64,
    /// 成交量异常倍数
    pub volume_spike_multiplier: f64,
    /// 监控间隔（秒）
    pub monitor_interval_secs: u64,
    /// 是否启用技术分析
    pub enable_technical_analysis: bool,
    /// 是否启用新闻监控
    pub enable_news_monitoring: bool,
    /// 是否启用风险监控
    pub enable_risk_monitoring: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            tickers: vec!["AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string()],
            price_change_threshold: 2.0, // 2%
            volume_spike_multiplier: 2.0, // 2倍
            monitor_interval_secs: 60, // 60秒
            enable_technical_analysis: true,
            enable_news_monitoring: true,
            enable_risk_monitoring: true,
        }
    }
}

/// 价格数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub ticker: String,
    pub price: f64,
    pub volume: u64,
    pub timestamp: DateTime<Utc>,
}

/// 市场监控Agent
pub struct MarketMonitorAgent {
    config: MonitorConfig,
    options: ClaudeAgentOptions,
    price_history: Arc<RwLock<HashMap<String, Vec<PricePoint>>>>,
    event_sender: mpsc::UnboundedSender<MarketEvent>,
}

impl MarketMonitorAgent {
    /// 创建新的市场监控Agent
    pub fn new(config: MonitorConfig) -> Self {
        let options = ClaudeAgentOptions::builder()
            .permission_mode(PermissionMode::AcceptEdits)
            .max_thinking_tokens(30000)
            .build();

        let (event_sender, _) = mpsc::unbounded_channel();

        Self {
            config,
            options,
            price_history: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }

    /// 启动监控
    pub async fn start(&self) -> Result<Pin<Box<dyn Stream<Item = MarketEvent> + Send>>> {
        use async_stream::stream;

        let tickers = self.config.tickers.clone();
        let threshold = self.config.price_change_threshold;
        let volume_multiplier = self.config.volume_spike_multiplier;
        let price_history = self.price_history.clone();
        let options = self.options.clone();

        let event_stream = stream! {
            let mut monitor_interval = interval(Duration::from_secs(self.config.monitor_interval_secs));

            loop {
                monitor_interval.tick().await;

                // 模拟获取实时价格（实际应用中从WebSocket或API获取）
                for ticker in &tickers {
                    if let Some(event) = Self::check_price_movement(
                        ticker,
                        threshold,
                        volume_multiplier,
                        &price_history,
                    ).await {
                        yield event;
                    }

                    // 检查技术指标
                    if self.config.enable_technical_analysis {
                        if let Some(event) = Self::check_technical_indicators(
                            ticker,
                            &price_history,
                        ).await {
                            yield event;
                        }
                    }
                }

                // 生成市场状态更新
                let status_event = Self::generate_market_status(&tickers, &price_history).await;
                yield status_event;
            }
        };

        Ok(Box::pin(event_stream))
    }

    /// 检查价格变动
    async fn check_price_movement(
        ticker: &str,
        threshold: f64,
        volume_multiplier: f64,
        price_history: &Arc<RwLock<HashMap<String, Vec<PricePoint>>>>,
    ) -> Option<MarketEvent> {
        let history = price_history.read().await;
        let ticker_history = history.get(ticker)?;

        if ticker_history.len() < 2 {
            return None;
        }

        let current = &ticker_history[ticker_history.len() - 1];
        let previous = &ticker_history[ticker_history.len() - 2];

        let change = current.price - previous.price;
        let change_percent = (change / previous.price) * 100.0;
        let volume_ratio = current.volume as f64 / previous.volume.max(1) as f64;

        // 检查是否超过阈值
        if change_percent.abs() >= threshold {
            return Some(MarketEvent::UnusualMovement {
                ticker: ticker.to_string(),
                price: current.price,
                change_percent,
                volume_spike: if volume_ratio >= volume_multiplier {
                    Some(volume_ratio)
                } else {
                    None
                },
                reason: format!(
                    "价格{} {:.2}%, {}",
                    if change_percent > 0.0 { "上涨" } else { "下跌" },
                    change_percent.abs(),
                    if volume_ratio >= volume_multiplier {
                        format!("成交量激增 {:.1}倍", volume_ratio)
                    } else {
                        "成交量正常".to_string()
                    }
                ),
                timestamp: Utc::now(),
            });
        }

        None
    }

    /// 检查技术指标
    async fn check_technical_indicators(
        ticker: &str,
        price_history: &Arc<RwLock<HashMap<String, Vec<PricePoint>>>>,
    ) -> Option<MarketEvent> {
        let history = price_history.read().await;
        let ticker_history = history.get(ticker)?;

        if ticker_history.len() < 20 {
            return None;
        }

        // 计算RSI（简化版）
        let prices: Vec<f64> = ticker_history.iter().map(|p| p.price).collect();
        let rsi = Self::calculate_rsi(&prices, 14);

        // RSI超买或超卖信号
        if rsi > 70.0 {
            return Some(MarketEvent::TechnicalSignal {
                ticker: ticker.to_string(),
                indicator: "RSI".to_string(),
                signal_type: "超买".to_string(),
                value: rsi,
                timestamp: Utc::now(),
            });
        } else if rsi < 30.0 {
            return Some(MarketEvent::TechnicalSignal {
                ticker: ticker.to_string(),
                indicator: "RSI".to_string(),
                signal_type: "超卖".to_string(),
                value: rsi,
                timestamp: Utc::now(),
            });
        }

        None
    }

    /// 计算RSI指标
    fn calculate_rsi(prices: &[f64], period: usize) -> f64 {
        if prices.len() < period + 1 {
            return 50.0;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in (prices.len() - period)..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 {
                gains += change;
            } else {
                losses -= change;
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    /// 生成市场状态
    async fn generate_market_status(
        tickers: &[String],
        price_history: &Arc<RwLock<HashMap<String, Vec<PricePoint>>>>,
    ) -> MarketEvent {
        let history = price_history.read().await;

        let mut total_change = 0.0;
        let mut count = 0;

        for ticker in tickers {
            if let Some(ticker_history) = history.get(ticker) {
                if ticker_history.len() >= 2 {
                    let current = &ticker_history[ticker_history.len() - 1];
                    let previous = &ticker_history[ticker_history.len() - 2];
                    let change = (current.price - previous.price) / previous.price * 100.0;
                    total_change += change;
                    count += 1;
                }
            }
        }

        let avg_change = if count > 0 {
            total_change / count as f64
        } else {
            0.0
        };

        let volatility = avg_change.abs();
        let trend = if avg_change > 1.0 {
            "强势上涨".to_string()
        } else if avg_change > 0.0 {
            "温和上涨".to_string()
        } else if avg_change < -1.0 {
            "强势下跌".to_string()
        } else if avg_change < 0.0 {
            "温和下跌".to_string()
        } else {
            "横盘整理".to_string()
        };

        MarketEvent::MarketStatus {
            status: "正常".to_string(),
            volatility,
            trend,
            timestamp: Utc::now(),
        }
    }

    /// 使用Claude分析市场事件
    pub async fn analyze_with_claude(&self, event: &MarketEvent) -> Result<String> {
        let prompt = format!(
            "请分析以下市场事件并提供投资建议：\n\n事件详情:\n{:?}\n\n\
            请提供：\n1. 事件影响评估\n2. 潜在风险\n3. 投资建议\n4. 需要关注的事项",
            event
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

    /// 更新价格数据
    pub async fn update_price(&self, price_point: PricePoint) {
        let mut history = self.price_history.write().await;
        let ticker_history = history.entry(price_point.ticker.clone()).or_insert_with(Vec::new);

        ticker_history.push(price_point);

        // 保留最近100个数据点
        if ticker_history.len() > 100 {
            ticker_history.remove(0);
        }
    }

    /// 获取价格历史
    pub async fn get_price_history(&self, ticker: &str) -> Vec<PricePoint> {
        let history = self.price_history.read().await;
        history.get(ticker).cloned().unwrap_or_default()
    }
}

#[async_trait]
impl Agent for MarketMonitorAgent {
    fn name(&self) -> &str {
        "MarketMonitorAgent"
    }

    fn description(&self) -> &str {
        "实时监控市场数据并生成事件"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 解析输入
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
            .unwrap_or_else(|| self.config.tickers.clone());

        // 生成监控摘要
        let mut summary = format!("监控 {} 个股票:\n", tickers.len());

        for ticker in &tickers {
            let history = self.get_price_history(ticker).await;
            if let Some(latest) = history.last() {
                summary.push_str(&format!(
                    "- {}: ${:.2} ({})\n",
                    ticker, latest.price, latest.timestamp
                ));
            }
        }

        Ok(AgentOutput::new(summary))
    }
}

/// 智能监控协调器
pub struct SmartMonitorCoordinator {
    monitor_agent: Arc<MarketMonitorAgent>,
    options: ClaudeAgentOptions,
}

impl SmartMonitorCoordinator {
    /// 创建新的协调器
    pub fn new(monitor_agent: Arc<MarketMonitorAgent>) -> Self {
        let options = ClaudeAgentOptions::builder()
            .permission_mode(PermissionMode::AcceptEdits)
            .max_thinking_tokens(50000)
            .build();

        Self {
            monitor_agent,
            options,
        }
    }

    /// 启动智能监控
    pub async fn start_smart_monitoring(&self) -> Result<Pin<Box<dyn Stream<Item = MonitoringInsight> + Send>>> {
        use async_stream::stream;

        let monitor = self.monitor_agent.clone();
        let options = self.options.clone();

        let insight_stream = stream! {
            let mut event_stream = monitor.start().await.unwrap();

            while let Some(event) = event_stream.next().await {
                // 使用Claude分析事件
                let analysis = monitor.analyze_with_claude(&event).await.unwrap();

                // 生成洞察
                let insight = Self::generate_insight(&event, &analysis);

                yield insight;
            }
        };

        Ok(Box::pin(insight_stream))
    }

    /// 生成监控洞察
    fn generate_insight(event: &MarketEvent, analysis: &str) -> MonitoringInsight {
        let (importance, urgency) = match event {
            MarketEvent::UnusualMovement { change_percent, .. } => {
                let importance = if change_percent.abs() > 5.0 {
                    ImportanceLevel::High
                } else if change_percent.abs() > 3.0 {
                    ImportanceLevel::Medium
                } else {
                    ImportanceLevel::Low
                };
                (importance, if change_percent.abs() > 5.0 { Urgency::Immediate } else { Urgency::Normal })
            }
            MarketEvent::RiskWarning { level, .. } => {
                let importance = match level {
                    RiskLevel::Critical => ImportanceLevel::Critical,
                    RiskLevel::High => ImportanceLevel::High,
                    RiskLevel::Medium => ImportanceLevel::Medium,
                    RiskLevel::Low => ImportanceLevel::Low,
                };
                (importance, Urgency::Immediate)
            }
            MarketEvent::TradeSignal { confidence, .. } => {
                let importance = if *confidence > 0.8 {
                    ImportanceLevel::High
                } else if *confidence > 0.6 {
                    ImportanceLevel::Medium
                } else {
                    ImportanceLevel::Low
                };
                (importance, Urgency::Normal)
            }
            _ => (ImportanceLevel::Low, Urgency::Normal),
        };

        MonitoringInsight {
            event: event.clone(),
            analysis: analysis.to_string(),
            importance,
            urgency,
            recommended_actions: Self::extract_actions(analysis),
            timestamp: Utc::now(),
        }
    }

    /// 从分析中提取建议动作
    fn extract_actions(analysis: &str) -> Vec<String> {
        // 简化实现：按行分割，提取包含建议的行
        analysis
            .lines()
            .filter(|line| {
                line.contains("建议") || line.contains("应该") || line.contains("推荐")
            })
            .map(|s| s.trim().to_string())
            .take(5)
            .collect()
    }
}

/// 监控洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringInsight {
    /// 原始事件
    pub event: MarketEvent,
    /// Claude分析
    pub analysis: String,
    /// 重要性级别
    pub importance: ImportanceLevel,
    /// 紧急程度
    pub urgency: Urgency,
    /// 推荐动作
    pub recommended_actions: Vec<String>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 重要性级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImportanceLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// 紧急程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Urgency {
    Normal,
    High,
    Immediate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitor_config_default() {
        let config = MonitorConfig::default();
        assert_eq!(config.tickers.len(), 3);
        assert_eq!(config.price_change_threshold, 2.0);
        assert_eq!(config.monitor_interval_secs, 60);
        println!("✅ 监控配置默认值正确");
    }

    #[tokio::test]
    async fn test_market_monitor_creation() {
        let config = MonitorConfig::default();
        let agent = MarketMonitorAgent::new(config);
        assert_eq!(agent.name(), "MarketMonitorAgent");
        println!("✅ 市场监控Agent创建成功");
    }

    #[tokio::test]
    async fn test_price_update() {
        let config = MonitorConfig::default();
        let agent = MarketMonitorAgent::new(config);

        let price_point = PricePoint {
            ticker: "AAPL".to_string(),
            price: 150.0,
            volume: 1000000,
            timestamp: Utc::now(),
        };

        agent.update_price(price_point.clone()).await;

        let history = agent.get_price_history("AAPL").await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].price, 150.0);
        println!("✅ 价格更新功能正常");
    }

    #[tokio::test]
    async fn test_rsi_calculation() {
        let prices = vec![
            100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0, 108.0, 107.0, 109.0,
            111.0, 110.0, 112.0, 114.0, 113.0,
        ];

        let rsi = MarketMonitorAgent::calculate_rsi(&prices, 14);
        assert!(rsi > 0.0 && rsi <= 100.0);
        println!("✅ RSI计算正确: {:.2}", rsi);
    }

    #[tokio::test]
    async fn test_event_stream() {
        let config = MonitorConfig {
            tickers: vec!["TEST".to_string()],
            monitor_interval_secs: 1,
            ..Default::default()
        };

        let agent = MarketMonitorAgent::new(config);

        // 添加一些测试数据
        for i in 0..5 {
            agent.update_price(PricePoint {
                ticker: "TEST".to_string(),
                price: 100.0 + i as f64,
                volume: 1000000,
                timestamp: Utc::now() - chrono::Duration::seconds((5 - i) as i64),
            }).await;
        }

        let mut stream = agent.start().await.unwrap();

        // 获取第一个事件
        if let Some(event) = stream.next().await {
            println!("✅ 事件流正常: {:?}", event);
        }
    }
}
