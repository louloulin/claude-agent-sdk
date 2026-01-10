//! 订单管理系统
//!
//! 基于Claude Agent SDK架构
//! 提供完整的订单生命周期管理

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::binance::{BinanceFuturesClient, OrderRequest, OrderResponse, OrderSide, PositionSide};
use super::okx::OkxClient;

/// 订单管理器
pub struct OrderManager {
    binance: Arc<BinanceFuturesClient>,
    okx: Arc<OkxClient>,
    orders: Arc<RwLock<HashMap<String, OrderRecord>>>,
    risk_engine: Arc<RiskEngine>,
    order_queue: Arc<RwLock<Vec<OrderRequest>>>,
    enabled: Arc<RwLock<bool>>,
}

impl OrderManager {
    /// 创建新的订单管理器
    pub fn new(
        binance: BinanceFuturesClient,
        okx: OkxClient,
        risk_engine: RiskEngine,
    ) -> Self {
        Self {
            binance: Arc::new(binance),
            okx: Arc::new(okx),
            orders: Arc::new(RwLock::new(HashMap::new())),
            risk_engine: Arc::new(risk_engine),
            order_queue: Arc::new(RwLock::new(Vec::new())),
            enabled: Arc::new(RwLock::new(true)),
        }
    }

    /// 下单
    pub async fn place_order(&self, request: OrderRequest, exchange: Exchange) -> Result<OrderReceipt> {
        // 检查是否启用
        if !*self.enabled.read().await {
            return Err(anyhow::anyhow!("Trading is currently disabled"));
        }

        // 1. 风险预检查
        self.risk_engine
            .pre_trade_check(&request)
            .await
            .context("Risk check failed")?;

        // 2. 创建订单记录
        let order_id = Uuid::new_v4().to_string();
        let record = OrderRecord {
            id: order_id.clone(),
            exchange_order_id: None,
            request: request.clone(),
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            error_message: None,
            exchange: exchange.clone(),
        };

        // 3. 存储订单记录
        {
            let mut orders = self.orders.write().await;
            orders.insert(order_id.clone(), record);
        }

        // 4. 提交到交易所
        let response = match exchange {
            Exchange::Binance => {
                self.binance
                    .place_order(request)
                    .await
                    .context("Failed to place order on Binance")?
            }
            Exchange::Okx => {
                // OKX订单转换 (这里简化处理)
                return Err(anyhow::anyhow!("OKX order placement not yet implemented"));
            }
        };

        // 5. 更新订单状态
        {
            let mut orders = self.orders.write().await;
            if let Some(record) = orders.get_mut(&order_id) {
                record.status = OrderStatus::Open;
                record.exchange_order_id = Some(response.order_id.clone());
                record.updated_at = Utc::now();
            }
        }

        Ok(OrderReceipt {
            id: order_id,
            exchange_order_id: response.order_id,
            status: OrderStatus::Open,
            created_at: Utc::now(),
        })
    }

    /// 批量下单
    pub async fn place_orders_batch(
        &self,
        requests: Vec<(OrderRequest, Exchange)>,
    ) -> Result<Vec<OrderReceipt>> {
        let mut receipts = Vec::new();

        for (request, exchange) in requests {
            match self.place_order(request, exchange).await {
                Ok(receipt) => receipts.push(receipt),
                Err(e) => {
                    eprintln!("Failed to place order: {}", e);
                    // 继续处理其他订单
                }
            }
        }

        Ok(receipts)
    }

    /// 取消订单
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        let orders = self.orders.read().await;
        let record = orders
            .get(order_id)
            .ok_or_else(|| anyhow::anyhow!("Order not found"))?;

        if record.status != OrderStatus::Open {
            return Err(anyhow::anyhow!("Order is not open, cannot cancel"));
        }

