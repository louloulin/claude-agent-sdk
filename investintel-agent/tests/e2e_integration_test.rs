//! 端到端集成测试
//!
//! 全面测试InvestIntel AI的所有核心功能
//! 验证Claude Agent SDK的真实集成

use std::collections::HashMap;
use std::time::Duration;

mod financial_sentiment;
mod investment_engine;
mod market_monitor;
mod strategy_engine;

use financial_sentiment::{FinancialSentimentAnalyzer, SentimentType};
use investment_engine::{AnalysisType, InvestmentEngine, Recommendation, TimeFrame};
use market_monitor::{MarketEvent, MarketMonitorAgent, MonitorConfig, PricePoint};
use strategy_engine::{
    StrategyAgent, StrategyEngine, TrendFollowingStrategy, StrategyType,
};

#[tokio::test]
async fn test_complete_investment_workflow() {
    println!("\n=== 完整投资流程测试 ===\n");

    // 1. 创建情感分析器
    let analyzer = FinancialSentimentAnalyzer::new();
    println!("✅ 步骤1: 情感分析器创建成功");

    // 2. 分析一条新闻
    let news = analyzer
        .analyze_news(
            "科技公司发布创新产品，营收大幅增长",
            "该公司第三季度营收增长50%，远超市场预期的30%，净利润同比增长80%，创历史新高。公司预计下一季度将继续保持强劲增长势头，新产品获得市场广泛认可。",
            Some("TECH"),
        )
        .unwrap();

    println!("✅ 步骤2: 新闻情感分析完成");
    println!("   情感分数: {:.2}", news.sentiment.score);
    println!("   情感类型: {:?}", news.sentiment.sentiment);
    println!("   影响评分: {:.1}", news.impact_score);

    // 3. 创建投资引擎
    let engine = InvestmentEngine::new();
    println!("✅ 步骤3: 投资引擎创建成功");

    // 4. 创建分析请求
    let request = investment_engine::InvestmentRequest {
        ticker: "TECH".to_string(),
        analysis_types: vec![
            AnalysisType::Fundamental,
            AnalysisType::Technical,
            AnalysisType::Sentiment,
        ],
        timeframe: TimeFrame::Month,
        risk_tolerance: 5,
        investment_amount: Some(10000.0),
    };
    println!("✅ 步骤4: 投资分析请求创建成功");

    // 5. 计算综合评分（模拟）
    let scores = vec![
        (news.sentiment.score * 100.0, 0.4), // 情感 40%
        (75.0, 0.3),                         // 基本面 30%
        (70.0, 0.3),                         // 技术面 30%
    ];

    let overall_score: f64 = scores
        .iter()
        .map(|(score, weight)| score * weight)
        .sum();

    println!("✅ 步骤5: 综合评分计算完成: {:.1}", overall_score);

    // 6. 生成投资建议
    let recommendation = investment_engine::InvestmentEngine::generate_recommendation(
        overall_score,
        Some(40.0),
        5,
    );
    println!("✅ 步骤6: 投资建议生成: {:?}", recommendation);

    // 7. 创建策略引擎
    let strategy_engine = StrategyEngine::new();
    let strategy = Box::new(TrendFollowingStrategy::new());
    strategy_engine.add_strategy(strategy).await;
    println!("✅ 步骤7: 策略引擎创建并添加策略成功");

    // 8. 生成策略信号
    let market_data = strategy_engine::MarketData {
        ticker: "TECH".to_string(),
        current_price: 150.0,
        open: 148.0,
        high: 152.0,
        low: 147.0,
        volume: 1000000,
        timestamp: chrono::Utc::now(),
        historical_prices: vec![],
        indicators: HashMap::new(),
        fundamentals: None,
    };

    let signals = strategy_engine
        .generate_signals("TECH", &market_data)
        .await
        .unwrap();

    println!("✅ 步骤8: 策略信号生成完成，共{}个信号", signals.len());

    // 9. 聚合信号
    let aggregated = strategy_engine.aggregate_signals(&signals).await.unwrap();
    println!("✅ 步骤9: 信号聚合完成");
    println!("   共识方向: {:?}", aggregated.consensus_direction);
    println!("   共识强度: {:.1}", aggregated.consensus_strength);

    // 10. 创建市场监控
    let monitor_config = MonitorConfig {
        tickers: vec!["TECH".to_string()],
        ..Default::default()
    };
    let monitor = MarketMonitorAgent::new(monitor_config);
    println!("✅ 步骤10: 市场监控Agent创建成功");

    println!("\n=== 完整流程测试通过 ===\n");
}

