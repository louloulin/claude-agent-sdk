//! Market Data Access Layer
//!
//! Provides unified access to multiple financial data sources including:
//! - Yahoo Finance API ✅
//! - Alpha Vantage API ✅
//! - WebSocket real-time data streams ✅
//! - Multi-source data fusion ✅

pub mod yahoo;
pub mod alpha_vantage;
pub mod websocket;
pub mod websocket_enhanced;
pub mod fusion;
pub mod quality;
// pub mod storage;  // TODO: Phase 1.5

pub use yahoo::{YahooFinanceClient, QuoteData, OHLCV, yahoo_finance_quote, yahoo_finance_historical, yahoo_finance_search};
pub use alpha_vantage::{
    AlphaVantageClient, GlobalQuote, NewsSentiment, TechnicalIndicator, CompanyOverview,
    alpha_vantage_quote, alpha_vantage_news_sentiment, alpha_vantage_technical, alpha_vantage_overview
};
pub use websocket::{MarketDataStream, MarketTick, AggregatedTick, websocket_start_polygon, websocket_subscribe_ticks, websocket_stats};
pub use websocket_enhanced::{
    EnhancedMarketDataStream, EnhancedMarketTick, DataSource, PriceAlert, AlertType,
    AnomalyDetection, AnomalyType, AnomalySeverity, DataQualityMetrics,
    subscribe_realtime_ticker, set_price_alert
};
pub use quality::{
    DataQualityValidator, ValidationResult, ValidationIssue, ValidationReport,
    AnomalyInfo, IssueSeverity, IssueCategory
};
pub use fusion::{
    DataFusionEngine, DataSource as FusionDataSource, DataPriority, YahooDataSource, AlphaVantageDataSource, UnifiedQuote,
    fusion_initialize, fusion_get_quote, fusion_stats, fusion_clear_cache
};
// pub use storage::{DataStorage, QueryBuilder};
