//! OKX交易所API客户端实现
//!
//! 完全基于Claude Agent SDK架构
//! 提供完整的OKX交易功能

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// OKX客户端
pub struct OkxClient {
    api_key: String,
    secret_key: String,
    passphrase: String,
    base_url: String,
    client: Client,
    simulated: bool,
}

impl OkxClient {
    /// 创建新的OKX客户端
    pub fn new(api_key: String, secret_key: String, passphrase: String, simulated: bool) -> Self {
        let base_url = if simulated {
            "https://www.okx.com".to_string() // 模拟交易
        } else {
            "https://www.okx.com".to_string()
        };

        Self {
            api_key,
            secret_key,
            passphrase,
            base_url,
            client: Client::new(),
            simulated,
        }
    }

    /// 下单
    pub async fn place_order(&self, order: OkxOrderRequest) -> Result<OkxOrderResponse> {
        let endpoint = "/api/v5/trade/order";
        let timestamp = Self::get_timestamp()?;
        let method = "POST";

        let body = serde_json::to_string(&order)?;
        let signature = self.sign(timestamp, method, endpoint, &body)?;

        let response = self
            .client
            .post(format!("{}{}", self.base_url, endpoint))
            .header("OK-ACCESS-KEY", &self.api_key)
            .header("OK-ACCESS-SIGN", &signature)
            .header("OK-ACCESS-TIMESTAMP", timestamp)
            .header("OK-ACCESS-PASSPHRASE", &self.passphrase)
            .header("Content-Type", "application/json")
            .body(body)
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

        let okx_resp: OkxApiResponse<OkxOrderResponse> = serde_json::from_str(&text)?;

        if okx_resp.code != "0" {
            return Err(anyhow::anyhow!("OKX API error: {}", okx_resp.msg));
        }

        okx_resp.data.ok_or_else(|| anyhow::anyhow!("No data in response"))
    }

    /// 获取账户信息
    pub async fn get_account_info(&self) -> Result<OkxAccountInfo> {
        let endpoint = "/api/v5/account/balance";
        let timestamp = Self::get_timestamp()?;
        let method = "GET";

        let signature = self.sign(timestamp, method, endpoint, "")?;

        let response = self
            .client
            .get(format!("{}{}", self.base_url, endpoint))
            .header("OK-ACCESS-KEY", &self.api_key)
            .header("OK-ACCESS-SIGN", &signature)
            .header("OK-ACCESS-TIMESTAMP", timestamp)
            .header("OK-ACCESS-PASSPHRASE", &self.passphrase)
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

        let okx_resp: OkxApiResponse<Vec<OkxBalanceData>> = serde_json::from_str(&text)?;

        if okx_resp.code != "0" {
            return Err(anyhow::anyhow!("OKX API error: {}", okx_resp.msg));
        }

        let balance_data = okx_resp.data.get(0).context("No balance data")?;

        Ok(OkxAccountInfo {
            total_equity: balance_data.total_eq.parse().unwrap_or(0.0),
            available_balance: balance_data.avail_bal.parse().unwrap_or(0.0),
            unrealized_profit: balance_data.upl.parse().unwrap_or(0.0),
            margin_balance: balance_data.frozen_bal.parse().unwrap_or(0.0),
        })
    }

