//! 回测策略定义
//!
//! 定义各种投资策略的回测实现

use crate::backtest::{BacktestDataPoint, DateRange, BacktestDataset};
use serde::{Deserialize, Serialize};

/// 策略信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySignal {
    /// 股票代码
    pub symbol: String,

    /// 信号类型
    pub signal_type: SignalType,

    /// 信号强度 (0-1)
    pub strength: f64,

    /// 目标价格
    pub target_price: Option<f64>,

    /// 建议仓位比例
    pub position_size: f64,

    /// 信号原因
    pub reason: String,

    /// 置信度 (0-1)
    pub confidence: f64,
}

/// 信号类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalType {
    /// 买入
    Buy,

    /// 卖出
    Sell,

    /// 持有
    Hold,

    /// 平仓
    Close,
}

/// 策略特征
pub trait Strategy: Send + Sync {
    /// 策略名称
    fn name(&self) -> &str;

    /// 分析数据点，生成信号
    fn analyze(
        &self,
        data_point: &BacktestDataPoint,
        historical_data: &[&BacktestDataPoint],
    ) -> Option<StrategySignal>;

    /// 是否需要历史数据
    fn requires_history(&self) -> bool {
        true
    }

    /// 所需历史数据长度（天数）
    fn history_length(&self) -> usize {
        252 // 默认1年交易日
    }
}

/// Graham价值投资策略
#[derive(Debug, Clone)]
pub struct GrahamStrategy {
    /// 安全边际阈值
    pub margin_of_safety_threshold: f64,

    /// Graham基础乘数
    pub graham_base_multiplier: f64,

    /// Graham增长率乘数
    pub graham_growth_multiplier: f64,
}

impl GrahamStrategy {
    /// 创建新的Graham策略
    pub fn new() -> Self {
        Self {
            margin_of_safety_threshold: 0.30, // 30%
            graham_base_multiplier: 8.5,
            graham_growth_multiplier: 2.0,
        }
    }

    /// 设置安全边际阈值
    pub fn with_margin_of_safety(mut self, threshold: f64) -> Self {
        self.margin_of_safety_threshold = threshold;
        self
    }

    /// 计算Graham内在价值
    fn calculate_intrinsic_value(&self, data_point: &BacktestDataPoint) -> Option<f64> {
        let eps = data_point.eps?;
        let growth_rate = self.estimate_growth_rate(data_point);

        let multiplier = self.graham_base_multiplier
            + self.graham_growth_multiplier * growth_rate * 100.0;

        Some(eps * multiplier)
    }

    /// 估算增长率（简化版本）
    fn estimate_growth_rate(&self, data_point: &BacktestDataPoint) -> f64 {
        // 简化：使用历史价格变化估算增长率
        // 实际应用中应使用EPS历史增长率
        0.05 // 默认5%
    }

    /// 计算安全边际
    fn calculate_margin_of_safety(&self, intrinsic_value: f64, current_price: f64) -> f64 {
        (intrinsic_value - current_price) / intrinsic_value
    }
}

impl Strategy for GrahamStrategy {
    fn name(&self) -> &str {
        "Graham Value Investing"
    }

    fn analyze(
        &self,
        data_point: &BacktestDataPoint,
        _historical_data: &[&BacktestDataPoint],
    ) -> Option<StrategySignal> {
        // 计算内在价值
        let intrinsic_value = self.calculate_intrinsic_value(data_point)?;

        // 计算安全边际
        let margin = self.calculate_margin_of_safety(intrinsic_value, data_point.close);

        // 根据安全边际生成信号
        if margin >= 0.50 {
            // 强烈买入
            Some(StrategySignal {
                symbol: data_point.symbol.clone(),
                signal_type: SignalType::Buy,
                strength: 0.9,
                target_price: Some(intrinsic_value),
                position_size: 0.20, // 20%仓位
                reason: format!(
                    "安全边际{:.1}%，内在价值${:.2}，远高于当前价格${:.2}",
                    margin * 100.0,
                    intrinsic_value,
                    data_point.close
                ),
                confidence: 0.85,
            })
        } else if margin >= self.margin_of_safety_threshold {
            // 买入
            Some(StrategySignal {
                symbol: data_point.symbol.clone(),
                signal_type: SignalType::Buy,
                strength: 0.7,
                target_price: Some(intrinsic_value),
                position_size: 0.15, // 15%仓位
                reason: format!(
                    "安全边际{:.1}%，内在价值${:.2}，高于当前价格${:.2}",
                    margin * 100.0,
                    intrinsic_value,
                    data_point.close
                ),
                confidence: 0.75,
            })
        } else if margin >= 0.0 {
            // 持有/观望
            Some(StrategySignal {
                symbol: data_point.symbol.clone(),
                signal_type: SignalType::Hold,
                strength: 0.5,
                target_price: Some(intrinsic_value),
                position_size: 0.0,
                reason: format!(
                    "安全边际{:.1}%，内在价值${:.2}，接近当前价格${:.2}",
                    margin * 100.0,
                    intrinsic_value,
                    data_point.close
                ),
                confidence: 0.6,
            })
        } else {
            // 避免/高估
            Some(StrategySignal {
                symbol: data_point.symbol.clone(),
                signal_type: SignalType::Sell,
                strength: 0.3,
                target_price: Some(intrinsic_value),
                position_size: 0.0,
                reason: format!(
                    "安全边际{:.1}%，内在价值${:.2}，低于当前价格${:.2}",
                    margin * 100.0,
                    intrinsic_value,
                    data_point.close
                ),
                confidence: 0.7,
            })
        }
    }
}

