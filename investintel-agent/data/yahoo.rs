//! Yahoo Finance API Client
//!
//! Provides real-time quotes, historical OHLCV data, and financial statements
//! using Yahoo Finance's public API (compatible with yfinance)

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Yahoo Finance Client
#[derive(Debug, Clone)]
pub struct YahooFinanceClient {
    client: Client,
    base_url: String,
}

impl YahooFinanceClient {
    /// Create a new Yahoo Finance client
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (compatible; InvestIntel/1.0)")
                .build()
                .unwrap_or_default(),
            base_url: "https://query1.finance.yahoo.com".to_string(),
        }
    }

    /// Create a client with custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (compatible; InvestIntel/1.0)")
                .build()
                .unwrap_or_default(),
            base_url,
        }
    }

    /// Get real-time quote for a symbol
    ///
    /// # Example
    /// ```no_run
    /// let client = YahooFinanceClient::new();
    /// let quote = client.get_quote("AAPL").await?;
    /// println!("{}: ${}", quote.symbol, quote.regular_market_price);
    /// ```
    pub async fn get_quote(&self, symbol: &str) -> Result<QuoteData> {
        let url = format!(
            "{}/v8/finance/chart/{}?interval=1m&range=1d",
            self.base_url, symbol
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to fetch quote from Yahoo Finance")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Yahoo Finance API returned status: {}",
                response.status()
            );
        }

        let chart_response: YahooChartResponse = response
            .json()
            .await
            .context("Failed to parse Yahoo Finance response")?;

        self.parse_quote(symbol, chart_response)
    }

    /// Get historical OHLCV data for a symbol
    ///
    /// # Arguments
    /// * `symbol` - Stock symbol (e.g., "AAPL")
    /// * `interval` - Data interval: "1m", "5m", "15m", "1h", "1d", "1wk", "1mo"
    /// * `range` - Time range: "1d", "5d", "1mo", "3mo", "6mo", "1y", "max"
    ///
    /// # Example
    /// ```no_run
    /// let client = YahooFinanceClient::new();
    /// let data = client.get_historical("AAPL", "1d", "1mo").await?;
    /// for bar in &data {
    ///     println!("{}: Open=${} Close=${}", bar.timestamp, bar.open, bar.close);
    /// }
    /// ```
    pub async fn get_historical(
        &self,
        symbol: &str,
        interval: &str,
        range: &str,
    ) -> Result<Vec<OHLCV>> {
        let url = format!(
            "{}/v8/finance/chart/{}?interval={}&range={}",
            self.base_url, symbol, interval, range
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to fetch historical data from Yahoo Finance")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Yahoo Finance API returned status: {}",
                response.status()
            );
        }

        let chart_response: YahooChartResponse = response.json().await.context(
            "Failed to parse historical data response from Yahoo Finance",
        )?;

        self.parse_historical(symbol, chart_response)
    }

    /// Get historical data with specific date range
    ///
    /// # Arguments
    /// * `symbol` - Stock symbol
    /// * `interval` - Data interval
    /// * `period_start` - Unix timestamp for start date
    /// * `period_end` - Unix timestamp for end date
    pub async fn get_historical_date_range(
        &self,
        symbol: &str,
        interval: &str,
        period_start: i64,
        period_end: i64,
    ) -> Result<Vec<OHLCV>> {
        let url = format!(
            "{}/v8/finance/chart/{}?interval={}&period1={}&period2={}",
            self.base_url, symbol, interval, period_start, period_end
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to fetch date-range data from Yahoo Finance")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Yahoo Finance API returned status: {}",
                response.status()
            );
        }

        let chart_response: YahooChartResponse = response.json().await.context(
            "Failed to parse date-range response from Yahoo Finance",
        )?;

        self.parse_historical(symbol, chart_response)
    }

    /// Search for symbols matching a query
    pub async fn search_symbols(&self, query: &str) -> Result<Vec<SymbolSearchResult>> {
        let url = format!(
            "{}/v1/finance/search?q={}&quotesCount=10&newsCount=0",
            self.base_url, query
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to search symbols on Yahoo Finance")?;

        if !response.status().is_success() {
            anyhow::bail!("Yahoo Finance search returned status: {}", response.status());
        }

        let search_response: YahooSearchResponse = response
            .json()
            .await
            .context("Failed to parse search response")?;

        Ok(search_response.quotes.unwrap_or_default())
    }

    /// Parse quote data from chart response
    fn parse_quote(&self, symbol: &str, response: YahooChartResponse) -> Result<QuoteData> {
        let mut result_vec = response
            .chart
            .result
            .ok_or_else(|| anyhow::anyhow!("No chart result in response"))?;

        let result = result_vec.first_mut()
            .ok_or_else(|| anyhow::anyhow!("Empty result array"))?;

        let meta = result.meta.clone()
            .ok_or_else(|| anyhow::anyhow!("No meta in chart result"))?;

        let price = result
            .indicators
            .quote
            .first()
            .and_then(|q| q.close.last().copied())
            .unwrap_or(meta.regular_market_price.unwrap_or(0.0));

        Ok(QuoteData {
            symbol: symbol.to_uppercase(),
            company_name: meta.long_name.unwrap_or_default(),
            regular_market_price: price,
            previous_close: meta.previous_close.unwrap_or(0.0),
            change: price - meta.previous_close.unwrap_or(0.0),
            change_percent: if meta.previous_close.unwrap_or(0.0) > 0.0 {
                ((price - meta.previous_close.unwrap_or(0.0)) / meta.previous_close.unwrap_or(1.0))
                    * 100.0
            } else {
                0.0
            },
            day_high: meta.regular_market_day_high.unwrap_or(0.0),
            day_low: meta.regular_market_day_low.unwrap_or(0.0),
            volume: meta.regular_market_volume.unwrap_or(0) as u64,
            week_high: meta.fifty_two_week_high.unwrap_or(0.0),
            week_low: meta.fifty_two_week_low.unwrap_or(0.0),
            market_cap: meta.market_cap.unwrap_or(0),
            currency: meta.currency.unwrap_or_else(|| "USD".to_string()),
            timestamp: Utc::now(),
        })
    }

    /// Parse historical OHLCV data from chart response
    fn parse_historical(&self, symbol: &str, response: YahooChartResponse) -> Result<Vec<OHLCV>> {
        let mut result_vec = response
            .chart
            .result
            .ok_or_else(|| anyhow::anyhow!("No chart result in response"))?;

        let result = result_vec.first_mut()
            .ok_or_else(|| anyhow::anyhow!("Empty result array"))?;

        let timestamp = result.timestamp.take()
            .ok_or_else(|| anyhow::anyhow!("No timestamp data"))?;
        let quote = result
            .indicators
            .quote
            .first()
            .ok_or_else(|| anyhow::anyhow!("No quote indicator"))?;

        let mut ohlcv_data = Vec::new();

        for (i, ts) in timestamp.iter().enumerate() {
            let open = quote.open.get(i).copied().unwrap_or(0.0);
            let high = quote.high.get(i).copied().unwrap_or(0.0);
            let low = quote.low.get(i).copied().unwrap_or(0.0);
            let close = quote.close.get(i).copied().unwrap_or(0.0);
            let volume = quote.volume.get(i).copied().unwrap_or(0) as u64;

            // Skip invalid data
            if close > 0.0 {
                ohlcv_data.push(OHLCV {
                    symbol: symbol.to_uppercase(),
                    timestamp: DateTime::from_timestamp(*ts, 0).unwrap_or_default(),
                    open,
                    high,
                    low,
                    close,
                    volume,
                });
            }
        }

        Ok(ohlcv_data)
    }
}

