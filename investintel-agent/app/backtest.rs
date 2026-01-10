// backtest.rs - Comprehensive backtesting engine with performance metrics
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trading signal
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

/// Trade record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub ticker: String,
    pub signal: Signal,
    pub price: f64,
    pub shares: f64,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
}

/// Backtest configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub initial_capital: f64,
    pub commission: f64,
    pub slippage: f64,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub symbols: Vec<String>,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 100_000.0,
            commission: 0.001, // 0.1% per trade
            slippage: 0.0001,  // 0.01% slippage
            start_date: Utc::now() - Duration::days(365),
            end_date: Utc::now(),
            symbols: vec!["AAPL".to_string()],
        }
    }
}

/// Backtest result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub strategy_name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub initial_capital: f64,
    pub final_value: f64,
    pub total_return: f64,
    pub annual_return: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<(DateTime<Utc>, f64)>,
    pub monthly_returns: Vec<f64>,
    pub parameters: serde_json::Value,
}

/// Portfolio state during backtest
#[derive(Debug, Clone)]
struct PortfolioState {
    cash: f64,
    positions: HashMap<String, f64>, // ticker -> shares
    equity: f64,
}

/// Backtesting engine
pub struct BacktestEngine {
    config: BacktestConfig,
    state: PortfolioState,
    trades: Vec<Trade>,
    equity_curve: Vec<(DateTime<Utc>, f64)>,
}

impl BacktestEngine {
    /// Create a new backtest engine
    pub fn new(config: BacktestConfig) -> Self {
        Self {
            config,
            state: PortfolioState {
                cash: config.initial_capital,
                positions: HashMap::new(),
                equity: config.initial_capital,
            },
            trades: vec![],
            equity_curve: vec![],
        }
    }

    /// Run a backtest with a strategy function
    pub fn run<F>(&mut self, mut strategy: F, data: &HashMap<String, Vec<PriceData>>) -> Result<BacktestResult>
    where
        F: FnMut(&str, &PriceData, &PortfolioState) -> Signal,
    {
        let mut timestamps = Vec::new();

        // Collect all unique timestamps across all symbols
        for (_, prices) in data.iter() {
            for price in prices {
                if !timestamps.contains(&price.timestamp) {
                    timestamps.push(price.timestamp);
                }
            }
        }
        timestamps.sort();

        // Process each timestamp
        for timestamp in &timestamps {
            // Update equity
            self.update_equity(data, *timestamp);

            // Process each symbol
            for ticker in &self.config.symbols {
                if let Some(prices) = data.get(ticker) {
                    if let Some(price_data) = prices.iter().find(|p| p.timestamp == *timestamp) {
                        // Generate signal
                        let signal = strategy(ticker, price_data, &self.state);

                        // Execute signal
                        self.execute_signal(ticker, signal, price_data, *timestamp)?;
                    }
                }
            }

            // Record equity curve
            self.equity_curve.push((*timestamp, self.state.equity));
        }

        // Calculate performance metrics
        self.calculate_metrics()
    }

    /// Execute a trading signal
    fn execute_signal(
        &mut self,
        ticker: &str,
        signal: Signal,
        price_data: &PriceData,
        timestamp: DateTime<Utc>,
    ) -> Result<()> {
        match signal {
            Signal::Buy => {
                let price = price_data.close * (1.0 + self.config.slippage);
                let commission = price * self.config.commission;

                if self.state.cash > price + commission {
                    let max_shares = (self.state.cash - commission) / price;
                    let shares = max_shares.floor();

                    if shares > 0.0 {
                        let cost = shares * price + commission;
                        self.state.cash -= cost;
                        *self.state.positions.entry(ticker.to_string()).or_insert(0.0) += shares;

                        self.trades.push(Trade {
                            ticker: ticker.to_string(),
                            signal,
                            price,
                            shares,
                            timestamp,
                            reason: "Strategy buy signal".to_string(),
                        });
                    }
                }
            }
            Signal::Sell => {
                let shares = self.state.positions.get(ticker).copied().unwrap_or(0.0);

                if shares > 0.0 {
                    let price = price_data.close * (1.0 - self.config.slippage);
                    let commission = price * shares * self.config.commission;
                    let proceeds = shares * price - commission;

                    self.state.cash += proceeds;
                    self.state.positions.remove(ticker);

                    self.trades.push(Trade {
                        ticker: ticker.to_string(),
                        signal,
                        price,
                        shares,
                        timestamp,
                        reason: "Strategy sell signal".to_string(),
                    });
                }
            }
            Signal::Hold => {}
        }

        Ok(())
    }

