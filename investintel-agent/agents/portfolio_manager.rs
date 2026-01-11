//! Portfolio Manager Agent - 投资组合管理Agent
//!
//! 基于现有portfolio-management skill的组合管理Agent
//!
//! ## 核心功能
//!
//! 1. **资产配置建议** - 基于风险偏好的资产配置
//! 2. **再平衡提醒** - 检查组合是否需要再平衡
//! 3. **绩效评估** - 计算组合收益和风险指标

use crate::agents::{Agent, AgentInput, AgentOutput, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Portfolio Manager Agent
///
/// 投资组合管理Agent，提供资产配置和再平衡建议
pub struct PortfolioManagerAgent {
    /// 默认再平衡阈值
    rebalance_threshold: f64,
}

impl PortfolioManagerAgent {
    /// 创建新的组合管理Agent
    pub fn new() -> Self {
        Self {
            rebalance_threshold: 0.05, // 5%偏离触发再平衡
        }
    }

    /// 设置再平衡阈值
    pub fn with_rebalance_threshold(mut self, threshold: f64) -> Self {
        self.rebalance_threshold = threshold;
        self
    }

    /// 分析投资组合
    async fn analyze_portfolio(&self, portfolio: &Portfolio) -> Result<PortfolioAnalysis> {
        let total_value = portfolio.holdings.iter().map(|h| h.value).sum();

        // 计算当前权重
        let current_weights: Vec<f64> = portfolio
            .holdings
            .iter()
            .map(|h| h.value / total_value)
            .collect();

        // 计算目标权重偏离
        let mut deviations = Vec::new();
        for (i, holding) in portfolio.holdings.iter().enumerate() {
            let target_weight = portfolio.target_weights.get(&holding.symbol).unwrap_or(&0.0);
            let deviation = current_weights[i] - target_weight;
            deviations.push((holding.symbol.clone(), deviation));
        }

        // 检查是否需要再平衡
        let needs_rebalance = deviations
            .iter()
            .any(|(_, deviation)| deviation.abs() > self.rebalance_threshold);

        // 计算组合指标
        let metrics = PortfolioMetrics {
            total_value,
            daily_return: self.calculate_daily_return(portfolio).await?,
            volatility: self.calculate_volatility(portfolio).await?,
            sharpe_ratio: 0.0, // TODO: 实际计算
            max_drawdown: 0.0, // TODO: 实际计算
        };

        Ok(PortfolioAnalysis {
            metrics,
            current_weights,
            deviations,
            needs_rebalance,
        })
    }

    /// 计算日收益率 (简化版)
    async fn calculate_daily_return(&self, portfolio: &Portfolio) -> Result<f64> {
        // TODO: 实际实现应该获取历史价格数据
        // 这里返回模拟值
        Ok(0.001) // 0.1%日收益
    }

    /// 计算波动率 (简化版)
    async fn calculate_volatility(&self, portfolio: &Portfolio) -> Result<f64> {
        // TODO: 实际实现应该计算历史波动率
        // 这里返回模拟值
        Ok(0.15) // 15%年化波动率
    }

    /// 生成再平衡建议
    async fn generate_rebalance_suggestions(
        &self,
        analysis: &PortfolioAnalysis,
    ) -> Vec<RebalanceAction> {
        let mut actions = Vec::new();

        for (symbol, deviation) in &analysis.deviations {
            if deviation.abs() > self.rebalance_threshold {
                let action = if *deviation > 0.0 {
                    RebalanceAction {
                        symbol: symbol.clone(),
                        action_type: RebalanceType::Sell,
                        amount: deviation.abs(),
                        reason: format!("超配 {:.1}%, 需要卖出", deviation.abs() * 100.0),
                    }
                } else {
                    RebalanceAction {
                        symbol: symbol.clone(),
                        action_type: RebalanceType::Buy,
                        amount: deviation.abs(),
                        reason: format!("低配 {:.1}%, 需要买入", deviation.abs() * 100.0),
                    }
                };

                actions.push(action);
            }
        }

        actions
    }
}

impl Default for PortfolioManagerAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for PortfolioManagerAgent {
    fn name(&self) -> &str {
        "PortfolioManagerAgent"
    }

    fn description(&self) -> &str {
        "投资组合管理Agent，提供资产配置和再平衡建议"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 解析输入获取组合信息
        let portfolio: Portfolio = serde_json::from_value(input.context)
            .map_err(|e| anyhow::anyhow!("解析组合信息失败: {}", e))?;

        // 分析组合
        let analysis = self.analyze_portfolio(&portfolio).await?;

        // 生成再平衡建议
        let rebalance_actions = if analysis.needs_rebalance {
            self.generate_rebalance_suggestions(&analysis).await
        } else {
            Vec::new()
        };

        // 格式化输出
        let mut content = format!(
            "📊 投资组合分析报告\n\n\
             💰 组合总值: ${:.2}\n\
             📈 日收益率: {:.3}%\n\
             📊 波动率: {:.1}%\n\n",
            analysis.metrics.total_value,
            analysis.metrics.daily_return * 100.0,
            analysis.metrics.volatility * 100.0,
        );

        if !rebalance_actions.is_empty() {
            content.push_str("⚠️ 再平衡建议:\n\n");
            for action in &rebalance_actions {
                content.push_str(&format!(
                    "  {} {}: {} ({:.1}%)\n",
                    action.action_type.display_name(),
                    action.symbol,
                    action.reason,
                    action.amount * 100.0
                ));
            }
        } else {
            content.push_str("✅ 组合配置合理，无需再平衡\n");
        }

        // 添加当前权重
        content.push_str("\n📊 当前权重:\n");
        for (i, holding) in portfolio.holdings.iter().enumerate() {
            let weight = analysis.current_weights[i];
            let target = portfolio.target_weights.get(&holding.symbol).unwrap_or(&0.0);
            content.push_str(&format!(
                "  {}: {:.1}% (目标: {:.1}%)\n",
                holding.symbol,
                weight * 100.0,
                target * 100.0
            ));
        }

        Ok(AgentOutput::new(content).with_data(serde_json::json!({
            "analysis": analysis,
            "rebalance_actions": rebalance_actions,
        })))
    }
}

/// 投资组合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    /// 持仓
    pub holdings: Vec<Holding>,

    /// 目标权重
    pub target_weights: std::collections::HashMap<String, f64>,
}

