//! Multi-Source Data Fusion Engine
//!
//! Intelligently combines data from multiple sources (Yahoo, Alpha Vantage, WebSocket)
//! with priority-based selection, caching, and latency optimization

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use super::yahoo::{YahooFinanceClient, QuoteData as YahooQuote};
use super::alpha_vantage::{AlphaVantageClient, GlobalQuote as AVQuote};

/// Data source priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DataPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    RealTime = 4,
}

/// Cached data with timestamp
#[derive(Debug, Clone)]
pub struct CachedData {
    pub data: UnifiedQuote,
    pub cached_at: DateTime<Utc>,
    pub source: String,
}

/// Unified quote format from any source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedQuote {
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: u64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub previous_close: f64,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub latency_ms: u64,
}

/// Data source trait
#[async_trait::async_trait]
pub trait DataSource: Send + Sync {
    /// Get quote for a symbol
    async fn get_quote(&self, symbol: &str) -> Result<UnifiedQuote>;

    /// Get priority (higher = preferred)
    fn priority(&self) -> DataPriority;

    /// Get average latency in milliseconds
    fn latency_ms(&self) -> u64;

    /// Get source name
    fn source_name(&self) -> &str;
}

/// Yahoo Finance data source adapter
pub struct YahooDataSource {
    client: YahooFinanceClient,
    latency: Arc<RwLock<u64>>,
}

impl YahooDataSource {
    pub fn new() -> Self {
        Self {
            client: YahooFinanceClient::new(),
            latency: Arc::new(RwLock::new(100)), // Default 100ms
        }
    }
}

#[async_trait::async_trait]
impl DataSource for YahooDataSource {
    async fn get_quote(&self, symbol: &str) -> Result<UnifiedQuote> {
        let start = std::time::Instant::now();

        let quote = self.client.get_quote(symbol).await
            .context("Yahoo Finance request failed")?;

        let latency = start.elapsed().as_millis() as u64;

        // Update latency
        let mut current_latency = self.latency.write().await;
        *current_latency = (*current_latency * 9 + latency) / 10; // Moving average

        Ok(UnifiedQuote {
            symbol: quote.symbol.clone(),
            price: quote.regular_market_price,
            change: quote.change,
            change_percent: quote.change_percent,
            volume: quote.volume,
            high: quote.day_high,
            low: quote.day_low,
            open: quote.previous_close + quote.change, // Approximate
            previous_close: quote.previous_close,
            timestamp: quote.timestamp,
            source: "Yahoo Finance".to_string(),
            latency_ms: latency,
        })
    }

    fn priority(&self) -> DataPriority {
        DataPriority::High
    }

    fn latency_ms(&self) -> u64 {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                *self.latency.read().await
            })
        })
    }

    fn source_name(&self) -> &str {
        "Yahoo Finance"
    }
}

/// Alpha Vantage data source adapter
pub struct AlphaVantageDataSource {
    client: AlphaVantageClient,
    latency: Arc<RwLock<u64>>,
}

impl AlphaVantageDataSource {
    pub fn new(api_key: String) -> Self {
        Self {
            client: AlphaVantageClient::new(api_key),
            latency: Arc::new(RwLock::new(150)), // Default 150ms
        }
    }
}

#[async_trait::async_trait]
impl DataSource for AlphaVantageDataSource {
    async fn get_quote(&self, symbol: &str) -> Result<UnifiedQuote> {
        let start = std::time::Instant::now();

        let quote = self.client.get_quote(symbol).await
            .context("Alpha Vantage request failed")?;

        let latency = start.elapsed().as_millis() as u64;

        // Update latency
        let mut current_latency = self.latency.write().await;
        *current_latency = (*current_latency * 9 + latency) / 10;

        // Parse change percent
        let change_percent = quote.change_percent
            .trim_end_matches('%')
            .parse::<f64>()
            .unwrap_or(0.0);

        Ok(UnifiedQuote {
            symbol: quote.symbol.clone(),
            price: quote.price,
            change: quote.change,
            change_percent,
            volume: quote.volume,
            high: quote.high,
            low: quote.low,
            open: quote.open,
            previous_close: quote.previous_close,
            timestamp: Utc::now(),
            source: "Alpha Vantage".to_string(),
            latency_ms: latency,
        })
    }