        match &record.exchange {
            Exchange::Binance => {
                if let Some(exchange_order_id) = &record.exchange_order_id {
                    self.binance
                        .cancel_order(&record.request.symbol, exchange_order_id)
                        .await?;

                    // 更新本地状态
                    drop(orders);
                    let mut orders = self.orders.write().await;
                    if let Some(record) = orders.get_mut(order_id) {
                        record.status = OrderStatus::Canceled;
                        record.updated_at = Utc::now();
                    }
                }
            }
            Exchange::Okx => {
                return Err(anyhow::anyhow!("OKX order cancellation not yet implemented"));
            }
        }

        Ok(())
    }

    /// 取消所有订单
    pub async fn cancel_all_orders(&self, symbol: Option<String>) -> Result<Vec<String>> {
        let orders = self.orders.read().await;
        let mut canceled_ids = Vec::new();

        for (order_id, record) in orders.iter() {
            if record.status == OrderStatus::Open {
                let should_cancel = if let Some(sym) = &symbol {
                    &record.request.symbol == sym
                } else {
                    true
                };

                if should_cancel {
                    drop(orders);
                    if let Err(e) = self.cancel_order(order_id).await {
                        eprintln!("Failed to cancel order {}: {}", order_id, e);
                    } else {
                        canceled_ids.push(order_id.clone());
                    }
                    let orders = self.orders.read().await;
                }
            }
        }

        Ok(canceled_ids)
    }

    /// 查询订单状态
    pub async fn get_order_status(&self, order_id: &str) -> Result<OrderStatusInfo> {
        let orders = self.orders.read().await;
        let record = orders
            .get(order_id)
            .ok_or_else(|| anyhow::anyhow!("Order not found"))?;

        Ok(OrderStatusInfo {
            id: record.id.clone(),
            status: record.status,
            exchange_order_id: record.exchange_order_id.clone(),
            symbol: record.request.symbol.clone(),
            side: record.request.side,
            quantity: record.request.quantity,
            price: record.request.price,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    /// 获取所有订单
    pub async fn get_all_orders(&self) -> Vec<OrderRecord> {
        let orders = self.orders.read().await;
        orders.values().cloned().collect()
    }

    /// 获取待处理订单
    pub async fn get_pending_orders(&self) -> Vec<OrderRecord> {
        let orders = self.orders.read().await;
        orders
            .values()
            .filter(|o| o.status == OrderStatus::Open || o.status == OrderStatus::Pending)
            .cloned()
            .collect()
    }

    /// 监控订单状态 (后台任务)
    pub async fn monitor_orders(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            interval.tick().await;

            let orders = self.orders.read().await;
            let open_orders: Vec<_> = orders
                .iter()
                .filter(|(_, o)| o.status == OrderStatus::Open)
                .map(|(id, o)| (id.clone(), o.clone()))
                .collect();

            drop(orders);

            for (order_id, record) in open_orders {
                match &record.exchange {
                    Exchange::Binance => {
                        if let Some(exchange_order_id) = &record.exchange_order_id {
                            match self
                                .binance
                                .get_order_status(&record.request.symbol, exchange_order_id)
                                .await
                            {
                                Ok(status) => {
                                    let mut orders = self.orders.write().await;
                                    if let Some(record) = orders.get_mut(&order_id) {
                                        record.status = status.status.into();
                                        record.updated_at = Utc::now();
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to get order status: {}", e);
                                }
                            }
                        }
                    }
                    Exchange::Okx => {
                        // TODO: Implement OKX order monitoring
                    }
                }
            }
        }
    }

    /// 启用交易
    pub async fn enable_trading(&self) {
        let mut enabled = self.enabled.write().await;
        *enabled = true;
    }

    /// 禁用交易
    pub async fn disable_trading(&self) {
        let mut enabled = self.enabled.write().await;
        *enabled = false;
    }

    /// 获取统计信息
    pub async fn get_statistics(&self) -> OrderStatistics {
        let orders = self.orders.read().await;

        let total_orders = orders.len();
        let pending_orders = orders.values().filter(|o| o.status == OrderStatus::Pending).count();
        let open_orders = orders.values().filter(|o| o.status == OrderStatus::Open).count();
        let filled_orders = orders.values().filter(|o| o.status == OrderStatus::Filled).count();
        let canceled_orders = orders
            .values()
            .filter(|o| o.status == OrderStatus::Canceled)
            .count();
        let failed_orders = orders.values().filter(|o| o.status == OrderStatus::Failed).count();

        OrderStatistics {
            total_orders,
            pending_orders,
            open_orders,
            filled_orders,
            canceled_orders,
            failed_orders,
        }
    }
}

/// 交易所
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Exchange {
    Binance,
    Okx,
}

/// 订单记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRecord {
    pub id: String,
    pub exchange_order_id: Option<String>,
    pub request: OrderRequest,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub error_message: Option<String>,
    pub exchange: Exchange,
}

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Open,
    Filled,
    PartiallyFilled,
    Canceled,
    Failed,
    Rejected,
}