    /// Update portfolio equity
    fn update_equity(&mut self, data: &HashMap<String, Vec<PriceData>>, timestamp: DateTime<Utc>) {
        let mut total_value = self.state.cash;

        for (ticker, shares) in &self.state.positions {
            if let Some(prices) = data.get(ticker) {
                if let Some(price_data) = prices.iter().find(|p| p.timestamp == timestamp) {
                    total_value += shares * price_data.close;
                }
            }
        }

        self.state.equity = total_value;
    }

    /// Calculate performance metrics
    fn calculate_metrics(&self) -> Result<BacktestResult> {
        if self.equity_curve.is_empty() {
            return Err(anyhow!("No equity data"));
        }

        let initial_value = self.config.initial_capital;
        let final_value = self.state.equity;
        let total_return = (final_value / initial_value - 1.0) * 100.0;

        // Calculate annual return
        let days = (self.config.end_date - self.config.start_date).num_days() as f64;
        let annual_return = ((final_value / initial_value).powf(365.0 / days) - 1.0) * 100.0;

        // Calculate equity returns
        let mut returns = Vec::new();
        for i in 1..self.equity_curve.len() {
            let prev_val = self.equity_curve[i - 1].1;
            let curr_val = self.equity_curve[i].1;
            returns.push((curr_val / prev_val) - 1.0);
        }

        // Calculate Sharpe Ratio (assuming 2% risk-free rate)
        let rf_daily = 0.02 / 365.0;
        let excess_returns: Vec<f64> = returns.iter().map(|r| r - rf_daily).collect();
        let avg_excess = excess_returns.iter().sum::<f64>() / excess_returns.len() as f64;
        let std_dev = std_deviation(&returns);
        let sharpe_ratio = if std_dev > 0.0 {
            (avg_excess / std_dev) * (365.0_f64).sqrt()
        } else {
            0.0
        };

        // Calculate Sortino Ratio
        let downside_returns: Vec<f64> = returns.iter().filter(|r| **r < 0.0).copied().collect();
        let downside_std = std_deviation(&downside_returns);
        let sortino_ratio = if downside_std > 0.0 {
            (avg_excess / downside_std) * (365.0_f64).sqrt()
        } else {
            0.0
        };

        // Calculate Max Drawdown
        let mut max_drawdown = 0.0;
        let mut peak = self.equity_curve[0].1;

        for (_, equity) in &self.equity_curve {
            if *equity > peak {
                peak = *equity;
            }
            let drawdown = (peak - *equity) / peak * 100.0;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        // Calculate trade statistics
        let total_trades = self.trades.len();
        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut total_profit = 0.0;
        let mut total_loss = 0.0;
        let mut largest_win = 0.0;
        let mut largest_loss = 0.0;

        // Group trades into pairs (buy-sell)
        let mut position_map: HashMap<String, (f64, DateTime<Utc>)> = HashMap::new();

        for trade in &self.trades {
            if trade.signal == Signal::Buy {
                position_map.insert(trade.ticker.clone(), (trade.shares, trade.timestamp));
            } else if trade.signal == Signal::Sell {
                if let Some((shares, buy_time)) = position_map.get(&trade.ticker) {
                    let profit = (trade.price - (trade.cost_per_share(buy_time, *shares))) * *shares;

                    if profit > 0.0 {
                        winning_trades += 1;
                        total_profit += profit;
                        if profit > largest_win {
                            largest_win = profit;
                        }
                    } else {
                        losing_trades += 1;
                        total_loss += profit.abs();
                        if profit < largest_loss {
                            largest_loss = profit;
                        }
                    }
                }
            }
        }

        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let avg_win = if winning_trades > 0 {
            total_profit / winning_trades as f64
        } else {
            0.0
        };

        let avg_loss = if losing_trades > 0 {
            total_loss / losing_trades as f64
        } else {
            0.0
        };

        let profit_factor = if total_loss > 0.0 {
            total_profit / total_loss
        } else {
            0.0
        };

        // Calculate monthly returns
        let monthly_returns = calculate_monthly_returns(&self.equity_curve);

        Ok(BacktestResult {
            strategy_name: "Default Strategy".to_string(),
            start_date: self.config.start_date,
            end_date: self.config.end_date,
            initial_capital: initial_value,
            final_value,
            total_return,
            annual_return,
            sharpe_ratio,
            sortino_ratio,
            max_drawdown,
            win_rate,
            profit_factor,
            total_trades,
            winning_trades,
            losing_trades,
            avg_win,
            avg_loss,
            largest_win,
            largest_loss,
            trades: self.trades.clone(),
            equity_curve: self.equity_curve.clone(),
            monthly_returns,
            parameters: serde_json::json!({
                "commission": self.config.commission,
                "slippage": self.config.slippage,
            }),
        })
    }
}

impl Trade {
    /// Helper to calculate cost per share (simplified)
    fn cost_per_share(&self, _buy_time: DateTime<Utc>, _shares: f64) -> f64 {
        self.price
    }
}

/// Price data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

/// Calculate standard deviation
fn std_deviation(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
    variance.sqrt()
}

/// Calculate monthly returns from equity curve
fn calculate_monthly_returns(equity_curve: &[(DateTime<Utc>, f64)]) -> Vec<f64> {
    let mut monthly_returns = Vec::new();
    let mut monthly_data: HashMap<String, Vec<f64>> = HashMap::new();

    for (timestamp, equity) in equity_curve {
        let month_key = format!("{}-{}", timestamp.year(), timestamp.month());
        monthly_data.entry(month_key).or_insert_with(Vec::new).push(*equity);
    }

    let mut prev_month_value = None;
    for month_key in monthly_data.keys().sorted() {
        if let Some(equities) = monthly_data.get(month_key) {
            if let Some(&first) = equities.first() {
                if let Some(&prev_value) = prev_month_value {
                    let monthly_return = (first / prev_value - 1.0) * 100.0;
                    monthly_returns.push(monthly_return);
                }
                prev_month_value = equities.last().copied();
            }
        }
    }

    monthly_returns
}

/// Predefined strategies

/// Simple moving average crossover strategy
pub fn sma_crossover_strategy(short_period: usize, long_period: usize) -> impl Fn(&str, &PriceData, &PortfolioState) -> Signal {
    move |_ticker: &str, price: &PriceData, _state: &PortfolioState| -> Signal {
        // This is a placeholder - actual implementation needs historical data
        Signal::Hold
    }
}

/// Mean reversion strategy using Bollinger Bands
pub fn bollinger_band_strategy() -> impl Fn(&str, &PriceData, &PortfolioState) -> Signal {
    move |_ticker: &str, price: &PriceData, _state: &PortfolioState| -> Signal {
        // Placeholder implementation
        if price.close < price.low * 1.02 {
            Signal::Buy
        } else if price.close > price.high * 0.98 {
            Signal::Sell
        } else {
            Signal::Hold
        }
    }
}

/// RSI-based strategy
pub fn rsi_strategy(oversold: f64, overbought: f64) -> impl Fn(&str, &PriceData, &PortfolioState) -> Signal {
    move |_ticker: &str, _price: &PriceData, _state: &PortfolioState| -> Signal {
        // Placeholder - needs RSI calculation
        Signal::Hold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backtest_engine() {
        let config = BacktestConfig {
            initial_capital: 100_000.0,
            commission: 0.001,
            slippage: 0.0001,
            start_date: Utc::now() - Duration::days(100),
            end_date: Utc::now(),
            symbols: vec!["AAPL".to_string()],
        };

        let mut engine = BacktestEngine::new(config);

        // Create simple test data
        let mut data = HashMap::new();
        let prices = (0..100)
            .map(|i| PriceData {
                timestamp: Utc::now() - Duration::days(100 - i),
                open: 150.0 + i as f64,
                high: 151.0 + i as f64,
                low: 149.0 + i as f64,
                close: 150.5 + i as f64,
                volume: 1_000_000,
            })
            .collect();

        data.insert("AAPL".to_string(), prices);

        // Run simple strategy
        let result = engine
            .run(
                |_ticker, price, state| {
                    if price.close < 155.0 && state.cash > price.close * 100 {
                        Signal::Buy
                    } else if price.close > 160.0 {
                        Signal::Sell
                    } else {
                        Signal::Hold
                    }
                },
                &data,
            )
            .unwrap();

        assert_eq!(result.initial_capital, 100_000.0);
        assert!(result.final_value > 0.0);
        assert!(!result.equity_curve.is_empty());
    }
}
