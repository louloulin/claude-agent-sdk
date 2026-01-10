//! 交易执行模块
//!
//! 基于Claude Agent SDK架构
//! 提供完整的交易执行功能

pub mod binance;
pub mod okx;
pub mod order_manager;
pub mod emergency_stop;

pub use binance::{BinanceFuturesClient, OrderRequest, OrderResponse, OrderSide, PositionSide};
pub use okx::{OkxClient, OkxOrderRequest, OkxAccountInfo, OkxPosition};
pub use order_manager::{
    OrderManager, RiskEngine, OrderRecord, OrderStatus, OrderReceipt,
    OrderStatistics, OrderStatusInfo, Exchange,
};
pub use emergency_stop::{
    EmergencyStopManager, EmergencyStopReason, EmergencyStopReport,
    RiskMonitor, EmergencyNotification,
};

// 常量定义

/// 默认最大仓位大小
pub const DEFAULT_MAX_POSITION_SIZE: f64 = 10000.0;

/// 默认最大每日亏损
pub const DEFAULT_MAX_DAILY_LOSS: f64 = 1000.0;

/// 默认最大订单大小
pub const DEFAULT_MAX_ORDER_SIZE: f64 = 5000.0;

/// 默认最大杠杆
pub const DEFAULT_MAX_LEVERAGE: u32 = 20;

/// 默认允许的交易对
pub const DEFAULT_ALLOWED_SYMBOLS: &[&str] = &[
    "BTCUSDT",
    "ETHUSDT",
    "BNBUSDT",
    "ADAUSDT",
    "SOLUSDT",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_MAX_POSITION_SIZE, 10000.0);
        assert_eq!(DEFAULT_MAX_DAILY_LOSS, 1000.0);
        assert_eq!(DEFAULT_MAX_ORDER_SIZE, 5000.0);
        assert_eq!(DEFAULT_MAX_LEVERAGE, 20);
        assert_eq!(DEFAULT_ALLOWED_SYMBOLS.len(), 5);
    }
}
