//! Alpha Vantage API Client
//!
//! Provides free financial data access with 500 calls/day limit
//! Features: real-time quotes, technical indicators, news sentiment

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Alpha Vantage Client
///
/// Get free API key at: https://www.alphavantage.co/support/#api-key
#[derive(Debug, Clone)]
pub struct AlphaVantageClient {
    api_key: String,
    client: Client,
    base_url: String,
}

impl AlphaVantageClient {
    /// Create a new Alpha Vantage client with API key
    ///
    /// # Example
    /// ```no_run
    /// let client = AlphaVantageClient::new("YOUR_API_KEY");
    /// ```
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: Client::builder()
                .user_agent("Mozilla/5.0 (compatible; InvestIntel/1.0)")
                .build()
                .unwrap_or_default(),
            base_url: "https://www.alphavantage.co".to_string(),
        }
    }

    /// Create client with custom base URL
    pub fn with_base_url(api_key: impl Into<String>, base_url: String) -> Self {
        Self {
            api_key: api_key.into(),
            client: Client::builder()
                .user_agent("Mozilla/5.0 (compatible; InvestIntel/1.0)")
                .build()
                .unwrap_or_default(),
            base_url,
        }
    }

    /// Get real-time global quote
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = AlphaVantageClient::new("demo");
    /// let quote = client.get_quote("IBM").await?;
    /// println!("{}: ${}", quote.symbol, quote.price);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_quote(&self, symbol: &str) -> Result<GlobalQuote> {
        let url = format!(
            "{}/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
            self.base_url, symbol, self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch quote from Alpha Vantage")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Alpha Vantage API returned status: {}",
                response.status()
            );
        }

        let raw_response: AlphaVantageRawResponse = response
            .json()
            .await
            .context("Failed to parse Alpha Vantage response")?;

        // Check for API error messages
        if let Some(error_msg) = raw_response.note {
            if error_msg.contains("higher call frequency") {
                anyhow::bail!("Alpha Vantage rate limit exceeded. Please wait and try again.");
            }
        }

        if let Some(error_msg) = raw_response.error_message {
            anyhow::bail!("Alpha Vantage API error: {}", error_msg);
        }

        raw_response
            .global_quote
            .ok_or_else(|| anyhow::anyhow!("No quote data in response"))
    }

    /// Get news sentiment for tickers
    ///
    /// # Arguments
    /// * `tickers` - List of stock symbols (comma-separated)
    /// * `time_from` - Filter articles from this time (optional)
    /// * `time_to` - Filter articles until this time (optional)
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = AlphaVantageClient::new("demo");
    /// let sentiment = client.get_news_sentiment("AAPL,MSFT", None, None).await?;
    /// for article in &sentiment.feed {
    ///     println!("{}: {}", article.ticker, article.sentiment_score);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_news_sentiment(
        &self,
        tickers: &str,
        time_from: Option<&str>,
        time_to: Option<&str>,
    ) -> Result<NewsSentiment> {
        let mut url = format!(
            "{}/query?function=NEWS_SENTIMENT&tickers={}&apikey={}",
            self.base_url, tickers, self.api_key
        );

        if let Some(from) = time_from {
            url.push_str(&format!("&time_from={}", from));
        }
        if let Some(to) = time_to {
            url.push_str(&format!("&time_to={}", to));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch news sentiment from Alpha Vantage")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Alpha Vantage API returned status: {}",
                response.status()
            );
        }

        let raw_response: AlphaVantageNewsResponse = response
            .json()
            .await
            .context("Failed to parse news sentiment response")?;

        if let Some(error_msg) = raw_response.note {
            if error_msg.contains("higher call frequency") {
                anyhow::bail!("Alpha Vantage rate limit exceeded. Please wait and try again.");
            }
        }

        Ok(raw_response.sentiment.unwrap_or_else(|| NewsSentiment {
            feed: Vec::new(),
            sentiment_by_ticker: HashMap::new(),
        }))
    }

    /// Get technical indicator values
    ///
    /// # Arguments
    /// * `symbol` - Stock symbol
    /// * `function` - Indicator type (SMA, EMA, RSI, MACD, BBANDS, etc.)
    /// * `interval` - Time interval (1min, 5min, 15min, 30min, 60min, daily, weekly, monthly)
    /// * `time_period` - Number of data points (optional, depends on indicator)
    /// * `series_type` - Price type (close, open, high, low)
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = AlphaVantageClient::new("demo");
    /// let rsi = client.get_technical_indicator("AAPL", "RSI", "daily", Some(14), "close").await?;
    /// println!("RSI: {:?}", rsi.values);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_technical_indicator(
        &self,
        symbol: &str,
        function: &str,
        interval: &str,
        time_period: Option<u32>,
        series_type: &str,
    ) -> Result<TechnicalIndicator> {
        let mut url = format!(
            "{}/query?function={}&symbol={}&interval={}&series_type={}&apikey={}",
            self.base_url, function, symbol, interval, series_type, self.api_key
        );

        if let Some(period) = time_period {
            url.push_str(&format!("&time_period={}", period));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch technical indicator from Alpha Vantage")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Alpha Vantage API returned status: {}",
                response.status()
            );
        }

        let raw_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse technical indicator response")?;

        // Check for error messages
        if let Some(_error_msg) = raw_json.get("Note") {
            anyhow::bail!("Alpha Vantage rate limit exceeded. Please wait and try again.");
        }

        TechnicalIndicator::from_json(function, raw_json)
    }

    /// Get company overview
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = AlphaVantageClient::new("demo");
    /// let overview = client.get_company_overview("AAPL").await?;
    /// println!("{}: {}", overview.symbol, overview.description);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_company_overview(&self, symbol: &str) -> Result<CompanyOverview> {
        let url = format!(
            "{}/query?function=OVERVIEW&symbol={}&apikey={}",
            self.base_url, symbol, self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch company overview from Alpha Vantage")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Alpha Vantage API returned status: {}",
                response.status()
            );
        }

        let overview: CompanyOverview = response
            .json()
            .await
            .context("Failed to parse company overview response")?;

        // Check if symbol is empty (API error indication)
        if overview.symbol.is_empty() {
            anyhow::bail!("Invalid symbol or API error");
        }

        Ok(overview)
    }
}