impl Default for GrahamStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Kelly仓位管理策略
#[derive(Debug, Clone)]
pub struct KellyStrategy {
    /// Kelly分数
    pub kelly_fraction: f64,

    /// 历史胜率（模拟）
    pub historical_win_rate: f64,

    /// 平均盈利（模拟）
    pub avg_win: f64,

    /// 平均亏损（模拟）
    pub avg_loss: f64,

    /// 最小仓位
    pub min_position: f64,

    /// 最大仓位
    pub max_position: f64,
}

impl KellyStrategy {
    /// 创建新的Kelly策略
    pub fn new() -> Self {
        Self {
            kelly_fraction: 0.25, // 1/4 Kelly
            historical_win_rate: 0.55,
            avg_win: 100.0,
            avg_loss: 80.0,
            min_position: 0.02, // 2%
            max_position: 0.25, // 25%
        }
    }

    /// 设置Kelly分数
    pub fn with_kelly_fraction(mut self, fraction: f64) -> Self {
        self.kelly_fraction = fraction;
        self
    }

    /// 设置历史参数
    pub fn with_historical_params(
        mut self,
        win_rate: f64,
        avg_win: f64,
        avg_loss: f64,
    ) -> Self {
        self.historical_win_rate = win_rate;
        self.avg_win = avg_win;
        self.avg_loss = avg_loss;
        self
    }

    /// 计算Kelly公式
    fn calculate_kelly(&self) -> f64 {
        if self.avg_loss <= 0.0 || self.historical_win_rate <= 0.0 {
            return 0.0;
        }

        let b = self.avg_win / self.avg_loss; // 盈亏比
        let p = self.historical_win_rate;
        let q = 1.0 - p;

        let kelly = (b * p - q) / b;
        kelly.max(0.0)
    }

    /// 应用分数Kelly
    fn apply_fractional_kelly(&self, full_kelly: f64) -> f64 {
        (full_kelly * self.kelly_fraction)
            .max(self.min_position)
            .min(self.max_position)
    }
}

impl Strategy for KellyStrategy {
    fn name(&self) -> &str {
        "Kelly Position Sizing"
    }

    fn analyze(
        &self,
        data_point: &BacktestDataPoint,
        _historical_data: &[&BacktestDataPoint],
    ) -> Option<StrategySignal> {
        // 计算Kelly仓位
        let full_kelly = self.calculate_kelly();
        let kelly_position = self.apply_fractional_kelly(full_kelly);

        // 根据Kelly结果生成信号
        if kelly_position >= self.min_position {
            Some(StrategySignal {
                symbol: data_point.symbol.clone(),
                signal_type: SignalType::Buy,
                strength: (kelly_position / self.max_position).min(1.0),
                target_price: None,
                position_size: kelly_position,
                reason: format!(
                    "Kelly建议仓位{:.1}%（完整Kelly：{:.1}%，使用{:.0}%Kelly）",
                    kelly_position * 100.0,
                    full_kelly * 100.0,
                    self.kelly_fraction * 100.0
                ),
                confidence: (self.historical_win_rate * 100.0) as f64 / 100.0,
            })
        } else {
            Some(StrategySignal {
                symbol: data_point.symbol.clone(),
                signal_type: SignalType::Hold,
                strength: 0.0,
                target_price: None,
                position_size: 0.0,
                reason: "Kelly公式建议不建仓".to_string(),
                confidence: 0.5,
            })
        }
    }

    fn requires_history(&self) -> bool {
        false // Kelly策略不需要历史价格数据
    }
}

impl Default for KellyStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// 组合策略（结合多个策略）
#[derive(Debug, Clone)]
pub struct CombinedStrategy {
    /// 子策略列表
    pub strategies: Vec<Box<dyn Strategy>>,

    /// 策略权重
    pub weights: Vec<f64>,
}

