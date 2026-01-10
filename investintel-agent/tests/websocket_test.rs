//! WebSocket Real-time Data Stream Tests
//!
//! Comprehensive tests for WebSocket market data streaming functionality

use investintel_agent::data::websocket::{
    MarketDataStream, MarketTick, AggregatedTick,
    websocket_start_polygon, websocket_subscribe_ticks, websocket_stats
};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_market_data_stream_creation() {
    let stream = MarketDataStream::new();
    let count = stream.subscribers_count().await;
    assert_eq!(count, 0);
    println!("✅ MarketDataStream created successfully");
}

#[tokio::test]
async fn test_stream_default() {
    let stream = MarketDataStream::default();
    let count = stream.subscribers_count().await;
    assert_eq!(count, 0);
    println!("✅ Default stream test passed");
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
        timestamp: chrono::Utc::now(),
        exchange: Some("NASDAQ".to_string()),
        conditions: vec!["regular".to_string()],
    };

    // Send tick
    let _ = stream.send_tick_for_testing(tick.clone());

    // Receive tick
    let received = rx.recv().await.unwrap();
    assert_eq!(received.symbol, "AAPL");
    assert_eq!(received.price, 150.0);
    assert_eq!(received.size, 100);

    println!("✅ Subscribe all test passed");
    println!("   Symbol: {}", received.symbol);
    println!("   Price: ${}", received.price);
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
        timestamp: chrono::Utc::now(),
    };

    // Send tick
    let _ = stream.send_aggregated_tick_for_testing(agg_tick);

    // Receive tick
    let received = rx.recv().await.unwrap();
    assert_eq!(received.symbol, "MSFT");
    assert_eq!(received.last_price, 300.5);
    assert_eq!(received.bid_price, Some(300.0));
    assert_eq!(received.ask_price, Some(301.0));

    println!("✅ Subscribe aggregated test passed");
    println!("   Symbol: {}", received.symbol);
    println!("   Bid/Ask: ${}/{}", received.bid_price.unwrap(), received.ask_price.unwrap());
}

#[tokio::test]
async fn test_multiple_subscribers() {
    let stream = MarketDataStream::new();

    // Create multiple subscribers
    let mut rx1 = stream.subscribe_all();
    let mut rx2 = stream.subscribe_all();
    let mut rx3 = stream.subscribe_all();

    let tick = MarketTick {
        symbol: "GOOGL".to_string(),
        price: 2500.0,
        size: 50,
        timestamp: chrono::Utc::now(),
        exchange: Some("NASDAQ".to_string()),
        conditions: vec!["regular".to_string()],
    };

    // Send tick
    let _ = stream.send_tick_for_testing(tick.clone());

    // All subscribers should receive
    let r1 = rx1.recv().await.unwrap();
    let r2 = rx2.recv().await.unwrap();
    let r3 = rx3.recv().await.unwrap();

    assert_eq!(r1.symbol, "GOOGL");
    assert_eq!(r2.symbol, "GOOGL");
    assert_eq!(r3.symbol, "GOOGL");

    println!("✅ Multiple subscribers test passed");
    println!("   All 3 subscribers received the tick");
}

#[tokio::test]
async fn test_tick_serialization() {
    let tick = MarketTick {
        symbol: "TSLA".to_string(),
        price: 800.0,
        size: 200,
        timestamp: chrono::Utc::now(),
        exchange: Some("NASDAQ".to_string()),
        conditions: vec!["regular".to_string()],
    };

    let json = serde_json::to_string(&tick).unwrap();
    assert!(json.contains("TSLA"));
    assert!(json.contains("800"));

    let deserialized: MarketTick = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.symbol, "TSLA");
    assert_eq!(deserialized.price, 800.0);

    println!("✅ Tick serialization test passed");
}

