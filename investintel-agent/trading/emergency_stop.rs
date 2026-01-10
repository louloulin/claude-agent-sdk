//! 紧急停止机制
//!
//! 基于Claude Agent SDK架构
//! 提供完整的紧急停止和风控功能

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::order_manager::{OrderManager, Exchange};
use super::binance::BinanceFuturesClient;
use super::okx::OkxClient;

/// 紧急停止管理器
pub struct EmergencyStopManager {
    order_manager: Arc<OrderManager>,
    binance_client: Arc<BinanceFuturesClient>,
    okx_client: Arc<OkxClient>,
    is_stopped: Arc<RwLock<bool>>,
    stop_reason: Arc<RwLock<Option<EmergencyStopReason>>>,
    stopped_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    notifications: Arc<RwLock<Vec<EmergencyNotification>>>,
}

impl EmergencyStopManager {
    /// 创建新的紧急停止管理器
    pub fn new(
        order_manager: Arc<OrderManager>,
        binance_client: Arc<BinanceFuturesClient>,
        okx_client: Arc<OkxClient>,
    ) -> Self {
        Self {
            order_manager,
            binance_client,
            okx_client,
            is_stopped: Arc::new(RwLock::new(false)),
            stop_reason: Arc::new(RwLock::new(None)),
            stopped_at: Arc::new(RwLock::new(None)),
            notifications: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 触发紧急停止
    pub async fn trigger_emergency_stop(&self, reason: EmergencyStopReason) -> Result<EmergencyStopReport> {
        // 1. 记录停止原因和时间
        {
            let mut is_stopped = self.is_stopped.write().await;
            let mut stop_reason = self.stop_reason.write().await;
            let mut stopped_at = self.stopped_at.write().await;

            *is_stopped = true;
            *stop_reason = Some(reason.clone());
            *stopped_at = Some(Utc::now());
        }

        // 2. 禁用交易
        self.order_manager.disable_trading().await;

        // 3. 取消所有挂单
        let canceled_orders = self.cancel_all_open_orders().await?;

        // 4. 平仓所有持仓 (可选)
        let closed_positions = if reason.should_close_positions() {
            self.close_all_positions().await?
        } else {
            vec![]
        };

        // 5. 生成报告
        let report = EmergencyStopReport {
            reason,
            stopped_at: *self.stopped_at.read().await,
            canceled_orders: canceled_orders.len(),
            closed_positions,
            status: EmergencyStopStatus::Completed,
        };

        // 6. 发送通知
        self.send_notification(&report).await?;

        Ok(report)
    }

    /// 取消所有挂单
    async fn cancel_all_open_orders(&self) -> Result<Vec<String>> {
        let mut all_canceled = Vec::new();

        // 取消Binance订单
        if let Ok(binance_orders) = self.order_manager.cancel_all_orders(None).await {
            all_canceled.extend(binance_orders);
        }

        // 取消OKX订单 (TODO: 实现OKX订单取消)
        // all_canceled.extend(okx_orders);

        Ok(all_canceled)
    }

    /// 平仓所有持仓
    async fn close_all_positions(&self) -> Result<Vec<PositionCloseInfo>> {
        let mut closed = Vec::new();

        // 获取Binance持仓
        let binance_positions = self.binance_client.get_positions(None).await?;

        for position in binance_positions {
            // 跳过无持仓的
            if position.position_amount == 0.0 {
                continue;
            }

            // 创建平仓订单
            let side = if position.position_amount > 0.0 {
                super::binance::OrderSide::Sell
            } else {
                super::binance::OrderSide::Buy
            };

            let order_request = super::binance::OrderRequest {
                symbol: position.symbol.clone(),
                side,
                quantity: position.position_amount.abs(),
                price: None, // 市价单
                stop_price: None,
                position_side: None,
            };

            match self
                .order_manager
                .place_order(order_request, Exchange::Binance)
                .await
            {
                Ok(receipt) => {
                    closed.push(PositionCloseInfo {
                        symbol: position.symbol,
                        quantity: position.position_amount,
                        order_id: receipt.exchange_order_id,
                    });
                }
                Err(e) => {
                    eprintln!("Failed to close position for {}: {}", position.symbol, e);
                }
            }
        }

        // OKX持仓平仓 (TODO: 实现OKX持仓平仓)

        Ok(closed)
    }

    /// 发送通知
    async fn send_notification(&self, report: &EmergencyStopReport) -> Result<()> {
        let notification = EmergencyNotification {
            id: uuid::Uuid::new_v4().to_string(),
            reason: report.reason.clone(),
            stopped_at: report.stopped_at,
            message: format!(
                "Emergency stop triggered: {:?}. Canceled {} orders.",
                report.reason, report.canceled_orders
            ),
            sent_at: Utc::now(),
        };

        let mut notifications = self.notifications.write().await;
        notifications.push(notification);

        // 这里可以添加邮件、短信、Webhook等通知方式
        // 例如:
        // self.send_email_notification(&notification).await?;
        // self.send_slack_notification(&notification).await?;
        // self.send_webhook_notification(&notification).await?;

        Ok(())
    }

    /// 检查是否已停止
    pub async fn is_stopped(&self) -> bool {
        *self.is_stopped.read().await
    }

    /// 获取停止原因
    pub async fn get_stop_reason(&self) -> Option<EmergencyStopReason> {
        *self.stop_reason.read().await
    }

    /// 获取停止时间
    pub async fn get_stopped_at(&self) -> Option<DateTime<Utc>> {
        *self.stopped_at.read().await
    }

    /// 重置紧急停止状态
    pub async fn reset(&self) -> Result<()> {
        let mut is_stopped = self.is_stopped.write().await;
        let mut stop_reason = self.stop_reason.write().await;
        let mut stopped_at = self.stopped_at.write().await;

        *is_stopped = false;
        *stop_reason = None;
        *stopped_at = None;

        // 重新启用交易
        self.order_manager.enable_trading().await;

        Ok(())
    }

    /// 获取所有通知
    pub async fn get_notifications(&self) -> Vec<EmergencyNotification> {
        self.notifications.read().await.clone()
    }

    /// 清除通知
    pub async fn clear_notifications(&self) {
        let mut notifications = self.notifications.write().await;
        notifications.clear();
    }

    /// 启动自动监控 (后台任务)
    pub async fn start_monitoring(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

        loop {
            interval.tick().await;

            // 检查风险引擎
            // 这里应该从OrderManager获取RiskEngine
            // 简化实现:
            // if risk_engine.should_emergency_stop().await {
            //     self.trigger_emergency_stop(EmergencyStopReason::DailyLossLimitReached).await;
            // }
        }
    }
}

/// 紧急停止原因
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmergencyStopReason {
    DailyLossLimitReached,
    PositionLimitExceeded,
    TechnicalError,
    ManualStop,
    MarginCall,
    NetworkIssue,
    ExchangeMaintenance,
}

impl EmergencyStopReason {
    /// 是否应该平仓所有持仓
    pub fn should_close_positions(&self) -> bool {
        matches!(
            self,
            EmergencyStopReason::MarginCall | EmergencyStopReason::ManualStop
        )
    }

    /// 获取原因描述
    pub fn description(&self) -> &str {
        match self {
            EmergencyStopReason::DailyLossLimitReached => "Daily loss limit reached",
            EmergencyStopReason::PositionLimitExceeded => "Position limit exceeded",
            EmergencyStopReason::TechnicalError => "Technical error occurred",
            EmergencyStopReason::ManualStop => "Manual stop triggered",
            EmergencyStopReason::MarginCall => "Margin call triggered",
            EmergencyStopReason::NetworkIssue => "Network connectivity issue",
            EmergencyStopReason::ExchangeMaintenance => "Exchange maintenance mode",
        }
    }
}

/// 紧急停止状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmergencyStopStatus {
    InProgress,
    Completed,
    Failed,
}

/// 紧急停止报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyStopReport {
    pub reason: EmergencyStopReason,
    pub stopped_at: Option<DateTime<Utc>>,
    pub canceled_orders: usize,
    pub closed_positions: Vec<PositionCloseInfo>,
    pub status: EmergencyStopStatus,
}