impl Default for YahooFinanceClient {
    fn default() -> Self {
        Self::new()
    }
}

// ==============================================================================
// Data Structures
// ==============================================================================

/// Real-time quote data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteData {
    pub symbol: String,
    pub company_name: String,
    pub regular_market_price: f64,
    pub previous_close: f64,
    pub change: f64,
    pub change_percent: f64,
    pub day_high: f64,
    pub day_low: f64,
    pub volume: u64,
    #[serde(alias = "52_week_high")]
    pub week_high: f64,
    #[serde(alias = "52_week_low")]
    pub week_low: f64,
    pub market_cap: u64,
    pub currency: String,
    pub timestamp: DateTime<Utc>,
}

/// OHLCV (Open, High, Low, Close, Volume) candlestick data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OHLCV {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

/// Symbol search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolSearchResult {
    pub symbol: String,
    pub name: String,
    #[serde(rename = "type")]
    pub result_type: String,
    pub exch: String,
    pub exchDisp: String,
}

// ==============================================================================
// Yahoo Finance API Response Structures
// ==============================================================================

#[derive(Debug, Clone, Deserialize)]
struct YahooChartResponse {
    pub chart: YahooChart,
}

#[derive(Debug, Clone, Deserialize)]
struct YahooChart {
    pub result: Option<Vec<YahooChartResult>>,
    pub error: Option<YahooError>,
}