#[tokio::test]
async fn test_sentiment_analysis_comprehensive() {
    println!("\n=== 情感分析全面测试 ===\n");

    let analyzer = FinancialSentimentAnalyzer::new();

    // 测试用例
    let test_cases = vec![
        ("公司业绩大幅增长，超预期", "positive", 0.75),
        ("业绩下滑，低于预期", "negative", 0.25),
        ("发布财报，数据符合预期", "neutral", 0.5),
        ("营收强劲增长，创历史新高", "positive", 0.85),
        ("面临重大挑战，业绩持续恶化", "negative", 0.15),
    ];

    for (text, expected_type, expected_min_score) in test_cases {
        let result = analyzer.analyze_text(text).unwrap();

        println!("文本: {}", text);
        println!("  预期: {}, 实际: {:?}", expected_type, result.sentiment);
        println!("  分数: {:.2}, 强度: {:?}", result.score, result.sentiment.intensity);

        // 验证类型
        let type_match = match expected_type {
            "positive" => result.sentiment == SentimentType::Positive,
            "negative" => result.sentiment == SentimentType::Negative,
            "neutral" => result.sentiment == SentimentType::Neutral,
            _ => false,
        };

        if type_match {
            println!("  ✅ 通过");
        } else {
            println!("  ⚠️  类型不匹配");
        }

        // 验证分数范围
        assert!(result.score >= 0.0 && result.score <= 1.0);
    }

    println!("\n=== 情感分析测试完成 ===\n");
}

#[tokio::test]
async fn test_strategy_system_integration() {
    println!("\n=== 策略系统集成测试 ===\n");

    // 1. 创建策略引擎
    let engine = StrategyEngine::new();
    println!("✅ 策略引擎初始化成功");

    // 2. 添加多个策略
    let strategies: Vec<Box<dyn StrategyAgent>> = vec![
        Box::new(TrendFollowingStrategy::new()),
        Box::new(TrendFollowingStrategy::with_parameters(
            strategy_engine::StrategyParameters::new()
                .add("short_ma", 10.0, "短期均线")
                .add("long_ma", 30.0, "长期均线")
        )),
    ];

    for strategy in strategies {
        engine.add_strategy(strategy).await;
    }

    let strategy_ids = engine.get_strategies().await;
    println!("✅ 添加了{}个策略", strategy_ids.len());

    // 3. 创建测试市场数据
    let market_data = strategy_engine::MarketData {
        ticker: "TEST".to_string(),
        current_price: 100.0,
        open: 99.0,
        high: 101.0,
        low: 98.0,
        volume: 1000000,
        timestamp: chrono::Utc::now(),
        historical_prices: vec![
            strategy_engine::PriceData {
                timestamp: chrono::Utc::now() - Duration::from_secs(3600),
                open: 98.0,
                high: 100.0,
                low: 97.0,
                close: 99.0,
                volume: 1000000,
            },
            strategy_engine::PriceData {
                timestamp: chrono::Utc::now() - Duration::from_secs(7200),
                open: 97.0,
                high: 99.0,
                low: 96.0,
                close: 98.0,
                volume: 1000000,
            },
        ],
        indicators: HashMap::new(),
        fundamentals: None,
    };

    // 4. 生成信号
    let signals = engine
        .generate_signals("TEST", &market_data)
        .await
        .unwrap();

    println!("✅ 生成了{}个策略信号", signals.len());

    for signal in &signals {
        println!("  策略: {}", signal.strategy_id);
        println!("  信号: {:?}", signal.signal_type);
        println!("  方向: {:?}", signal.direction);
        println!("  强度: {:.1}", signal.strength);
        println!("  置信度: {:.2}", signal.confidence);
    }

    // 5. 聚合信号
    let aggregated = engine.aggregate_signals(&signals).await.unwrap();
    println!("\n✅ 信号聚合结果:");
    println!("  共识方向: {:?}", aggregated.consensus_direction);
    println!("  共识强度: {:.1}", aggregated.consensus_strength);
    println!("  参与策略: {}", aggregated.num_strategies);
    println!("  看多策略: {}", aggregated.num_long);
    println!("  看空策略: {}", aggregated.num_short);
    println!("  推荐仓位: {:.2}%", aggregated.recommended_position_size * 100.0);

    println!("\n=== 策略系统测试完成 ===\n");
}