impl CombinedStrategy {
    /// 创建新的组合策略
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
            weights: Vec::new(),
        }
    }

    /// 添加策略
    pub fn add_strategy(mut self, strategy: Box<dyn Strategy>, weight: f64) -> Self {
        self.strategies.push(strategy);
        self.weights.push(weight);
        self
    }
}

impl Default for CombinedStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for CombinedStrategy {
    fn name(&self) -> &str {
        "Combined Strategy"
    }

    fn analyze(
        &self,
        data_point: &BacktestDataPoint,
        historical_data: &[&BacktestDataPoint],
    ) -> Option<StrategySignal> {
        let mut buy_score = 0.0;
        let mut sell_score = 0.0;
        let mut total_weight = 0.0;
        let mut reasons = Vec::new();

        // 收集所有策略的信号
        for (strategy, weight) in self.strategies.iter().zip(self.weights.iter()) {
            if let Some(signal) = strategy.analyze(data_point, historical_data) {
                match signal.signal_type {
                    SignalType::Buy => {
                        buy_score += signal.strength * signal.confidence * weight;
                        reasons.push(format!(
                            "{}: 买入（强度{:.1}，置信度{:.1}）",
                            strategy.name(),
                            signal.strength,
                            signal.confidence
                        ));
                    },
                    SignalType::Sell => {
                        sell_score += signal.strength * signal.confidence * weight;
                        reasons.push(format!(
                            "{}: 卖出（强度{:.1}，置信度{:.1}）",
                            strategy.name(),
                            signal.strength,
                            signal.confidence
                        ));
                    },
                    SignalType::Hold => {},
                    SignalType::Close => {
                        sell_score += 0.3 * weight;
                    },
                }
                total_weight += weight;
            }
        }

        // 综合判断
        if total_weight == 0.0 {
            return None;
        }

        let avg_buy_score = buy_score / total_weight;
        let avg_sell_score = sell_score / total_weight;

        if avg_buy_score > avg_sell_score && avg_buy_score > 0.3 {
            Some(StrategySignal {
                symbol: data_point.symbol.clone(),
                signal_type: SignalType::Buy,
                strength: avg_buy_score,
                target_price: None,
                position_size: 0.15, // 默认15%仓位
                reason: format!(
                    "综合分析建议买入（买入得分{:.2}，卖出得分{:.2}）\n{}",
                    avg_buy_score,
                    avg_sell_score,
                    reasons.join("\n")
                ),
                confidence: (avg_buy_score / (avg_buy_score + avg_sell_score)),
            })
        } else if avg_sell_score > avg_buy_score && avg_sell_score > 0.3 {
            Some(StrategySignal {
                symbol: data_point.symbol.clone(),
                signal_type: SignalType::Sell,
                strength: avg_sell_score,
                target_price: None,
                position_size: 0.0,
                reason: format!(
                    "综合分析建议卖出（买入得分{:.2}，卖出得分{:.2}）\n{}",
                    avg_buy_score,
                    avg_sell_score,
                    reasons.join("\n")
                ),
                confidence: (avg_sell_score / (avg_buy_score + avg_sell_score)),
            })
        } else {
            Some(StrategySignal {
                symbol: data_point.symbol.clone(),
                signal_type: SignalType::Hold,
                strength: 0.5,
                target_price: None,
                position_size: 0.0,
                reason: "综合分析建议持有".to_string(),
                confidence: 0.5,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graham_strategy() {
        let strategy = GrahamStrategy::new();

        let data_point = BacktestDataPoint {
            date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            symbol: "TEST".to_string(),
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 100.0,
            volume: 1_000_000.0,
            eps: Some(10.0),
            dividend: None,
        };

        let signal = strategy.analyze(&data_point, &[]);
        assert!(signal.is_some());

        let signal = signal.unwrap();
        assert_eq!(signal.symbol, "TEST");
    }

    #[test]
    fn test_kelly_strategy() {
        let strategy = KellyStrategy::new()
            .with_historical_params(0.6, 120.0, 80.0);

        let data_point = BacktestDataPoint {
            date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            symbol: "TEST".to_string(),
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 100.0,
            volume: 1_000_000.0,
            eps: Some(5.0),
            dividend: None,
        };

        let signal = strategy.analyze(&data_point, &[]);
        assert!(signal.is_some());
    }

    #[test]
    fn test_combined_strategy() {
        let graham = Box::new(GrahamStrategy::new()) as Box<dyn Strategy>;
        let kelly = Box::new(KellyStrategy::new()) as Box<dyn Strategy>;

        let combined = CombinedStrategy::new()
            .add_strategy(graham, 0.6)
            .add_strategy(kelly, 0.4);

        assert_eq!(combined.strategies.len(), 2);
        assert_eq!(combined.weights.len(), 2);
    }
}
