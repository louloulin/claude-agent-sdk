//! WebSocket Real-time Market Data Stream
//!
//! Provides real-time market data using WebSocket connections
//! Supports Polygon.io (stocks, crypto, forex)

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::time::{Duration, SystemTime};

/// Market tick data from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTick {
    pub symbol: String,
    pub price: f64,
    pub size: u64,
    pub timestamp: DateTime<Utc>,
    pub exchange: Option<String>,
    pub conditions: Vec<String>,
}

/// Aggregated tick data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedTick {
    pub symbol: String,
    pub bid_price: Option<f64>,
    pub ask_price: Option<f64>,
    pub bid_size: Option<u64>,
    pub ask_size: Option<u64>,
    pub last_price: f64,
    pub volume: u64,
    pub timestamp: DateTime<Utc>,
}

/// WebSocket market data stream
#[derive(Clone)]
pub struct MarketDataStream {
    tx: broadcast::Sender<MarketTick>,
    agg_tx: broadcast::Sender<AggregatedTick>,
    subscribers: Arc<Mutex<HashMap<String, Vec<broadcast::Sender<MarketTick>>>>>,
}

impl MarketDataStream {
    /// Create a new market data stream
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        let (agg_tx, _) = broadcast::channel(1000);

        Self {
            tx,
            agg_tx,
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Subscribe to all market ticks
    pub fn subscribe_all(&self) -> broadcast::Receiver<MarketTick> {
        self.tx.subscribe()
    }

    /// Subscribe to aggregated ticks
    pub fn subscribe_aggregated(&self) -> broadcast::Receiver<AggregatedTick> {
        self.agg_tx.subscribe()
    }

    /// Subscribe to specific symbol
    pub async fn subscribe_symbol(&self, symbol: String) -> broadcast::Receiver<MarketTick> {
        let (tx, rx) = broadcast::channel(100);
        let mut subscribers = self.subscribers.lock().await;
        subscribers.entry(symbol).or_insert_with(Vec::new).push(tx);
        rx
    }

    /// Send a tick (for testing purposes)
    pub fn send_tick_for_testing(&self, tick: MarketTick) {
        let _ = self.tx.send(tick);
    }

    /// Send an aggregated tick (for testing purposes)
    pub fn send_aggregated_tick_for_testing(&self, tick: AggregatedTick) {
        let _ = self.agg_tx.send(tick);
    }

    /// Connect to Polygon.io WebSocket (free tier available)
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// let stream = MarketDataStream::new();
    /// stream.connect_polygon("YOUR_API_KEY").await?;
    ///
    /// // Subscribe to ticks
    /// let mut rx = stream.subscribe_all();
    /// while let Ok(tick) = rx.recv().await {
    ///     println!("{}: ${}", tick.symbol, tick.price);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_polygon(&self, api_key: &str) -> Result<()> {
        let ws_url = "wss://stream.polygon.io/stocks";
        let (ws_stream, _) = connect_async(ws_url)
            .await
            .context("Failed to connect to Polygon.io WebSocket")?;

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Authentication
        let auth_msg = serde_json::json!({
            "action": "auth",
            "params": api_key
        });
        ws_sender
            .send(Message::Text(auth_msg.to_string()))
            .await
            .context("Failed to send auth message")?;

        tokio::time::sleep(Duration::from_millis(500)).await;

        // Subscribe to trades by default (can be customized)
        let subscribe_msg = serde_json::json!({
            "action": "subscribe",
            "params": "T.*"  // All trades
        });
        ws_sender
            .send(Message::Text(subscribe_msg.to_string()))
            .await
            .context("Failed to send subscribe message")?;

        // Receive loop
        let tx = self.tx.clone();
        let agg_tx = self.agg_tx.clone();

        tokio::spawn(async move {
            let mut last_prices: Arc<RwLock<HashMap<String, f64>>> = Arc::new(RwLock::new(HashMap::new()));
            let mut volumes: Arc<RwLock<HashMap<String, u64>>> = Arc::new(RwLock::new(HashMap::new()));

            while let Some(msg_result) = ws_receiver.next().await {
                match msg_result {
                    Ok(Message::Text(data)) => {
                        if let Ok(polygon_msg) = serde_json::from_str::<PolygonMessage>(&data) {
                            match polygon_msg {
                                PolygonMessage::Trade(trade) => {
                                    let tick = MarketTick {
                                        symbol: trade.sym,
                                        price: trade.p,
                                        size: trade.q,
                                        timestamp: DateTime::from_timestamp(trade.t / 1000, 0).unwrap_or_default(),
                                        exchange: Some(trade.e),
                                        conditions: trade.c.iter().map(|c| c.to_string()).collect(),
                                    };

                                    // Update volume
                                    let mut volumes = volumes.write().await;
                                    *volumes.entry(tick.symbol.clone()).or_insert(0) += tick.size;
                                    let volume = *volumes.get(&tick.symbol).unwrap_or(&0);

                                    // Send tick
                                    let _ = tx.send(tick.clone());

                                    // Create aggregated tick
                                    let mut last_prices = last_prices.write().await;
                                    last_prices.insert(tick.symbol.clone(), tick.price);

                                    let agg_tick = AggregatedTick {
                                        symbol: tick.symbol.clone(),
                                        bid_price: None,
                                        ask_price: None,
                                        bid_size: None,
                                        ask_size: None,
                                        last_price: tick.price,
                                        volume,
                                        timestamp: tick.timestamp,
                                    };

                                    let _ = agg_tx.send(agg_tick);
                                }
                                PolygonMessage::Agg(agg) => {
                                    // Handle aggregated data
                                    let agg_tick = AggregatedTick {
                                        symbol: agg.sym,
                                        bid_price: Some(agg.b),
                                        ask_price: Some(agg.a),
                                        bid_size: Some(agg.bz),
                                        ask_size: Some(agg.az),
                                        last_price: (agg.b + agg.a) / 2.0,
                                        volume: 0,
                                        timestamp: DateTime::from_timestamp(agg.t / 1000, 0).unwrap_or_default(),
                                    };

                                    let _ = agg_tx.send(agg_tick);
                                }
                                PolygonMessage::Status(status) => {
                                    eprintln!("Polygon.io status: {:?}", status);
                                }
                                PolygonMessage::Error(error) => {
                                    eprintln!("Polygon.io error: {:?}", error);
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        eprintln!("Polygon.io WebSocket connection closed");
                        break;
                    }
                    Err(e) => {
                        eprintln!("Polygon.io WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Subscribe to specific symbols
    pub async fn subscribe_symbols(&self, symbols: Vec<String>) -> Result<()> {
        let params = symbols
            .iter()
            .map(|s| format!("T.{}", s))
            .collect::<Vec<_>>()
            .join(",");

        // This would need to be sent through the WebSocket connection
        // For now, just log it
        println!("Subscribing to symbols: {}", params);

        Ok(())
    }

    /// Get current subscribers count
    pub async fn subscribers_count(&self) -> usize {
        let subscribers = self.subscribers.lock().await;
        subscribers.len()
    }
}

impl Default for MarketDataStream {
    fn default() -> Self {
        Self::new()
    }
}

// ==============================================================================
// Polygon.io WebSocket Message Types
// ==============================================================================

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PolygonMessage {
    Trade(PolygonTrade),
    Agg(PolygonAgg),
    Status(PolygonStatus),
    Error(PolygonError),
}

#[derive(Debug, Deserialize, Clone)]
struct PolygonTrade {
    #[serde(rename = "sym")]
    pub sym: String,
    #[serde(rename = "p")]
    pub p: f64,
    #[serde(rename = "q")]
    pub q: u64,
    #[serde(rename = "t")]
    pub t: i64,
    #[serde(rename = "e")]
    pub e: String,
    #[serde(rename = "c")]
    pub c: Vec<i32>,
}

#[derive(Debug, Deserialize, Clone)]
struct PolygonAgg {
    #[serde(rename = "sym")]
    pub sym: String,
    #[serde(rename = "b")]
    pub b: f64,
    #[serde(rename = "a")]
    pub a: f64,
    #[serde(rename = "bz")]
    pub bz: u64,
    #[serde(rename = "az")]
    pub az: u64,
    #[serde(rename = "t")]
    pub t: i64,
}

#[derive(Debug, Deserialize)]
struct PolygonStatus {
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PolygonError {
    pub error: String,
    pub message: Option<String>,
}

// ==============================================================================
// MCP Tool Functions
// ==============================================================================

use claude_agent_sdk_rs::ToolResult;

/// Global stream instance (lazy initialized)
static GLOBAL_STREAM: std::sync::OnceLock<std::sync::Arc<Mutex<Option<MarketDataStream>>>> =
    std::sync::OnceLock::new();

/// Initialize global stream
fn get_global_stream() -> std::sync::Arc<Mutex<Option<MarketDataStream>>> {
    GLOBAL_STREAM.get_or_init(|| std::sync::Arc::new(Mutex::new(None))).clone()
}

/// MCP Tool: Start WebSocket connection to Polygon.io
pub async fn websocket_start_polygon(args: serde_json::Value) -> Result<ToolResult> {
    let api_key = std::env::var("POLYGON_API_KEY")
        .unwrap_or_else(|_| args.get("api_key")
            .and_then(|v| v.as_str())
            .unwrap_or("demo")
            .to_string());

    let stream = MarketDataStream::new();

    match stream.connect_polygon(&api_key).await {
        Ok(_) => {
            // Store global stream
            let global = get_global_stream();
            let mut guard = global.lock().await;
            *guard = Some(stream);

            Ok(ToolResult {
                content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                    text: format!("✅ WebSocket connection to Polygon.io established\n\nAPI Key: {}\n\nSubscribers can now receive real-time market data.", mask_api_key(&api_key)),
                }],
                is_error: false,
            })
        }
        Err(e) => {
            Ok(ToolResult {
                content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                    text: format!("❌ Failed to connect to Polygon.io: {}\n\nMake sure you have a valid API key from https://polygon.io/", e),
                }],
                is_error: true,
            })
        }
    }
}

/// MCP Tool: Subscribe to real-time ticks for a symbol
pub async fn websocket_subscribe_ticks(args: serde_json::Value) -> Result<ToolResult> {
    let symbol = args
        .get("symbol")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'symbol' parameter"))?;

    let global = get_global_stream();
    let guard = global.lock().await;

    match guard.as_ref() {
        Some(stream) => {
            // Subscribe to the symbol
            let mut rx = stream.subscribe_all();

            // Receive for a short time to get some ticks
            let mut ticks = Vec::new();
            let timeout_duration = Duration::from_secs(5);

            let start = SystemTime::now();
            loop {
                match tokio::time::timeout(timeout_duration, rx.recv()).await {
                    Ok(Ok(tick)) => {
                        if tick.symbol == symbol || symbol == "*" {
                            ticks.push(tick);
                            if ticks.len() >= 10 {
                                break;
                            }
                        }
                    }
                    _ => break,
                }

                if start.elapsed().unwrap_or(Duration::ZERO) >= timeout_duration {
                    break;
                }
            }

            if ticks.is_empty() {
                Ok(ToolResult {
                    content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                        text: format!("⏳ No ticks received for {} in the last 5 seconds\n\nNote: Market data may be limited outside trading hours or with free API tier.", symbol),
                    }],
                    is_error: false,
                })
            } else {
                let output = serde_json::to_string_pretty(&ticks)
                    .context("Failed to serialize ticks")?;

                Ok(ToolResult {
                    content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                        text: format!("Real-time ticks for {}\n\n{}\n\nTotal ticks: {}", symbol, output, ticks.len()),
                    }],
                    is_error: false,
                })
            }
        }
        None => {
            Ok(ToolResult {
                content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                    text: "❌ WebSocket not connected. Please call websocket_start_polygon first.".to_string(),
                }],
                is_error: true,
            })
        }
    }
}

/// MCP Tool: Get stream statistics
pub async fn websocket_stats(args: serde_json::Value) -> Result<ToolResult> {
    let global = get_global_stream();
    let guard = global.lock().await;

    match guard.as_ref() {
        Some(stream) => {
            let subscribers_count = stream.subscribers_count().await;

            let stats = serde_json::json!({
                "status": "connected",
                "subscribers": subscribers_count,
                "channels": 2,
                "buffer_size": 1000
            });

            Ok(ToolResult {
                content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                    text: format!("WebSocket Stream Statistics\n\n{}", serde_json::to_string_pretty(&stats)?),
                }],
                is_error: false,
            })
        }
        None => {
            Ok(ToolResult {
                content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                    text: "WebSocket not connected".to_string(),
                }],
                is_error: false,
            })
        }
    }
}