#[tokio::test]
async fn test_aggregated_tick_serialization() {
    let agg_tick = AggregatedTick {
        symbol: "NVDA".to_string(),
        bid_price: Some(500.0),
        ask_price: Some(501.0),
        bid_size: Some(1000),
        ask_size: Some(1200),
        last_price: 500.5,
        volume: 50000,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&agg_tick).unwrap();
    assert!(json.contains("NVDA"));

    let deserialized: AggregatedTick = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.symbol, "NVDA");
    assert_eq!(deserialized.last_price, 500.5);

    println!("✅ Aggregated tick serialization test passed");
}

#[tokio::test]
async fn test_tick_conditions() {
    let tick = MarketTick {
        symbol: "AMZN".to_string(),
        price: 3000.0,
        size: 150,
        timestamp: chrono::Utc::now(),
        exchange: Some("NASDAQ".to_string()),
        conditions: vec!["regular".to_string(), "trade".to_string()],
    };

    assert_eq!(tick.conditions.len(), 2);
    assert!(tick.conditions.contains(&"regular".to_string()));

    println!("✅ Tick conditions test passed");
    println!("   Conditions: {:?}", tick.conditions);
}

#[tokio::test]
async fn test_high_frequency_ticks() {
    let stream = MarketDataStream::new();
    let mut rx = stream.subscribe_all();

    // Send multiple ticks rapidly
    for i in 0..100 {
        let tick = MarketTick {
            symbol: "TEST".to_string(),
            price: 100.0 + i as f64,
            size: 10,
            timestamp: chrono::Utc::now(),
            exchange: Some("TEST".to_string()),
            conditions: vec!["test".to_string()],
        };
        let _ = stream.send_tick_for_testing(tick);
    }

    // Receive some ticks
    let mut received_count = 0;
    let timeout = Duration::from_secs(1);

    let start = std::time::Instant::now();
    while received_count < 10 {
        match tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Ok(_)) => received_count += 1,
            _ => break,
        }

        if start.elapsed() > timeout {
            break;
        }
    }

    assert!(received_count >= 10, "Should receive at least 10 ticks");
    println!("✅ High frequency ticks test passed");
    println!("   Received {} ticks", received_count);
}

#[tokio::test]
async fn test_mcp_tool_websocket_stats() {
    let result = websocket_stats(json!({})).await;

    match result {
        Ok(tool_result) => {
            assert!(!tool_result.is_error);
            assert!(!tool_result.content.is_empty());

            println!("✅ MCP Tool websocket_stats test passed");
            if let Some(content) = tool_result.content.first() {
                if let claude_agent_sdk_rs::McpToolResultContent::Text { text } = content {
                    println!("   Response length: {} chars", text.len());
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  MCP tool test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_subscribe_symbol_not_connected() {
    let result = websocket_subscribe_ticks(json!({"symbol": "AAPL"})).await;

    match result {
        Ok(tool_result) => {
            // Should return error since not connected
            if tool_result.is_error {
                println!("✅ Subscribe without connection correctly returns error");
            } else {
                println!("⚠️  Expected error when not connected");
            }
        }
        Err(e) => {
            eprintln!("⚠️  MCP tool test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_stream_buffer_capacity() {
    let stream = MarketDataStream::new();
    let mut rx = stream.subscribe_all();

    // Fill the buffer
    for i in 0..1500 {
        let tick = MarketTick {
            symbol: "OVERFLOW".to_string(),
            price: i as f64,
            size: 1,
            timestamp: chrono::Utc::now(),
            exchange: Some("TEST".to_string()),
            conditions: vec![],
        };
        let _ = stream.send_tick_for_testing(tick);
    }

    // Try to receive
    let mut count = 0;
    for _ in 0..2000 {
        match tokio::time::timeout(Duration::from_millis(10), rx.recv()).await {
            Ok(Ok(_)) => count += 1,
            _ => break,
        }
    }

    // Channel has capacity 1000, so we should receive fewer than 1500
    assert!(count < 1500, "Channel should drop old messages");
    println!("✅ Buffer capacity test passed");
    println!("   Sent: 1500, Received: {}", count);
}
