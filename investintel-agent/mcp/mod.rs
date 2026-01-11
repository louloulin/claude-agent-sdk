//! MCP Gateway - 统一数据源和交易接口连接
//!
//! 基于Model Context Protocol (MCP)开放标准
//! 为AI投资助手提供统一的数据访问和交易执行接口
//!
//! ## 核心功能
//!
//! 1. **统一数据源** - 通过MCP连接多个数据提供者
//! 2. **统一交易接口** - 通过MCP连接多个券商API
//! 3. **热插拔** - 动态添加/移除MCP服务器
//! 4. **智能路由** - 自动选择最佳数据源
//!
//! ## 架构设计
//!
//! ```text
//! InvestmentAssistant
//!     ↓
//! MCPGateway (统一网关)
//!     ├─ Data Sources (数据源MCP)
//!     │   ├─ Yahoo Finance MCP
//!     │   ├─ Alpha Vantage MCP
//!     │   ├─ Tushare MCP (A股)
//!     │   └─ Binance MCP (加密货币)
//!     ├─ Trading APIs (交易API MCP)
//!     │   ├─ QMT Broker MCP (A股)
//!     │   ├─ Interactive Brokers MCP (美股)
//!     │   └─ Binance Trading MCP (加密货币)
//!     └─ Tools (工具MCP)
//!         ├─ News API MCP
//!         ├─ SEC Filings MCP
//!         └─ Analysis Tools MCP
//! ```

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod client;
pub mod config;

use client::MCPClient;
use config::MCPConfig;

/// MCP Gateway - 统一网关
///
/// 负责管理所有MCP连接，提供统一的数据查询和交易执行接口
pub struct MCPGateway {
    /// MCP客户端连接池
    connections: Arc<RwLock<HashMap<String, Arc<MCPClient>>>>,

    /// 数据源MCP服务器
    data_sources: Arc<RwLock<HashMap<String, String>>>,

    /// 交易API MCP服务器
    trading_apis: Arc<RwLock<HashMap<String, String>>>,

    /// 工具MCP服务器
    tools: Arc<RwLock<HashMap<String, String>>>,

    /// 网关配置
    config: GatewayConfig,
}

/// 网关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// 启用的数据源
    pub enabled_data_sources: Vec<String>,

    /// 启用的交易API
    pub enabled_trading_apis: Vec<String>,

    /// 启用的工具
    pub enabled_tools: Vec<String>,

    /// 连接超时（秒）
    pub connection_timeout: u64,

    /// 查询超时（秒）
    pub query_timeout: u64,

    /// 是否启用缓存
    pub enable_cache: bool,

    /// 缓存TTL（秒）
    pub cache_ttl: u64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            enabled_data_sources: vec![
                "yahoo-finance-mcp".to_string(),
                "alpha-vantage-mcp".to_string(),
            ],
            enabled_trading_apis: vec![],
            enabled_tools: vec![
                "news-api-mcp".to_string(),
            ],
            connection_timeout: 30,
            query_timeout: 60,
            enable_cache: true,
            cache_ttl: 60,
        }
    }
}

impl MCPGateway {
    /// 创建新的MCP网关
    pub async fn new(config: GatewayConfig) -> Result<Self> {
        let gateway = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            data_sources: Arc::new(RwLock::new(HashMap::new())),
            trading_apis: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
            config,
        };

