//! 回测引擎
//!
//! 执行策略回测的核心引擎

use crate::backtest::{
    BacktestConfig, BacktestDataset, BacktestPortfolio, Strategy, StrategySignal,
    DateRange, Trade, TradeType,
};
use crate::backtest::metrics::{PerformanceMetrics, RiskMetrics};
use anyhow::Result;
use chrono::NaiveDate;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 回测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    /// 策略名称
    pub strategy_name: String,

    /// 配置
    pub config: BacktestConfig,

    /// 绩效指标
    pub performance: PerformanceMetrics,

    /// 风险指标
    pub risk: RiskMetrics,

    /// 交易列表
    pub trades: Vec<Trade>,

    /// 每日净值曲线
    pub equity_curve: Vec<(NaiveDate, f64)>,

    /// 成功标记
    pub success: bool,
}

/// 回测引擎
pub struct BacktestEngine {
    /// 配置
    config: BacktestConfig,

    /// 数据集
    dataset: BacktestDataset,

    /// 组合
    portfolio: BacktestPortfolio,

    /// 当前日期
    current_date: NaiveDate,
}

impl BacktestEngine {
    /// 创建新的回测引擎
    pub fn new(config: BacktestConfig, dataset: BacktestDataset) -> Self {
        let start_date = config.date_range.start_date;
        let initial_capital = config.initial_capital;

        let portfolio = BacktestPortfolio::new(
            initial_capital,
            start_date,
        );

        Self {
            config,
            dataset,
            portfolio,
            current_date: start_date,
        }
    }

    /// 运行回测
    pub fn run(mut self, strategy: &dyn Strategy) -> Result<BacktestResult> {
        let mut equity_curve = Vec::new();
        let start_value = self.portfolio.total_value;

        // 记录初始净值
        equity_curve.push((self.current_date, self.portfolio.total_value));

        // 逐日回测
        while self.current_date <= self.config.date_range.end_date {
            // 获取当日数据并收集信号
            let today_data = self.dataset.get_date_data(self.current_date);
            let mut signals = Vec::new();

            for data_point in today_data {
                if let Some(signal) = self.analyze_symbol(strategy, data_point) {
                    signals.push(signal);
                }
            }

            // 执行所有信号
            for signal in signals {
                self.execute_signal(&signal)?;
            }

            // 更新组合市值
            self.update_portfolio_value()?;

            // 记录净值
            equity_curve.push((self.current_date, self.portfolio.total_value));

            // 移动到下一天
            self.current_date = self.current_date.succ_opt().unwrap();
        }

        // 计算绩效指标
        let performance = self.calculate_performance(&equity_curve, start_value)?;
        let risk = self.calculate_risk(&equity_curve)?;

        Ok(BacktestResult {
            strategy_name: strategy.name().to_string(),
            config: self.config,
            performance,
            risk,
            trades: self.portfolio.trades,
            equity_curve,
            success: true,
        })
    }

    /// 分析单个股票
    fn analyze_symbol(
        &self,
        strategy: &dyn Strategy,
        data_point: &crate::backtest::BacktestDataPoint,
    ) -> Option<StrategySignal> {
        // 获取历史数据
        let historical = if strategy.requires_history() {
            let history_length = strategy.history_length();
            let mut historical_data = Vec::new();

            let mut date = data_point.date;
            let mut count = 0;

            while count < history_length {
                date = date.pred_opt()?;
                if let Some(dp) = self.dataset.get_data(&data_point.symbol, date) {
                    historical_data.push(dp);
                    count += 1;
                }
            }

            historical_data.reverse()
        } else {
            Vec::new()
        };

        strategy.analyze(data_point, &historical)
    }

