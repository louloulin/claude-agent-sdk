//!
//! MCP Gateway - 统一连接所有数据源和交易接口
//!
//! 基于Model Context Protocol (MCP) 开放标准
//! 参考: https://modelcontextprotocol.io/
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                   MCPGateway                        │
//! │  (统一网关 - 单一入口点)                             │
//! └─────────────────────────────────────────────────────┘
//!                         │
//!         ┌────────────────┼────────────────┐
//!         │                │                │
//!    ┌────▼────┐     ┌────▼────┐     ┌────▼────┐
//!    │  Data   │     │ Trading │     │  Tools  │
//!    │Sources  │     │   APIs  │     │   MCPs  │
//!    └─────────┘     └─────────┘     └─────────┘
//!         │                │                │
//!    ┌────▼─────────┐ ┌──▼──────────┐ ┌──▼──────────┐
//!    │ Yahoo Finance│ │   QMT       │ │  Research   │
//!    │ Tushare      │ │   IBKR      │ │  Analysis   │
//!    │ Binance      │ │  Binance    │ │  Reporting  │
//!    │ SEC-EDGAR    │ │             │ │             │
//!    └──────────────┘ └─────────────┘ └─────────────┘
//! ```

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// MCP Gateway Core
// ============================================================================

/// MCP Gateway - 统一网关
///
/// 负责管理所有MCP服务器连接，提供统一的数据查询和交易执行接口
pub struct MCPGateway {
    /// MCP客户端连接池
    connections: Arc<RwLock<HashMap<String, Box<dyn MCPClient>>>>,

    /// 数据源MCP服务器
    data_sources: Arc<RwLock<HashMap<String, String>>>,

    /// 交易API MCP服务器
    trading_apis: Arc<RwLock<HashMap<String, String>>>,

    /// 工具MCP服务器
    tools: Arc<RwLock<HashMap<String, String>>>,
}

impl MCPGateway {
    /// 创建新的MCP网关
    pub async fn new() -> Result<Self> {
        let gateway = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            data_sources: Arc::new(RwLock::new(HashMap::new())),
            trading_apis: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
        };

        // 初始化时连接配置的MCP服务器
        // 在实际实现中，这里会读取配置文件
        tracing::info!("MCP Gateway initialized");

        Ok(gateway)
    }

    /// 连接MCP服务器
    pub async fn connect_mcp_server(
        &self,
        name: String,
        url: String,
        config: MCPConfig,
    ) -> Result<()> {
        tracing::info!("Connecting to MCP server: {} at {}", name, url);

        // 创建MCP客户端
        let client = create_mcp_client(&url, config).await?;

        // 保存连接
        let mut connections = self.connections.write().await;
        connections.insert(name.clone(), client);

        // 根据类型分类存储
        if url.contains("data") || url.contains("finance") || url.contains("tushare") {
            let mut data_sources = self.data_sources.write().await;
            data_sources.insert(name, url);
        } else if url.contains("trading") || url.contains("broker") {
            let mut trading_apis = self.trading_apis.write().await;
            trading_apis.insert(name, url);
        } else {
            let mut tools = self.tools.write().await;
            tools.insert(name, url);
        }

        Ok(())
    }

    /// 查询数据 (统一接口)
    ///
    /// # Arguments
    /// * `query` - 数据查询请求
    ///
    /// # Returns
    /// 查询结果
    pub async fn query_data(&self, query: DataQuery) -> Result<Data> {
        tracing::debug!("Querying data: {:?}", query);

        // 获取对应域的MCP客户端
        let connections = self.connections.read().await;
        let mcp_client = connections.get(&query.domain)
            .ok_or_else(|| anyhow::anyhow!("MCP client not found for domain: {}", query.domain))?;

        // 构建查询参数
        let params = serde_json::json!({
            "symbol": query.symbol,
            "start_date": query.start_date,
            "end_date": query.end_date,
            "fields": query.fields,
        });

        // 调用MCP工具
        let response = mcp_client.call_tool("get_data", params).await?;

        Ok(serde_json::from_value(response)?)
    }

    /// 执行交易 (统一接口)
    ///
    /// # Arguments
    /// * `order` - 订单请求
    ///
    /// # Returns
    /// 订单响应
    pub async fn execute_trade(&self, order: OrderRequest) -> Result<OrderResponse> {
        tracing::info!("Executing trade: {:?}", order);

        // 获取对应市场的券商MCP名称
        let broker_name = self.get_broker_for_market(&order.market).await?;

        // 构建订单参数
        let params = serde_json::to_value(order)?;

        // 获取client并调用
        let connections = self.connections.read().await;
        let mcp_client = connections.get(&broker_name)
            .ok_or_else(|| anyhow::anyhow!("MCP client not found: {}", broker_name))?;

        // 调用MCP工具
        let response = mcp_client.call_tool("place_order", params).await?;

        Ok(serde_json::from_value(response)?)
    }

    /// 获取市场对应的券商MCP
    async fn get_broker_for_market(&self, market: &str) -> Result<String> {
        // 简单的路由逻辑，返回broker名称而不是client
        let broker_name = match market {
            "CN" => "qmt-broker-mcp",
            "US" => "interactive-brokers-mcp",
            "CRYPTO" => "binance-trading-mcp",
            _ => return Err(anyhow::anyhow!("Unsupported market: {}", market)),
        };

        // 检查broker是否存在
        let connections = self.connections.read().await;
        if !connections.contains_key(broker_name) {
            return Err(anyhow::anyhow!("Broker MCP not found: {}", broker_name));
        }

        Ok(broker_name.to_string())
    }

    /// 获取网关状态
    pub async fn status(&self) -> GatewayStatus {
        let connections = self.connections.read().await;
        let data_sources = self.data_sources.read().await;
        let trading_apis = self.trading_apis.read().await;
        let tools = self.tools.read().await;

        GatewayStatus {
            total_connections: connections.len(),
            data_sources: data_sources.clone(),
            trading_apis: trading_apis.clone(),
            tools: tools.clone(),
            is_healthy: true,
        }
    }
}

