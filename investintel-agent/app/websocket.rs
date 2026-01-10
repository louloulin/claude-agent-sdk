// websocket.rs - Real-time WebSocket market data streaming
use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

/// WebSocket message types for market data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMarketMessage {
    /// Real-time price update
    PriceUpdate {
        ticker: String,
        price: f64,
        change: f64,
        change_percent: f64,
        volume: u64,
        timestamp: i64,
    },
    /// Trade execution
    Trade {
        ticker: String,
        price: f64,
        size: u64,
        timestamp: i64,
    },
    /// Quote update (bid/ask)
    Quote {
        ticker: String,
        bid_price: f64,
        ask_price: f64,
        bid_size: u64,
        ask_size: u64,
        timestamp: i64,
    },
    /// Aggregated volume
    Volume {
        ticker: String,
        volume: u64,
        vwap: f64,
        timestamp: i64,
    },
    /// Heartbeat
    Heartbeat {
        timestamp: i64,
    },
}

/// WebSocket configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// WebSocket URL
    pub url: String,
    /// Subscribed tickers
    pub tickers: Vec<String>,
    /// Reconnection interval
    pub reconnect_interval: std::time::Duration,
    /// Message type subscription
    pub message_types: Vec<String>,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            url: "wss://stream.finance.yahoo.com/".to_string(),
            tickers: vec![],
            reconnect_interval: std::time::Duration::from_secs(5),
            message_types: vec!["quote".to_string(), "trade".to_string()],
        }
    }
}

/// WebSocket client for real-time market data
pub struct MarketDataWebSocket {
    config: WebSocketConfig,
    tx: broadcast::Sender<WsMarketMessage>,
    rx: broadcast::Receiver<WsMarketMessage>,
    connected: Arc<RwLock<bool>>,
}

