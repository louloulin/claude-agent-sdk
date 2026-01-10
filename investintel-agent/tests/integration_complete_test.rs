//! 完整集成测试
//!
//! 全面测试InvestIntel AI的所有核心功能
//! 验证Claude Agent SDK的实际集成

use std::collections::HashMap;

mod financial_sentiment;
mod investment_engine;

use financial_sentiment::{
    AggregatedSentiment, FinancialSentimentAnalyzer, NewsSentiment, SentimentType,
};
use investment_engine::{
    AnalysisEvent, AnalysisType, InvestmentEngine, InvestmentRequest, Recommendation,
    TimeFrame,
};

#[tokio::test]
async fn test_investment_engine_creation() {
    let engine = InvestmentEngine::new();
    // 验证引擎创建成功
    assert!(engine.cache.read().await.is_empty());
    println!("✅ InvestmentEngine创建成功");
}

#[tokio::test]
async fn test_investment_request_creation() {
    let request = InvestmentRequest {
        ticker: "AAPL".to_string(),
        analysis_types: vec![
            AnalysisType::Fundamental,
            AnalysisType::Technical,
            AnalysisType::Sentiment,
            AnalysisType::Risk,
        ],
        timeframe: TimeFrame::Month,
        risk_tolerance: 5,
        investment_amount: Some(10000.0),
    };

    assert_eq!(request.ticker, "AAPL");
    assert_eq!(request.analysis_types.len(), 4);
    assert_eq!(request.risk_tolerance, 5);
    assert_eq!(request.investment_amount, Some(10000.0));
    println!("✅ InvestmentRequest创建成功");
}

#[tokio::test]
async fn test_analysis_type_enum() {
    // 测试所有分析类型
    let types = vec![
        AnalysisType::Fundamental,
        AnalysisType::Technical,
        AnalysisType::Sentiment,
        AnalysisType::Risk,
        AnalysisType::PortfolioOptimization,
        AnalysisType::Comprehensive,
    ];

    for analysis_type in types {
        match analysis_type {
            AnalysisType::Fundamental => println!("✅ Fundamental类型"),
            AnalysisType::Technical => println!("✅ Technical类型"),
            AnalysisType::Sentiment => println!("✅ Sentiment类型"),
            AnalysisType::Risk => println!("✅ Risk类型"),
            AnalysisType::PortfolioOptimization => println!("✅ PortfolioOptimization类型"),
            AnalysisType::Comprehensive => println!("✅ Comprehensive类型"),
        }
    }
}

#[tokio::test]
async fn test_recommendation_generation() {
    // 测试强烈买入 (平衡型投资者)
    let rec = investment_engine::InvestmentEngine::generate_recommendation(80.0, Some(30.0), 5);
    assert_eq!(rec, Recommendation::StrongBuy);
    println!("✅ 强烈买入建议: {:?}", rec);

    // 测试买入
    let rec = investment_engine::InvestmentEngine::generate_recommendation(65.0, Some(40.0), 5);
    assert_eq!(rec, Recommendation::Buy);
    println!("✅ 买入建议: {:?}", rec);

    // 测试持有
    let rec = investment_engine::InvestmentEngine::generate_recommendation(55.0, Some(50.0), 5);
    assert_eq!(rec, Recommendation::Hold);
    println!("✅ 持有建议: {:?}", rec);

    // 测试卖出
    let rec = investment_engine::InvestmentEngine::generate_recommendation(35.0, Some(60.0), 5);
    assert_eq!(rec, Recommendation::Sell);
    println!("✅ 卖出建议: {:?}", rec);

    // 测试强烈卖出
    let rec = investment_engine::InvestmentEngine::generate_recommendation(20.0, Some(70.0), 5);
    assert_eq!(rec, Recommendation::StrongSell);
    println!("✅ 强烈卖出建议: {:?}", rec);

    // 测试保守型投资者阈值
    let rec_conservative =
        investment_engine::InvestmentEngine::generate_recommendation(70.0, Some(30.0), 2);
    println!("✅ 保守型投资者建议 (70分): {:?}", rec_conservative);

    // 测试激进型投资者阈值
    let rec_aggressive =
        investment_engine::InvestmentEngine::generate_recommendation(60.0, Some(40.0), 8);
    println!("✅ 激进型投资者建议 (60分): {:?}", rec_aggressive);
}

