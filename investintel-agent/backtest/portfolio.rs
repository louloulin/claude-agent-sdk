//! 回测组合管理
//!
//! 管理回测过程中的持仓、交易、资金

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 持仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// 股票代码
    pub symbol: String,

    /// 持仓数量
    pub shares: f64,

    /// 平均成本
    pub cost_basis: f64,

    /// 当前市值
    pub market_value: f64,

    /// 未实现盈亏
    pub unrealized_pnl: f64,

    /// 开仓日期
    pub open_date: chrono::NaiveDate,
}

impl Position {
    /// 创建新的持仓
    pub fn new(
        symbol: String,
        shares: f64,
        cost_basis: f64,
        open_date: chrono::NaiveDate,
    ) -> Self {
        Self {
            symbol,
            shares,
            cost_basis,
            market_value: cost_basis * shares,
            unrealized_pnl: 0.0,
            open_date,
        }
    }

    /// 更新市值
    pub fn update_market_value(&mut self, current_price: f64) {
        self.market_value = self.shares * current_price;
        self.unrealized_pnl = self.market_value - (self.cost_basis * self.shares);
    }

    /// 获取收益率
    pub fn return_pct(&self) -> f64 {
        if self.cost_basis > 0.0 {
            self.unrealized_pnl / (self.cost_basis * self.shares)
        } else {
            0.0
        }
    }
}

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// 交易ID
    pub id: String,

    /// 交易日期
    pub date: chrono::NaiveDate,

    /// 股票代码
    pub symbol: String,

    /// 交易类型
    pub trade_type: TradeType,

    /// 数量
    pub shares: f64,

    /// 价格
    pub price: f64,

    /// 金额
    pub amount: f64,

    /// 佣金
    pub commission: f64,

    /// 滑点
    pub slippage: f64,
}

/// 交易类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeType {
    /// 买入
    Buy,

    /// 卖出
    Sell,

    /// 分红
    Dividend,
}

impl Trade {
    /// 创建新的交易
    pub fn new(
        date: chrono::NaiveDate,
        symbol: String,
        trade_type: TradeType,
        shares: f64,
        price: f64,
        commission: f64,
        slippage: f64,
    ) -> Self {
        let amount = shares * price;

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            date,
            symbol,
            trade_type,
            shares,
            price,
            amount,
            commission,
            slippage,
        }
    }

    /// 获取总成本（含佣金和滑点）
    pub fn total_cost(&self) -> f64 {
        self.amount + self.commission + self.slippage
    }
}

/// 回测组合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestPortfolio {
    /// 现金
    pub cash: f64,

    /// 持仓
    pub positions: HashMap<String, Position>,

    /// 交易历史
    pub trades: Vec<Trade>,

    /// 总市值
    pub total_value: f64,

    /// 创建日期
    pub created_date: chrono::NaiveDate,
}

impl BacktestPortfolio {
    /// 创建新的组合
    pub fn new(initial_capital: f64, created_date: chrono::NaiveDate) -> Self {
        Self {
            cash: initial_capital,
            positions: HashMap::new(),
            trades: Vec::new(),
            total_value: initial_capital,
            created_date,
        }
    }

    /// 买入股票
    pub fn buy(
        &mut self,
        symbol: String,
        shares: f64,
        price: f64,
        date: chrono::NaiveDate,
        commission_rate: f64,
        slippage_rate: f64,
    ) -> Result<(), anyhow::Error> {
        let amount = shares * price;
        let commission = amount * commission_rate;
        let slippage = amount * slippage_rate;
        let total_cost = amount + commission + slippage;

        if total_cost > self.cash {
            return Err(anyhow::anyhow!("资金不足：需要${:.2}，现金${:.2}", total_cost, self.cash));
        }

        // 扣除现金
        self.cash -= total_cost;

        // 更新或创建持仓
        if let Some(position) = self.positions.get_mut(&symbol) {
            // 加仓
            let old_cost = position.cost_basis * position.shares;
            position.shares += shares;
            position.cost_basis = (old_cost + amount) / position.shares;
            position.open_date = date; // 更新开仓日期
        } else {
            // 新建仓位
            self.positions.insert(
                symbol.clone(),
                Position::new(symbol.clone(), shares, price, date),
            );
        }

        // 记录交易
        self.trades.push(Trade::new(
            date,
            symbol,
            TradeType::Buy,
            shares,
            price,
            commission,
            slippage,
        ));

        Ok(())
    }

