//! Portfolio Management
//!
//! Manages investment portfolio with:
//! - Position tracking
//! - Performance history
//! - Risk metrics
//! - Concentration monitoring

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::agreement::ConcentrationLimits;

/// Investment portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    /// Portfolio positions
    pub positions: HashMap<String, Position>,

    /// Cash balance
    pub cash: f64,

    /// Total value (cash + positions)
    pub total_value: f64,

    /// Performance history
    pub performance_history: PerformanceHistory,

    /// Creation date
    pub created_at: DateTime<Utc>,
}

impl Portfolio {
    /// Create new portfolio with initial capital
    pub fn new(initial_capital: f64) -> Self {
        Self {
            positions: HashMap::new(),
            cash: initial_capital,
            total_value: initial_capital,
            performance_history: PerformanceHistory::new(),
            created_at: Utc::now(),
        }
    }

    /// Add position
    pub fn add_position(&mut self, symbol: String, position: Position) -> anyhow::Result<()> {
        // Check concentration limits
        let position_value = position.shares * position.avg_cost;
        let new_weight = position_value / self.total_value;

        if new_weight > 0.50 {
            anyhow::bail!("Position too large: {}%", new_weight * 100.0);
        }

        // Update cash
        self.cash -= position_value;

        // Store position data for transaction record before moving
        let shares = position.shares;
        let avg_cost = position.avg_cost;

        // Add position
        self.positions.insert(symbol.clone(), position);

        // Update total value
        self.update_total_value();

        // Record in history
        self.performance_history.record_transaction(PortfolioTransaction {
            symbol,
            transaction_type: TransactionType::Buy,
            shares,
            price: avg_cost,
            timestamp: Utc::now(),
        });

        Ok(())
    }

    /// Update position price
    pub fn update_price(&mut self, symbol: &str, current_price: f64) {
        if let Some(position) = self.positions.get_mut(symbol) {
            position.current_price = Some(current_price);
            position.current_value = Some(position.shares * current_price);
            position.unrealized_pnl = Some(position.current_value.unwrap() - (position.shares * position.avg_cost));
        }
        self.update_total_value();
    }

    /// Check concentration limits
    pub fn check_concentration(&self, limits: &ConcentrationLimits) -> ConcentrationReport {
        let mut top_positions = Vec::new();
        let mut industry_exposure: HashMap<String, f64> = HashMap::new();

        // Calculate position weights
        for (symbol, position) in &self.positions {
            let weight = position.current_value.unwrap_or(position.shares * position.avg_cost) / self.total_value;
            top_positions.push((symbol.clone(), weight));
        }

        // Sort by weight
        top_positions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Check single position limit
        let max_single = top_positions.first().map(|(_, w)| *w).unwrap_or(0.0);
        let single_position_ok = max_single <= limits.max_single_position;

        // Check top 5 concentration
        let top_5_sum: f64 = top_positions.iter().take(5).map(|(_, w)| *w).sum();
        let top_5_ok = top_5_sum <= limits.max_top_5_concentration;

        // Check minimum positions
        let min_positions_ok = self.positions.len() >= limits.min_positions;

        ConcentrationReport {
            single_position_ok,
            top_5_ok,
            min_positions_ok,
            max_single_position: max_single,
            top_5_concentration: top_5_sum,
            position_count: self.positions.len(),
        }
    }

    /// Update total value
    fn update_total_value(&mut self) {
        let positions_value: f64 = self.positions
            .values()
            .map(|p| p.current_value.unwrap_or(p.shares * p.avg_cost))
            .sum();

        self.total_value = self.cash + positions_value;
    }

    /// Calculate performance metrics
    pub fn calculate_metrics(&self) -> PerformanceMetrics {
        let total_return = if self.created_at.timestamp() > 0 {
            (self.total_value - (self.cash / (1.0 - self.cash / self.total_value))) / self.total_value
        } else {
            0.0
        };

        PerformanceMetrics {
            total_value: self.total_value,
            total_return,
            cash_ratio: self.cash / self.total_value,
            position_count: self.positions.len(),
            sharpe_ratio: None,
            max_drawdown: None,
            volatility: None,
        }
    }
}

/// Portfolio position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Number of shares
    pub shares: f64,

    /// Average cost basis
    pub avg_cost: f64,

    /// Current price (if available)
    pub current_price: Option<f64>,

    /// Current market value
    pub current_value: Option<f64>,

    /// Unrealized P&L
    pub unrealized_pnl: Option<f64>,

    /// Industry sector
    pub sector: Option<String>,

    /// Purchase date
    pub purchased_at: DateTime<Utc>,
}

/// Performance history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHistory {
    /// Historical snapshots
    pub snapshots: Vec<PortfolioSnapshot>,

    /// Transactions
    pub transactions: Vec<PortfolioTransaction>,
}

impl PerformanceHistory {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            transactions: Vec::new(),
        }
    }

    pub fn record_snapshot(&mut self, snapshot: PortfolioSnapshot) {
        self.snapshots.push(snapshot);
    }

    pub fn record_transaction(&mut self, transaction: PortfolioTransaction) {
        self.transactions.push(transaction);
    }
}

/// Portfolio snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub timestamp: DateTime<Utc>,
    pub total_value: f64,
    pub positions: HashMap<String, f64>,
    pub cash: f64,
}

/// Transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioTransaction {
    pub symbol: String,
    pub transaction_type: TransactionType,
    pub shares: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionType {
    Buy,
    Sell,
    Dividend,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_value: f64,
    pub total_return: f64,
    pub cash_ratio: f64,
    pub position_count: usize,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub volatility: Option<f64>,
}

/// Concentration report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentrationReport {
    pub single_position_ok: bool,
    pub top_5_ok: bool,
    pub min_positions_ok: bool,
    pub max_single_position: f64,
    pub top_5_concentration: f64,
    pub position_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_creation() {
        let portfolio = Portfolio::new(100_000.0);
        assert_eq!(portfolio.cash, 100_000.0);
        assert_eq!(portfolio.total_value, 100_000.0);
        assert_eq!(portfolio.positions.len(), 0);
    }

    #[test]
    fn test_add_position() {
        let mut portfolio = Portfolio::new(100_000.0);

        let position = Position {
            shares: 100.0,
            avg_cost: 150.0,
            current_price: Some(150.0),
            current_value: Some(15_000.0),
            unrealized_pnl: Some(0.0),
            sector: Some("Technology".to_string()),
            purchased_at: Utc::now(),
        };

        portfolio.add_position("AAPL".to_string(), position).unwrap();

        assert_eq!(portfolio.positions.len(), 1);
        assert_eq!(portfolio.cash, 85_000.0);
    }
}