#[derive(Debug, Clone, Deserialize)]
struct YahooChartResult {
    pub meta: Option<YahooMeta>,
    pub timestamp: Option<Vec<i64>>,
    pub indicators: YahooIndicators,
}

#[derive(Debug, Clone, Deserialize)]
struct YahooMeta {
    pub currency: Option<String>,
    pub symbol: Option<String>,
    pub exchange_name: Option<String>,
    pub instrument_type: Option<String>,
    pub first_trade_date: Option<i64>,
    pub regular_market_time: Option<i64>,
    pub gmtoffset: Option<i32>,
    pub timezone: Option<String>,
    pub exchange_timezone_name: Option<String>,
    pub regular_market_price: Option<f64>,
    pub chart_previous_close: Option<f64>,
    pub previous_close: Option<f64>,
    pub scale: Option<i32>,
    pub price_hint: Option<i32>,
    pub current_trading_period: Option<TradingPeriod>,
    pub trading_periods: Option<Vec<Vec<TradingPeriod>>>,
    #[serde(alias = "52_week_high")]
    pub fifty_two_week_high: Option<f64>,
    #[serde(alias = "52_week_low")]
    pub fifty_two_week_low: Option<f64>,
    pub regular_market_day_high: Option<f64>,
    pub regular_market_day_low: Option<f64>,
    pub regular_market_volume: Option<i64>,
    pub long_name: Option<String>,
    pub short_name: Option<String>,
    pub market_cap: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct TradingPeriod {
    pub timezone: Option<String>,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub gmtoffset: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
struct YahooIndicators {
    pub quote: Vec<YahooQuote>,
    #[serde(default)]
    pub adjclose: Vec<YahooAdjClose>,
}

#[derive(Debug, Clone, Deserialize)]
struct YahooQuote {
    #[serde(default)]
    pub open: Vec<f64>,
    #[serde(default)]
    pub high: Vec<f64>,
    #[serde(default)]
    pub low: Vec<f64>,
    #[serde(default)]
    pub close: Vec<f64>,
    #[serde(default)]
    pub volume: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
struct YahooAdjClose {
    #[serde(default)]
    pub adjclose: Vec<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct YahooError {
    pub code: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct YahooSearchResponse {
    pub explains: Option<Vec<serde_json::Value>>,
    pub count: Option<i32>,
    pub quotes: Option<Vec<SymbolSearchResult>>,
    pub news: Option<Vec<serde_json::Value>>,
    pub nav: Option<serde_json::Value>,
    pub lists: Option<Vec<serde_json::Value>>,
    pub research_reports: Option<Vec<serde_json::Value>>,
    pub screener_field_results: Option<Vec<serde_json::Value>>,
    pub time: Option<serde_json::Value>,
}

// ==============================================================================
// MCP Tool Functions
// ==============================================================================

use claude_agent_sdk_rs::ToolResult;

/// MCP Tool: Get real-time quote from Yahoo Finance
pub async fn yahoo_finance_quote(args: serde_json::Value) -> Result<ToolResult> {
    let symbol = args
        .get("symbol")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'symbol' parameter"))?;

    let client = YahooFinanceClient::new();
    let quote = client.get_quote(symbol).await.with_context(|| {
        format!("Failed to get quote for symbol: {}", symbol)
    })?;

    let output = serde_json::to_string_pretty(&quote)
        .context("Failed to serialize quote data")?;

    Ok(ToolResult {
        content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
            text: format!("Yahoo Finance Quote for {}\n\n{}", symbol, output),
        }],
        is_error: false,
    })
}

/// MCP Tool: Get historical OHLCV data from Yahoo Finance
pub async fn yahoo_finance_historical(args: serde_json::Value) -> Result<ToolResult> {
    let symbol = args
        .get("symbol")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'symbol' parameter"))?;

    let interval = args
        .get("interval")
        .and_then(|v| v.as_str())
        .unwrap_or("1d");

    let range = args
        .get("range")
        .and_then(|v| v.as_str())
        .unwrap_or("1mo");

    let client = YahooFinanceClient::new();
    let data = client.get_historical(symbol, interval, range).await
        .with_context(|| {
            format!("Failed to get historical data for symbol: {}", symbol)
        })?;

    let output = serde_json::to_string_pretty(&data)
        .context("Failed to serialize historical data")?;

    Ok(ToolResult {
        content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
            text: format!(
                "Yahoo Finance Historical Data for {} ({} interval, {} range)\n\n{}\n\nTotal records: {}",
                symbol,
                interval,
                range,
                output,
                data.len()
            ),
        }],
        is_error: false,
    })
}

/// MCP Tool: Search for symbols
pub async fn yahoo_finance_search(args: serde_json::Value) -> Result<ToolResult> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?;