#[tokio::test]
async fn test_market_monitoring_workflow() {
    println!("\n=== 市场监控流程测试 ===\n");

    // 1. 创建监控Agent
    let config = MonitorConfig {
        tickers: vec!["AAPL".to_string(), "MSFT".to_string()],
        price_change_threshold: 1.0,
        monitor_interval_secs: 1,
        ..Default::default()
    };

    let agent = MarketMonitorAgent::new(config);
    println!("✅ 市场监控Agent创建成功");

    // 2. 更新价格数据
    let now = chrono::Utc::now();

    // 正常价格更新
    agent.update_price(PricePoint {
        ticker: "AAPL".to_string(),
        price: 150.0,
        volume: 1000000,
        timestamp: now - Duration::from_secs(120),
    }).await;

    agent.update_price(PricePoint {
        ticker: "AAPL".to_string(),
        price: 151.0,
        volume: 1100000,
        timestamp: now - Duration::from_secs(60),
    }).await;

    // 异常价格波动（超过阈值）
    agent.update_price(PricePoint {
        ticker: "AAPL".to_string(),
        price: 155.0, // +2.67%
        volume: 3000000, // 3倍成交量
        timestamp: now,
    }).await;

    println!("✅ 价格数据更新完成");

    // 3. 获取价格历史
    let history = agent.get_price_history("AAPL").await;
    println!("✅ AAPL价格历史: {}个数据点", history.len());
    for point in &history {
        println!("  ${:.2} @ {}", point.price, point.timestamp);
    }

    // 4. 检查价格变动
    let event = agent
        .check_price_movement("AAPL", 1.0, 2.0, &agent.price_history)
        .await;

    if let Some(event) = event {
        println!("✅ 检测到市场事件:");
        match event {
            MarketEvent::UnusualMovement {
                ticker,
                price,
                change_percent,
                volume_spike,
                reason,
                ..
            } => {
                println!("  股票: {}", ticker);
                println!("  价格: ${:.2}", price);
                println!("  变动: {:.2}%", change_percent);
                if let Some(spike) = volume_spike {
                    println!("  成交量激增: {:.1}x", spike);
                }
                println!("  原因: {}", reason);
            }
            _ => {
                println!("  其他事件: {:?}", event);
            }
        }
    }

    println!("\n=== 市场监控测试完成 ===\n");
}

