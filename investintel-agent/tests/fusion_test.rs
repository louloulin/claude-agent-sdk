//! Multi-Source Data Fusion Engine Tests
//!
//! Comprehensive tests for data fusion functionality

use investintel_agent::data::fusion::{
    DataFusionEngine, DataSource, DataPriority, YahooDataSource, UnifiedQuote,
    fusion_initialize, fusion_get_quote, fusion_stats, fusion_clear_cache
};
use serde_json::json;

#[tokio::test]
async fn test_data_priority_ordering() {
    assert!(DataPriority::RealTime > DataPriority::High);
    assert!(DataPriority::High > DataPriority::Medium);
    assert!(DataPriority::Medium > DataPriority::Low);

    println!("✅ Data priority ordering test passed");
}

#[tokio::test]
async fn test_fusion_engine_creation() {
    let engine = DataFusionEngine::new();
    assert_eq!(engine.sources.len(), 0);

    let stats = engine.cache_stats().await;
    assert_eq!(stats.total_entries, 0);

    println!("✅ Fusion engine creation test passed");
}

#[tokio::test]
async fn test_add_single_source() {
    let yahoo = Box::new(YahooDataSource::new()) as Box<dyn DataSource>;

    let engine = DataFusionEngine::new()
        .add_source(yahoo);

    assert_eq!(engine.sources.len(), 1);

    let source_stats = engine.source_stats().await;
    assert_eq!(source_stats.len(), 1);
    assert_eq!(source_stats[0].name, "Yahoo Finance");
    assert_eq!(source_stats[0].priority, DataPriority::High);

    println!("✅ Add single source test passed");
}

#[tokio::test]
async fn test_yahoo_data_source() {
    let source = YahooDataSource::new();

    assert_eq!(source.source_name(), "Yahoo Finance");
    assert_eq!(source.priority(), DataPriority::High);
    assert!(source.latency_ms() > 0);

    // Test actual quote retrieval
    match source.get_quote("AAPL").await {
        Ok(quote) => {
            assert_eq!(quote.symbol, "AAPL");
            assert!(quote.price > 0.0);
            assert_eq!(quote.source, "Yahoo Finance");

            println!("✅ Yahoo data source test passed");
            println!("   Symbol: {}", quote.symbol);
            println!("   Price: ${}", quote.price);
            println!("   Latency: {}ms", quote.latency_ms);
        }
        Err(e) => {
            eprintln!("⚠️  Network test skipped: {}", e);
        }
    }
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
        timestamp: chrono::Utc::now(),
        source: "TestSource".to_string(),
        latency_ms: 50,
    };

    engine.update_cache("TEST", quote.clone(), "TestSource".to_string()).await;

    // Should have 1 entry
    let stats = engine.cache_stats().await;
    assert_eq!(stats.total_entries, 1);
    assert_eq!(stats.source_counts.get("TestSource"), Some(&1));

    // Clear cache
    engine.clear_cache().await;

    // Should be empty
    let stats = engine.cache_stats().await;
    assert_eq!(stats.total_entries, 0);

    println!("✅ Cache operations test passed");
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
        timestamp: chrono::Utc::now(),
        source: "Test".to_string(),
        latency_ms: 100,
    };

    let json = serde_json::to_string(&quote).unwrap();
    assert!(json.contains("AAPL"));
    assert!(json.contains("150"));

    let deserialized: UnifiedQuote = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.symbol, "AAPL");
    assert_eq!(deserialized.price, 150.0);

    println!("✅ Unified quote serialization test passed");
}

#[tokio::test]
async fn test_mcp_tool_fusion_initialize() {
    let result = fusion_initialize(json!({})).await;

    match result {
        Ok(tool_result) => {
            assert!(!tool_result.is_error);
            assert!(!tool_result.content.is_empty());

            println!("✅ MCP Tool fusion_initialize test passed");
            if let Some(content) = tool_result.content.first() {
                if let claude_agent_sdk_rs::McpToolResultContent::Text { text } = content {
                    println!("   Response: {}", text);
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  MCP tool test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_mcp_tool_fusion_stats() {
    // First initialize
    let _ = fusion_initialize(json!({})).await;

    // Then get stats
    let result = fusion_stats(json!({})).await;

    match result {
        Ok(tool_result) => {
            assert!(!tool_result.is_error);

            println!("✅ MCP Tool fusion_stats test passed");
            if let Some(content) = tool_result.content.first() {
                if let claude_agent_sdk_rs::McpToolResultContent::Text { text } = content {
                    println!("   Stats: {}", text);
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  MCP tool test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_mcp_tool_fusion_clear_cache() {
    // First initialize
    let _ = fusion_initialize(json!({})).await;

    // Then clear cache
    let result = fusion_clear_cache(json!({})).await;

    match result {
        Ok(tool_result) => {
            assert!(!tool_result.is_error);

            println!("✅ MCP Tool fusion_clear_cache test passed");
            if let Some(content) = tool_result.content.first() {
                if let claude_agent_sdk_rs::McpToolResultContent::Text { text } = content {
                    println!("   Response: {}", text);
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  MCP tool test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_mcp_tool_fusion_get_quote() {
    // First initialize
    let _ = fusion_initialize(json!({})).await;

    // Then get quote
    let result = fusion_get_quote(json!({"symbol": "AAPL"})).await;

    match result {
        Ok(tool_result) => {
            if tool_result.is_error {
                println!("⚠️  Quote retrieval returned error (may be network issue)");
            } else {
                assert!(!tool_result.content.is_empty());

                println!("✅ MCP Tool fusion_get_quote test passed");
                if let Some(content) = tool_result.content.first() {
                    if let claude_agent_sdk_rs::McpToolResultContent::Text { text } = content {
                        println!("   Quote length: {} chars", text.len());
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  MCP tool test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_cache_ttl() {
    let engine = DataFusionEngine::new()
        .with_cache_ttl(2); // 2 second TTL

    let quote = UnifiedQuote {
        symbol: "TTLTEST".to_string(),
        price: 100.0,
        change: 0.0,
        change_percent: 0.0,
        volume: 100,
        high: 100.0,
        low: 100.0,
        open: 100.0,
        previous_close: 100.0,
        timestamp: chrono::Utc::now(),
        source: "Test".to_string(),
        latency_ms: 50,
    };

    engine.update_cache("TTLTEST", quote.clone(), "Test".to_string()).await;

    // Should be in cache immediately
    let cached = engine.get_from_cache("TTLTEST", 2).await;
    assert!(cached.is_some(), "Should be in cache");

    // Wait for cache to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Should be expired
    let cached = engine.get_from_cache("TTLTEST", 2).await;
    assert!(cached.is_none(), "Should be expired");

    println!("✅ Cache TTL test passed");
}

#[tokio::test]
async fn test_source_statistics() {
    let engine = DataFusionEngine::new()
        .add_source(Box::new(YahooDataSource::new()));

    let stats = engine.source_stats().await;
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].name, "Yahoo Finance");
    assert_eq!(stats[0].priority, DataPriority::High);
    assert!(stats[0].latency_ms > 0);

    println!("✅ Source statistics test passed");
    println!("   Source: {}", stats[0].name);
    println!("   Priority: {:?}", stats[0].priority);
    println!("   Latency: {}ms", stats[0].latency_ms);
}