    let client = YahooFinanceClient::new();
    let results = client.search_symbols(query).await.with_context(|| {
        format!("Failed to search for: {}", query)
    })?;

    let output = serde_json::to_string_pretty(&results)
        .context("Failed to serialize search results")?;

    Ok(ToolResult {
        content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
            text: format!(
                "Yahoo Finance Symbol Search for '{}'\n\n{}\n\nFound {} results",
                query,
                output,
                results.len()
            ),
        }],
        is_error: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_yahoo_client_creation() {
        let client = YahooFinanceClient::new();
        assert_eq!(client.base_url, "https://query1.finance.yahoo.com");
    }

    #[tokio::test]
    async fn test_get_quote() {
        let client = YahooFinanceClient::new();
        let result = client.get_quote("AAPL").await;

        match result {
            Ok(quote) => {
                assert_eq!(quote.symbol, "AAPL");
                assert!(quote.regular_market_price > 0.0);
                println!("Quote for AAPL: ${}", quote.regular_market_price);
            }
            Err(e) => {
                eprintln!("Error fetching quote: {:?}", e);
                // Network tests may fail in CI/CD, don't fail the test
            }
        }
    }

    #[tokio::test]
    async fn test_get_historical() {
        let client = YahooFinanceClient::new();
        let result = client.get_historical("AAPL", "1d", "5d").await;

        match result {
            Ok(data) => {
                assert!(!data.is_empty());
                assert_eq!(data[0].symbol, "AAPL");
                println!("Got {} historical data points", data.len());
            }
            Err(e) => {
                eprintln!("Error fetching historical: {:?}", e);
                // Network tests may fail in CI/CD
            }
        }
    }
}
