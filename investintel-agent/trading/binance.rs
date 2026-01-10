//! Binance Futures API客户端实现
//!
//! 完全基于Claude Agent SDK架构
//! 提供完整的Binance期货交易功能

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac, NewHmac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// Binance期货客户端
pub struct BinanceFuturesClient {
    api_key: String,
    secret_key: String,
    base_url: String,
    client: Client,
    testnet: bool,
}

impl BinanceFuturesClient {
    /// 创建新的Binance期货客户端
    pub fn new(api_key: String, secret_key: String, testnet: bool) -> Self {
        let base_url = if testnet {
            "https://testnet.binancefuture.com".to_string()
        } else {
            "https://fapi.binance.com".to_string()
        };

        Self {
            api_key,
            secret_key,
            base_url,
            client: Client::new(),
            testnet,
        }
    }

    /// 下单
    pub async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse> {
        let endpoint = "/fapi/v1/order";
        let timestamp = Self::get_timestamp()?;

        let mut params = vec![
            ("symbol".to_string(), order.symbol.clone()),
            ("side".to_string(), order.side.to_string()),
            ("type".to_string(), "MARKET".to_string()),
            ("quantity".to_string(), order.quantity.to_string()),
            ("timestamp".to_string(), timestamp.to_string()),
        ];

        // 可选参数
        if let Some(price) = order.price {
            params.push(("price".to_string(), price.to_string()));
            params[2].1 = "LIMIT".to_string(); // 改为限价单
            params.push(("timeInForce".to_string(), "GTC".to_string()));
        }

        if let Some(position_side) = order.position_side {
            params.push(("positionSide".to_string(), position_side.to_string()));
        }

        if let Some(stop_price) = order.stop_price {
            params.push(("stopPrice".to_string(), stop_price.to_string()));
            params[2].1 = "STOP".to_string();
        }

        let query_string = Self::build_query(&params);
        let signature = self.sign(&query_string)?;

        let response = self
            .client
            .post(format!("{}{}", self.base_url, endpoint))
            .header("X-MBX-APIKEY", &self.api_key)
            .query(&[("signature", &signature)])
            .form(&params)
            .send()
            .await
            .context("Failed to place order")?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Order failed with status {}: {}",
                status,
                text
            ));
        }

        let order_resp: BinanceOrderResponse = serde_json::from_str(&text)?;
        Ok(OrderResponse {
            order_id: order_resp.order_id.to_string(),
            client_order_id: order_resp.client_order_id,
            symbol: order_resp.symbol,
            status: order_resp.status.into(),
            side: order_resp.side.into(),
            price: order_resp.price,
            quantity: order_resp.orig_qty,
            executed_qty: order_resp.executed_qty,
            created_at: DateTime::from_timestamp_millis(order_resp.time)
                .unwrap_or_else(|| Utc::now()),
        })
    }

    /// 获取账户信息
    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        let endpoint = "/fapi/v2/account";
        let timestamp = Self::get_timestamp()?;
        let query = format!("timestamp={}", timestamp);
        let signature = self.sign(&query)?;

        let response = self
            .client
            .get(format!("{}{}", self.base_url, endpoint))
            .header("X-MBX-APIKEY", &self.api_key)
            .query(&[("timestamp", ×tamp)])
            .query(&[("signature", &signature)])
            .send()
            .await
            .context("Failed to get account info")?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Get account info failed with status {}: {}",
                status,
                text
            ));
        }

        let account: BinanceAccountResponse = serde_json::from_str(&text)?;
        Ok(AccountInfo {
            total_wallet_balance: account.total_wallet_balance,
            available_balance: account.available_balance,
            unrealized_profit: account.total_unrealized_profit,
            margin_balance: account.total_margin_balance,
            positions: account
                .positions
                .into_iter()
                .map(|p| PositionInfo {
                    symbol: p.symbol,
                    position_amount: p.position_amt,
                    entry_price: p.entry_price,
                    mark_price: p.mark_price,
                    unrealized_profit: p.unrealized_profit,
                    leverage: p.leverage,
                })
                .collect(),
        })
    }

    /// 获取当前持仓
    pub async fn get_positions(&self, symbol: Option<String>) -> Result<Vec<PositionInfo>> {
        let account = self.get_account_info().await?;

        let positions = if let Some(sym) = symbol {
            account
                .positions
                .into_iter()
                .filter(|p| p.symbol == sym)
                .collect()
        } else {
            account.positions
        };

        Ok(positions)
    }

    /// 获取订单状态
    pub async fn get_order_status(&self, symbol: &str, order_id: &str) -> Result<OrderStatus> {
        let endpoint = "/fapi/v1/order";
        let timestamp = Self::get_timestamp()?;
        let query = format!(
            "symbol={}&orderId={}&timestamp={}",
            symbol, order_id, timestamp
        );
        let signature = self.sign(&query)?;

        let response = self
            .client
            .get(format!("{}{}", self.base_url, endpoint))
            .header("X-MBX-APIKEY", &self.api_key)
            .query(&[("symbol", symbol)])
            .query(&[("orderId", order_id)])
            .query(&[("timestamp", ×tamp)])
            .query(&[("signature", &signature)])
            .send()
            .await
            .context("Failed to get order status")?;

        let status_code = response.status();
        let text = response.text().await?;

        if !status_code.is_success() {
            return Err(anyhow::anyhow!(
                "Get order status failed with status {}: {}",
                status_code,
                text
            ));
        }

        let order: BinanceOrderResponse = serde_json::from_str(&text)?;
        Ok(OrderStatus {
            order_id: order.order_id.to_string(),
            symbol: order.symbol,
            status: order.status.into(),
            side: order.side.into(),
            price: order.price,
            quantity: order.orig_qty,
            executed_qty: order.executed_qty,
            created_at: DateTime::from_timestamp_millis(order.time)
                .unwrap_or_else(|| Utc::now()),
        })
    }

    /// 取消订单
    pub async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<()> {
        let endpoint = "/fapi/v1/order";
        let timestamp = Self::get_timestamp()?;
        let query = format!(
            "symbol={}&orderId={}&timestamp={}",
            symbol, order_id, timestamp
        );
        let signature = self.sign(&query)?;

        let response = self
            .client
            .delete(format!("{}{}", self.base_url, endpoint))
            .header("X-MBX-APIKEY", &self.api_key)
            .query(&[("symbol", symbol)])
            .query(&[("orderId", order_id)])
            .query(&[("timestamp", ×tamp)])
            .query(&[("signature", &signature)])
            .send()
            .await
            .context("Failed to cancel order")?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Cancel order failed with status {}: {}",
                status,
                text
            ));
        }

        Ok(())
    }

    /// 取消所有订单
    pub async fn cancel_all_orders(&self, symbol: &str) -> Result<Vec<String>> {
        let endpoint = "/fapi/v1/allOpenOrders";
        let timestamp = Self::get_timestamp()?;
        let query = format!("symbol={}&timestamp={}", symbol, timestamp);
        let signature = self.sign(&query)?;

        let response = self
            .client
            .delete(format!("{}{}", self.base_url, endpoint))
            .header("X-MBX-APIKEY", &self.api_key)
            .query(&[("symbol", symbol)])
            .query(&[("timestamp", ×tamp)])
            .query(&[("signature", &signature)])
            .send()
            .await
            .context("Failed to cancel all orders")?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Cancel all orders failed with status {}: {}",
                status,
                text
            ));
        }

        // 返回取消的订单ID列表
        let canceled: Vec<BinanceOrderResponse> = serde_json::from_str(&text)?;
        Ok(canceled.iter().map(|o| o.order_id.to_string()).collect())
    }

    /// 修改订单
    pub async fn modify_order(
        &self,
        symbol: &str,
        order_id: &str,
        new_quantity: f64,
        new_price: Option<f64>,
    ) -> Result<OrderResponse> {
        let endpoint = "/fapi/v1/order";
        let timestamp = Self::get_timestamp()?;

        let mut params = vec![
            ("symbol".to_string(), symbol.to_string()),
            ("orderId".to_string(), order_id.to_string()),
            ("quantity".to_string(), new_quantity.to_string()),
            ("timestamp".to_string(), timestamp.to_string()),
        ];

        if let Some(price) = new_price {
            params.push(("price".to_string(), price.to_string()));
        }

        let query_string = Self::build_query(&params);
        let signature = self.sign(&query_string)?;

        let response = self
            .client
            .put(format!("{}{}", self.base_url, endpoint))
            .header("X-MBX-APIKEY", &self.api_key)
            .query(&[("signature", &signature)])
            .form(&params)
            .send()
            .await
            .context("Failed to modify order")?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Modify order failed with status {}: {}",
                status,
                text
            ));
        }

        let order_resp: BinanceOrderResponse = serde_json::from_str(&text)?;
        Ok(OrderResponse {
            order_id: order_resp.order_id.to_string(),
            client_order_id: order_resp.client_order_id,
            symbol: order_resp.symbol,
            status: order_resp.status.into(),
            side: order_resp.side.into(),
            price: order_resp.price,
            quantity: order_resp.orig_qty,
            executed_qty: order_resp.executed_qty,
            created_at: DateTime::from_timestamp_millis(order_resp.time)
                .unwrap_or_else(|| Utc::now()),
        })
    }

    /// 获取24小时价格变动统计
    pub async fn get_24h_ticker(&self, symbol: &str) -> Result<TickerStatistics> {
        let endpoint = "/fapi/v1/ticker/24hr";

        let response = self
            .client
            .get(format!("{}{}", self.base_url, endpoint))
            .query(&[("symbol", symbol)])
            .send()
            .await
            .context("Failed to get 24h ticker")?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Get 24h ticker failed with status {}: {}",
                status,
                text
            ));
        }

        let ticker: BinanceTickerResponse = serde_json::from_str(&text)?;
        Ok(TickerStatistics {
            symbol: ticker.symbol,
            price_change: ticker.price_change,
            price_change_percent: ticker.price_change_percent,
            weighted_avg_price: ticker.weighted_avg_price,
            open_price: ticker.open_price,
            high_price: ticker.high_price,
            low_price: ticker.low_price,
            last_price: ticker.last_price,
            volume: ticker.volume,
        })
    }

    /// 设置杠杆
    pub async fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<()> {
        let endpoint = "/fapi/v1/leverage";
        let timestamp = Self::get_timestamp()?;
        let query = format!(
            "symbol={}&leverage={}&timestamp={}",
            symbol, leverage, timestamp
        );
        let signature = self.sign(&query)?;

        let response = self
            .client
            .post(format!("{}{}", self.base_url, endpoint))
            .header("X-MBX-APIKEY", &self.api_key)
            .query(&[("symbol", symbol)])
            .query(&[("leverage", &leverage.to_string())])
            .query(&[("timestamp", ×tamp)])
            .query(&[("signature", &signature)])
            .send()
            .await
            .context("Failed to set leverage")?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Set leverage failed with status {}: {}",
                status,
                text
            ));
        }

        Ok(())
    }

    /// 签名请求
    fn sign(&self, query: &str) -> Result<String> {
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .context("Failed to create HMAC")?;
        mac.update(query.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    /// 构建查询字符串
    fn build_query(params: &[(String, String)]) -> String {
        params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// 获取时间戳
    fn get_timestamp() -> Result<u64> {
        Ok(Utc::now().timestamp_millis() as u64)
    }
}

/// 订单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub position_side: Option<PositionSide>,
}

/// 订单方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    pub fn to_string(&self) -> String {
        match self {
            OrderSide::Buy => "BUY".to_string(),
            OrderSide::Sell => "SELL".to_string(),
        }
    }
}