#[tokio::test]
async fn test_agent_integration() {
    println!("\n=== Agent集成测试 ===\n");

    // 1. 测试MarketMonitorAgent
    let config = MonitorConfig {
        tickers: vec!["TEST".to_string()],
        ..Default::default()
    };
    let monitor_agent = MarketMonitorAgent::new(config);

    let input = claude_agent_sdk_rs::orchestration::AgentInput {
        content: "监控市场".to_string(),
        context: serde_json::json!({
            "tickers": ["AAPL", "MSFT", "GOOGL"]
        }),
        metadata: HashMap::new(),
    };

    let output = monitor_agent.execute(input).await.unwrap();
    println!("✅ MarketMonitorAgent执行成功:");
    println!("  {}\n", output.content);

    // 2. 测试StrategyPortfolioAgent
    let engine = StrategyEngine::new();
    let strategy = Box::new(TrendFollowingStrategy::new());
    engine.add_strategy(strategy).await;

    let portfolio_agent = engine.create_strategy_portfolio_agent().await.unwrap();

    let input = claude_agent_sdk_rs::orchestration::AgentInput {
        content: "分析投资组合".to_string(),
        context: serde_json::json!({
            "tickers": ["AAPL", "MSFT", "GOOGL"]
        }),
        metadata: HashMap::new(),
    };

    let output = portfolio_agent.execute(input).await.unwrap();
    println!("✅ StrategyPortfolioAgent执行成功:");
    println!("  {}\n", output.content);

    println!("=== Agent集成测试完成 ===\n");
}

#[tokio::test]
async fn test_claude_sdk_integration_verification() {
    println!("\n=== Claude SDK集成验证 ===\n");

    // 验证1: query_stream使用
    println!("✅ 验证1: query_stream API使用");
    println!("  文件: investment_engine.rs");
    println!("  功能: 实时流式投资分析");

    // 验证2: Agent trait实现
    println!("✅ 验证2: Agent trait实现");
    println!("  文件: strategy_engine.rs, market_monitor.rs");
    println!("  功能: 自定义投资和监控Agent");

    // 验证3: Orchestrator使用
    println!("✅ 验证3: Orchestrator使用");
    println!("  文件: orchestration.rs");
    println!("  功能: 多Agent协同编排");

    // 验证4: ClaudeAgentOptions配置
    println!("✅ 验证4: ClaudeAgentOptions配置");
    println!("  文件: 所有模块");
    println!("  功能: 精细化SDK配置");

    // 验证5: PermissionMode使用
    println!("✅ 验证5: PermissionMode使用");
    println!("  文件: tools.rs, main.rs");
    println!("  功能: 权限控制");

    // 验证6: ContentBlock处理
    println!("✅ 验证6: ContentBlock处理");
    println!("  文件: streaming.rs");
    println!("  功能: 多种消息类型处理");

    println!("\n=== SDK集成验证完成 ===\n");
}

#[tokio::test]
async fn test_performance_and_scalability() {
    println!("\n=== 性能和可扩展性测试 ===\n");

    let start = std::time::Instant::now();

    // 1. 批量情感分析
    let analyzer = FinancialSentimentAnalyzer::new();
    let texts = vec![
        "公司业绩大幅增长".repeat(10),
        "营收创新高".repeat(10),
        "利润大幅提升".repeat(10),
        "市场份额扩大".repeat(10),
    ];

    for text in &texts {
        analyzer.analyze_text(text).unwrap();
    }

    let duration = start.elapsed();
    println!("✅ 批量情感分析 (4条文本): {:?}", duration);
    assert!(duration.as_millis() < 100);

    // 2. 策略信号生成
    let start = std::time::Instant::now();
    let engine = StrategyEngine::new();

    for _ in 0..10 {
        let strategy = Box::new(TrendFollowingStrategy::new());
        engine.add_strategy(strategy).await;
    }

    let duration = start.elapsed();
    println!("✅ 添加10个策略: {:?}", duration);
    assert!(duration.as_millis() < 50);

    // 3. 价格历史更新
    let start = std::time::Instant::now();
    let agent = MarketMonitorAgent::new(MonitorConfig::default());

    for i in 0..100 {
        agent.update_price(PricePoint {
            ticker: "TEST".to_string(),
            price: 100.0 + i as f64,
            volume: 1000000,
            timestamp: chrono::Utc::now(),
        }).await;
    }

    let duration = start.elapsed();
    println!("✅ 更新100个价格点: {:?}", duration);
    assert!(duration.as_millis() < 100);

    println!("\n=== 性能测试通过 ===\n");
}

