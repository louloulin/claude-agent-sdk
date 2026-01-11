//! 回测系统模块
//!
//! 提供完整的投资策略历史回测功能

pub mod engine;
pub mod strategy;
pub mod metrics;
pub mod portfolio;

pub use engine::{BacktestEngine, BacktestConfig, BacktestResult};
pub use strategy::{Strategy, StrategySignal, GrahamStrategy, KellyStrategy};
pub use metrics::{PerformanceMetrics, RiskMetrics, TradeAnalysis};
pub use portfolio::{BacktestPortfolio, Position, Trade};

use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 回测时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    /// 开始日期
    pub start_date: NaiveDate,

    /// 结束日期
    pub end_date: NaiveDate,
}

impl DateRange {
    /// 创建新的时间范围
    pub fn new(start_date: &str, end_date: &str) -> anyhow::Result<Self> {
        Ok(Self {
            start_date: NaiveDate::parse_from_str(start_date, "%Y-%m-%d")?,
            end_date: NaiveDate::parse_from_str(end_date, "%Y-%m-%d")?,
        })
    }

    /// 获取天数
    pub fn days(&self) -> i64 {
        (self.end_date - self.start_date).num_days()
    }

    /// 获取年数
    pub fn years(&self) -> f64 {
        self.days() as f64 / 365.25
    }
}

/// 回测数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestDataPoint {
    /// 日期
    pub date: NaiveDate,

    /// 股票代码
    pub symbol: String,

    /// 开盘价
    pub open: f64,

    /// 最高价
    pub high: f64,

    /// 最低价
    pub low: f64,

    /// 收盘价
    pub close: f64,

    /// 成交量
    pub volume: f64,

    /// 每股收益 (EPS)
    pub eps: Option<f64>,

    /// 股息
    pub dividend: Option<f64>,
}

/// 回测数据集
#[derive(Debug, Clone)]
pub struct BacktestDataset {
    /// 数据点（按symbol和date排序）
    pub data_points: Vec<BacktestDataPoint>,

    /// 股票列表
    pub symbols: Vec<String>,
}

impl BacktestDataset {
    /// 创建新的数据集
    pub fn new(data_points: Vec<BacktestDataPoint>) -> Self {
        let symbols: std::collections::HashSet<String> = data_points
            .iter()
            .map(|dp| dp.symbol.clone())
            .collect();

        Self {
            data_points,
            symbols: symbols.into_iter().collect(),
        }
    }

    /// 获取指定股票在指定日期的数据
    pub fn get_data(&self, symbol: &str, date: NaiveDate) -> Option<&BacktestDataPoint> {
        self.data_points
            .iter()
            .find(|dp| dp.symbol == symbol && dp.date == date)
    }

    /// 获取指定股票的所有数据
    pub fn get_symbol_data(&self, symbol: &str) -> Vec<&BacktestDataPoint> {
        self.data_points
            .iter()
            .filter(|dp| dp.symbol == symbol)
            .collect()
    }

    /// 获取指定日期的所有数据
    pub fn get_date_data(&self, date: NaiveDate) -> Vec<&BacktestDataPoint> {
        self.data_points
            .iter()
            .filter(|dp| dp.date == date)
            .collect()
    }

    /// 获取日期范围内的数据
    pub fn get_range_data(&self, symbol: &str, range: &DateRange) -> Vec<&BacktestDataPoint> {
        self.data_points
            .iter()
            .filter(|dp| {
                dp.symbol == symbol
                    && dp.date >= range.start_date
                    && dp.date <= range.end_date
            })
            .collect()
    }

    /// 生成模拟数据（用于测试）
    #[cfg(test)]
    pub fn generate_mock_data(
        symbols: Vec<&str>,
        range: &DateRange,
        initial_price: f64,
    ) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let mut data_points = Vec::new();

        let mut current_prices: HashMap<String, f64> = symbols
            .iter()
            .map(|s| (s.to_string(), initial_price))
            .collect();

        let mut current_date = range.start_date;

        while current_date <= range.end_date {
            // 跳过周末
            if current_date.weekday().num_days_from_monday() < 5 {
                for symbol in &symbols {
                    let price = current_prices.get(symbol).unwrap();

                    // 随机价格变动 (-3% 到 +3%)
                    let change = (rng.gen::<f64>() - 0.5) * 0.06;
                    let new_price = price * (1.0 + change);

                    current_prices.insert(symbol.clone(), new_price);

                    data_points.push(BacktestDataPoint {
                        date: current_date,
                        symbol: symbol.to_string(),
                        open: *price,
                        high: new_price * 1.01,
                        low: *price * 0.99,
                        close: new_price,
                        volume: 1_000_000.0,
                        eps: Some(new_price / 25.0),
                        dividend: Some(new_price * 0.002 / 4.0), // 季度股息 0.5%
                    });
                }
            }

            current_date = current_date.succ_opt().unwrap();
        }

        Self {
            data_points,
            symbols: symbols.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// 回测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// 初始资金
    pub initial_capital: f64,

    /// 时间范围
    pub date_range: DateRange,

    /// 基准指数（可选）
    pub benchmark: Option<String>,

    /// 交易费率
    pub commission_rate: f64,

    /// 滑点
    pub slippage: f64,

    /// 是否允许做空
    pub allow_short: bool,

    /// 是否允许杠杆
    pub allow_margin: bool,

    /// 最大仓位比例
    pub max_position_size: f64,

    /// 最小交易金额
    pub min_trade_amount: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 100_000.0,
            date_range: DateRange {
                start_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(),
            },
            benchmark: Some("SPY".to_string()),
            commission_rate: 0.001, // 0.1%
            slippage: 0.0001,         // 0.01%
            allow_short: false,
            allow_margin: false,
            max_position_size: 0.25,
            min_trade_amount: 100.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_range() {
        let range = DateRange::new("2020-01-01", "2020-12-31").unwrap();
        assert_eq!(range.days(), 365);
        assert!((range.years() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_generate_mock_data() {
        let range = DateRange::new("2023-01-01", "2023-01-31").unwrap();
        let dataset = BacktestDataset::generate_mock_data(
            vec!["AAPL", "MSFT"],
            &range,
            150.0,
        );

        assert_eq!(dataset.symbols.len(), 2);
        assert!(!dataset.data_points.is_empty());

        // 验证数据点格式正确
        let dp = &dataset.data_points[0];
        assert!(dp.close > 0.0);
        assert!(dp.high >= dp.low);
    }
}
