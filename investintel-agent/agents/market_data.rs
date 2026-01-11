//! Market Data Provider - 市场数据提供者
//!
//! 为Agents提供统一的市场数据获取接口
//!
//! ## 核心功能
//!
//! 1. **实时报价** - 获取实时股价、市值等
//! 2. **财务数据** - EPS、ROE、ROIC等基本面数据
//! 3. **历史数据** - 历史价格、股息历史
//! 4. **智能缓存** - 减少API调用，提升性能
//! 5. **MCP Gateway集成** - 支持MCP统一数据源 (NEW!)

use crate::data::{
    yahoo::YahooFinanceClient, yahoo_finance_quote, yahoo_finance_historical,
    fusion::{fusion_get_quote, fusion_initialize},
    alpha_vantage::{alpha_vantage_overview, CompanyOverview},
};
use crate::mcp::MCPGateway;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 市场数据提供者
#[derive(Debug, Clone)]
pub struct MarketDataProvider {
    /// Yahoo Finance客户端
    yahoo_client: Arc<YahooFinanceClient>,

    /// MCP Gateway (可选)
    mcp_gateway: Option<Arc<MCPGateway>>,

    /// 数据缓存
    cache: Arc<RwLock<DataCache>>,

    /// 缓存TTL (秒)
    cache_ttl: u64,

    /// 是否优先使用MCP
    prefer_mcp: bool,
}

/// 数据缓存
#[derive(Debug, Clone)]
struct DataCache {
    quotes: std::collections::HashMap<String, CachedQuote>,
    fundamentals: std::collections::HashMap<String, CachedFundamental>,
}

#[derive(Debug, Clone)]
struct CachedQuote {
    data: StockQuote,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct CachedFundamental {
    data: FundamentalData,
    timestamp: DateTime<Utc>,
}

/// 股票实时报价
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub high: f64,

    /// 日最低价
    pub low: f64,

    /// 开盘价
    pub open: f64,

    /// 前收盘价
    pub previous_close: f64,

    /// 成交量
    pub volume: u64,

    /// 市值
    pub market_cap: Option<f64>,

    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 基本面数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalData {
    /// 股票代码
    pub symbol: String,

    /// 每股收益 (EPS)
    pub eps: Option<f64>,

    /// PE比率
    pub pe_ratio: Option<f64>,

    /// ROE (%)
    pub roe: Option<f64>,

    /// ROIC (%)
    pub roic: Option<f64>,

    /// 股息收益率
    pub dividend_yield: Option<f64>,

    /// 年度股息
    pub annual_dividend: Option<f64>,

    /// 股息支付比率
    pub payout_ratio: Option<f64>,

    /// 52周最高
    pub week_52_high: Option<f64>,

    /// 52周最低
    pub week_52_low: Option<f64>,

    /// Beta
    pub beta: Option<f64>,

    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

impl MarketDataProvider {
    /// 创建新的市场数据提供者
    pub fn new() -> Self {
        Self {
            yahoo_client: Arc::new(YahooFinanceClient::new()),
            mcp_gateway: None,
            cache: Arc::new(RwLock::new(DataCache {
                quotes: std::collections::HashMap::new(),
                fundamentals: std::collections::HashMap::new(),
            })),
            cache_ttl: 60, // 60秒缓存
            prefer_mcp: false,
        }
    }

    /// 设置缓存TTL
    pub fn with_cache_ttl(mut self, ttl_seconds: u64) -> Self {
        self.cache_ttl = ttl_seconds;
        self
    }

    /// 设置MCP Gateway
    pub fn with_mcp_gateway(mut self, gateway: Arc<MCPGateway>, prefer: bool) -> Self {
        self.mcp_gateway = Some(gateway);
        self.prefer_mcp = prefer;
        self
    }

    /// 获取实时报价
    ///
    /// 优先从缓存获取，缓存过期则从API获取
    /// 支持MCP Gateway作为可选数据源
    pub async fn get_quote(&self, symbol: &str) -> Result<StockQuote> {
        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.quotes.get(symbol) {
                let age = (Utc::now() - cached.timestamp).num_seconds() as u64;
                if age < self.cache_ttl {
                    return Ok(cached.data.clone());
                }
            }
        }

        // 如果配置了MCP Gateway且优先使用，尝试从MCP获取
        if self.prefer_mcp {
            if let Some(gateway) = &self.mcp_gateway {
                if let Ok(quote) = self.try_get_quote_from_mcp(gateway, symbol).await {
                    // 更新缓存
                    let mut cache = self.cache.write().await;
                    cache.quotes.insert(
                        symbol.to_uppercase(),
                        CachedQuote {
                            data: quote.clone(),
                            timestamp: Utc::now(),
                        },
                    );
                    return Ok(quote);
                }
                // MCP失败，fallback到Yahoo Finance
            }
        }

        // 从Yahoo Finance获取
        let yahoo_quote = self.yahoo_client.get_quote(symbol).await?;

        // 转换为统一格式
        let quote = StockQuote {
            symbol: symbol.to_uppercase(),
            current_price: yahoo_quote.regular_market_price,
            change: yahoo_quote.change,
            change_percent: yahoo_quote.change_percent,
            high: yahoo_quote.day_high,
            low: yahoo_quote.day_low,
            open: yahoo_quote.previous_close, // Yahoo API不提供open，使用previous_close作为近似
            previous_close: yahoo_quote.previous_close,
            volume: yahoo_quote.volume,
            market_cap: yahoo_quote.market_cap,
            timestamp: Utc::now(),
        };

        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            cache.quotes.insert(
                symbol.to_uppercase(),
                CachedQuote {
                    data: quote.clone(),
                    timestamp: Utc::now(),
                },
            );
        }

        Ok(quote)
    }