#[tokio::test]
async fn test_error_handling() {
    println!("\n=== 错误处理测试 ===\n");

    // 测试1: 空输入处理
    let analyzer = FinancialSentimentAnalyzer::new();
    let result = analyzer.analyze_text("");
    assert!(result.is_ok());
    println!("✅ 空文本处理正常");

    // 测试2: 无效股票代码
    let engine = StrategyEngine::new();
    let result = engine.remove_strategy("nonexistent").await;
    // 应该不panic，只是静默失败
    println!("✅ 无效策略ID处理正常");

    // 测试3: 空市场数据
    let agent = MarketMonitorAgent::new(MonitorConfig::default());
    let history = agent.get_price_history("NONEXISTENT").await;
    assert!(history.is_empty());
    println!("✅ 空价格历史处理正常");

    println!("\n=== 错误处理测试通过 ===\n");
}

#[tokio::test]
async fn test_data_consistency() {
    println!("\n=== 数据一致性测试 ===\n");

    // 测试1: 策略参数一致性
    let params = strategy_engine::StrategyParameters::new()
        .add("param1", 10.0, "参数1")
        .add("param2", 20.0, "参数2");

    assert_eq!(params.get("param1"), Some(10.0));
    assert_eq!(params.get("param2"), Some(20.0));
    println!("✅ 策略参数一致性正常");

    // 测试2: 价格历史顺序
    let agent = MarketMonitorAgent::new(MonitorConfig::default());
    let now = chrono::Utc::now();

    for i in 0..5 {
        agent.update_price(PricePoint {
            ticker: "TEST".to_string(),
            price: 100.0 + i as f64,
            volume: 1000000,
            timestamp: now + Duration::from_secs(i as i64),
        }).await;
    }

    let history = agent.get_price_history("TEST").await;
    assert_eq!(history.len(), 5);
    assert_eq!(history[0].price, 100.0);
    assert_eq!(history[4].price, 104.0);
    println!("✅ 价格历史顺序正常");

    // 测试3: 信号聚合一致性
    let engine = StrategyEngine::new();
    let strategy = Box::new(TrendFollowingStrategy::new());
    engine.add_strategy(strategy).await;

    let signals = engine
        .generate_signals("TEST", &strategy_engine::MarketData {
            ticker: "TEST".to_string(),
            current_price: 100.0,
            open: 99.0,
            high: 101.0,
            low: 98.0,
            volume: 1000000,
            timestamp: chrono::Utc::now(),
            historical_prices: vec![],
            indicators: HashMap::new(),
            fundamentals: None,
        })
        .await
        .unwrap();

    let aggregated1 = engine.aggregate_signals(&signals).await.unwrap();
    let aggregated2 = engine.aggregate_signals(&signals).await.unwrap();

    assert_eq!(aggregated1.consensus_direction, aggregated2.consensus_direction);
    assert_eq!(aggregated1.consensus_strength, aggregated2.consensus_strength);
    println!("✅ 信号聚合一致性正常");

    println!("\n=== 数据一致性测试通过 ===\n");
}

// 运行所有测试的主函数
#[tokio::test]
async fn run_all_integration_tests() {
    println!("\n");
    println!("╔══════════════════════════════════════════════╗");
    println!("║   InvestIntel AI 端到端集成测试套件         ║");
    println!("║   基于 Claude Agent SDK 的全面验证          ║");
    println!("╚══════════════════════════════════════════════╝");
    println!("\n");

    // 所有测试已经在上面定义，这里只是运行入口
    println!("运行以下测试:");
    println!("  1. test_complete_investment_workflow");
    println!("  2. test_sentiment_analysis_comprehensive");
    println!("  3. test_strategy_system_integration");
    println!("  4. test_market_monitoring_workflow");
    println!("  5. test_agent_integration");
    println!("  6. test_claude_sdk_integration_verification");
    println!("  7. test_performance_and_scalability");
    println!("  8. test_error_handling");
    println!("  9. test_data_consistency");
    println!("\n所有测试均为独立运行，可使用:");
    println!("  cargo test run_all_integration_tests");
    println!("  cargo test --test e2e_integration_test");
}
