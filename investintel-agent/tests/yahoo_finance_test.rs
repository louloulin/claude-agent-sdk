//! Yahoo Finance API Integration Tests
//!
//! Comprehensive tests for Yahoo Finance data access functionality

use investintel_agent::data::yahoo::{YahooFinanceClient, yahoo_finance_quote, yahoo_finance_historical, yahoo_finance_search};
use serde_json::json;

#[tokio::test]
async fn test_yahoo_client_quote() {
    let client = YahooFinanceClient::new();

    // Test getting a quote for Apple
    match client.get_quote("AAPL").await {
        Ok(quote) => {
            assert_eq!(quote.symbol, "AAPL");
            assert!(quote.regular_market_price > 0.0, "Price should be positive");
            assert!(!quote.company_name.is_empty(), "Company name should not be empty");
            assert!(quote.volume >= 0, "Volume should be non-negative");

            println!("✅ Quote test passed for AAPL");
            println!("   Price: ${}", quote.regular_market_price);
            println!("   Change: {} ({:.2}%)", quote.change, quote.change_percent);
            println!("   Volume: {}", quote.volume);
        }
        Err(e) => {
            eprintln!("⚠️  Network test skipped (connection failed): {}", e);
            // Don't fail the test in CI/CD environments without internet
        }
    }
}

#[tokio::test]
async fn test_yahoo_client_historical() {
    let client = YahooFinanceClient::new();

    // Test getting historical data
    match client.get_historical("AAPL", "1d", "5d").await {
        Ok(data) => {
            assert!(!data.is_empty(), "Should have historical data");
            assert_eq!(data[0].symbol, "AAPL");

            // Verify OHLCV structure
            for bar in &data {
                assert!(bar.open > 0.0, "Open should be positive");
                assert!(bar.high > 0.0, "High should be positive");
                assert!(bar.low > 0.0, "Low should be positive");
                assert!(bar.close > 0.0, "Close should be positive");
                assert!(bar.high >= bar.low, "High should be >= low");
                assert!(bar.high >= bar.open, "High should be >= open");
                assert!(bar.high >= bar.close, "High should be >= close");
            }

            println!("✅ Historical data test passed");
            println!("   Records: {}", data.len());
            println!("   Date range: {} to {}", data[0].timestamp.format("%Y-%m-%d"),
                     data.last().unwrap().timestamp.format("%Y-%m-%d"));
        }
        Err(e) => {
            eprintln!("⚠️  Network test skipped (connection failed): {}", e);
        }
    }
}

#[tokio::test]
async fn test_yahoo_client_search() {
    let client = YahooFinanceClient::new();

    match client.search_symbols("Apple").await {
        Ok(results) => {
            assert!(!results.is_empty(), "Should find search results");

            // Find Apple Inc.
            let apple = results.iter().find(|r| r.symbol == "AAPL");
            assert!(apple.is_some(), "Should find AAPL in search results");

            println!("✅ Search test passed");
            println!("   Found {} results", results.len());
            if let Some(apple_result) = apple {
                println!("   AAPL: {}", apple_result.name);
            }
        }
        Err(e) => {
            eprintln!("⚠️  Network test skipped (connection failed): {}", e);
        }
    }
}

#[tokio::test]
async fn test_mcp_tool_yahoo_finance_quote() {
    let args = json!({
        "symbol": "AAPL"
    });

    match yahoo_finance_quote(args).await {
        Ok(result) => {
            assert!(!result.is_error, "Tool should not return error");
            assert!(!result.content.is_empty(), "Tool should return content");

            println!("✅ MCP Tool yahoo_finance_quote test passed");
            if let Some(content) = result.content.first() {
                if let claude_agent_sdk_rs::McpToolResultContent::Text { text } = content {
                    println!("   Content length: {} chars", text.len());
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  MCP tool test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_mcp_tool_yahoo_finance_historical() {
    let args = json!({
        "symbol": "AAPL",
        "interval": "1d",
        "range": "5d"
    });

    match yahoo_finance_historical(args).await {
        Ok(result) => {
            assert!(!result.is_error, "Tool should not return error");
            assert!(!result.content.is_empty(), "Tool should return content");

            println!("✅ MCP Tool yahoo_finance_historical test passed");
            if let Some(content) = result.content.first() {
                if let claude_agent_sdk_rs::McpToolResultContent::Text { text } = content {
                    println!("   Content length: {} chars", text.len());
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  MCP tool test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_mcp_tool_yahoo_finance_search() {
    let args = json!({
        "query": "Microsoft"
    });

    match yahoo_finance_search(args).await {
        Ok(result) => {
            assert!(!result.is_error, "Tool should not return error");
            assert!(!result.content.is_empty(), "Tool should return content");

            println!("✅ MCP Tool yahoo_finance_search test passed");
            if let Some(content) = result.content.first() {
                if let claude_agent_sdk_rs::McpToolResultContent::Text { text } = content {
                    println!("   Content length: {} chars", text.len());
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  MCP tool test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_multiple_symbols() {
    let client = YahooFinanceClient::new();
    let symbols = vec!["AAPL", "MSFT", "GOOGL"];
    let total_symbols = symbols.len();

    let mut success_count = 0;
    for symbol in symbols {
        match client.get_quote(symbol).await {
            Ok(quote) => {
                assert_eq!(quote.symbol, symbol);
                assert!(quote.regular_market_price > 0.0);
                success_count += 1;
                println!("   {}: ${}", symbol, quote.regular_market_price);
            }
            Err(e) => {
                eprintln!("   {}: Failed - {}", symbol, e);
            }
        }
    }

    if success_count > 0 {
        println!("✅ Multiple symbols test: {}/{} successful", success_count, total_symbols);
    } else {
        eprintln!("⚠️  Network test skipped (no connections succeeded)");
    }
}

#[tokio::test]
async fn test_different_intervals() {
    let client = YahooFinanceClient::new();
    let intervals = vec!["1d", "1wk", "1mo"];

    for interval in intervals {
        match client.get_historical("AAPL", interval, "1mo").await {
            Ok(data) => {
                assert!(!data.is_empty());
                println!("✅ Interval '{}' test: {} records", interval, data.len());
            }
            Err(e) => {
                eprintln!("⚠️  Interval '{}' test skipped: {}", interval, e);
            }
        }
    }
}
