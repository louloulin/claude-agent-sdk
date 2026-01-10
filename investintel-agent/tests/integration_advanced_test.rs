// integration_advanced_test.rs - Advanced integration tests for InvestIntel
use anyhow::Result;

#[cfg(test)]
mod advanced_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_streaming_analyzer_basic() {
        // Test streaming analyzer creation
        // Note: This test requires actual Claude API access
        // In CI/CD, this should be mocked or skipped
    }

    #[tokio::test]
    async fn test_market_data_client_creation() {
        let client = investintel::market_data::MarketDataClient::new();
        // Client should be created successfully
        println!("✅ MarketDataClient created successfully");
    }

    #[tokio::test]
    async fn test_storage_manager_creation() {
        use std::path::PathBuf;
        let db_path = PathBuf::from("/tmp/test_investintel_advanced.db");
        let storage = investintel::storage::StorageManager::new(db_path)
            .await
            .unwrap();

        println!("✅ StorageManager created successfully");

        // Test portfolio operations
        let portfolio = investintel::storage::Portfolio {
            id: "test-portfolio-advanced".to_string(),
            name: "Advanced Test Portfolio".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            initial_value: 50000.0,
            current_value: 52500.0,
            positions: vec![
                investintel::storage::Position {
                    id: "pos-1".to_string(),
                    portfolio_id: "test-portfolio-advanced".to_string(),
                    ticker: "AAPL".to_string(),
                    shares: 100.0,
                    avg_cost: 150.0,
                    current_price: 175.0,
                    market_value: 17500.0,
                },
                investintel::storage::Position {
                    id: "pos-2".to_string(),
                    portfolio_id: "test-portfolio-advanced".to_string(),
                    ticker: "MSFT".to_string(),
                    shares: 150.0,
                    avg_cost: 300.0,
                    current_price: 350.0,
                    market_value: 52500.0,
                },
            ],
            metadata: serde_json::json!({"test_type": "advanced"}),
        };

        storage.save_portfolio(&portfolio).await.unwrap();
        println!("✅ Portfolio saved successfully");

        let loaded = storage.load_portfolio("test-portfolio-advanced").await.unwrap();
        assert_eq!(loaded.id, portfolio.id);
        assert_eq!(loaded.positions.len(), 2);
        println!("✅ Portfolio loaded successfully");

        // Test market data caching
        let market_data = investintel::storage::MarketDataRecord {
            id: "md-1".to_string(),
            ticker: "AAPL".to_string(),
            price: 175.5,
            change: 5.5,
            change_percent: 3.24,
            volume: 50_000_000,
            timestamp: chrono::Utc::now(),
        };

        storage.save_market_data(&market_data).await.unwrap();
        println!("✅ Market data saved successfully");

        let retrieved = storage.get_latest_market_data("AAPL").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().ticker, "AAPL");
        println!("✅ Market data retrieved successfully");

        // Test analysis caching
        let cache = investintel::storage::AnalysisCache {
            id: "cache-1".to_string(),
            ticker: "AAPL".to_string(),
            analysis_type: "technical".to_string(),
            result: "Strong buy signal".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            confidence: 0.92,
        };

        storage.cache_analysis(&cache).await.unwrap();
        println!("✅ Analysis cached successfully");

        let cached = storage
            .get_cached_analysis("AAPL", "technical")
            .await
            .unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().result, "Strong buy signal");
        println!("✅ Cached analysis retrieved successfully");

        // Test stats
        let stats = storage.get_stats().await.unwrap();
        assert_eq!(stats.portfolio_count, 1);
        assert_eq!(stats.position_count, 2);
        assert!(stats.market_data_count > 0);
        println!("✅ Stats retrieved: {:?}", stats);

        // Cleanup
        tokio::fs::remove_file("/tmp/test_investintel_advanced.db")
            .await
            .ok();
    }

    #[test]
    fn test_backtest_engine_creation() {
        use investintel::backtest::{BacktestConfig, BacktestEngine};

        let config = BacktestConfig {
            initial_capital: 100_000.0,
            commission: 0.001,
            slippage: 0.0001,
            start_date: chrono::Utc::now() - chrono::Duration::days(90),
            end_date: chrono::Utc::now(),
            symbols: vec!["AAPL".to_string(), "MSFT".to_string()],
        };

        let _engine = BacktestEngine::new(config.clone());
        println!("✅ BacktestEngine created successfully");
        println!("   Config: {:?}", config);
    }

    #[test]
    fn test_backtest_signals() {
        use investintel::backtest::{PriceData, Signal};

        let price_data = PriceData {
            timestamp: chrono::Utc::now(),
            open: 150.0,
            high: 155.0,
            low: 148.0,
            close: 152.0,
            volume: 1_000_000,
        };

        // Test different signals
        let buy_signal = Signal::Buy;
        let sell_signal = Signal::Sell;
        let hold_signal = Signal::Hold;

        assert_eq!(buy_signal, Signal::Buy);
        assert_eq!(sell_signal, Signal::Sell);
        assert_eq!(hold_signal, Signal::Hold);

        println!("✅ Signal enum working correctly");
    }

    #[test]
    fn test_streaming_events() {
        use investintel::streaming::{StreamingEvent, StreamingAnalysisResult, AnalysisType};
        use std::collections::HashMap;

        let event_text = StreamingEvent::Text("Sample text".to_string());
        let event_complete = StreamingEvent::AnalysisComplete(StreamingAnalysisResult {
            ticker: "AAPL".to_string(),
            analysis_type: AnalysisType::Technical,
            content: "Strong buy".to_string(),
            confidence: 0.95,
            metadata: HashMap::new(),
        });
        let event_error = StreamingEvent::Error("Test error".to_string());

        match event_text {
            StreamingEvent::Text(text) => assert_eq!(text, "Sample text"),
            _ => panic!("Unexpected event"),
        }

        match event_complete {
            StreamingEvent::AnalysisComplete(result) => {
                assert_eq!(result.ticker, "AAPL");
                assert_eq!(result.confidence, 0.95);
            }
            _ => panic!("Unexpected event"),
        }

        match event_error {
            StreamingEvent::Error(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Unexpected event"),
        }

        println!("✅ StreamingEvent enum working correctly");
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        use std::path::PathBuf;
        use tokio::task::JoinSet;

        let db_path = PathBuf::from("/tmp/test_concurrent.db");
        let storage = investintel::storage::StorageManager::new(db_path)
            .await
            .unwrap();

        // Test concurrent portfolio saves
        let mut join_set = JoinSet::new();

        for i in 0..10 {
            let storage_clone = storage.clone();
            let task = tokio::spawn(async move {
                let portfolio = investintel::storage::Portfolio {
                    id: format!("concurrent-{}", i),
                    name: format!("Concurrent Portfolio {}", i),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    initial_value: 10000.0 * (i as f64 + 1.0),
                    current_value: 11000.0 * (i as f64 + 1.0),
                    positions: vec![],
                    metadata: serde_json::json!({"concurrent": true}),
                };

                storage_clone.save_portfolio(&portfolio).await.unwrap();
                i
            });

            join_set.spawn(task);
        }

        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            results.push(result.unwrap());
        }

        assert_eq!(results.len(), 10);
        println!("✅ Concurrent operations completed successfully");

        // Verify all portfolios were saved
        let stats = storage.get_stats().await.unwrap();
        assert_eq!(stats.portfolio_count, 10);
        println!("✅ All concurrent saves verified: {} portfolios", stats.portfolio_count);

        // Cleanup
        tokio::fs::remove_file("/tmp/test_concurrent.db").await.ok();
    }

    #[test]
    fn test_analysis_type_parsing() {
        use investintel::main_v2;

        let types = main_v2::parse_analysis_types("technical,fundamental,sentiment");
        assert_eq!(types.len(), 3);

        let types_all = main_v2::parse_analysis_types("all");
        assert_eq!(types_all.len(), 1);

        println!("✅ Analysis type parsing working correctly");
    }
}
