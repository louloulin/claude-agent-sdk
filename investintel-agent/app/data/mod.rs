//! Market Data Access Layer
//!
//! Provides unified access to multiple financial data sources including:
//! - Yahoo Finance API
//! - Alpha Vantage API
//! - WebSocket real-time data streams
//! - Multi-source data fusion

pub mod yahoo;
pub mod alpha_vantage;
pub mod websocket;
pub mod fusion;
pub mod storage;

pub use yahoo::{YahooFinanceClient, QuoteData, OHLCV};
pub use alpha_vantage::{AlphaVantageClient, GlobalQuote, NewsSentiment, TechnicalIndicator};
pub use websocket::{MarketDataStream, MarketTick};
pub use fusion::{DataFusionEngine, DataSource, DataPriority};
pub use storage::{DataStorage, QueryBuilder};