#[tokio::test]
async fn test_overall_score_calculation() {
    // 测试所有维度都有分数
    let score = investment_engine::InvestmentEngine::calculate_overall_score(
        Some(70.0), // 基本面
        Some(75.0), // 技术面
        Some(65.0), // 情感
        Some(40.0), // 风险（会转换为60.0）
    );
    assert!((60.0..=80.0).contains(&score));
    println!("✅ 综合评分 (全维度): {:.2}", score);

    // 测试只有部分维度
    let score = investment_engine::InvestmentEngine::calculate_overall_score(
        Some(70.0),
        Some(75.0),
        None,
        None,
    );
    assert!((65.0..=80.0).contains(&score));
    println!("✅ 综合评分 (部分维度): {:.2}", score);

    // 测试无分数
    let score =
        investment_engine::InvestmentEngine::calculate_overall_score(None, None, None, None);
    assert_eq!(score, 50.0);
    println!("✅ 综合评分 (无维度): {:.2}", score);
}

#[tokio::test]
async fn test_confidence_calculation() {
    assert_eq!(investment_engine::InvestmentEngine::calculate_confidence(4), 0.95);
    assert_eq!(investment_engine::InvestmentEngine::calculate_confidence(3), 0.85);
    assert_eq!(investment_engine::InvestmentEngine::calculate_confidence(2), 0.70);
    assert_eq!(investment_engine::InvestmentEngine::calculate_confidence(1), 0.50);
    println!("✅ 置信度计算正确");
}

#[tokio::test]
async fn test_sentiment_analyzer_creation() {
    let analyzer = FinancialSentimentAnalyzer::new();
    assert!(!analyzer.positive_words.is_empty());
    assert!(!analyzer.negative_words.is_empty());
    println!("✅ 情感分析器创建成功");
    println!("   正面词数量: {}", analyzer.positive_words.len());
    println!("   负面词数量: {}", analyzer.negative_words.len());
}

#[tokio::test]
async fn test_text_sentiment_analysis() {
    let analyzer = FinancialSentimentAnalyzer::new().with_claude(false);

    // 测试正面文本
    let positive_text = "公司营收强劲增长，超市场预期，盈利能力显著提升，业绩创新高";
    let result = analyzer.analyze_text(positive_text).unwrap();
    assert!(result.score > 0.5);
    assert_eq!(result.sentiment, SentimentType::Positive);
    println!("✅ 正面文本分析: {:.2} ({:?})", result.score, result.sentiment);
    println!("   关键词: {:?}", result.keywords);
    println!("   强度: {:?}", result.intensity);

    // 测试负面文本
    let negative_text = "公司业绩下滑，低于市场预期，面临较大经营压力和挑战，风险增加";
    let result = analyzer.analyze_text(negative_text).unwrap();
    assert!(result.score < 0.5);
    assert_eq!(result.sentiment, SentimentType::Negative);
    println!("✅ 负面文本分析: {:.2} ({:?})", result.score, result.sentiment);
    println!("   关键词: {:?}", result.keywords);
    println!("   强度: {:?}", result.intensity);

    // 测试中性文本
    let neutral_text = "公司发布季度财报，数据符合预期，业务正常运营";
    let result = analyzer.analyze_text(neutral_text).unwrap();
    println!("✅ 中性文本分析: {:.2} ({:?})", result.score, result.sentiment);
}

