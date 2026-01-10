// market_data.rs - Real-time market data integration
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Market data client for fetching real-time data
pub struct MarketDataClient {
    client: Client,
    cache: RwLock<HashMap<String, CachedData>>,
    cache_ttl: std::time::Duration,
}

/// Cached market data with expiry
#[derive(Debug, Clone)]
struct CachedData {
    data: MarketData,
    expires_at: DateTime<Utc>,
}

/// Market data for a ticker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub ticker: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub previous_close: f64,
    pub market_cap: Option<u64>,
    pub timestamp: DateTime<Utc>,
}

/// Historical data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataPoint {
    pub date: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub adjusted_close: f64,
}

/// Technical indicators calculated from price data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIndicators {
    pub sma_20: Option<f64>,
    pub sma_50: Option<f64>,
    pub ema_12: Option<f64>,
    pub ema_26: Option<f64>,
    pub rsi: Option<f64>,
    pub macd: Option<MACD>,
    pub bollinger_bands: Option<BollingerBands>,
    pub support_levels: Vec<f64>,
    pub resistance_levels: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACD {
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BollingerBands {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
    pub bandwidth: f64,
}

/// News article for sentiment analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub title: String,
    pub url: String,
    pub published_at: DateTime<Utc>,
    pub source: String,
    pub sentiment_score: Option<f64>,
    pub summary: Option<String>,
}