impl From<String> for OrderSide {
    fn from(s: String) -> Self {
        match s.to_uppercase().as_str() {
            "BUY" => OrderSide::Buy,
            "SELL" => OrderSide::Sell,
            _ => panic!("Invalid order side: {}", s),
        }
    }
}

/// 持仓方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionSide {
    Long,
    Short,
}

impl PositionSide {
    pub fn to_string(&self) -> String {
        match self {
            PositionSide::Long => "LONG".to_string(),
            PositionSide::Short => "SHORT".to_string(),
        }
    }
}

impl From<String> for PositionSide {
    fn from(s: String) -> Self {
        match s.to_uppercase().as_str() {
            "LONG" => PositionSide::Long,
            "SHORT" => PositionSide::Short,
            _ => panic!("Invalid position side: {}", s),
        }
    }
}

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinanceOrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    PendingCancel,
    Rejected,
    Expired,
}

impl From<String> for BinanceOrderStatus {
    fn from(s: String) -> Self {
        match s.to_uppercase().as_str() {
            "NEW" => BinanceOrderStatus::New,
            "PARTIALLY_FILLED" => BinanceOrderStatus::PartiallyFilled,
            "FILLED" => BinanceOrderStatus::Filled,
            "CANCELED" => BinanceOrderStatus::Canceled,
            "PENDING_CANCEL" => BinanceOrderStatus::PendingCancel,
            "REJECTED" => BinanceOrderStatus::Rejected,
            "EXPIRED" => BinanceOrderStatus::Expired,
            _ => panic!("Invalid order status: {}", s),
        }
    }
}