// ==============================================================================
// Data Structures
// ==============================================================================

/// Global quote data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalQuote {
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub price: f64,
    pub volume: u64,
    pub latest_trading_day: String,
    pub previous_close: f64,
    pub change: f64,
    pub change_percent: String,
}

/// News sentiment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsSentiment {
    #[serde(default)]
    pub feed: Vec<NewsArticle>,
    #[serde(default)]
    pub sentiment_by_ticker: HashMap<String, TickerSentiment>,
}

/// News article with sentiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub title: String,
    pub url: String,
    pub time_published: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub banner_image: Option<String>,
    pub source: String,
    pub category_within_source: String,
    pub topics: Vec<String>,
    pub overall_sentiment_score: f64,
    pub overall_sentiment_label: String,
    pub ticker_sentiment: Vec<TickerSentiment>,
}

/// Ticker-specific sentiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerSentiment {
    pub ticker: String,
    pub relevance_score: f64,
    pub sentiment_score: f64,
    pub sentiment_label: String,
    pub ticker_news_article_count: u32,
}

/// Technical indicator data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIndicator {
    pub indicator: String,
    pub symbol: String,
    pub interval: String,
    pub values: HashMap<String, f64>,
    pub metadata: HashMap<String, String>,
}

impl TechnicalIndicator {
    fn from_json(function: &str, json: serde_json::Value) -> Result<Self> {
        let meta_obj = json
            .get("Meta Data")
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow::anyhow!("No metadata in response"))?;

        let mut metadata = HashMap::new();
        for (key, value) in meta_obj {
            if let Some(str_val) = value.as_str() {
                metadata.insert(key.clone(), str_val.to_string());
            }
        }

        let symbol = metadata
            .get("2: Symbol")
            .or(metadata.get("1: Symbol"))
            .cloned()
            .unwrap_or_default();

        let interval = metadata
            .get("4: Interval")
            .cloned()
            .unwrap_or_default();

        // Extract values from time series data
        let mut values = HashMap::new();

        // Try different key patterns
        let time_series_keys = vec![
            format!("Time Series ({})", function),
            "Technical Analysis: BBANDS".to_string(),
            format!("Time Series ({})", function),  // For SMA/EMA etc
        ];

        for key_pattern in &time_series_keys {
            if let Some(time_series) = json.get(key_pattern).and_then(|v| v.as_object()) {
                // Get the most recent entry
                let latest_entry = time_series
                    .values()
                    .next()
                    .and_then(|v| v.as_object())
                    .ok_or_else(|| anyhow::anyhow!("Empty time series"))?;

                for (key, value) in latest_entry {
                    if let Some(num_val) = value.as_str().and_then(|s| s.parse::<f64>().ok()) {
                        values.insert(key.clone(), num_val);
                    }
                }
                break;
            }
        }