impl MarketDataClient {
    /// Create a new market data client
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            cache: RwLock::new(HashMap::new()),
            cache_ttl: std::time::Duration::from_secs(60), // 1 minute cache
        }
    }

    /// Create client with custom cache TTL
    pub fn with_cache_ttl(ttl: std::time::Duration) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            cache: RwLock::new(HashMap::new()),
            cache_ttl: ttl,
        }
    }

    /// Get current market data for a ticker (with caching)
    pub async fn get_quote(&self, ticker: &str) -> Result<MarketData> {
        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(ticker) {
                if cached.expires_at > Utc::now() {
                    return Ok(cached.data.clone());
                }
            }
        }

        // Fetch fresh data
        let data = self.fetch_quote(ticker).await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                ticker.to_uppercase(),
                CachedData {
                    data: data.clone(),
                    expires_at: Utc::now() + chrono::Duration::seconds(self.cache_ttl.as_secs() as i64),
                },
            );
        }

        Ok(data)
    }

    /// Fetch quote from Yahoo Finance API
    async fn fetch_quote(&self, ticker: &str) -> Result<MarketData> {
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1m&range=1d",
            ticker
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch data for {}", ticker));
        }

        let json: serde_json::Value = response.json().await?;
        let result = json["chart"]["result"]
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| anyhow!("No data found for {}", ticker))?;

        let meta = &result["meta"];
        let quote = &result["indicators"]["quote"][0];

        let timestamp = meta["regularMarketTime"]
            .as_i64()
            .unwrap_or_else(|| chrono::Utc::now().timestamp());

        let price = quote["close"]
            .as_array()
            .and_then(|arr| arr.last())
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("No price data"))?;

        let previous_close = meta["previousClose"]
            .as_f64()
            .unwrap_or(price);

        let change = price - previous_close;
        let change_percent = (change / previous_close) * 100.0;

        Ok(MarketData {
            ticker: ticker.to_uppercase(),
            price,
            change,
            change_percent,
            volume: quote["volume"]
                .as_array()
                .and_then(|arr| arr.last())
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            open: quote["open"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_f64())
                .unwrap_or(price),
            high: quote["high"]
                .as_array()
                .and_then(|arr| arr.iter().filter_map(|v| v.as_f64()).fold(f64::NAN, f64::max)),
            low: quote["low"]
                .as_array()
                .and_then(|arr| arr.iter().filter_map(|v| v.as_f64()).fold(f64::NAN, f64::min)),
            previous_close,
            market_cap: meta["marketCap"].as_u64(),
            timestamp: DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now()),
        })
    }

    /// Get historical data for a ticker
    pub async fn get_historical_data(
        &self,
        ticker: &str,
        period: &str, // "1mo", "3mo", "6mo", "1y", "2y", "5y", "max"
    ) -> Result<Vec<HistoricalDataPoint>> {
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range={}",
            ticker, period
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch historical data for {}", ticker));
        }

        let json: serde_json::Value = response.json().await?;
        let result = json["chart"]["result"]
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| anyhow!("No historical data found"))?;

        let timestamps = result["timestamp"]
            .as_array()
            .ok_or_else(|| anyhow!("No timestamps"))?;

        let quote = &result["indicators"]["quote"][0];
        let opens = quote["open"].as_array().unwrap_or(&vec![]);
        let highs = quote["high"].as_array().unwrap_or(&vec![]);
        let lows = quote["low"].as_array().unwrap_or(&vec![]);
        let closes = quote["close"].as_array().unwrap_or(&vec![]);
        let volumes = quote["volume"].as_array().unwrap_or(&vec![]);

        let mut data = Vec::new();
        for (i, ts) in timestamps.iter().enumerate() {
            if let Some(timestamp) = ts.as_i64() {
                if let (Some(open), Some(high), Some(low), Some(close), Some(volume)) = (
                    opens.get(i).and_then(|v| v.as_f64()),
                    highs.get(i).and_then(|v| v.as_f64()),
                    lows.get(i).and_then(|v| v.as_f64()),
                    closes.get(i).and_then(|v| v.as_f64()),
                    volumes.get(i).and_then(|v| v.as_u64()),
                ) {
                    data.push(HistoricalDataPoint {
                        date: DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now()),
                        open,
                        high,
                        low,
                        close,
                        volume,
                        adjusted_close: close, // Simplified
                    });
                }
            }
        }

        Ok(data)
    }

    /// Calculate technical indicators from historical data
    pub fn calculate_indicators(&self, data: &[HistoricalDataPoint]) -> Result<TechnicalIndicators> {
        if data.len() < 50 {
            return Err(anyhow!("Not enough data points for indicators"));
        }

        let closes: Vec<f64> = data.iter().map(|d| d.close).collect();

        // Simple Moving Averages
        let sma_20 = self.calculate_sma(&closes, 20);
        let sma_50 = self.calculate_sma(&closes, 50);

        // Exponential Moving Averages
        let ema_12 = self.calculate_ema(&closes, 12);
        let ema_26 = self.calculate_ema(&closes, 26);

        // RSI (14-period)
        let rsi = self.calculate_rsi(&closes, 14);

        // MACD
        let macd = if let (Some(ema_12_val), Some(ema_26_val)) = (ema_12, ema_26) {
            let macd_line = ema_12_val - ema_26_val;
            let macd_history: Vec<f64> = closes
                .windows(26)
                .map(|w| {
                    let e12 = self.calculate_ema_array(&w.to_vec(), 12).unwrap_or(0.0);
                    let e26 = self.calculate_ema_array(&w.to_vec(), 26).unwrap_or(0.0);
                    e12 - e26
                })
                .collect();

            let signal = self.calculate_ema_array(&macd_history, 9);
            signal.map(|s| MACD {
                macd: macd_line,
                signal: s,
                histogram: macd_line - s,
            })
        } else {
            None
        };

        // Bollinger Bands (20-period, 2 standard deviations)
        let bollinger = if let Some(sma_20_val) = sma_20 {
            let last_20: Vec<f64> = closes.iter().rev().take(20).copied().collect();
            let variance = last_20
                .iter()
                .map(|&x| (x - sma_20_val).powi(2))
                .sum::<f64>() / 20.0;
            let std_dev = variance.sqrt();

            Some(BollingerBands {
                upper: sma_20_val + 2.0 * std_dev,
                middle: sma_20_val,
                lower: sma_20_val - 2.0 * std_dev,
                bandwidth: (4.0 * std_dev / sma_20_val) * 100.0,
            })
        } else {
            None
        };

        // Support and Resistance levels (pivot points)
        let last = &data[data.len() - 1];
        let pivot = (last.high + last.low + last.close) / 3.0;
        let support_levels = vec![
            2.0 * pivot - last.high,  // S1
            pivot - (last.high - last.low), // S2
        ];
        let resistance_levels = vec![
            2.0 * pivot - last.low,   // R1
            pivot + (last.high - last.low), // R2
        ];

        Ok(TechnicalIndicators {
            sma_20,
            sma_50,
            ema_12,
            ema_26,
            rsi,
            macd,
            bollinger_bands: bollinger,
            support_levels,
            resistance_levels,
        })
    }

    /// Calculate Simple Moving Average
    fn calculate_sma(&self, data: &[f64], period: usize) -> Option<f64> {
        if data.len() < period {
            return None;
        }
        let sum: f64 = data.iter().rev().take(period).sum();
        Some(sum / period as f64)
    }

    /// Calculate Exponential Moving Average
    fn calculate_ema(&self, data: &[f64], period: usize) -> Option<f64> {
        if data.len() < period {
            return None;
        }
        self.calculate_ema_array(data, period)
    }

    fn calculate_ema_array(&self, data: &[f64], period: usize) -> Option<f64> {
        if data.len() < period {
            return None;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = data[..period].iter().sum::<f64>() / period as f64;

        for &price in &data[period..] {
            ema = (price - ema) * multiplier + ema;
        }

        Some(ema)
    }

    /// Calculate RSI
    fn calculate_rsi(&self, data: &[f64], period: usize) -> Option<f64> {
        if data.len() < period + 1 {
            return None;
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..=period {
            let change = data[data.len() - i] - data[data.len() - i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        let avg_gain: f64 = gains.iter().sum::<f64>() / period as f64;
        let avg_loss: f64 = losses.iter().sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Some(rsi)
    }

    /// Get news for a ticker
    pub async fn get_news(&self, ticker: &str, limit: usize) -> Result<Vec<NewsArticle>> {
        let url = format!(
            "https://newsapi.org/v2/everything?q={}&apiKey=YOUR_API_KEY",
            ticker
        );

        // For now, return a mock implementation
        // In production, integrate with actual news API
        Ok(vec![NewsArticle {
            title: format!("Latest updates for {}", ticker),
            url: format!("https://example.com/{}", ticker),
            published_at: Utc::now(),
            source: "Mock Source".to_string(),
            sentiment_score: Some(0.0),
            summary: Some("This is a mock news article".to_string()),
        }])
    }

    /// Batch get quotes for multiple tickers
    pub async fn get_batch_quotes(&self, tickers: &[String]) -> Result<HashMap<String, MarketData>> {
        let mut results = HashMap::new();

        for ticker in tickers {
            match self.get_quote(ticker).await {
                Ok(data) => {
                    results.insert(ticker.clone(), data);
                }
                Err(e) => {
                    eprintln!("Error fetching {}: {}", ticker, e);
                }
            }
        }

        Ok(results)
    }

    /// Clear the cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

impl Default for MarketDataClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_market_data_client() {
        let client = MarketDataClient::new();

        // Test with a common ticker
        match client.get_quote("AAPL").await {
            Ok(data) => {
                println!("AAPL Price: ${}", data.price);
                println!("Change: {:+.2} ({:+.2}%)", data.change, data.change_percent);
                assert!(data.price > 0.0);
            }
            Err(e) => {
                eprintln!("Error fetching AAPL data: {}", e);
                // This is expected if running in offline mode
            }
        }
    }

    #[tokio::test]
    async fn test_historical_data() {
        let client = MarketDataClient::new();

        match client.get_historical_data("AAPL", "1mo").await {
            Ok(data) => {
                println!("Got {} data points", data.len());
                assert!(!data.is_empty());
            }
            Err(e) => {
                eprintln!("Error fetching historical data: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_technical_indicators() {
        let client = MarketDataClient::new();

        if let Ok(data) = client.get_historical_data("AAPL", "3mo").await {
            match client.calculate_indicators(&data) {
                Ok(indicators) => {
                    println!("SMA 20: {:?}", indicators.sma_20);
                    println!("RSI: {:?}", indicators.rsi);
                    println!("Support levels: {:?}", indicators.support_levels);
                    println!("Resistance levels: {:?}", indicators.resistance_levels);
                }
                Err(e) => {
                    eprintln!("Error calculating indicators: {}", e);
                }
            }
        }
    }
}