// ============================================================================
// MCP Client Trait
// ============================================================================

/// MCP客户端trait
#[async_trait]
pub trait MCPClient: Send + Sync {
    /// 调用MCP工具
    async fn call_tool(&self, tool: &str, params: serde_json::Value) -> Result<serde_json::Value>;

    /// 获取服务器信息
    async fn server_info(&self) -> Result<ServerInfo>;

    /// 列出可用工具
    async fn list_tools(&self) -> Result<Vec<String>>;

    /// 健康检查
    async fn health_check(&self) -> Result<bool>;
}

// ============================================================================
// Data Types
// ============================================================================

/// MCP配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    /// API密钥
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// 密钥
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,

    /// 超时时间(秒)
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// 重试次数
    #[serde(default = "default_retries")]
    pub max_retries: usize,
}

fn default_timeout() -> u64 { 30 }
fn default_retries() -> usize { 3 }

impl Default for MCPConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            secret: None,
            timeout: 30,
            max_retries: 3,
        }
    }
}

/// 数据查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQuery {
    /// 数据域 (yahoo-finance, tushare, binance等)
    pub domain: String,

    /// 代码符号
    pub symbol: String,

    /// 开始日期
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,

    /// 结束日期
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,

    /// 查询字段
    #[serde(default)]
    pub fields: Vec<String>,
}

/// 数据响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    /// 数据内容
    pub content: serde_json::Value,

    /// 元数据
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// 时间戳
    pub timestamp: i64,
}

/// 订单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    /// 市场 (CN, US, CRYPTO)
    pub market: String,

    /// 代码符号
    pub symbol: String,

    /// 操作 (buy, sell)
    pub action: String,

    /// 数量
    pub quantity: f64,

    /// 价格类型 (market, limit)
    #[serde(default = "default_price_type")]
    pub price_type: String,

    /// 限价价格
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<f64>,
}

fn default_price_type() -> String { "market".to_string() }

/// 订单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    /// 订单ID
    pub order_id: String,

    /// 状态
    pub status: String,

    /// 成交价格
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filled_price: Option<f64>,

    /// 成交数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filled_quantity: Option<f64>,

    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// 时间戳
    pub timestamp: i64,
}

/// 服务器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// 服务器名称
    pub name: String,

    /// 版本
    pub version: String,

    /// 支持的工具列表
    pub tools: Vec<String>,

    /// 描述
    pub description: String,
}

/// 网关状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStatus {
    /// 总连接数
    pub total_connections: usize,

    /// 数据源列表
    pub data_sources: HashMap<String, String>,

    /// 交易API列表
    pub trading_apis: HashMap<String, String>,

    /// 工具列表
    pub tools: HashMap<String, String>,

    /// 是否健康
    pub is_healthy: bool,
}

// ============================================================================
// Mock MCP Client (用于测试)
// ============================================================================