impl From<BinanceOrderStatus> for String {
    fn from(status: BinanceOrderStatus) -> String {
        match status {
            BinanceOrderStatus::New => "NEW".to_string(),
            BinanceOrderStatus::PartiallyFilled => "PARTIALLY_FILLED".to_string(),
            BinanceOrderStatus::Filled => "FILLED".to_string(),
            BinanceOrderStatus::Canceled => "CANCELED".to_string(),
            BinanceOrderStatus::PendingCancel => "PENDING_CANCEL".to_string(),
            BinanceOrderStatus::Rejected => "REJECTED".to_string(),
            BinanceOrderStatus::Expired => "EXPIRED".to_string(),
        }
    }
}

/// 订单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub client_order_id: String,
    pub symbol: String,
    pub status: BinanceOrderStatus,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
    pub executed_qty: f64,
    pub created_at: DateTime<Utc>,
}

/// 账户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub total_wallet_balance: f64,
    pub available_balance: f64,
    pub unrealized_profit: f64,
    pub margin_balance: f64,
    pub positions: Vec<PositionInfo>,
}

/// 持仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionInfo {
    pub symbol: String,
    pub position_amount: f64,
    pub entry_price: f64,
    pub mark_price: f64,
    pub unrealized_profit: f64,
    pub leverage: f64,
}