/// 持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holding {
    /// 股票代码
    pub symbol: String,

    /// 数量
    pub shares: u64,

    /// 当前价值
    pub value: f64,

    /// 平均成本
    pub cost_basis: f64,
}

/// 组合分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAnalysis {
    /// 组合指标
    pub metrics: PortfolioMetrics,

    /// 当前权重
    pub current_weights: Vec<f64>,

    /// 权重偏离
    pub deviations: Vec<(String, f64)>,

    /// 是否需要再平衡
    pub needs_rebalance: bool,
}

/// 组合指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMetrics {
    /// 总价值
    pub total_value: f64,

    /// 日收益率
    pub daily_return: f64,

    /// 波动率
    pub volatility: f64,

    /// 夏普比率
    pub sharpe_ratio: f64,

    /// 最大回撤
    pub max_drawdown: f64,
}

/// 再平衡操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceAction {
    /// 股票代码
    pub symbol: String,

    /// 操作类型
    pub action_type: RebalanceType,

    /// 调整比例
    pub amount: f64,

    /// 原因
    pub reason: String,
}

/// 再平衡类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RebalanceType {
    /// 买入
    Buy,

    /// 卖出
    Sell,
}

impl RebalanceType {
    pub fn display_name(&self) -> &str {
        match self {
            RebalanceType::Buy => "买入",
            RebalanceType::Sell => "卖出",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_portfolio_manager_agent() {
        let agent = PortfolioManagerAgent::new();

        let mut target_weights = HashMap::new();
        target_weights.insert("AAPL".to_string(), 0.4);
        target_weights.insert("MSFT".to_string(), 0.3);
        target_weights.insert("GOOGL".to_string(), 0.3);

        let portfolio = Portfolio {
            holdings: vec![
                Holding {
                    symbol: "AAPL".to_string(),
                    shares: 100,
                    value: 15000.0,
                    cost_basis: 12000.0,
                },
                Holding {
                    symbol: "MSFT".to_string(),
                    shares: 50,
                    value: 10000.0,
                    cost_basis: 9000.0,
                },
                Holding {
                    symbol: "GOOGL".to_string(),
                    shares: 30,
                    value: 5000.0,
                    cost_basis: 6000.0,
                },
            ],
            target_weights,
        };

        let result = agent
            .execute(AgentInput::new("分析组合").with_context(serde_json::to_value(portfolio).unwrap()))
            .await
            .unwrap();

        assert!(!result.content.is_empty());
        assert!(result.content.contains("投资组合分析报告"));
    }
}