impl From<super::binance::BinanceOrderStatus> for OrderStatus {
    fn from(status: super::binance::BinanceOrderStatus) -> Self {
        match status {
            super::binance::BinanceOrderStatus::New => OrderStatus::Open,
            super::binance::BinanceOrderStatus::PartiallyFilled => OrderStatus::PartiallyFilled,
            super::binance::BinanceOrderStatus::Filled => OrderStatus::Filled,
            super::binance::BinanceOrderStatus::Canceled => OrderStatus::Canceled,
            super::binance::BinanceOrderStatus::Rejected => OrderStatus::Rejected,
            _ => OrderStatus::Open,
        }
    }
}

/// 订单收据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderReceipt {
    pub id: String,
    pub exchange_order_id: String,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
}

/// 订单状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatusInfo {
    pub id: String,
    pub status: OrderStatus,
    pub exchange_order_id: Option<String>,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub price: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 订单统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatistics {
    pub total_orders: usize,
    pub pending_orders: usize,
    pub open_orders: usize,
    pub filled_orders: usize,
    pub canceled_orders: usize,
    pub failed_orders: usize,
}

/// 风险引擎
pub struct RiskEngine {
    max_position_size: f64,
    max_daily_loss: f64,
    max_order_size: f64,
    daily_pnl: Arc<RwLock<f64>>,
    max_leverage: u32,
    allowed_symbols: Vec<String>,
}

impl RiskEngine {
    /// 创建新的风险引擎
    pub fn new(
        max_position_size: f64,
        max_daily_loss: f64,
        max_order_size: f64,
        max_leverage: u32,
        allowed_symbols: Vec<String>,
    ) -> Self {
        Self {
            max_position_size,
            max_daily_loss,
            max_order_size,
            daily_pnl: Arc::new(RwLock::new(0.0)),
            max_leverage,
            allowed_symbols,
        }
    }

    /// 交易前风险检查
    pub async fn pre_trade_check(&self, request: &OrderRequest) -> Result<()> {
        // 1. 检查交易对是否允许
        if !self.allowed_symbols.contains(&request.symbol) {
            return Err(anyhow::anyhow!(
                "Symbol {} is not in the allowed list",
                request.symbol
            ));
        }

        // 2. 检查订单大小
        let notional = request.quantity * request.price.unwrap_or(0.0);
        if notional > self.max_order_size {
            return Err(anyhow::anyhow!(
                "Order size {} exceeds limit {}",
                notional,
                self.max_order_size
            ));
        }

        // 3. 检查每日亏损
        let current_pnl = *self.daily_pnl.read().await;
        if current_pnl < -self.max_daily_loss {
            return Err(anyhow::anyhow!(
                "Daily loss limit reached: {}/{}",
                current_pnl.abs(),
                self.max_daily_loss
            ));
        }

        // 4. 检查仓位大小 (简化版,实际应该查询当前持仓)
        if notional > self.max_position_size {
            return Err(anyhow::anyhow!(
                "Position size {} exceeds limit {}",
                notional,
                self.max_position_size
            ));
        }

        Ok(())
    }