        Ok(gateway)
    }

    /// 初始化网关（连接所有配置的MCP服务器）
    pub async fn initialize(&self) -> Result<()> {
        // 连接数据源
        if !self.config.enabled_data_sources.is_empty() {
            self.connect_data_sources().await?;
        }

        // 连接交易API
        if !self.config.enabled_trading_apis.is_empty() {
            self.connect_trading_apis().await?;
        }

        // 连接工具
        if !self.config.enabled_tools.is_empty() {
            self.connect_tools().await?;
        }

        Ok(())
    }

    /// 连接所有数据源MCP服务器
    async fn connect_data_sources(&self) -> Result<()> {
        for source_name in &self.config.enabled_data_sources {
            let config = self.get_data_source_config(source_name)?;
            self.connect_data_source(source_name, config).await?;
        }
        Ok(())
    }

    /// 连接单个数据源
    async fn connect_data_source(&self, name: &str, config: MCPConfig) -> Result<()> {
        let client = MCPClient::new(name, config).await
            .context(format!("Failed to connect to data source: {}", name))?;

        {
            let mut connections = self.connections.write().await;
            connections.insert(name.to_string(), Arc::new(client));
        }

        {
            let mut data_sources = self.data_sources.write().await;
            data_sources.insert(name.to_string(), "data".to_string());
        }

        tracing::info!("Connected to data source MCP: {}", name);
        Ok(())
    }

    /// 连接所有交易API MCP服务器
    async fn connect_trading_apis(&self) -> Result<()> {
        for api_name in &self.config.enabled_trading_apis {
            let config = self.get_trading_api_config(api_name)?;
            self.connect_trading_api(api_name, config).await?;
        }
        Ok(())
    }

    /// 连接单个交易API
    async fn connect_trading_api(&self, name: &str, config: MCPConfig) -> Result<()> {
        let client = MCPClient::new(name, config).await
            .context(format!("Failed to connect to trading API: {}", name))?;

        {
            let mut connections = self.connections.write().await;
            connections.insert(name.to_string(), Arc::new(client));
        }

        {
            let mut trading_apis = self.trading_apis.write().await;
            trading_apis.insert(name.to_string(), "trading".to_string());
        }

        tracing::info!("Connected to trading API MCP: {}", name);
        Ok(())
    }

    /// 连接所有工具MCP服务器
    async fn connect_tools(&self) -> Result<()> {
        for tool_name in &self.config.enabled_tools {
            let config = self.get_tool_config(tool_name)?;
            self.connect_tool(tool_name, config).await?;
        }
        Ok(())
    }

    /// 连接单个工具
    async fn connect_tool(&self, name: &str, config: MCPConfig) -> Result<()> {
        let client = MCPClient::new(name, config).await
            .context(format!("Failed to connect to tool: {}", name))?;

        {
            let mut connections = self.connections.write().await;
            connections.insert(name.to_string(), Arc::new(client));
        }

        {
            let mut tools = self.tools.write().await;
            tools.insert(name.to_string(), "tool".to_string());
        }

        tracing::info!("Connected to tool MCP: {}", name);
        Ok(())
    }

    /// 获取数据源配置
    fn get_data_source_config(&self, name: &str) -> Result<MCPConfig> {
        match name {
            "yahoo-finance-mcp" => Ok(MCPConfig::default()),
            "alpha-vantage-mcp" => Ok(MCPConfig {
                api_key: std::env::var("ALPHA_VANTAGE_API_KEY").ok(),
                ..Default::default()
            }),
            "tushare-mcp" => Ok(MCPConfig {
                api_key: std::env::var("TUSHARE_API_KEY").ok(),
                ..Default::default()
            }),
            "binance-mcp" => Ok(MCPConfig::default()),
            _ => Ok(MCPConfig::default()),
        }
    }

    /// 获取交易API配置
    fn get_trading_api_config(&self, name: &str) -> Result<MCPConfig> {
        match name {
            "qmt-broker-mcp" => Ok(MCPConfig {
                api_key: std::env::var("QMT_API_KEY").ok(),
                ..Default::default()
            }),
            "interactive-brokers-mcp" => Ok(MCPConfig {
                api_key: std::env::var("IBKR_API_KEY").ok(),
                ..Default::default()
            }),
            "binance-trading-mcp" => Ok(MCPConfig {
                api_key: std::env::var("BINANCE_API_KEY").ok(),
                secret: std::env::var("BINANCE_SECRET").ok(),
                ..Default::default()
            }),
            _ => Ok(MCPConfig::default()),
        }
    }

    /// 获取工具配置
    fn get_tool_config(&self, name: &str) -> Result<MCPConfig> {
        match name {
            "news-api-mcp" => Ok(MCPConfig {
                api_key: std::env::var("NEWS_API_KEY").ok(),
                ..Default::default()
            }),
            "sec-filings-mcp" => Ok(MCPConfig::default()),
            _ => Ok(MCPConfig::default()),
        }
    }

    /// 查询数据（统一接口）
    pub async fn query_data(&self, query: DataQuery) -> Result<Data> {
        // 根据domain选择最佳数据源
        let mcp_name = self.select_best_data_source(&query.domain)?;

        let connections = self.connections.read().await;
        let client = connections.get(&mcp_name)
            .context(format!("MCP client not found: {}", mcp_name))?;

        // 调用MCP工具
        let response = client.call_tool("get_data", serde_json::to_value(query)?)
            .await
            .context("Failed to call MCP tool")?;

        Ok(serde_json::from_value(response)?)
    }

    /// 执行交易（统一接口）
    pub async fn execute_trade(&self, order: OrderRequest) -> Result<OrderResponse> {
        let mcp_name = self.get_broker_for_market(&order.market)?;

        let connections = self.connections.read().await;
        let client = connections.get(&mcp_name)
            .context(format!("Trading MCP client not found: {}", mcp_name))?;

        let response = client.call_tool("place_order", serde_json::to_value(order)?)
            .await
            .context("Failed to execute trade")?;

        Ok(serde_json::from_value(response)?)
    }

    /// 选择最佳数据源
    fn select_best_data_source(&self, domain: &str) -> Result<String> {
        // 简单策略：根据domain选择数据源
        // 实际实现可以更智能，考虑可用性、延迟、成本等
        match domain {
            "us-stock" => Ok("yahoo-finance-mcp".to_string()),
            "us-stock-fundamental" => Ok("alpha-vantage-mcp".to_string()),
            "china-stock" => Ok("tushare-mcp".to_string()),
            "crypto" => Ok("binance-mcp".to_string()),
            _ => Ok("yahoo-finance-mcp".to_string()), // 默认
        }
    }

    /// 获取市场对应的券商MCP
    fn get_broker_for_market(&self, market: &str) -> Result<String> {
        match market {
            "china" => Ok("qmt-broker-mcp".to_string()),
            "us" => Ok("interactive-brokers-mcp".to_string()),
            "crypto" => Ok("binance-trading-mcp".to_string()),
            _ => anyhow::bail!("Unknown market: {}", market),
        }
    }

    /// 动态添加MCP服务器
    pub async fn add_mcp_server(&self, name: String, url: String, server_type: &str) -> Result<()> {
        let config = MCPConfig {
            url: Some(url),
            ..Default::default()
        };

        match server_type {
            "data" => self.connect_data_source(&name, config).await?,
            "trading" => self.connect_trading_api(&name, config).await?,
            "tool" => self.connect_tool(&name, config).await?,
            _ => anyhow::bail!("Unknown server type: {}", server_type),
        }

        Ok(())
    }

    /// 移除MCP服务器
    pub async fn remove_mcp_server(&self, name: &str) -> Result<()> {
        {
            let mut connections = self.connections.write().await;
            connections.remove(name);
        }

        {
            let mut data_sources = self.data_sources.write().await;
            data_sources.remove(name);
        }

        {
            let mut trading_apis = self.trading_apis.write().await;
            trading_apis.remove(name);
        }

        {
            let mut tools = self.tools.write().await;
            tools.remove(name);
        }

        tracing::info!("Removed MCP server: {}", name);
        Ok(())
    }

    /// 获取连接状态
    pub async fn get_connection_status(&self) -> ConnectionStatus {
        let connections = self.connections.read().await;
        let data_sources = self.data_sources.read().await;
        let trading_apis = self.trading_apis.read().await;
        let tools = self.tools.read().await;

        ConnectionStatus {
            total_connections: connections.len(),
            data_sources: data_sources.len(),
            trading_apis: trading_apis.len(),
            tools: tools.len(),
            connected_servers: connections.keys().cloned().collect(),
        }
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let connections = self.connections.read().await;
        let mut healthy_servers = Vec::new();
        let mut unhealthy_servers = Vec::new();

        for (name, client) in connections.iter() {
            match client.ping().await {
                Ok(_) => healthy_servers.push(name.clone()),
                Err(_) => unhealthy_servers.push(name.clone()),
            }
        }

        Ok(HealthStatus {
            total: connections.len(),
            healthy: healthy_servers.len(),
            unhealthy: unhealthy_servers.len(),
            healthy_servers,
            unhealthy_servers,
        })
    }
}