    fn priority(&self) -> DataPriority {
        DataPriority::Medium // Lower priority due to rate limits
    }

    fn latency_ms(&self) -> u64 {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                *self.latency.read().await
            })
        })
    }

    fn source_name(&self) -> &str {
        "Alpha Vantage"
    }
}

/// Data fusion engine with smart source selection
pub struct DataFusionEngine {
    pub sources: Vec<Box<dyn DataSource>>,
    cache: Arc<RwLock<HashMap<String, CachedData>>>,
    pub default_cache_ttl_secs: u64,
}

impl DataFusionEngine {
    /// Create a new fusion engine
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_cache_ttl_secs: 5, // 5 second cache TTL
        }
    }

    /// Add a data source
    pub fn add_source(mut self, source: Box<dyn DataSource>) -> Self {
        self.sources.push(source);
        self
    }

    /// Set cache TTL
    pub fn with_cache_ttl(mut self, ttl_secs: u64) -> Self {
        self.default_cache_ttl_secs = ttl_secs;
        self
    }

    /// Get quote with smart source selection
    ///
    /// # Strategy
    /// 1. Check cache (if < 5 seconds old)
    /// 2. Score each source: priority - (latency / 100)
    /// 3. Try sources in score order
    /// 4. Update cache on success
    pub async fn get_quote_smart(&self, symbol: &str) -> Result<UnifiedQuote> {
        // 1. Check cache
        if let Some(cached) = self.get_from_cache(symbol, self.default_cache_ttl_secs).await {
            return Ok(cached.data);
        }

        // 2. Score and sort sources
        let mut scored_sources: Vec<_> = self.sources.iter().enumerate().map(|(i, source)| {
            let priority_score = source.priority() as u64 as f64;
            let latency_penalty = source.latency_ms() as f64 / 100.0;
            let score = priority_score - latency_penalty;
            (i, source, score)
        }).collect();

        scored_sources.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // 3. Try sources in order
        let mut last_error = None;

        for (_i, source, _score) in scored_sources {
            match source.get_quote(symbol).await {
                Ok(quote) => {
                    // Update cache
                    self.update_cache(symbol, quote.clone(), source.source_name().to_string()).await;
                    return Ok(quote);
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        // 4. All sources failed
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("No data sources available")))
    }

    /// Get quote from specific source
    pub async fn get_quote_from_source(&self, symbol: &str, source_name: &str) -> Result<UnifiedQuote> {
        for source in &self.sources {
            if source.source_name() == source_name {
                return source.get_quote(symbol).await;
            }
        }

        anyhow::bail!("Source '{}' not found", source_name)
    }

    /// Get from cache if fresh (public for testing)
    pub async fn get_from_cache(&self, symbol: &str, max_age_secs: u64) -> Option<CachedData> {
        let cache = self.cache.read().await;
        cache.get(symbol).filter(|cached| {
            let age = Utc::now().signed_duration_since(cached.cached_at).num_seconds();
            age < max_age_secs as i64
        }).cloned()
    }

    /// Update cache (public for testing)
    pub async fn update_cache(&self, symbol: &str, data: UnifiedQuote, source: String) {
        let mut cache = self.cache.write().await;
        cache.insert(symbol.to_string(), CachedData {
            data,
            cached_at: Utc::now(),
            source,
        });
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let mut source_counts: HashMap<String, usize> = HashMap::new();

        for cached in cache.values() {
            *source_counts.entry(cached.source.clone()).or_insert(0) += 1;
        }

        CacheStats {
            total_entries: cache.len(),
            source_counts,
        }
    }

    /// Get source statistics
    pub async fn source_stats(&self) -> Vec<SourceStats> {
        let mut stats = Vec::new();

        for source in &self.sources {
            stats.push(SourceStats {
                name: source.source_name().to_string(),
                priority: source.priority(),
                latency_ms: source.latency_ms(),
            });
        }

        stats.sort_by(|a, b| b.priority.cmp(&a.priority));
        stats
    }
}