    /// 更新每日盈亏
    pub async fn update_pnl(&self, pnl: f64) {
        let mut daily_pnl = self.daily_pnl.write().await;
        *daily_pnl += pnl;
    }

    /// 重置每日盈亏
    pub async fn reset_daily_pnl(&self) {
        let mut daily_pnl = self.daily_pnl.write().await;
        *daily_pnl = 0.0;
    }

    /// 获取当前每日盈亏
    pub async fn get_daily_pnl(&self) -> f64 {
        *self.daily_pnl.read().await
    }

    /// 检查是否应该触发紧急停止
    pub async fn should_emergency_stop(&self) -> bool {
        let current_pnl = *self.daily_pnl.read().await;
        current_pnl < -self.max_daily_loss
    }

    /// 获取最大杠杆
    pub fn get_max_leverage(&self) -> u32 {
        self.max_leverage
    }

    /// 获取允许的交易对
    pub fn get_allowed_symbols(&self) -> &[String] {
        &self.allowed_symbols
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_enum() {
        assert_eq!(Exchange::Binance, Exchange::Binance);
        assert_eq!(Exchange::Okx, Exchange::Okx);
    }

    #[test]
    fn test_risk_engine_creation() {
        let risk_engine = RiskEngine::new(
            10000.0,
            1000.0,
            5000.0,
            20,
            vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        );

        assert_eq!(risk_engine.max_position_size, 10000.0);
        assert_eq!(risk_engine.max_daily_loss, 1000.0);
        assert_eq!(risk_engine.max_order_size, 5000.0);
        assert_eq!(risk_engine.max_leverage, 20);
        assert_eq!(risk_engine.allowed_symbols.len(), 2);
    }

    #[tokio::test]
    async fn test_risk_engine_pre_trade_check() {
        let risk_engine = RiskEngine::new(
            10000.0,
            1000.0,
            5000.0,
            20,
            vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        );

        // 测试允许的交易对
        let request = OrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            quantity: 1.0,
            price: Some(3000.0),
            stop_price: None,
            position_side: None,
        };

        assert!(risk_engine.pre_trade_check(&request).await.is_ok());

        // 测试不允许的交易对
        let invalid_request = OrderRequest {
            symbol: "INVALID".to_string(),
            side: OrderSide::Buy,
            quantity: 1.0,
            price: Some(3000.0),
            stop_price: None,
            position_side: None,
        };

        assert!(risk_engine.pre_trade_check(&invalid_request).await.is_err());
    }

    #[tokio::test]
    async fn test_risk_engine_daily_pnl() {
        let risk_engine = RiskEngine::new(
            10000.0,
            1000.0,
            5000.0,
            20,
            vec!["BTCUSDT".to_string()],
        );

        // 初始PnL为0
        assert_eq!(risk_engine.get_daily_pnl().await, 0.0);

        // 更新PnL
        risk_engine.update_pnl(100.0).await;
        assert_eq!(risk_engine.get_daily_pnl().await, 100.0);

        // 测试紧急停止
        assert!(!risk_engine.should_emergency_stop().await);

        // 模拟大额亏损
        risk_engine.update_pnl(-1500.0).await;
        assert!(risk_engine.should_emergency_stop().await);
    }

    #[test]
    fn test_order_status_conversion() {
        use super::super::binance::BinanceOrderStatus;

        // 测试状态转换
        let status = OrderStatus::from(BinanceOrderStatus::New);
        assert_eq!(status, OrderStatus::Open);

        let status = OrderStatus::from(BinanceOrderStatus::Filled);
        assert_eq!(status, OrderStatus::Filled);
    }
}