/// 数据查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQuery {
    /// 查询域
    pub domain: String,

    /// 查询类型
    pub query_type: String,

    /// 参数
    pub params: serde_json::Value,
}

/// 数据响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    /// 数据内容
    pub content: serde_json::Value,

    /// 元数据
    pub metadata: HashMap<String, String>,

    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 交易请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    /// 市场
    pub market: String,

    /// 交易对
    pub symbol: String,

    /// 订单类型
    pub order_type: String,

    /// 方向
    pub side: String,

    /// 数量
    pub quantity: f64,

    /// 价格（限价单）
    pub price: Option<f64>,
}

/// 交易响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    /// 订单ID
    pub order_id: String,

    /// 状态
    pub status: String,

    /// 成交数量
    pub filled_quantity: Option<f64>,

    /// 成交价格
    pub filled_price: Option<f64>,

    /// 错误信息
    pub error: Option<String>,
}

/// 连接状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    /// 总连接数
    pub total_connections: usize,

    /// 数据源数量
    pub data_sources: usize,

    /// 交易API数量
    pub trading_apis: usize,

    /// 工具数量
    pub tools: usize,

    /// 已连接的服务器列表
    pub connected_servers: Vec<String>,
}

/// 健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// 总服务器数
    pub total: usize,

    /// 健康服务器数
    pub healthy: usize,

    /// 不健康服务器数
    pub unhealthy: usize,

    /// 健康服务器列表
    pub healthy_servers: Vec<String>,

    /// 不健康服务器列表
    pub unhealthy_servers: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gateway_creation() {
        let config = GatewayConfig::default();
        let gateway = MCPGateway::new(config).await;
        assert!(gateway.is_ok());
    }

    #[tokio::test]
    async fn test_data_source_selection() {
        let config = GatewayConfig::default();
        let gateway = MCPGateway::new(config).await.unwrap();

        assert_eq!(
            gateway.select_best_data_source("us-stock").unwrap(),
            "yahoo-finance-mcp"
        );
        assert_eq!(
            gateway.select_best_data_source("crypto").unwrap(),
            "binance-mcp"
        );
    }

    #[tokio::test]
    async fn test_broker_selection() {
        let config = GatewayConfig::default();
        let gateway = MCPGateway::new(config).await.unwrap();

        assert_eq!(
            gateway.get_broker_for_market("us").unwrap(),
            "interactive-brokers-mcp"
        );
        assert_eq!(
            gateway.get_broker_for_market("crypto").unwrap(),
            "binance-trading-mcp"
        );
    }
}
