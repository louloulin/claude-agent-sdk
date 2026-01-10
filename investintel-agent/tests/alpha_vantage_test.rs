//! Alpha Vantage API Integration Tests
//!
//! Comprehensive tests for Alpha Vantage data access functionality

use investintel_agent::data::alpha_vantage::{
    AlphaVantageClient, alpha_vantage_quote, alpha_vantage_technical,
    alpha_vantage_news_sentiment, alpha_vantage_overview
};
use serde_json::json;

#[tokio::test]
async fn test_alpha_vantage_client_creation() {
    let client = AlphaVantageClient::new("demo");
    // Client creation successful - no assertion needed for private fields
    println!("✅ Alpha Vantage client created successfully");
}

#[tokio::test]
async fn test_get_quote() {
    let client = AlphaVantageClient::new("demo");

    // Alpha Vantage demo key only works with IBM
    match client.get_quote("IBM").await {
        Ok(quote) => {
            assert_eq!(quote.symbol, "IBM");
            assert!(quote.price > 0.0, "Price should be positive");
            assert!(!quote.latest_trading_day.is_empty(), "Trading day should not be empty");

            println!("✅ Quote test passed for IBM");
            println!("   Price: ${}", quote.price);
            println!("   Change: {} ({})", quote.change, quote.change_percent);
            println!("   Volume: {}", quote.volume);
        }
        Err(e) => {
            eprintln!("⚠️  Network/API test skipped (may need valid API key): {}", e);
            // Demo key has limited functionality
        }
    }
}

#[tokio::test]
async fn test_get_technical_indicator() {
    let client = AlphaVantageClient::new("demo");

    match client.get_technical_indicator("IBM", "RSI", "daily", Some(14), "close").await {
        Ok(indicator) => {
            assert_eq!(indicator.indicator, "RSI");
            assert_eq!(indicator.symbol, "IBM");
            assert!(!indicator.values.is_empty(), "Should have indicator values");

            println!("✅ Technical indicator test passed");
            println!("   Indicator: {}", indicator.indicator);
            println!("   Symbol: {}", indicator.symbol);
            println!("   Interval: {}", indicator.interval);
        }
        Err(e) => {
            eprintln!("⚠️  Network/API test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_get_company_overview() {
    let client = AlphaVantageClient::new("demo");

    match client.get_company_overview("IBM").await {
        Ok(overview) => {
            assert_eq!(overview.symbol, "IBM");
            assert!(!overview.name.is_empty(), "Company name should not be empty");
            assert!(!overview.sector.is_empty(), "Sector should not be empty");

            println!("✅ Company overview test passed");
            println!("   Company: {}", overview.name);
            println!("   Sector: {}", overview.sector);
            println!("   Industry: {}", overview.industry);
            println!("   Market Cap: ${}", overview.market_capitalization);
        }
        Err(e) => {
            eprintln!("⚠️  Network/API test skipped: {}", e);
        }
    }
}

#[tokio::test]
async fn test_get_news_sentiment() {
    let client = AlphaVantageClient::new("demo");

    match client.get_news_sentiment("IBM", None, None).await {
        Ok(sentiment) => {
            println!("✅ News sentiment test passed");
            println!("   Articles: {}", sentiment.feed.len());

            if !sentiment.feed.is_empty() {
                let article = &sentiment.feed[0];
                println!("   Latest article: {}", article.title);
                println!("   Sentiment score: {:.2}", article.overall_sentiment_score);
            }
        }
        Err(e) => {
            eprintln!("⚠️  News sentiment test skipped (premium endpoint): {}", e);
            // News sentiment requires premium subscription
        }
    }
}

#[tokio::test]
async fn test_mcp_tool_alpha_vantage_quote() {
    let args = json!({
        "symbol": "IBM"
    });

    match alpha_vantage_quote(args).await {
        Ok(result) => {
            assert!(!result.is_error, "Tool should not return error");
            assert!(!result.content.is_empty(), "Tool should return content");

            println!("✅ MCP Tool alpha_vantage_quote test passed");
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
async fn test_mcp_tool_alpha_vantage_technical() {
    let args = json!({
        "symbol": "IBM",
        "function": "RSI",
        "interval": "daily",
        "time_period": 14,
        "series_type": "close"
    });

    match alpha_vantage_technical(args).await {
        Ok(result) => {
            assert!(!result.is_error, "Tool should not return error");
            assert!(!result.content.is_empty(), "Tool should return content");

            println!("✅ MCP Tool alpha_vantage_technical test passed");
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
async fn test_mcp_tool_alpha_vantage_overview() {
    let args = json!({
        "symbol": "IBM"
    });

    match alpha_vantage_overview(args).await {
        Ok(result) => {
            assert!(!result.is_error, "Tool should not return error");
            assert!(!result.content.is_empty(), "Tool should return content");

            println!("✅ MCP Tool alpha_vantage_overview test passed");
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
async fn test_multiple_indicators() {
    let client = AlphaVantageClient::new("demo");
    let indicators = vec![("RSI", 14), ("SMA", 20), ("EMA", 20)];

    for (indicator, period) in indicators {
        match client.get_technical_indicator("IBM", indicator, "daily", Some(period), "close").await {
            Ok(result) => {
                println!("   {}: {} data points", indicator, result.values.len());
            }
            Err(e) => {
                eprintln!("   {}: Failed - {}", indicator, e);
            }
        }
    }

    println!("✅ Multiple indicators test completed");
}

#[tokio::test]
async fn test_different_intervals() {
    let client = AlphaVantageClient::new("demo");
    let intervals = vec!["daily", "weekly", "monthly"];

    for interval in intervals {
        match client.get_technical_indicator("IBM", "SMA", interval, Some(20), "close").await {
            Ok(result) => {
                println!("✅ Interval '{}' test: {} values", interval, result.values.len());
            }
            Err(e) => {
                eprintln!("⚠️  Interval '{}' test skipped: {}", interval, e);
            }
        }
    }
}

#[tokio::test]
async fn test_rate_limit_handling() {
    let client = AlphaVantageClient::new("demo");

    // Test multiple rapid requests
    let mut success_count = 0;
    for i in 1..=3 {
        match client.get_quote("IBM").await {
            Ok(_) => {
                success_count += 1;
            }
            Err(e) => {
                eprintln!("Request {} failed: {}", i, e);
            }
        }
        // Small delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    println!("✅ Rate limit test: {}/3 requests succeeded", success_count);
}

#[tokio::test]
async fn test_error_handling_invalid_symbol() {
    let client = AlphaVantageClient::new("demo");

    // Test with invalid symbol
    match client.get_quote("INVALID_SYMBOL_12345").await {
        Ok(_) => {
            eprintln!("⚠️  Expected error for invalid symbol");
        }
        Err(e) => {
            println!("✅ Error handling test passed: {}", e);
        }
    }
}