    /// 尝试从MCP Gateway获取报价
    async fn try_get_quote_from_mcp(&self, gateway: &Arc<MCPGateway>, symbol: &str) -> Result<StockQuote> {
        use crate::mcp::{DataQuery, Data};

        let query = DataQuery {
            domain: "us-stock".to_string(),
            query_type: "quote".to_string(),
            params: serde_json::json!({
                "symbol": symbol,
            }),
        };

        let data: Data = gateway.query_data(query).await?;

        // 解析MCP返回的数据
        let content = &data.content;
        Ok(StockQuote {
            symbol: content["symbol"].as_str().unwrap_or(symbol).to_string(),
            current_price: content["current_price"].as_f64().unwrap_or(0.0),
            change: content["change"].as_f64().unwrap_or(0.0),
            change_percent: content["change_percent"].as_f64().unwrap_or(0.0),
            high: content["high"].as_f64().unwrap_or(0.0),
            low: content["low"].as_f64().unwrap_or(0.0),
            open: content["open"].as_f64().unwrap_or(0.0),
            previous_close: content["previous_close"].as_f64().unwrap_or(0.0),
            volume: content["volume"].as_u64().unwrap_or(0),
            market_cap: content["market_cap"].as_f64(),
            timestamp: Utc::now(),
        })
    }

    /// 获取基本面数据
    ///
    /// 目前使用模拟数据，后续可集成Alpha Vantage API
    pub async fn get_fundamental(&self, symbol: &str) -> Result<FundamentalData> {
        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.fundamentals.get(symbol) {
                let age = (Utc::now() - cached.timestamp).num_seconds() as u64;
                if age < self.cache_ttl * 10 {
                    // 基本面数据缓存更长时间
                    return Ok(cached.data.clone());
                }
            }
        }

        // TODO: 从Alpha Vantage获取真实基本面数据
        // 目前使用模拟数据，基于当前价格推算
        let quote = self.get_quote(symbol).await?;