/// Mask API key for logging
fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        "***".to_string()
    } else {
        format!("{}***{}", &key[..4], &key[key.len()-4..])
    }
}

// ==============================================================================
// Tests
// ==============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_market_data_stream_creation() {
        let stream = MarketDataStream::new();
        assert_eq!(stream.subscribers_count().await, 0);
        println!("✅ MarketDataStream created successfully");
    }

    #[tokio::test]
    async fn test_subscribe_all() {
        let stream = MarketDataStream::new();
        let mut rx = stream.subscribe_all();

        // Create a test tick
        let tick = MarketTick {
            symbol: "AAPL".to_string(),
            price: 150.0,
            size: 100,
            timestamp: Utc::now(),
            exchange: Some("NASDAQ".to_string()),
            conditions: vec!["regular".to_string()],
        };

        // Send tick
        let _ = stream.tx.send(tick.clone());

        // Receive tick
        let received = rx.recv().await.unwrap();
        assert_eq!(received.symbol, "AAPL");
        assert_eq!(received.price, 150.0);

        println!("✅ Subscribe all test passed");
    }

    #[tokio::test]
    async fn test_subscribe_aggregated() {
        let stream = MarketDataStream::new();
        let mut rx = stream.subscribe_aggregated();

        // Create an aggregated tick
        let agg_tick = AggregatedTick {
            symbol: "MSFT".to_string(),
            bid_price: Some(300.0),
            ask_price: Some(301.0),
            bid_size: Some(100),
            ask_size: Some(150),
            last_price: 300.5,
            volume: 1000,
            timestamp: Utc::now(),
        };

        // Send tick
        let _ = stream.agg_tx.send(agg_tick);

        // Receive tick
        let received = rx.recv().await.unwrap();
        assert_eq!(received.symbol, "MSFT");
        assert_eq!(received.last_price, 300.5);

        println!("✅ Subscribe aggregated test passed");
    }

    #[tokio::test]
    async fn test_tick_serialization() {
        let tick = MarketTick {
            symbol: "GOOGL".to_string(),
            price: 2500.0,
            size: 50,
            timestamp: Utc::now(),
            exchange: Some("NASDAQ".to_string()),
            conditions: vec!["regular".to_string()],
        };

        let json = serde_json::to_string(&tick).unwrap();
        assert!(json.contains("GOOGL"));
        assert!(json.contains("2500"));

        println!("✅ Tick serialization test passed");
    }

    #[tokio::test]
    async fn test_stream_default() {
        let stream = MarketDataStream::default();
        assert_eq!(stream.subscribers_count().await, 0);
        println!("✅ Default stream test passed");
    }
}