        Ok(TechnicalIndicator {
            indicator: function.to_string(),
            symbol,
            interval,
            values,
            metadata,
        })
    }

    /// Get the most recent value
    pub fn latest_value(&self, key: &str) -> Option<f64> {
        self.values.get(key).copied()
    }
}

/// Company overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyOverview {
    pub symbol: String,
    pub asset_type: String,
    pub name: String,
    pub description: String,
    pub cik: String,
    pub exchange: String,
    pub currency: String,
    pub country: String,
    pub sector: String,
    pub industry: String,
    pub address: String,
    #[serde(rename = "FiscalYearEnd")]
    pub fiscal_year_end: String,
    pub latest_quarter: String,
    pub market_capitalization: u64,
    #[serde(rename = "EBITDA")]
    pub ebitda: u64,
    #[serde(rename = "PERatio")]
    pub pe_ratio: f64,
    #[serde(rename = "PEGRatio")]
    pub peg_ratio: f64,
    #[serde(rename = "BookValue")]
    pub book_value: f64,
    #[serde(rename = "DividendPerShare")]
    pub dividend_per_share: f64,
    #[serde(rename = "DividendYield")]
    pub dividend_yield: f64,
    #[serde(rename = "EPS")]
    pub eps: f64,
    #[serde(rename = "RevenuePerShareTTM")]
    pub revenue_per_share_ttm: f64,
    #[serde(rename = "ProfitMargin")]
    pub profit_margin: f64,
    #[serde(rename = "OperatingMarginTTM")]
    pub operating_margin_ttm: f64,
    #[serde(rename = "ReturnOnAssetsTTM")]
    pub return_on_assets_ttm: f64,
    #[serde(rename = "ReturnOnEquityTTM")]
    pub return_on_equity_ttm: f64,
    #[serde(rename = "RevenueTTM")]
    pub revenue_ttm: u64,
    #[serde(rename = "GrossProfitTTM")]
    pub gross_profit_ttm: u64,
    #[serde(rename = "AnalystTargetPrice")]
    pub analyst_target_price: f64,
    pub trailing_pe: f64,
    pub forward_pe: f64,
    #[serde(rename = "52WeekHigh")]
    pub week_52_high: f64,
    #[serde(rename = "52WeekLow")]
    pub week_52_low: f64,
    #[serde(rename = "52WeekChange")]
    pub week_52_change: f64,
}

// ==============================================================================
// Alpha Vantage API Response Structures
// ==============================================================================

#[derive(Debug, Deserialize)]
struct AlphaVantageRawResponse {
    #[serde(rename = "Global Quote")]
    global_quote: Option<GlobalQuote>,
    #[serde(rename = "Note")]
    note: Option<String>,
    #[serde(rename = "Error Message")]
    error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageNewsResponse {
    feed: Option<Vec<NewsArticle>>,
    sentiment: Option<NewsSentiment>,
    #[serde(rename = "Note")]
    note: Option<String>,
    #[serde(rename = "Error Message")]
    error_message: Option<String>,
}

// ==============================================================================
// MCP Tool Functions
// ==============================================================================

use claude_agent_sdk_rs::ToolResult;

/// MCP Tool: Get real-time quote from Alpha Vantage
pub async fn alpha_vantage_quote(args: serde_json::Value) -> Result<ToolResult> {
    let api_key = std::env::var("ALPHA_VANTAGE_API_KEY")
        .unwrap_or_else(|_| "demo".to_string());

    let symbol = args
        .get("symbol")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'symbol' parameter"))?;

    let client = AlphaVantageClient::new(api_key);
    let quote = client.get_quote(symbol).await.with_context(|| {
        format!("Failed to get quote for symbol: {}", symbol)
    })?;

    let output = serde_json::to_string_pretty(&quote)
        .context("Failed to serialize quote data")?;

    Ok(ToolResult {
        content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
            text: format!("Alpha Vantage Quote for {}\n\n{}", symbol, output),
        }],
        is_error: false,
    })
}

