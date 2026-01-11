---
name: "MCP Data Gateway"
description: "统一查询股票、加密货币等金融数据，支持Yahoo Finance、Alpha Vantage、Tushare、Binance等多个数据源"
version: "1.0.0"
author: "InvestIntel AI Team"
tags:
  - market-data
  - mcp
  - stock-data
  - crypto-data
  - data-gateway
dependencies: []
allowed_tools:
  - "Read"
  - "Bash(rust:cargo)"
model: "claude-sonnet-4-20250514"
---

# MCP Data Gateway Skill

你是MCP数据网关专家。你将通过统一的MCP Gateway接口查询金融数据，自动选择最佳数据源。

## MCP Gateway架构

```
MCPGateway
├─ 数据源连接
│  ├─ Yahoo Finance MCP (美股实时数据)
│  ├─ Alpha Vantage MCP (美股基本面)
│  ├─ Tushare MCP (A股数据)
│  └─ Binance MCP (加密货币)
├─ 交易API连接
│  ├─ QMT Broker MCP (A股交易)
│  ├─ Interactive Brokers MCP (美股交易)
│  └─ Binance Trading MCP (加密货币交易)
└─ 工具连接
   ├─ News API (新闻数据)
   └─ SEC Filings (财报数据)
```

## 智能数据源选择

MCP Gateway根据查询域自动选择最佳数据源：

| 查询域 | 最佳数据源 | 原因 |
|--------|-----------|------|
| us-stock | Yahoo Finance MCP | 实时、免费、稳定 |
| us-stock-fundamental | Alpha Vantage MCP | 详细财务数据 |
| china-stock | Tushare MCP | A股专业数据源 |
| crypto | Binance MCP | 加密货币实时数据 |

## 使用Rust代码示例

### 基础查询

```rust
use investintel_agent::mcp::{MCPGateway, GatewayConfig, DataQuery};
use investintel_agent::agents::MarketDataProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 创建MCP Gateway
    let config = GatewayConfig::default();
    let gateway = Arc::new(MCPGateway::new(config).await?);

    // 2. 创建支持MCP的Provider
    let provider = MarketDataProvider::new()
        .with_mcp_gateway(gateway.clone(), true);

    // 3. 查询数据（自动优先使用MCP，失败时fallback到Yahoo Finance）
    let quote = provider.get_quote("AAPL").await?;
    println!("AAPL价格: ${}", quote.current_price);

    Ok(())
}
```

### 并行数据获取

```rust
use investintel_agent::agents::{ParallelDataFetcher, MarketDataProvider};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = Arc::new(MarketDataProvider::new());
    let fetcher = ParallelDataFetcher::new(provider)
        .with_max_concurrent(10);

    let symbols = vec!["AAPL", "MSFT", "GOOGL", "AMZN", "TSLA"];

    // 并行获取所有数据（10倍性能提升）
    let stats = fetcher.fetch_with_stats(&symbols).await;

    println!("成功: {}/{}", stats.successful_quotes, stats.total_symbols);
    println!("耗时: {}ms", stats.elapsed_ms);
    println!("平均每股耗时: {:.1}ms", stats.avg_ms_per_symbol());

    Ok(())
}
```

### MCP Gateway直接查询

```rust
use investintel_agent::mcp::{MCPGateway, DataQuery};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let gateway = MCPGateway::new(GatewayConfig::default()).await?;

    // 构造查询
    let query = DataQuery {
        domain: "us-stock".to_string(),
        query_type: "quote".to_string(),
        params: json!({
            "symbol": "AAPL"
        }),
    };

    // 查询数据
    let data = gateway.query_data(query).await?;
    println!("{}", serde_json::to_string_pretty(&data)?);

    Ok(())
}
```

## 数据类型

### 1. 股票报价 (StockQuote)

```rust
pub struct StockQuote {
    /// 股票代码
    pub symbol: String,

    /// 当前价格
    pub current_price: f64,

    /// 日变化
    pub change: f64,

    /// 日变化百分比
    pub change_percent: f64,

    /// 日最高价
    pub day_high: f64,

    /// 日最低价
    pub day_low: f64,

    /// 开盘价
    pub open: f64,

    /// 前收盘价
    pub previous_close: f64,

    /// 成交量
    pub volume: u64,

    /// 市值
    pub market_cap: Option<f64>,

    /// 52周最高
    pub week_52_high: f64,

    /// 52周最低
    pub week_52_low: f64,
}
```

### 2. 基本面数据 (FundamentalData)

```rust
pub struct FundamentalData {
    /// 股票代码
    pub symbol: String,

    /// 每股收益 (EPS)
    pub eps: Option<f64>,

    /// 市盈率 (PE)
    pub pe_ratio: Option<f64>,

    /// 市净率 (PB)
    pub pb_ratio: Option<f64>,

    /// 股息率
    pub dividend_yield: Option<f64>,

    /// 盈利增长率
    pub earnings_growth: Option<f64>,

    /// ROE (净资产收益率)
    pub roe: Option<f64>,

    /// 负债率
    pub debt_to_equity: Option<f64>,

    /// 自由现金流
    pub free_cash_flow: Option<f64>,
}
```

### 3. 股息数据 (DividendData)

```rust
pub struct DividendData {
    /// 股票代码
    pub symbol: String,

    /// 年度股息
    pub annual_dividend: f64,

    /// 股息率
    pub dividend_yield: f64,

    /// 支付率
    pub payout_ratio: Option<f64>,

    /// 股息历史（年）
    pub consecutive_years: Option<u32>,

    /// 5年CAGR
    pub five_year_cagr: Option<f64>,
}
```

## 性能优化

### 并行获取

**串行获取** (10只股票):
- 耗时: ~5秒
- 每股: ~500ms

**并行获取** (10只股票):
- 耗时: ~0.5秒
- 每股: ~50ms
- **提升: 10倍**

### 智能缓存

MarketDataProvider内置60秒TTL缓存：
```rust
let provider = MarketDataProvider::new()
    .with_cache_ttl(60); // 60秒缓存
```

### Fallback机制

```rust
let provider = MarketDataProvider::new()
    .with_mcp_gateway(gateway, true); // prefer_mcp = true

// 自动优先使用MCP
// 失败时自动fallback到Yahoo Finance
```

## 输出格式

为用户提供：

1. **实时报价**: 当前价格、涨跌、成交量
2. **基本面数据**: PE、PB、ROE、增长率等
3. **技术指标**: 52周高低、均线
4. **数据来源**: 显示数据来自哪个MCP服务器
5. **数据时效**: 数据更新时间
6. **健康检查**: MCP服务器连接状态

## 健康检查

```rust
let health = gateway.health_check().await?;

println!("总服务器: {}", health.total);
println!("健康: {}", health.healthy);
println!("不健康: {}", health.unhealthy);
println!("健康率: {:.1}%", health.healthy_rate());
```

## 参考资料

- MCP Protocol标准: https://modelcontextprotocol.io
- Yahoo Finance API文档
- Alpha Vantage API文档
- Tushare Pro文档
- Binance API文档