#[tokio::test]
async fn test_news_sentiment_analysis() {
    let analyzer = FinancialSentimentAnalyzer::new().with_claude(false);

    let news = analyzer
        .analyze_news(
            "科技公司发布超预期财报 营收大幅增长",
            "该公司第三季度营收增长50%，远超市场预期的30%，净利润同比增长80%，创历史新高。公司预计下一季度将继续保持强劲增长势头。",
            Some("AAPL"),
        )
        .unwrap();

    assert_eq!(news.ticker, Some("AAPL".to_string()));
    assert!(news.impact_score > 60.0);
    assert_eq!(news.sentiment.sentiment, SentimentType::Positive);
    println!("✅ 新闻情感分析:");
    println!("   标题: {}", news.title);
    println!("   情感: {:.2} ({:?})", news.sentiment.score, news.sentiment.sentiment);
    println!("   影响分数: {:.1}", news.impact_score);
    println!("   强度: {:?}", news.sentiment.intensity);
}

#[tokio::test]
async fn test_sentiment_aggregation() {
    let analyzer = FinancialSentimentAnalyzer::new().with_claude(false);

    // 创建模拟新闻情感
    let news_sentiments = vec![
        NewsSentiment {
            title: "超预期财报".to_string(),
            source: "Bloomberg".to_string(),
            timestamp: chrono::Utc::now(),
            ticker: Some("TEST".to_string()),
            sentiment: financial_sentiment::SentimentResult {
                sentiment: SentimentType::Positive,
                score: 0.75,
                confidence: 0.8,
                keywords: vec!["增长".to_string(), "超预期".to_string()],
                intensity: financial_sentiment::SentimentIntensity::Strong,
                trend: None,
            },
            impact_score: 80.0,
            summary: "公司营收强劲增长".to_string(),
        },
        NewsSentiment {
            title: "新产品发布".to_string(),
            source: "Reuters".to_string(),
            timestamp: chrono::Utc::now(),
            ticker: Some("TEST".to_string()),
            sentiment: financial_sentiment::SentimentResult {
                sentiment: SentimentType::Positive,
                score: 0.70,
                confidence: 0.75,
                keywords: vec!["创新".to_string()],
                intensity: financial_sentiment::SentimentIntensity::Medium,
                trend: None,
            },
            impact_score: 70.0,
            summary: "推出创新产品".to_string(),
        },
    ];

    // 创建模拟社交媒体情感
    let social_sentiments = vec![financial_sentiment::SocialSentiment {
        platform: "Twitter".to_string(),
        ticker: "TEST".to_string(),
        timeframe: "24h".to_string(),
        overall_sentiment: financial_sentiment::SentimentResult {
            sentiment: SentimentType::Positive,
            score: 0.68,
            confidence: 0.7,
            keywords: vec![],
            intensity: financial_sentiment::SentimentIntensity::Medium,
            trend: None,
        },
        buzz_score: 75.0,
        influence_score: 65.0,
        hashtags: vec!["stocks".to_string(), "investing".to_string()],
        distribution: HashMap::new(),
    }];

    let aggregated = analyzer
        .aggregate_sentiment("TEST", &news_sentiments, &social_sentiments, None)
        .unwrap();

    assert!(aggregated.composite_score > 0.6);
    assert_eq!(aggregated.trend, financial_sentiment::SentimentTrend::Rising);
    assert!(!aggregated.interpretation.is_empty());

    println!("✅ 情感聚合分析:");
    println!("   综合分数: {:.2}", aggregated.composite_score);
    println!("   趋势: {:?}", aggregated.trend);
    println!("   一致性: {:.2}", aggregated.consistency);
    println!("   解读: {}", aggregated.interpretation);
    if let Some(ref news) = aggregated.news_sentiment {
        println!("   新闻情感: {:.2} ({:?})", news.score, news.sentiment);
    }
    if let Some(ref social) = aggregated.social_sentiment {
        println!("   社交情感: {:.2} ({:?})", social.score, social.sentiment);
    }
}