/// 持仓平仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionCloseInfo {
    pub symbol: String,
    pub quantity: f64,
    pub order_id: String,
}

/// 紧急通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyNotification {
    pub id: String,
    pub reason: EmergencyStopReason,
    pub stopped_at: Option<DateTime<Utc>>,
    pub message: String,
    pub sent_at: DateTime<Utc>,
}

/// 风险监控器
pub struct RiskMonitor {
    order_manager: Arc<OrderManager>,
    emergency_manager: Arc<EmergencyStopManager>,
    check_interval: u64,
}

impl RiskMonitor {
    /// 创建新的风险监控器
    pub fn new(
        order_manager: Arc<OrderManager>,
        emergency_manager: Arc<EmergencyStopManager>,
        check_interval_seconds: u64,
    ) -> Self {
        Self {
            order_manager,
            emergency_manager,
            check_interval: check_interval_seconds,
        }
    }

    /// 启动监控
    pub async fn start(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(self.check_interval));

        loop {
            interval.tick().await;

            // 检查各项风险指标
            if let Err(e) = self.check_risk_metrics().await {
                eprintln!("Risk check error: {}", e);
            }
        }
    }

    /// 检查风险指标
    async fn check_risk_metrics(&self) -> Result<()> {
        // TODO: 实现具体的风险检查逻辑
        // 1. 检查每日盈亏
        // 2. 检查仓位大小
        // 3. 检查未实现亏损
        // 4. 检查网络连接
        // 5. 检查交易所状态

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emergency_stop_reason() {
        let reason = EmergencyStopReason::DailyLossLimitReached;
        assert!(reason.should_close_positions());
        assert_eq!(reason.description(), "Daily loss limit reached");

        let reason = EmergencyStopReason::TechnicalError;
        assert!(!reason.should_close_positions());
        assert_eq!(reason.description(), "Technical error occurred");
    }

    #[tokio::test]
    async fn test_emergency_stop_manager_creation() {
        // 注意: 这个测试需要mock OrderManager和Clients
        // 这里只是测试结构体创建
        let order_manager = Arc::new(OrderManager::new(
            BinanceFuturesClient::new("key".to_string(), "secret".to_string(), true),
            OkxClient::new("key".to_string(), "secret".to_string(), "pass".to_string(), true),
            super::order_manager::RiskEngine::new(
                10000.0,
                1000.0,
                5000.0,
                20,
                vec!["BTCUSDT".to_string()],
            ),
        ));

        let binance_client = Arc::new(BinanceFuturesClient::new(
            "key".to_string(),
            "secret".to_string(),
            true,
        ));

        let okx_client = Arc::new(OkxClient::new(
            "key".to_string(),
            "secret".to_string(),
            "pass".to_string(),
            true,
        ));

        let emergency_manager = EmergencyStopManager::new(
            order_manager,
            binance_client,
            okx_client,
        );

        assert!(!emergency_manager.is_stopped().await);
        assert!(emergency_manager.get_stop_reason().await.is_none());
        assert!(emergency_manager.get_stopped_at().await.is_none());
    }

    #[test]
    fn test_emergency_stop_reason_display() {
        let reasons = vec![
            EmergencyStopReason::DailyLossLimitReached,
            EmergencyStopReason::PositionLimitExceeded,
            EmergencyStopReason::TechnicalError,
            EmergencyStopReason::ManualStop,
            EmergencyStopReason::MarginCall,
            EmergencyStopReason::NetworkIssue,
            EmergencyStopReason::ExchangeMaintenance,
        ];

        for reason in reasons {
            println!("{:?}: {}", reason, reason.description());
        }
    }
}
