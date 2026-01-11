//! InvestIntel AI Investment Assistant Tests
//!
//! 集成测试验证投资助手的核心功能

use investintel_agent::InvestmentAssistant;

#[tokio::test]
async fn test_value_investment_agent() {
    // 测试价值投资Agent
    let agent = investintel_agent::ValueInvestmentAgent::new();

    let result = agent
        .execute(claude_agent_sdk_rs::orchestration::AgentInput::new("AAPL"))
        .await
        .unwrap();

    assert!(!result.content.is_empty());
    assert!(result.confidence > 0.0);
    assert!(result.content.contains("价值投资分析报告"));
}

#[tokio::test]
async fn test_investment_assistant_stock_analysis() {
    // 测试投资助手股票分析功能
    let assistant = InvestmentAssistant::new();

    let analysis = assistant.analyze_stock("AAPL").await.unwrap();

    assert_eq!(analysis.symbol, "AAPL");
    assert!(!analysis.value_analysis.is_empty());
    assert!(!analysis.trading_advice.is_empty());
}

#[tokio::test]
async fn test_investment_assistant_chat() {
    // 测试投资助手交互式咨询
    let assistant = InvestmentAssistant::new();

    // 测试股票分析
    let response1 = assistant.chat("分析AAPL").await.unwrap();
    assert!(response1.contains("AAPL"));

    // 测试一般建议
    let response2 = assistant.chat("有什么投资建议").await.unwrap();
    assert!(response2.contains("安全边际") || response2.contains("价值投资"));
}

#[tokio::test]
async fn test_graham_formula() {
    // 测试Graham公式计算
    let agent = investintel_agent::ValueInvestmentAgent::new();

    // V = EPS × (8.5 + 2g)
    // EPS = $10, g = 10% -> V = 10 × (8.5 + 20) = $285
    let intrinsic_value = agent.calculate_graham_intrinsic_value(10.0, 0.1);
    assert_eq!(intrinsic_value, 285.0);

    // 安全边际 = (100 - 70) / 100 = 30%
    let margin = agent.calculate_margin_of_safety(100.0, 70.0);
    assert_eq!(margin, 0.3);
}

#[tokio::test]
async fn test_portfolio_manager() {
    // 测试组合管理Agent
    use std::collections::HashMap;
    use investintel_agent::agents::{Portfolio, Holding, PortfolioManagerAgent};

    let agent = PortfolioManagerAgent::new();

    let mut target_weights = HashMap::new();
    target_weights.insert("AAPL".to_string(), 0.4);
    target_weights.insert("MSFT".to_string(), 0.3);
    target_weights.insert("GOOGL".to_string(), 0.3);

    let portfolio = Portfolio {
        holdings: vec![
            Holding {
                symbol: "AAPL".to_string(),
                shares: 100,
                value: 15000.0,
                cost_basis: 12000.0,
            },
            Holding {
                symbol: "MSFT".to_string(),
                shares: 50,
                value: 10000.0,
                cost_basis: 9000.0,
            },
            Holding {
                symbol: "GOOGL".to_string(),
                shares: 30,
                value: 5000.0,
                cost_basis: 6000.0,
            },
        ],
        target_weights,
    };

    let result = agent
        .execute(
            claude_agent_sdk_rs::orchestration::AgentInput::new("分析组合")
                .with_context(serde_json::to_value(portfolio).unwrap())
        )
        .await
        .unwrap();

    assert!(!result.content.is_empty());
    assert!(result.content.contains("投资组合分析报告"));
}

#[tokio::test]
async fn test_trading_advisor() {
    // 测试交易建议Agent
    use investintel_agent::agents::{TradingAdvisorAgent, InvestmentAction};

    let agent = TradingAdvisorAgent::new();

    let input = claude_agent_sdk_rs::orchestration::AgentInput::new("AAPL")
        .with_context(serde_json::json!({
            "action": "Buy",
            "current_price": 150.0,
            "confidence": 0.8
        }));

    let result = agent.execute(input).await.unwrap();

    assert!(!result.content.is_empty());
    assert!(result.content.contains("交易建议"));
}