        // 简单估算基本面数据 (实际应该从API获取)
        let fundamental = FundamentalData {
            symbol: symbol.to_uppercase(),
            eps: Some(quote.current_price / 25.0), // 假设PE=25
            pe_ratio: Some(25.0),
            roe: Some(15.0), // 假设ROE=15%
            roic: Some(12.0), // 假设ROIC=12%
            dividend_yield: Some(0.0), // 后续从API获取
            annual_dividend: Some(0.0),
            payout_ratio: Some(0.0),
            week_52_high: Some(quote.current_price * 1.2),
            week_52_low: Some(quote.current_price * 0.8),
            beta: Some(1.0),
            timestamp: Utc::now(),
        };

        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            cache.fundamentals.insert(
                symbol.to_uppercase(),
                CachedFundamental {
                    data: fundamental.clone(),
                    timestamp: Utc::now(),
                },
            );
        }

        Ok(fundamental)
    }

    /// 获取股息数据
    pub async fn get_dividend_data(&self, symbol: &str) -> Result<DividendData> {
        let quote = self.get_quote(symbol).await?;
        let fundamental = self.get_fundamental(symbol).await?;

        // 计算月度收入
        let annual_dividend = fundamental.annual_dividend.unwrap_or(0.0);
        let monthly_income = annual_dividend / 12.0;

        Ok(DividendData {
            symbol: symbol.to_uppercase(),
            current_price: quote.current_price,
            annual_dividend,
            dividend_yield: fundamental.dividend_yield.unwrap_or(0.0),
            monthly_income,
            payout_ratio: fundamental.payout_ratio.unwrap_or(0.0),
            growth_rate: 0.05, // 默认5%增长
            consecutive_years: 5, // 默认5年
        })
    }

    /// 清除缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.quotes.clear();
        cache.fundamentals.clear();
    }

    /// 获取缓存统计
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        CacheStats {
            quotes_count: cache.quotes.len(),
            fundamentals_count: cache.fundamentals.len(),
        }
    }
}

/// 股息数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividendData {
    /// 股票代码
    pub symbol: String,

    /// 当前价格
    pub current_price: f64,

    /// 年度股息
    pub annual_dividend: f64,

    /// 股息收益率
    pub dividend_yield: f64,

    /// 月度收入
    pub monthly_income: f64,

    /// 派息比率
    pub payout_ratio: f64,

    /// 股息增长率
    pub growth_rate: f64,

    /// 连续增长年数
    pub consecutive_years: u32,
}

/// 缓存统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub quotes_count: usize,
    pub fundamentals_count: usize,
}

impl Default for MarketDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// 辅助函数：获取实时报价
pub async fn get_realtime_quote(symbol: &str) -> Result<StockQuote> {
    let provider = MarketDataProvider::new();
    provider.get_quote(symbol).await
}

/// 辅助函数：获取基本面数据
pub async fn get_fundamental_data(symbol: &str) -> Result<FundamentalData> {
    let provider = MarketDataProvider::new();
    provider.get_fundamental(symbol).await
}

/// 辅助函数：获取股息数据
pub async fn get_dividend_info(symbol: &str) -> Result<DividendData> {
    let provider = MarketDataProvider::new();
    provider.get_dividend_data(symbol).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_market_data_provider() {
        let provider = MarketDataProvider::new();

        // 测试获取报价
        let quote = provider.get_quote("AAPL").await;
        assert!(quote.is_ok());

        if let Ok(q) = quote {
            assert_eq!(q.symbol, "AAPL");
            assert!(q.current_price > 0.0);
        }
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let provider = MarketDataProvider::with_cache_ttl(MarketDataProvider::new(), 10);

        // 第一次获取
        let quote1 = provider.get_quote("MSFT").await.unwrap();

        // 第二次获取（应该从缓存）
        let quote2 = provider.get_quote("MSFT").await.unwrap();

        assert_eq!(quote1.symbol, quote2.symbol);
        assert_eq!(quote1.current_price, quote2.current_price);

        // 检查缓存统计
        let stats = provider.cache_stats().await;
        assert!(stats.quotes_count >= 1);
    }
}