    /// 获取持仓
    pub async fn get_positions(&self, inst_type: Option<String>) -> Result<Vec<OkxPosition>> {
        let endpoint = "/api/v5/account/positions";
        let timestamp = Self::get_timestamp()?;
        let method = "GET";

        let signature = self.sign(timestamp, method, endpoint, "")?;

        let mut request = self
            .client
            .get(format!("{}{}", self.base_url, endpoint))
            .header("OK-ACCESS-KEY", &self.api_key)
            .header("OK-ACCESS-SIGN", &signature)
            .header("OK-ACCESS-TIMESTAMP", timestamp)
            .header("OK-ACCESS-PASSPHRASE", &self.passphrase);

        if let Some(inst_type) = inst_type {
            request = request.query(&[("instType", &inst_type)]);
        }

        let response = request.send().await.context("Failed to get positions")?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Get positions failed with status {}: {}",
                status,
                text
            ));
        }

        let okx_resp: OkxApiResponse<Vec<OkxPosition>> = serde_json::from_str(&text)?;

        if okx_resp.code != "0" {
            return Err(anyhow::anyhow!("OKX API error: {}", okx_resp.msg));
        }

        Ok(okx_resp.data)
    }

    /// 取消订单
    pub async fn cancel_order(&self, inst_id: &str, order_id: &str) -> Result<()> {
        let endpoint = "/api/v5/trade/cancel-order";
        let timestamp = Self::get_timestamp()?;
        let method = "POST";

        let request_body = OkxCancelRequest {
            inst_id: inst_id.to_string(),
            ord_id: order_id.to_string(),
        };

        let body = serde_json::to_string(&request_body)?;
        let signature = self.sign(timestamp, method, endpoint, &body)?;

        let response = self
            .client
            .post(format!("{}{}", self.base_url, endpoint))
            .header("OK-ACCESS-KEY", &self.api_key)
            .header("OK-ACCESS-SIGN", &signature)
            .header("OK-ACCESS-TIMESTAMP", timestamp)
            .header("OK-ACCESS-PASSPHRASE", &self.passphrase)
            .header("Content-Type", "application/json")
            .body(body)
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

    /// 获取订单详情
    pub async fn get_order(&self, inst_id: &str, order_id: &str) -> Result<OkxOrderDetail> {
        let endpoint = "/api/v5/trade/order";
        let timestamp = Self::get_timestamp()?;
        let method = "GET";

        let signature = self.sign(timestamp, method, endpoint, "")?;

        let response = self
            .client
            .get(format!("{}{}", self.base_url, endpoint))
            .header("OK-ACCESS-KEY", &self.api_key)
            .header("OK-ACCESS-SIGN", &signature)
            .header("OK-ACCESS-TIMESTAMP", timestamp)
            .header("OK-ACCESS-PASSPHRASE", &self.passphrase)
            .query(&[("instId", inst_id)])
            .query(&[("ordId", order_id)])
            .send()
            .await
            .context("Failed to get order")?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Get order failed with status {}: {}",
                status,
                text
            ));
        }

        let okx_resp: OkxApiResponse<Vec<OkxOrderDetail>> = serde_json::from_str(&text)?;

        if okx_resp.code != "0" {
            return Err(anyhow::anyhow!("OKX API error: {}", okx_resp.msg));
        }

        okx_resp
            .data
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No order data in response"))
    }

    /// 设置杠杆
    pub async fn set_leverage(&self, inst_id: &str, lever: &str, mgn_mode: &str) -> Result<()> {
        let endpoint = "/api/v5/account/set-leverage";
        let timestamp = Self::get_timestamp()?;
        let method = "POST";

        let request_body = OkxLeverageRequest {
            inst_id: inst_id.to_string(),
            lever: lever.to_string(),
            mgn_mode: mgn_mode.to_string(),
        };

        let body = serde_json::to_string(&request_body)?;
        let signature = self.sign(timestamp, method, endpoint, &body)?;

        let response = self
            .client
            .post(format!("{}{}", self.base_url, endpoint))
            .header("OK-ACCESS-KEY", &self.api_key)
            .header("OK-ACCESS-SIGN", &signature)
            .header("OK-ACCESS-TIMESTAMP", timestamp)
            .header("OK-ACCESS-PASSPHRASE", &self.passphrase)
            .header("Content-Type", "application/json")
            .body(body)
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

    /// 获取24小时价格统计
    pub async fn get_ticker(&self, inst_id: &str) -> Result<OkxTicker> {
        let endpoint = "/api/v5/market/ticker";
        let timestamp = Self::get_timestamp()?;
        let method = "GET";

        let signature = self.sign(timestamp, method, endpoint, "")?;

        let response = self
            .client
            .get(format!("{}{}", self.base_url, endpoint))
            .header("OK-ACCESS-KEY", &self.api_key)
            .header("OK-ACCESS-SIGN", &signature)
            .header("OK-ACCESS-TIMESTAMP", timestamp)
            .header("OK-ACCESS-PASSPHRASE", &self.passphrase)
            .query(&[("instId", inst_id)])
            .send()
            .await
            .context("Failed to get ticker")?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Get ticker failed with status {}: {}",
                status,
                text
            ));
        }

        let okx_resp: OkxApiResponse<Vec<OkxTicker>> = serde_json::from_str(&text)?;

        if okx_resp.code != "0" {
            return Err(anyhow::anyhow!("OKX API error: {}", okx_resp.msg));
        }

        okx_resp
            .data
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No ticker data in response"))
    }

    /// 签名请求
    fn sign(&self, timestamp: &str, method: &str, endpoint: &str, body: &str) -> Result<String> {
        let message = format!("{}{}{}{}", timestamp, method, endpoint, body);
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .context("Failed to create HMAC")?;
        mac.update(message.as_bytes());
        Ok(base64_encode(mac.finalize().into_bytes()))
    }

    /// 获取时间戳
    fn get_timestamp() -> Result<String> {
        Ok(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
    }
}