/// Mock MCP客户端 - 用于开发和测试
pub struct MockMCPClient {
    server_name: String,
}

impl MockMCPClient {
    pub fn new(name: String) -> Self {
        Self { server_name: name }
    }
}

#[async_trait]
impl MCPClient for MockMCPClient {
    async fn call_tool(&self, tool: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        tracing::debug!("MockMCPClient[{}]: calling tool {} with params {:?}",
                       self.server_name, tool, params);

        // 根据不同的工具返回符合预期格式的模拟数据
        match tool {
            "get_data" => {
                // 返回Data结构体所需的格式
                Ok(serde_json::json!({
                    "content": {
                        "mock": true,
                        "tool": tool,
                        "params": params,
                    },
                    "metadata": {
                        "source": self.server_name.clone(),
                    },
                    "timestamp": chrono::Utc::now().timestamp(),
                }))
            }
            "place_order" => {
                // 返回OrderResponse结构体所需的格式
                let quantity = params.get("quantity")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                Ok(serde_json::json!({
                    "order_id": format!("mock-order-{}", uuid::Uuid::new_v4()),
                    "status": "filled",
                    "filled_price": 100.0,
                    "filled_quantity": quantity,
                    "timestamp": chrono::Utc::now().timestamp(),
                }))
            }
            _ => {
                Ok(serde_json::json!({
                    "status": "success",
                    "data": {
                        "mock": true,
                        "tool": tool,
                        "params": params,
                    }
                }))
            }
        }
    }

    async fn server_info(&self) -> Result<ServerInfo> {
        Ok(ServerInfo {
            name: self.server_name.clone(),
            version: "0.1.0-mock".to_string(),
            tools: vec!["get_data".to_string(), "place_order".to_string()],
            description: "Mock MCP client for testing".to_string(),
        })
    }

    async fn list_tools(&self) -> Result<Vec<String>> {
        Ok(vec!["get_data".to_string(), "place_order".to_string()])
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// 创建MCP客户端 (工厂函数)
async fn create_mcp_client(url: &str, config: MCPConfig) -> Result<Box<dyn MCPClient>> {
    // 在实际实现中，这里会:
    // 1. 连接到真实的MCP服务器
    // 2. 协商能力
    // 3. 返回具体的客户端实现

    // 当前返回Mock客户端用于开发
    tracing::warn!("Using MockMCPClient for: {} (生产环境需要实现真实MCP客户端)", url);
    
    Ok(Box::new(MockMCPClient::new(url.to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_gateway_creation() {
        let gateway = MCPGateway::new().await.unwrap();
        let status = gateway.status().await;

        assert_eq!(status.total_connections, 0);
        assert!(status.is_healthy);
    }

    #[tokio::test]
    async fn test_connect_mcp_server() {
        let gateway = MCPGateway::new().await.unwrap();

        gateway.connect_mcp_server(
            "test-mcp".to_string(),
            "http://localhost:3000".to_string(),
            MCPConfig::default(),
        ).await.unwrap();

        let status = gateway.status().await;
        assert_eq!(status.total_connections, 1);
    }

    #[tokio::test]
    async fn test_query_data() {
        let gateway = MCPGateway::new().await.unwrap();

        // 先连接一个数据源
        gateway.connect_mcp_server(
            "yahoo-finance".to_string(),
            "http://localhost:3000".to_string(),
            MCPConfig::default(),
        ).await.unwrap();

        let query = DataQuery {
            domain: "yahoo-finance".to_string(),
            symbol: "AAPL".to_string(),
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-12-31".to_string()),
            fields: vec!["price".to_string(), "volume".to_string()],
        };

        let result = gateway.query_data(query).await;
        if let Err(e) = &result {
            eprintln!("query_data error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_trade() {
        let gateway = MCPGateway::new().await.unwrap();

        // 连接券商MCP
        gateway.connect_mcp_server(
            "qmt-broker-mcp".to_string(),
            "http://localhost:3001".to_string(),
            MCPConfig::default(),
        ).await.unwrap();

        let order = OrderRequest {
            market: "CN".to_string(),
            symbol: "600000".to_string(),
            action: "buy".to_string(),
            quantity: 100.0,
            price_type: "market".to_string(),
            limit_price: None,
        };

        let result = gateway.execute_trade(order).await;
        if let Err(e) = &result {
            eprintln!("execute_trade error: {:?}", e);
        }
        assert!(result.is_ok());
    }
}