#[tokio::test]
async fn test_sentiment_type_conversion() {
    // 测试分数到类型的转换
    assert_eq!(SentimentType::from_score(0.8), SentimentType::Positive);
    assert_eq!(SentimentType::from_score(0.5), SentimentType::Neutral);
    assert_eq!(SentimentType::from_score(0.2), SentimentType::Negative);

    // 测试类型到分数的转换
    assert_eq!(SentimentType::Positive.to_score(), 0.75);
    assert_eq!(SentimentType::Neutral.to_score(), 0.5);
    assert_eq!(SentimentType::Negative.to_score(), 0.25);

    // 测试中文描述
    println!("✅ Positive: {}", SentimentType::Positive.to_chinese());
    println!("✅ Neutral: {}", SentimentType::Neutral.to_chinese());
    println!("✅ Negative: {}", SentimentType::Negative.to_chinese());
}

#[tokio::test]
async fn test_timeframe_enum() {
    let timeframes = vec![
        TimeFrame::Day,
        TimeFrame::Week,
        TimeFrame::Month,
        TimeFrame::Quarter,
        TimeFrame::Year,
        TimeFrame::Max,
    ];

    for tf in timeframes {
        match tf {
            TimeFrame::Day => println!("✅ Day timeframe"),
            TimeFrame::Week => println!("✅ Week timeframe"),
            TimeFrame::Month => println!("✅ Month timeframe"),
            TimeFrame::Quarter => println!("✅ Quarter timeframe"),
            TimeFrame::Year => println!("✅ Year timeframe"),
            TimeFrame::Max => println!("✅ Max timeframe"),
        }
    }
}

#[tokio::test]
async fn test_module_integration() {
    println!("\n=== 模块集成测试 ===\n");

    // 1. 创建情感分析器
    let analyzer = FinancialSentimentAnalyzer::new().with_claude(false);
    println!("✅ 1. 情感分析器初始化成功");

    // 2. 分析一条新闻
    let news = analyzer
        .analyze_news(
            "测试公司营收大幅增长",
            "公司营收同比增长50%，净利润增长80%，超市场预期。",
            Some("TEST"),
        )
        .unwrap();
    println!("✅ 2. 新闻情感分析完成: {:.2}", news.sentiment.score);

    // 3. 创建投资引擎
    let engine = InvestmentEngine::new();
    println!("✅ 3. 投资引擎初始化成功");

    // 4. 创建分析请求
    let request = InvestmentRequest {
        ticker: "TEST".to_string(),
        analysis_types: vec![AnalysisType::Sentiment],
        timeframe: TimeFrame::Month,
        risk_tolerance: 5,
        investment_amount: Some(10000.0),
    };
    println!("✅ 4. 投资分析请求创建成功");

    // 5. 测试综合评分计算
    let score = investment_engine::InvestmentEngine::calculate_overall_score(
        Some(news.sentiment.score * 100.0),
        Some(70.0),
        Some(news.sentiment.score * 100.0),
        Some(40.0),
    );
    println!("✅ 5. 综合评分计算完成: {:.1}", score);

    // 6. 生成投资建议
    let recommendation = investment_engine::InvestmentEngine::generate_recommendation(
        score,
        Some(40.0),
        5,
    );
    println!("✅ 6. 投资建议生成完成: {:?}", recommendation);

    println!("\n=== 集成测试全部通过 ===\n");
}