/// 订单状态查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatus {
    pub order_id: String,
    pub symbol: String,
    pub status: BinanceOrderStatus,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
    pub executed_qty: f64,
    pub created_at: DateTime<Utc>,
}

/// 24小时价格统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerStatistics {
    pub symbol: String,
    pub price_change: f64,
    pub price_change_percent: f64,
    pub weighted_avg_price: f64,
    pub open_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub last_price: f64,
    pub volume: f64,
}

// Binance API响应结构

#[derive(Debug, Serialize, Deserialize)]
struct BinanceOrderResponse {
    #[serde(rename = "orderId")]
    order_id: u64,
    #[serde(rename = "clientOrderId")]
    client_order_id: String,
    symbol: String,
    status: String,
    side: String,
    price: f64,
    #[serde(rename = "origQty")]
    orig_qty: f64,
    #[serde(rename = "executedQty")]
    executed_qty: f64,
    time: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BinanceAccountResponse {
    #[serde(rename = "totalWalletBalance")]
    total_wallet_balance: f64,
    #[serde(rename = "availableBalance")]
    available_balance: f64,
    #[serde(rename = "totalUnrealizedProfit")]
    total_unrealized_profit: f64,
    #[serde(rename = "totalMarginBalance")]
    total_margin_balance: f64,
    positions: Vec<BinancePosition>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BinancePosition {
    symbol: String,
    #[serde(rename = "positionAmt")]
    position_amt: f64,
    #[serde(rename = "entryPrice")]
    entry_price: f64,
    #[serde(rename = "markPrice")]
    mark_price: f64,
    #[serde(rename = "unrealizedProfit")]
    unrealized_profit: f64,
    leverage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BinanceTickerResponse {
    symbol: String,
    #[serde(rename = "priceChange")]
    price_change: f64,
    #[serde(rename = "priceChangePercent")]
    price_change_percent: f64,
    #[serde(rename = "weightedAvgPrice")]
    weighted_avg_price: f64,
    #[serde(rename = "openPrice")]
    open_price: f64,
    #[serde(rename = "highPrice")]
    high_price: f64,
    #[serde(rename = "lowPrice")]
    low_price: f64,
    #[serde(rename = "lastPrice")]
    last_price: f64,
    volume: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_side_conversion() {
        assert_eq!(OrderSide::Buy.to_string(), "BUY");
        assert_eq!(OrderSide::Sell.to_string(), "SELL");
    }

    #[test]
    fn test_position_side_conversion() {
        assert_eq!(PositionSide::Long.to_string(), "LONG");
        assert_eq!(PositionSide::Short.to_string(), "SHORT");
    }

    #[test]
    fn test_binance_client_creation() {
        let client = BinanceFuturesClient::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            true, // testnet
        );

        assert_eq!(client.api_key, "test_key");
        assert_eq!(client.secret_key, "test_secret");
        assert!(client.testnet);
        assert_eq!(client.base_url, "https://testnet.binancefuture.com");
    }

    #[test]
    fn test_build_query() {
        let params = vec![
            ("symbol".to_string(), "BTCUSDT".to_string()),
            ("side".to_string(), "BUY".to_string()),
            ("quantity".to_string(), "0.001".to_string()),
        ];

        let query = BinanceFuturesClient::build_query(&params);
        assert_eq!(query, "symbol=BTCUSDT&side=BUY&quantity=0.001");
    }
}