#[tokio::test]
async fn test_end_to_end_flow() {
    // 端到端测试完整流程
    let assistant = InvestmentAssistant::new();

    // 1. 股票分析
    let stock_analysis = assistant.analyze_stock("MSFT").await.unwrap();
    assert!(!stock_analysis.value_analysis.is_empty());
    assert!(!stock_analysis.trading_advice.is_empty());

    // 2. 交互式咨询
    let chat_response = assistant.chat("MSFT值得投资吗").await.unwrap();
    assert!(!chat_response.is_empty());

    println!("✅ 端到端测试通过");
}

#[test]
fn test_investment_action_scoring() {
    // 测试投资行动评分
    use investintel_agent::agents::InvestmentAction;

    // 强烈买入
    assert!(matches!(
        InvestmentAction::from_score(0.9),
        InvestmentAction::StrongBuy
    ));

    // 买入
    assert!(matches!(
        InvestmentAction::from_score(0.7),
        InvestmentAction::Buy
    ));

    // 持有
    assert!(matches!(
        InvestmentAction::from_score(0.5),
        InvestmentAction::Hold
    ));

    // 卖出
    assert!(matches!(
        InvestmentAction::from_score(0.3),
        InvestmentAction::Sell
    ));

    // 强烈卖出
    assert!(matches!(
        InvestmentAction::from_score(0.1),
        InvestmentAction::StrongSell
    ));
}

#[tokio::test]
async fn test_dividend_investor_agent() {
    // 测试股息投资Agent
    let agent = investintel_agent::DividendInvestorAgent::new();

    let result = agent
        .execute(claude_agent_sdk_rs::orchestration::AgentInput::new("AAPL"))
        .await
        .unwrap();

    assert!(!result.content.is_empty());
    assert!(result.content.contains("股息投资分析报告"));
    assert!(result.confidence > 0.0);
}

#[tokio::test]
async fn test_dividend_safety_score() {
    // 测试股息安全性评分
    let agent = investintel_agent::DividendInvestorAgent::new();

    // 通过反射测试安全性评分计算
    // 高安全性: 低派息率+高增长
    let score_high = agent.calculate_safety_score(0.50, 0.10);
    assert_eq!(score_high, 5);

    // 中等安全性
    let score_mid = agent.calculate_safety_score(0.65, 0.06);
    assert!(score_mid >= 2 && score_mid <= 4);

    // 低安全性: 高派息率
    let score_low = agent.calculate_safety_score(0.80, 0.02);
    assert!(score_low <= 2);
}

#[tokio::test]
async fn test_dividend_compounding() {
    // 测试股息复利计算
    let agent = investintel_agent::DividendInvestorAgent::new();

    let result = agent.calculate_dividend_compounding(10000.0, 0.04, 10);

    // 复利效应: 10000 * (1.04)^10 - 10000
    let expected_reinvested = 10000.0 * 1.04_f64.powi(10) - 10000.0;

    assert!((result.reinvested_dividends - expected_reinvested).abs() < 1.0);
    assert!(result.total_return > 0.45); // >45%总回报
    assert_eq!(result.initial_investment, 10000.0);
}

#[tokio::test]
async fn test_investment_assistant_dividend_analysis() {
    // 测试投资助手的股息分析功能
    let assistant = investintel_agent::InvestmentAssistant::new();

    // 测试股息分析
    let response = assistant.chat("股息分析AAPL").await.unwrap();
    assert!(response.contains("AAPL"));
    assert!(response.contains("股息"));

    // 测试另一种询问方式
    let response2 = assistant.chat("AAPL的股息怎么样").await.unwrap();
    assert!(response2.contains("AAPL"));
}

#[tokio::test]
async fn test_kelly_position_agent() {
    // 测试Kelly仓位管理Agent
    let agent = investintel_agent::KellyPositionAgent::new();

    // 测试单标的仓位分析
    let output = agent
        .execute(claude_agent_sdk_rs::orchestration::AgentInput::new("AAPL"))
        .await
        .unwrap();

    assert!(!output.content.is_empty());
    assert!(output.content.contains("Kelly"));
    assert!(output.confidence > 0.0);
}

#[tokio::test]
async fn test_kelly_formula_calculation() {
    let agent = investintel_agent::KellyPositionAgent::new();

    // 测试基础Kelly公式
    let kelly = agent.calculate_kelly(0.6, 100.0, 50.0);
    assert!((kelly - 0.2).abs() < 0.01);

    // 测试Fractional Kelly
    let fractional = agent.apply_fractional_kelly(0.20);
    // 1/4 Kelly应该是5%
    assert!((fractional - 0.05).abs() < 0.01);
}