    /// 卖出股票
    pub fn sell(
        &mut self,
        symbol: &str,
        shares: f64,
        price: f64,
        date: chrono::NaiveDate,
        commission_rate: f64,
        slippage_rate: f64,
    ) -> Result<(), anyhow::Error> {
        if let Some(position) = self.positions.get_mut(symbol) {
            if shares > position.shares {
                return Err(anyhow::anyhow!("持仓不足：尝试卖出{}股，持仓{}股", shares, position.shares));
            }

            let amount = shares * price;
            let commission = amount * commission_rate;
            let slippage = amount * slippage_rate;
            let net_proceeds = amount - commission - slippage;

            // 增加现金
            self.cash += net_proceeds;

            // 更新持仓
            position.shares -= shares;

            // 如果全部卖出，移除持仓
            if position.shares < 0.0001 {
                self.positions.remove(symbol);
            }

            // 记录交易
            self.trades.push(Trade::new(
                date,
                symbol.to_string(),
                TradeType::Sell,
                shares,
                price,
                commission,
                slippage,
            ));

            Ok(())
        } else {
            Err(anyhow::anyhow!("无持仓：{}", symbol))
        }
    }

    /// 更新市值
    pub fn update_value(&mut self, prices: &HashMap<String, f64>) {
        let mut positions_value = 0.0;

        for (symbol, position) in self.positions.iter_mut() {
            if let Some(&price) = prices.get(symbol) {
                position.update_market_value(price);
                positions_value += position.market_value;
            }
        }

        self.total_value = self.cash + positions_value;
    }

    /// 获取持仓列表
    pub fn get_positions(&self) -> Vec<&Position> {
        self.positions.values().collect()
    }

    /// 获取指定股票的持仓
    pub fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    /// 计算权重
    pub fn calculate_weights(&self) -> HashMap<String, f64> {
        let mut weights = HashMap::new();

        for (symbol, position) in &self.positions {
            let weight = if self.total_value > 0.0 {
                position.market_value / self.total_value
            } else {
                0.0
            };
            weights.insert(symbol.clone(), weight);
        }

        weights
    }

    /// 获取现金比例
    pub fn cash_ratio(&self) -> f64 {
        if self.total_value > 0.0 {
            self.cash / self.total_value
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_creation() {
        let portfolio = BacktestPortfolio::new(
            100_000.0,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        );

        assert_eq!(portfolio.cash, 100_000.0);
        assert_eq!(portfolio.total_value, 100_000.0);
        assert!(portfolio.positions.is_empty());
    }

    #[test]
    fn test_buy_stock() {
        let mut portfolio = BacktestPortfolio::new(
            100_000.0,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        );

        portfolio
            .buy(
                "AAPL".to_string(),
                100.0,
                150.0,
                NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
                0.001,
                0.0001,
            )
            .unwrap();

        assert_eq!(portfolio.trades.len(), 1);
        assert!(portfolio.positions.contains_key("AAPL"));

        let position = portfolio.get_position("AAPL").unwrap();
        assert_eq!(position.shares, 100.0);
    }

    #[test]
    fn test_sell_stock() {
        let mut portfolio = BacktestPortfolio::new(
            100_000.0,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        );

        // 先买入
        portfolio
            .buy(
                "AAPL".to_string(),
                100.0,
                150.0,
                NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
                0.001,
                0.0001,
            )
            .unwrap();

        // 再卖出
        portfolio
            .sell(
                "AAPL",
                50.0,
                160.0,
                NaiveDate::from_ymd_opt(2023, 1, 3).unwrap(),
                0.001,
                0.0001,
            )
            .unwrap();

        assert_eq!(portfolio.trades.len(), 2);
        let position = portfolio.get_position("AAPL").unwrap();
        assert_eq!(position.shares, 50.0);
    }

    #[test]
    fn test_update_value() {
        let mut portfolio = BacktestPortfolio::new(
            100_000.0,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        );

        portfolio
            .buy(
                "AAPL".to_string(),
                100.0,
                150.0,
                NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
                0.001,
                0.0001,
            )
            .unwrap();

        let mut prices = HashMap::new();
        prices.insert("AAPL".to_string(), 160.0);

        portfolio.update_value(&prices);

        assert!(portfolio.total_value > 100_000.0); // 应该有盈利
    }
}
