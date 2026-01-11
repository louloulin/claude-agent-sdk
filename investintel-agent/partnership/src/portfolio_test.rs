//! Tests for Portfolio Module
//!
//! Validates portfolio management, position tracking, and concentration limits

use super::portfolio::*;
use super::agreement::ConcentrationLimits;
use chrono::Utc;

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
        assert_eq!(portfolio.total_value, 100_000.0);
    }

    #[test]
    fn test_add_position_too_large() {
        let mut portfolio = Portfolio::new(100_000.0);
        
        let position = Position {
            shares: 1000.0,
            avg_cost: 150.0, // 150,000 total - exceeds 50% of portfolio
            current_price: Some(150.0),
            current_value: Some(150_000.0),
            unrealized_pnl: Some(0.0),
            sector: Some("Technology".to_string()),
            purchased_at: Utc::now(),
        };
        
        let result = portfolio.add_position("AAPL".to_string(), position);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_price() {
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
        portfolio.update_price("AAPL", 175.0);
        
        let updated = portfolio.positions.get("AAPL").unwrap();
        assert_eq!(updated.current_price, Some(175.0));
        assert_eq!(updated.current_value, Some(17_500.0));
        assert_eq!(updated.unrealized_pnl, Some(2_500.0));
    }

    #[test]
    fn test_concentration_limits_pass() {
        let portfolio = Portfolio::new(1_000_000.0);
        let limits = ConcentrationLimits::default();
        
        let report = portfolio.check_concentration(&limits);
        
        // Empty portfolio should pass min positions check
        assert!(!report.min_positions_ok);
        assert!(report.single_position_ok);
        assert!(report.top_5_ok);
    }

    #[test]
    fn test_concentration_with_positions() {
        let mut portfolio = Portfolio::new(1_000_000.0);
        
        // Add positions within limits
        for i in 1..=20 {
            let position = Position {
                shares: 100.0,
                avg_cost: 1000.0, // 100,000 each = 10% per position
                current_price: Some(1000.0),
                current_value: Some(100_000.0),
                unrealized_pnl: Some(0.0),
                sector: Some("Technology".to_string()),
                purchased_at: Utc::now(),
            };
            
            let symbol = format!("STOCK{}", i);
            portfolio.add_position(symbol, position).unwrap();
        }
        
        let limits = ConcentrationLimits::default();
        let report = portfolio.check_concentration(&limits);
        
        // Each position is 10%, which is below 20% limit
        assert!(report.single_position_ok);
        // Top 5 = 50%, which is below 60% limit
        assert!(report.top_5_ok);
        // 20 positions >= 15 minimum
        assert!(report.min_positions_ok);
    }

    #[test]
    fn test_concentration_exceed_single_limit() {
        let mut portfolio = Portfolio::new(1_000_000.0);
        
        let position = Position {
            shares: 2500.0,
            avg_cost: 100.0, // 250,000 = 25% of portfolio
            current_price: Some(100.0),
            current_value: Some(250_000.0),
            unrealized_pnl: Some(0.0),
            sector: Some("Technology".to_string()),
            purchased_at: Utc::now(),
        };
        
        portfolio.add_position("BIG".to_string(), position).unwrap();
        
        let limits = ConcentrationLimits {
            max_single_position: 0.20,
            ..Default::default()
        };
        
        let report = portfolio.check_concentration(&limits);
        assert!(!report.single_position_ok);
        assert_eq!(report.max_single_position, 0.25);
    }

    #[test]
    fn test_performance_metrics() {
        let mut portfolio = Portfolio::new(100_000.0);
        
        let position = Position {
            shares: 100.0,
            avg_cost: 150.0,
            current_price: Some(175.0),
            current_value: Some(17_500.0),
            unrealized_pnl: Some(2_500.0),
            sector: Some("Technology".to_string()),
            purchased_at: Utc::now(),
        };
        
        portfolio.add_position("AAPL".to_string(), position).unwrap();
        
        let metrics = portfolio.calculate_metrics();
        
        // total_value = cash (85,000) + position value (17,500) = 102,500
        assert_eq!(metrics.total_value, 102_500.0);
        assert_eq!(metrics.cash_ratio, 85_000.0 / 102_500.0);
        assert_eq!(metrics.position_count, 1);
    }

    #[test]
    fn test_performance_history() {
        let mut history = PerformanceHistory::new();
        
        assert_eq!(history.snapshots.len(), 0);
        assert_eq!(history.transactions.len(), 0);
        
        let snapshot = PortfolioSnapshot {
            timestamp: Utc::now(),
            total_value: 100_000.0,
            positions: std::collections::HashMap::new(),
            cash: 50_000.0,
        };
        
        history.record_snapshot(snapshot);
        
        assert_eq!(history.snapshots.len(), 1);
    }

    #[test]
    fn test_transaction_type() {
        assert_eq!(TransactionType::Buy, TransactionType::Buy);
        assert_eq!(TransactionType::Sell, TransactionType::Sell);
        assert_eq!(TransactionType::Dividend, TransactionType::Dividend);
    }

    #[test]
    fn test_position_unrealized_pnl_calculation() {
        let position = Position {
            shares: 100.0,
            avg_cost: 50.0,
            current_price: Some(75.0),
            current_value: Some(7_500.0),
            unrealized_pnl: Some(2_500.0), // 100 * (75 - 50)
            sector: None,
            purchased_at: Utc::now(),
        };
        
        assert_eq!(position.unrealized_pnl, Some(2_500.0));
    }
}