#[tokio::test]
async fn test_investment_assistant_kelly_analysis() {
    let assistant = investintel_agent::InvestmentAssistant::new();

    // 测试Kelly仓位分析
    let response = assistant.chat("Kelly仓位建议").await.unwrap();
    assert!(response.contains("Kelly"));
    assert!(response.contains("仓位"));

    // 测试另一种询问方式
    let response2 = assistant.chat("仓位分析AAPL").await.unwrap();
    assert!(response2.contains("Kelly"));
}

#[tokio::test]
async fn test_munger_framework_agent() {
    // 测试Munger框架Agent
    let agent = investintel_agent::MungerFrameworkAgent::new();

    // 测试Munger分析
    let output = agent
        .execute(claude_agent_sdk_rs::orchestration::AgentInput::new("AAPL"))
        .await
        .unwrap();

    assert!(!output.content.is_empty());
    assert!(output.content.contains("Munger"));
    assert!(output.confidence > 0.0);
}

#[tokio::test]
async fn test_munger_mental_models() {
    let agent = investintel_agent::MungerFrameworkAgent::new();

    // 测试思维模型数量
    assert_eq!(agent.mental_models.len(), 6);

    // 测试分析结果结构
    let analysis = agent.analyze_with_models("AAPL").await.unwrap();

    assert_eq!(analysis.symbol, "AAPL");
    assert!(!analysis.mental_model_insights.is_empty());
    assert!(analysis.average_score >= 0.0 && analysis.average_score <= 1.0);
    assert!(analysis.resonance_ratio >= 0.0 && analysis.resonance_ratio <= 1.0);
}

#[tokio::test]
async fn test_investment_assistant_munger_analysis() {
    let assistant = investintel_agent::InvestmentAssistant::new();

    // 测试Munger框架分析
    let response = assistant.chat("Munger分析AAPL").await.unwrap();
    assert!(response.contains("Munger"));
    assert!(response.contains("AAPL"));

    // 测试另一种询问方式
    let response2 = assistant.chat("思维模型分析").await.unwrap();
    assert!(response2.contains("Munger"));
}

#[tokio::test]
async fn test_end_to_end_three_in_one_analysis() {
    // 测试Graham-Buffett-Munger三位一体分析
    let assistant = investintel_agent::InvestmentAssistant::new();

    // 价值投资分析 (Graham-Buffett)
    let value_response = assistant.chat("分析AAPL").await.unwrap();
    assert!(value_response.contains("AAPL"));
    assert!(value_response.contains("价值分析"));

    // 股息投资分析
    let dividend_response = assistant.chat("股息分析AAPL").await.unwrap();
    assert!(dividend_response.contains("AAPL"));
    assert!(dividend_response.contains("股息"));

    // Munger框架分析 (Munger思维模型)
    let munger_response = assistant.chat("Munger分析AAPL").await.unwrap();
    assert!(munger_response.contains("AAPL"));
    assert!(munger_response.contains("Munger"));

    // Kelly仓位管理
    let kelly_response = assistant.chat("Kelly仓位建议").await.unwrap();
    assert!(kelly_response.contains("Kelly"));
}

#[tokio::test]
async fn test_complete_investment_workflow() {
    // 测试完整投资工作流
    let assistant = investintel_agent::InvestmentAssistant::new();

    // 1. 获取一般投资建议
    let advice = assistant.chat("有什么投资建议").await.unwrap();
    assert!(advice.contains("Graham"));
    assert!(advice.contains("Buffett"));
    assert!(advice.contains("Munger"));

    // 2. 分析具体股票
    let analysis = assistant.chat("分析MSFT").await.unwrap();
    assert!(analysis.contains("MSFT"));

    // 3. 评估股息
    let dividend = assistant.chat("MSFT的股息怎么样").await.unwrap();
    assert!(dividend.contains("MSFT"));

    // 4. Kelly仓位建议
    let kelly = assistant.chat("Kelly仓位建议").await.unwrap();
    assert!(kelly.contains("Kelly"));

    // 5. Munger思维模型分析
    let munger = assistant.chat("Munger分析MSFT").await.unwrap();
    assert!(munger.contains("Munger"));
}