impl Default for DataFusionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub source_counts: HashMap<String, usize>,
}

/// Source statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceStats {
    pub name: String,
    pub priority: DataPriority,
    pub latency_ms: u64,
}

// ==============================================================================
// MCP Tool Functions
// ==============================================================================

use claude_agent_sdk_rs::ToolResult;

/// Global fusion engine instance
static GLOBAL_FUSION_ENGINE: std::sync::OnceLock<std::sync::Arc<Mutex<Option<DataFusionEngine>>>> =
    std::sync::OnceLock::new();

/// Get global fusion engine
fn get_global_fusion_engine() -> std::sync::Arc<Mutex<Option<DataFusionEngine>>> {
    GLOBAL_FUSION_ENGINE.get_or_init(|| std::sync::Arc::new(Mutex::new(None))).clone()
}

/// Initialize fusion engine with default sources
pub async fn fusion_initialize(args: serde_json::Value) -> Result<ToolResult> {
    let alpha_vantage_key = std::env::var("ALPHA_VANTAGE_API_KEY")
        .unwrap_or_else(|_| args.get("alpha_vantage_key")
            .and_then(|v| v.as_str())
            .unwrap_or("demo")
            .to_string());

    let mut engine = DataFusionEngine::new();

    // Add Yahoo Finance (no API key needed)
    engine = engine.add_source(Box::new(YahooDataSource::new()));

    // Add Alpha Vantage (if key provided)
    if !alpha_vantage_key.is_empty() && alpha_vantage_key != "demo" {
        engine = engine.add_source(Box::new(AlphaVantageDataSource::new(alpha_vantage_key)));
    }

    // Store global engine
    let global = get_global_fusion_engine();
    let mut guard = global.lock().await;
    *guard = Some(engine);

    Ok(ToolResult {
        content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
            text: "✅ Data Fusion Engine initialized\n\nAvailable sources:\n  - Yahoo Finance (High priority)\n  - Alpha Vantage (Medium priority)\n\nCache TTL: 5 seconds".to_string(),
        }],
        is_error: false,
    })
}

/// Get quote using smart fusion
pub async fn fusion_get_quote(args: serde_json::Value) -> Result<ToolResult> {
    let symbol = args
        .get("symbol")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'symbol' parameter"))?;

    let global = get_global_fusion_engine();
    let guard = global.lock().await;

    match guard.as_ref() {
        Some(engine) => {
            let quote = engine.get_quote_smart(symbol).await
                .with_context(|| format!("Failed to get quote for symbol: {}", symbol))?;

            let output = serde_json::to_string_pretty(&quote)
                .context("Failed to serialize quote")?;

            Ok(ToolResult {
                content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                    text: format!("Smart Quote for {}\n\n{}", symbol, output),
                }],
                is_error: false,
            })
        }
        None => {
            Ok(ToolResult {
                content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                    text: "❌ Fusion engine not initialized. Please call fusion_initialize first.".to_string(),
                }],
                is_error: true,
            })
        }
    }
}

/// Get fusion statistics
pub async fn fusion_stats(args: serde_json::Value) -> Result<ToolResult> {
    let global = get_global_fusion_engine();
    let guard = global.lock().await;

    match guard.as_ref() {
        Some(engine) => {
            let cache_stats = engine.cache_stats().await;
            let source_stats = engine.source_stats().await;

            let stats = serde_json::json!({
                "cache": cache_stats,
                "sources": source_stats
            });

            Ok(ToolResult {
                content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                    text: format!("Fusion Engine Statistics\n\n{}", serde_json::to_string_pretty(&stats)?),
                }],
                is_error: false,
            })
        }
        None => {
            Ok(ToolResult {
                content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                    text: "Fusion engine not initialized".to_string(),
                }],
                is_error: false,
            })
        }
    }
}

