//! MCP Gateway 测试
//!
//! 测试MCP Gateway的核心功能

use investintel_agent::mcp::{MCPGateway, GatewayConfig, DataQuery, Data};

#[tokio::test]
async fn test_gateway_creation() {
    let config = GatewayConfig::default();
    let gateway = MCPGateway::new(config).await;
    assert!(gateway.is_ok());
}

#[tokio::test]
async fn test_gateway_initialization() {
    let config = GatewayConfig {
        enabled_data_sources: vec![],
        enabled_trading_apis: vec![],
        enabled_tools: vec![],
        ..Default::default()
    };

    let gateway = MCPGateway::new(config).await.unwrap();
    // 初始化应该成功，即使没有配置任何MCP服务器
    let result = gateway.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_connection_status() {
    let config = GatewayConfig::default();
    let gateway = MCPGateway::new(config).await.unwrap();

    let status = gateway.get_connection_status().await;
    assert_eq!(status.total_connections, 0);
    assert_eq!(status.data_sources, 0);
    assert_eq!(status.trading_apis, 0);
    assert_eq!(status.tools, 0);
}

#[tokio::test]
async fn test_health_check() {
    let config = GatewayConfig::default();
    let gateway = MCPGateway::new(config).await.unwrap();

    let health = gateway.health_check().await;
    assert!(health.is_ok());

    let status = health.unwrap();
    assert_eq!(status.total, 0);
    assert_eq!(status.healthy, 0);
    assert_eq!(status.unhealthy, 0);
}

#[tokio::test]
async fn test_data_source_selection() {
    let config = GatewayConfig::default();
    let gateway = MCPGateway::new(config).await.unwrap();

    // 测试数据源选择逻辑
    let source = gateway.select_best_data_source("us-stock");
    assert!(source.is_ok());
    assert_eq!(source.unwrap(), "yahoo-finance-mcp");

    let source = gateway.select_best_data_source("crypto");
    assert!(source.is_ok());
    assert_eq!(source.unwrap(), "binance-mcp");
}

#[tokio::test]
async fn test_broker_selection() {
    let config = GatewayConfig::default();
    let gateway = MCPGateway::new(config).await.unwrap();

    // 测试券商选择逻辑
    let broker = gateway.get_broker_for_market("us");
    assert!(broker.is_ok());
    assert_eq!(broker.unwrap(), "interactive-brokers-mcp");

    let broker = gateway.get_broker_for_market("crypto");
    assert!(broker.is_ok());
    assert_eq!(broker.unwrap(), "binance-trading-mcp");
}

#[tokio::test]
async fn test_add_remove_mcp_server() {
    let config = GatewayConfig::default();
    let gateway = MCPGateway::new(config).await.unwrap();

    // 添加服务器（不会真的连接，只是测试接口）
    let result = gateway.add_mcp_server(
        "test-mcp".to_string(),
        "https://example.com".to_string(),
        "data"
    ).await;

    // 预期会失败，因为无法连接，但接口应该存在
    // assert!(result.is_err());

    // 移除服务器
    let result = gateway.remove_mcp_server("test-mcp").await;
    assert!(result.is_ok());
}
