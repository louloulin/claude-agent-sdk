//! 回测绩效和风险评估指标
//!
//! 计算各种回测绩效和风险指标

use serde::{Deserialize, Serialize};

/// 绩效指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
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

    /// 总交易次数
    pub total_trades: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_return: 0.0,
            annualized_return: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            win_rate: 0.0,
            total_trades: 0,
        }
    }
}

/// 风险指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    /// 波动率
    pub volatility: f64,

    /// VaR（Value at Risk）
    pub var_95: f64,

    /// CVaR（Conditional VaR）
    pub cvar_95: f64,

    /// Beta
    pub beta: f64,

    /// Alpha
    pub alpha: f64,
}

impl Default for RiskMetrics {
    fn default() -> Self {
        Self {
            volatility: 0.0,
            var_95: 0.0,
            cvar_95: 0.0,
            beta: 1.0,
            alpha: 0.0,
        }
    }
}

/// 交易分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAnalysis {
    /// 盈利交易数
    pub winning_trades: usize,

    /// 亏损交易数
    pub losing_trades: usize,

    /// 平均盈利
    pub avg_win: f64,

    /// 平均亏损
    pub avg_loss: f64,

    /// 最大盈利
    pub max_win: f64,

    /// 最大亏损
    pub max_loss: f64,

    /// 盈亏比
    pub profit_loss_ratio: f64,
}

/// 格式化绩效指标为可读文本
pub fn format_performance_metrics(metrics: &PerformanceMetrics) -> String {
    format!(
        "📊 绩效指标：
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
总收益率：{:.2}%
年化收益率：{:.2}%
夏普比率：{:.2}
最大回撤：{:.2}%
胜率：{:.1}%
总交易次数：{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        metrics.total_return * 100.0,
        metrics.annualized_return * 100.0,
        metrics.sharpe_ratio,
        metrics.max_drawdown * 100.0,
        metrics.win_rate * 100.0,
        metrics.total_trades
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.total_return, 0.0);
        assert_eq!(metrics.total_trades, 0);
    }

    #[test]
    fn test_format_performance_metrics() {
        let metrics = PerformanceMetrics {
            total_return: 0.50,
            annualized_return: 0.15,
            sharpe_ratio: 1.5,
            max_drawdown: -0.10,
            win_rate: 0.60,
            total_trades: 100,
        };

        let text = format_performance_metrics(&metrics);
        assert!(text.contains("50.00%"));
        assert!(text.contains("100"));
    }
}