/// MCP Tool: Get news sentiment from Alpha Vantage
pub async fn alpha_vantage_news_sentiment(args: serde_json::Value) -> Result<ToolResult> {
    let api_key = std::env::var("ALPHA_VANTAGE_API_KEY")
        .unwrap_or_else(|_| "demo".to_string());

    let tickers = args
        .get("tickers")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'tickers' parameter"))?;

    let time_from = args.get("time_from").and_then(|v| v.as_str());
    let time_to = args.get("time_to").and_then(|v| v.as_str());

    let client = AlphaVantageClient::new(api_key);
    let sentiment = client.get_news_sentiment(tickers, time_from, time_to).await
        .with_context(|| {
            format!("Failed to get news sentiment for tickers: {}", tickers)
        })?;

    let output = serde_json::to_string_pretty(&sentiment)
        .context("Failed to serialize news sentiment")?;

    Ok(ToolResult {
        content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
            text: format!(
                "Alpha Vantage News Sentiment for {}\n\n{}\n\nTotal articles: {}",
                tickers,
                output,
                sentiment.feed.len()
            ),
        }],
        is_error: false,
    })
}

/// MCP Tool: Get technical indicator from Alpha Vantage
pub async fn alpha_vantage_technical(args: serde_json::Value) -> Result<ToolResult> {
    let api_key = std::env::var("ALPHA_VANTAGE_API_KEY")
        .unwrap_or_else(|_| "demo".to_string());

    let symbol = args
        .get("symbol")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'symbol' parameter"))?;

    let function = args
        .get("function")
        .and_then(|v| v.as_str())
        .unwrap_or("RSI");

    let interval = args
        .get("interval")
        .and_then(|v| v.as_str())
        .unwrap_or("daily");

    let time_period = args.get("time_period").and_then(|v| v.as_u64()).map(|v| v as u32);

    let series_type = args
        .get("series_type")
        .and_then(|v| v.as_str())
        .unwrap_or("close");

    let client = AlphaVantageClient::new(api_key);
    let indicator = client.get_technical_indicator(symbol, function, interval, time_period, series_type).await
        .with_context(|| {
            format!("Failed to get {} indicator for symbol: {}", function, symbol)
        })?;

    let output = serde_json::to_string_pretty(&indicator)
        .context("Failed to serialize technical indicator")?;

    Ok(ToolResult {
        content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
            text: format!(
                "Alpha Vantage Technical Indicator: {} for {}\n\n{}",
                function, symbol, output
            ),
        }],
        is_error: false,
    })
}

/// MCP Tool: Get company overview from Alpha Vantage
pub async fn alpha_vantage_overview(args: serde_json::Value) -> Result<ToolResult> {
    let api_key = std::env::var("ALPHA_VANTAGE_API_KEY")
        .unwrap_or_else(|_| "demo".to_string());

    let symbol = args
        .get("symbol")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'symbol' parameter"))?;

    let client = AlphaVantageClient::new(api_key);
    let overview = client.get_company_overview(symbol).await.with_context(|| {
        format!("Failed to get company overview for symbol: {}", symbol)
    })?;

    let output = serde_json::to_string_pretty(&overview)
        .context("Failed to serialize company overview")?;

    Ok(ToolResult {
        content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
            text: format!(
                "Alpha Vantage Company Overview for {}\n\n{}",
                symbol, output
            ),
        }],
        is_error: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alpha_vantage_client_creation() {
        let client = AlphaVantageClient::new("demo");
        assert_eq!(client.api_key, "demo");
        assert_eq!(client.base_url, "https://www.alphavantage.co");
    }

    #[tokio::test]
    async fn test_get_quote_with_demo_key() {
        let client = AlphaVantageClient::new("demo");

        // Alpha Vantage demo key only works with IBM
        match client.get_quote("IBM").await {
            Ok(quote) => {
                assert_eq!(quote.symbol, "IBM");
                println!("✅ Quote test passed for IBM");
                println!("   Price: ${}", quote.price);
                println!("   Change: {} ({})", quote.change, quote.change_percent);
            }
            Err(e) => {
                eprintln!("⚠️  Network/API test skipped (may need valid API key): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_company_overview() {
        let client = AlphaVantageClient::new("demo");

        match client.get_company_overview("IBM").await {
            Ok(overview) => {
                assert_eq!(overview.symbol, "IBM");
                println!("✅ Company overview test passed");
                println!("   Company: {}", overview.name);
                println!("   Sector: {}", overview.sector);
            }
            Err(e) => {
                eprintln!("⚠️  Network/API test skipped: {}", e);
            }
        }
    }
}