impl MarketDataWebSocket {
    /// Create a new WebSocket client
    pub fn new(config: WebSocketConfig) -> Self {
        let (tx, rx) = broadcast::channel(1000);

        Self {
            config,
            tx,
            rx,
            connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Connect to WebSocket server
    pub async fn connect(&self) -> Result<()> {
        let url = Url::parse(&self.config.url)?;
        let (ws_stream, _) = connect_async(url).await?;

        *self.connected.write().await = true;

        Ok(())
    }

    /// Subscribe to ticker updates
    pub async fn subscribe(&self, tickers: Vec<String>) -> Result<()> {
        let subscription_msg = serde_json::json!({
            "type": "subscribe",
            "tickers": tickers,
            "message_types": self.config.message_types,
        });

        // Send subscription message
        // Note: This would be sent through the WebSocket connection
        // Implementation depends on the specific WebSocket API

        Ok(())
    }

    /// Unsubscribe from ticker updates
    pub async fn unsubscribe(&self, tickers: Vec<String>) -> Result<()> {
        let unsubscription_msg = serde_json::json!({
            "type": "unsubscribe",
            "tickers": tickers,
        });

        // Send unsubscription message

        Ok(())
    }

    /// Get message receiver
    pub fn receiver(&self) -> broadcast::Receiver<WsMarketMessage> {
        self.tx.subscribe()
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Start message processing loop
    pub async fn run(&self) -> Result<()> {
        // Message processing loop
        // In a real implementation, this would:
        // 1. Connect to WebSocket
        // 2. Receive messages
        // 3. Parse into WsMarketMessage
        // 4. Broadcast to subscribers
        // 5. Handle reconnection on disconnect

        Ok(())
    }
}

/// Simulated WebSocket client for testing (when real WebSocket is not available)
pub struct SimulatedWebSocket {
    config: WebSocketConfig,
    tx: broadcast::Sender<WsMarketMessage>,
    rx: broadcast::Receiver<WsMarketMessage>,
    running: Arc<RwLock<bool>>,
}

impl SimulatedWebSocket {
    /// Create a new simulated WebSocket client
    pub fn new(config: WebSocketConfig) -> Self {
        let (tx, rx) = broadcast::channel(1000);

        Self {
            config,
            tx,
            rx,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Get message receiver
    pub fn receiver(&self) -> broadcast::Receiver<WsMarketMessage> {
        self.tx.subscribe()
    }

    /// Start generating simulated data
    pub async fn start(&self) -> Result<()> {
        *self.running.write().await = true;

        let tickers = self.config.tickers.clone();
        let tx = self.tx.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

            while *running.read().await {
                interval.tick().await;

                for ticker in &tickers {
                    // Simulate price movements
                    let base_price = match ticker.as_str() {
                        "AAPL" => 175.0,
                        "MSFT" => 380.0,
                        "GOOGL" => 140.0,
                        "TSLA" => 240.0,
                        _ => 100.0,
                    };

                    let random_change = (rand::random::<f64>() - 0.5) * 2.0; // -1 to +1
                    let price = base_price + random_change;
                    let change = price - base_price;
                    let change_percent = (change / base_price) * 100.0;

                    let msg = WsMarketMessage::PriceUpdate {
                        ticker: ticker.clone(),
                        price,
                        change,
                        change_percent,
                        volume: 1000000 + (rand::random::<u64>() % 5000000),
                        timestamp: chrono::Utc::now().timestamp(),
                    };

                    let _ = tx.send(msg);
                }

                // Send heartbeat every 10 ticks
                if rand::random::<u8>() % 10 == 0 {
                    let heartbeat = WsMarketMessage::Heartbeat {
                        timestamp: chrono::Utc::now().timestamp(),
                    };
                    let _ = tx.send(heartbeat);
                }
            }
        });

        Ok(())
    }

    /// Stop generating simulated data
    pub async fn stop(&self) {
        *self.running.write().await = false;
    }

    /// Subscribe to ticker updates
    pub async fn subscribe(&self, tickers: Vec<String>) -> Result<()> {
        self.config.tickers = tickers;
        Ok(())
    }
}

/// Real-time market aggregator that combines multiple data sources
pub struct RealTimeMarketAggregator {
    websocket: Option<SimulatedWebSocket>,
    tickers: Vec<String>,
    latest_prices: Arc<RwLock<std::collections::HashMap<String, f64>>>,
}

impl RealTimeMarketAggregator {
    /// Create a new aggregator
    pub fn new() -> Self {
        Self {
            websocket: None,
            tickers: vec![],
            latest_prices: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Initialize with WebSocket (simulated for now)
    pub async fn initialize(&mut self, tickers: Vec<String>) -> Result<()> {
        self.tickers = tickers.clone();

        let config = WebSocketConfig {
            tickers,
            ..Default::default()
        };

        let ws = SimulatedWebSocket::new(config);
        ws.start().await?;

        self.websocket = Some(ws);

        Ok(())
    }

    /// Get latest price for a ticker
    pub async fn get_latest_price(&self, ticker: &str) -> Option<f64> {
        self.latest_prices.read().await.get(ticker).copied()
    }

    /// Get all latest prices
    pub async fn get_all_prices(&self) -> std::collections::HashMap<String, f64> {
        self.latest_prices.read().await.clone()
    }

    /// Start processing messages
    pub async fn run(&mut self) -> Result<()> {
        if let Some(ws) = &self.websocket {
            let mut rx = ws.receiver();
            let prices = self.latest_prices.clone();

            tokio::spawn(async move {
                while let Ok(msg) = rx.recv().await {
                    match msg {
                        WsMarketMessage::PriceUpdate { ticker, price, .. } => {
                            prices.write().await.insert(ticker, price);
                        }
                        WsMarketMessage::Heartbeat { .. } => {
                            // Heartbeat received
                        }
                        _ => {}
                    }
                }
            });
        }

        Ok(())
    }

    /// Stop the aggregator
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(ws) = &self.websocket {
            ws.stop().await;
        }
        Ok(())
    }
}

impl Default for RealTimeMarketAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simulated_websocket() {
        let config = WebSocketConfig {
            tickers: vec!["AAPL".to_string(), "MSFT".to_string()],
            ..Default::default()
        };

        let ws = SimulatedWebSocket::new(config);
        ws.start().await.unwrap();

        let mut rx = ws.receiver();

        // Receive a few messages
        for _ in 0..5 {
            let msg = rx.recv().await.unwrap();
            match msg {
                WsMarketMessage::PriceUpdate { ticker, price, .. } => {
                    println!("{}: ${}", ticker, price);
                    assert!(price > 0.0);
                }
                _ => {}
            }
        }

        ws.stop().await;
    }

    #[tokio::test]
    async fn test_market_aggregator() {
        let mut aggregator = RealTimeMarketAggregator::new();

        aggregator.initialize(vec!["AAPL".to_string(), "MSFT".to_string()]).await.unwrap();
        aggregator.run().await.unwrap();

        // Wait for some data
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let prices = aggregator.get_all_prices().await;
        println!("Prices: {:?}", prices);

        aggregator.stop().await.unwrap();
    }
}