    /// 执行交易信号
    fn execute_signal(&mut self, signal: &StrategySignal) -> Result<()> {
        match signal.signal_type {
            crate::backtest::SignalType::Buy => {
                // 计算买入金额
                let invest_amount = self.portfolio.total_value * signal.position_size;

                // 获取当前价格
                if let Some(dp) = self.dataset.get_data(&signal.symbol, self.current_date) {
                    let shares = (invest_amount / dp.close).floor();
                    if shares > 0.0 && invest_amount >= self.config.min_trade_amount {
                        self.portfolio.buy(
                            signal.symbol.clone(),
                            shares,
                            dp.close,
                            self.current_date,
                            self.config.commission_rate,
                            self.config.slippage,
                        )?;
                    }
                }
            },
            crate::backtest::SignalType::Sell => {
                // 卖出全部持仓
                if let Some(position) = self.portfolio.get_position(&signal.symbol) {
                    if let Some(dp) = self.dataset.get_data(&signal.symbol, self.current_date) {
                        self.portfolio.sell(
                            &signal.symbol,
                            position.shares,
                            dp.close,
                            self.current_date,
                            self.config.commission_rate,
                            self.config.slippage,
                        )?;
                    }
                }
            },
            _ => {},
        }

        Ok(())
    }

    /// 更新组合市值
    fn update_portfolio_value(&mut self) -> Result<()> {
        let mut prices = HashMap::new();

        for (symbol, _) in &self.portfolio.positions {
            if let Some(dp) = self.dataset.get_data(symbol, self.current_date) {
                prices.insert(symbol.clone(), dp.close);
            }
        }

        self.portfolio.update_value(&prices);
        Ok(())
    }

    /// 计算绩效指标
    fn calculate_performance(
        &self,
        equity_curve: &[(NaiveDate, f64)],
        start_value: f64,
    ) -> Result<PerformanceMetrics> {
        if equity_curve.is_empty() {
            return Ok(PerformanceMetrics::default());
        }

        let end_value = equity_curve.last().unwrap().1;
        let total_return = (end_value / start_value) - 1.0;

        // 计算年化收益率
        let years = self.config.date_range.years();
        let annualized_return = if years > 0.0 {
            ((end_value / start_value).powf(1.0 / years) - 1.0)
        } else {
            0.0
        };

        // 计算每日收益率
        let daily_returns: Vec<f64> = equity_curve
            .windows(2)
            .map(|w| (w[1].1 / w[0].1) - 1.0)
            .collect();

        // 计算夏普比率（简化）
        let avg_return = if !daily_returns.is_empty() {
            daily_returns.iter().sum::<f64>() / daily_returns.len() as f64
        } else {
            0.0
        };

        let variance = daily_returns
            .iter()
            .map(|r| (r - avg_return).powi(2))
            .sum::<f64>() / daily_returns.len() as f64;

        let std_dev = variance.sqrt();

        let sharpe_ratio = if std_dev > 0.0 {
            (avg_return * 252.0) / (std_dev * (252.0_f64).sqrt())
        } else {
            0.0
        };

        Ok(PerformanceMetrics {
            total_return,
            annualized_return,
            sharpe_ratio,
            max_drawdown: 0.0, // 简化
            win_rate: 0.0,     // 从交易记录计算
            total_trades: self.portfolio.trades.len(),
        })
    }

    /// 计算风险指标
    fn calculate_risk(&self, equity_curve: &[(NaiveDate, f64)]) -> Result<RiskMetrics> {
        Ok(RiskMetrics::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::{GrahamStrategy, BacktestDataPoint};

    #[test]
    fn test_backtest_engine_creation() {
        let config = BacktestConfig::default();
        let dataset = BacktestDataset {
            data_points: vec![],
            symbols: vec![],
        };

        let engine = BacktestEngine::new(config, dataset);
        assert_eq!(engine.portfolio.cash, 100_000.0);
    }

    #[test]
    fn test_run_simple_backtest() {
        let config = BacktestConfig {
            initial_capital: 100_000.0,
            date_range: DateRange::new("2023-01-01", "2023-01-31").unwrap(),
            ..Default::default()
        };

        // 创建模拟数据
        let dataset = BacktestDataset::generate_mock_data(
            vec!["TEST"],
            &config.date_range,
            100.0,
        );

        let engine = BacktestEngine::new(config, dataset);
        let strategy = GrahamStrategy::new();

        let result = engine.run(&strategy);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.strategy_name, "Graham Value Investing");
        assert!(result.success);
    }
}