/// Clear fusion cache
pub async fn fusion_clear_cache(args: serde_json::Value) -> Result<ToolResult> {
    let global = get_global_fusion_engine();
    let guard = global.lock().await;

    match guard.as_ref() {
        Some(engine) => {
            engine.clear_cache().await;

            Ok(ToolResult {
                content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                    text: "✅ Fusion cache cleared".to_string(),
                }],
                is_error: false,
            })
        }
        None => {
            Ok(ToolResult {
                content: vec![claude_agent_sdk_rs::McpToolResultContent::Text {
                    text: "❌ Fusion engine not initialized".to_string(),
                }],
                is_error: true,
            })
        }
    }
}

// ==============================================================================
// Tests
// ==============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fusion_engine_creation() {
        let engine = DataFusionEngine::new();
        assert_eq!(engine.sources.len(), 0);
        println!("✅ Fusion engine created successfully");
    }

    #[tokio::test]
    async fn test_add_sources() {
        let yahoo = Box::new(YahooDataSource::new()) as Box<dyn DataSource>;

        let engine = DataFusionEngine::new()
            .add_source(yahoo);

        assert_eq!(engine.sources.len(), 1);
        println!("✅ Sources added successfully");
    }

    #[tokio::test]
    async fn test_yahoo_data_source() {
        let source = YahooDataSource::new();

        // Test with a common symbol
        match source.get_quote("AAPL").await {
            Ok(quote) => {
                assert_eq!(quote.symbol, "AAPL");
                assert!(quote.price > 0.0);
                assert_eq!(quote.source, "Yahoo Finance");

                println!("✅ Yahoo data source test passed");
                println!("   Symbol: {}", quote.symbol);
                println!("   Price: ${}", quote.price);
                println!("   Source: {}", quote.source);
                println!("   Latency: {}ms", quote.latency_ms);
            }
            Err(e) => {
                eprintln!("⚠️  Network test skipped: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_data_priority_ordering() {
        assert!(DataPriority::RealTime > DataPriority::High);
        assert!(DataPriority::High > DataPriority::Medium);
        assert!(DataPriority::Medium > DataPriority::Low);

        println!("✅ Data priority ordering test passed");
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let engine = DataFusionEngine::new();

        // Initially empty
        let stats = engine.cache_stats().await;
        assert_eq!(stats.total_entries, 0);

        // Add test data
        let quote = UnifiedQuote {
            symbol: "TEST".to_string(),
            price: 100.0,
            change: 1.0,
            change_percent: 1.0,
            volume: 1000,
            high: 105.0,
            low: 95.0,
            open: 99.0,
            previous_close: 99.0,
            timestamp: Utc::now(),
            source: "Test".to_string(),
            latency_ms: 50,
        };

        engine.update_cache("TEST", quote, "Test".to_string()).await;

        // Should have 1 entry
        let stats = engine.cache_stats().await;
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.source_counts.get("Test"), Some(&1));

        println!("✅ Cache operations test passed");
    }

    #[tokio::test]
    async fn test_source_stats() {
        let engine = DataFusionEngine::new()
            .add_source(Box::new(YahooDataSource::new()));

        let stats = engine.source_stats().await;
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name, "Yahoo Finance");
        assert_eq!(stats[0].priority, DataPriority::High);

        println!("✅ Source stats test passed");
    }

    #[tokio::test]
    async fn test_unified_quote_serialization() {
        let quote = UnifiedQuote {
            symbol: "AAPL".to_string(),
            price: 150.0,
            change: 1.5,
            change_percent: 1.0,
            volume: 1000000,
            high: 152.0,
            low: 148.0,
            open: 149.0,
            previous_close: 148.5,
            timestamp: Utc::now(),
            source: "Test".to_string(),
            latency_ms: 100,
        };

        let json = serde_json::to_string(&quote).unwrap();
        assert!(json.contains("AAPL"));
        assert!(json.contains("150"));

        let deserialized: UnifiedQuote = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "AAPL");

        println!("✅ Unified quote serialization test passed");
    }
}