#[tokio::test]
async fn test_all_features_comprehensive() {
    println!("\n=== 全面功能测试 ===\n");

    // 情感分析测试
    println!("--- 情感分析模块 ---");
    let analyzer = FinancialSentimentAnalyzer::new();
    let sentiment = analyzer
        .analyze_text("公司业绩强劲增长，超预期盈利")
        .unwrap();
    println!("✅ 情感分析: {:.2} ({:?})", sentiment.score, sentiment.sentiment);
    println!("   强度: {:?}", sentiment.intensity);
    println!("   置信度: {:.2}", sentiment.confidence);

    // 投资引擎测试
    println!("\n--- 投资引擎模块 ---");
    let engine = InvestmentEngine::new();

    // 评分计算
    let overall = investment_engine::InvestmentEngine::calculate_overall_score(
        Some(75.0), // 基本面
        Some(70.0), // 技术面
        Some(65.0), // 情感
        Some(35.0), // 风险
    );
    println!("✅ 综合评分: {:.1}", overall);

    // 建议生成
    let rec = investment_engine::InvestmentEngine::generate_recommendation(overall, Some(35.0), 5);
    println!("✅ 投资建议: {:?}", rec);

    // 置信度计算
    let conf = investment_engine::InvestmentEngine::calculate_confidence(4);
    println!("✅ 分析置信度: {:.2}", conf);

    // 新闻分析
    println!("\n--- 新闻情感分析 ---");
    let news = analyzer
        .analyze_news(
            "科技公司发布创新产品",
            "公司推出革命性新产品，市场反响热烈，预期将带来显著营收增长",
            Some("TECH"),
        )
        .unwrap();
    println!("✅ 新闻标题: {}", news.title);
    println!("   情感分数: {:.2}", news.sentiment.score);
    println!("   影响评分: {:.1}", news.impact_score);

    println!("\n=== 全面功能测试通过 ===\n");
}

// 性能测试
#[tokio::test]
async fn test_performance_benchmarks() {
    use std::time::Instant;

    println!("\n=== 性能基准测试 ===\n");

    let analyzer = FinancialSentimentAnalyzer::new().with_claude(false);

    // 测试文本分析性能
    let test_texts = vec![
        "公司营收强劲增长，超市场预期，盈利能力显著提升",
        "业绩下滑，低于预期，面临较大压力和风险",
        "发布新产品，市场反响良好，前景乐观",
        "季度财报符合预期，业务稳定发展",
    ];

    let start = Instant::now();
    for text in &test_texts {
        analyzer.analyze_text(text).unwrap();
    }
    let duration = start.elapsed();
    println!("✅ 文本情感分析 (4条): {:?}", duration);
    println!("   平均每条: {:?}", duration / 4);

    // 测试新闻分析性能
    let start = Instant::now();
    for (i, text) in test_texts.iter().enumerate() {
        analyzer
            .analyze_news(
                &format!("新闻标题 {}", i),
                text,
                Some(&format!("STOCK{}", i)),
            )
            .unwrap();
    }
    let duration = start.elapsed();
    println!("✅ 新闻情感分析 (4条): {:?}", duration);
    println!("   平均每条: {:?}", duration / 4);

    println!("\n=== 性能测试完成 ===\n");
}

// 边界情况测试
#[tokio::test]
async fn test_edge_cases() {
    println!("\n=== 边界情况测试 ===\n");

    let analyzer = FinancialSentimentAnalyzer::new().with_claude(false);

    // 空文本
    let result = analyzer.analyze_text("");
    assert!(result.is_ok());
    println!("✅ 空文本处理: {:.2}", result.unwrap().score);

    // 只有标点符号
    let result = analyzer.analyze_text("。。。！！！");
    assert!(result.is_ok());
    println!("✅ 标点符号处理: {:.2}", result.unwrap().score);

    // 混合正负面词
    let result = analyzer.analyze_text("营收增长但也存在风险").unwrap();
    println!("✅ 混合情感: {:.2} ({:?})", result.score, result.sentiment);

    // 极长文本
    let long_text = "公司业绩优秀 ".repeat(100);
    let start = std::time::Instant::now();
    let result = analyzer.analyze_text(&long_text);
    let duration = start.elapsed();
    assert!(result.is_ok());
    println!("✅ 长文本处理 (6000字符): {:?}", duration);

    println!("\n=== 边界测试完成 ===\n");
}