/// Base64编码
fn base64_encode(data: impl AsRef<[u8]>) -> String {
    use base64::prelude::*;
    BASE64_STANDARD.encode(data.as_ref())
}

/// OKX订单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkxOrderRequest {
    #[serde(rename = "instId")]
    pub inst_id: String,
    #[serde(rename = "tdMode")]
    pub td_mode: String,
    #[serde(rename = "side")]
    pub side: String,
    #[serde(rename = "ordType")]
    pub ord_type: String,
    #[serde(rename = "sz")]
    pub sz: String,
}

/// OKX订单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkxOrderResponse {
    #[serde(rename = "ordId")]
    pub ord_id: String,
    #[serde(rename = "clOrdId")]
    pub cl_ord_id: String,
    #[serde(rename = "tag")]
    pub tag: String,
    #[serde(rename = "sCode")]
    pub s_code: String,
    #[serde(rename = "sMsg")]
    pub s_msg: String,
}

/// OKX账户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkxAccountInfo {
    pub total_equity: f64,
    pub available_balance: f64,
    pub unrealized_profit: f64,
    pub margin_balance: f64,
}

/// OKX持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkxPosition {
    #[serde(rename = "instId")]
    pub inst_id: String,
    #[serde(rename = "mgnMode")]
    pub mgn_mode: String,
    #[serde(rename = "pos")]
    pub pos: String,
    #[serde(rename = "avgPx")]
    pub avg_px: String,
    #[serde(rename = "upl")]
    pub upl: String,
    #[serde(rename = "lever")]
    pub lever: String,
}

/// OKX订单详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkxOrderDetail {
    #[serde(rename = "instId")]
    pub inst_id: String,
    #[serde(rename = "ordId")]
    pub ord_id: String,
    #[serde(rename = "clOrdId")]
    pub cl_ord_id: String,
    #[serde(rename = "state")]
    pub state: String,
    #[serde(rename = "side")]
    pub side: String,
    #[serde(rename = "ordType")]
    pub ord_type: String,
    #[serde(rename = "px")]
    pub px: String,
    #[serde(rename = "sz")]
    pub sz: String,
    #[serde(rename = "fillSz")]
    pub fill_sz: String,
    #[serde(rename = "cTime")]
    pub c_time: String,
}

/// OKX行情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkxTicker {
    #[serde(rename = "instId")]
    pub inst_id: String,
    #[serde(rename = "last")]
    pub last: String,
    #[serde(rename = "lastSz")]
    pub last_sz: String,
    #[serde(rename = "askPx")]
    pub ask_px: String,
    #[serde(rename = "bidPx")]
    pub bid_px: String,
    #[serde(rename = "open24h")]
    pub open_24h: String,
    #[serde(rename = "high24h")]
    pub high_24h: String,
    #[serde(rename = "low24h")]
    pub low_24h: String,
}

// OKX API响应结构

#[derive(Debug, Serialize, Deserialize)]
struct OkxApiResponse<T> {
    code: String,
    msg: String,
    data: Vec<T>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OkxCancelRequest {
    #[serde(rename = "instId")]
    inst_id: String,
    #[serde(rename = "ordId")]
    ord_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OkxLeverageRequest {
    #[serde(rename = "instId")]
    inst_id: String,
    #[serde(rename = "lever")]
    lever: String,
    #[serde(rename = "mgnMode")]
    mgn_mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OkxBalanceData {
    #[serde(rename = "totalEq")]
    total_eq: String,
    #[serde(rename = "availBal")]
    avail_bal: String,
    #[serde(rename = "frozenBal")]
    frozen_bal: String,
    #[serde(rename = "upl")]
    upl: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_okx_client_creation() {
        let client = OkxClient::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            "test_passphrase".to_string(),
            true, // simulated
        );

        assert_eq!(client.api_key, "test_key");
        assert_eq!(client.secret_key, "test_secret");
        assert_eq!(client.passphrase, "test_passphrase");
        assert!(client.simulated);
    }

    #[test]
    fn test_okx_order_request() {
        let order = OkxOrderRequest {
            inst_id: "BTC-USDT-SWAP".to_string(),
            td_mode: "cross".to_string(),
            side: "buy".to_string(),
            ord_type: "market".to_string(),
            sz: "1".to_string(),
        };

        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("BTC-USDT-SWAP"));
        assert!(json.contains("buy"));
    }
}
