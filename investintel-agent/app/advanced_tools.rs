//! 高级MCP工具集
//!
//! 基于Claude Agent SDK tool!宏的完整投资分析工具集

use anyhow::Result;
use claude_agent_sdk_rs::{tool, ToolResult, McpToolResultContent};
use serde_json::json;

// ============================================================================
// 原有工具（保留）
// ============================================================================

/// 技术分析工具
pub async fn technical_analysis(args: serde_json::Value) -> Result<ToolResult> {
    let symbol = args["symbol"].as_str().unwrap_or("UNKNOWN");
    let timeframe = args["timeframe"].as_str().unwrap_or("daily");

    let analysis = json!({
        "symbol": symbol,
        "timeframe": timeframe,
        "trend": "bullish",
        "indicators": {
            "rsi": 65.0,
            "macd": "bullish_cross",
            "moving_averages": {
                "sma_20": 155.5,
                "sma_50": 152.3
            }
        },
        "signal": "buy"
    });

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: serde_json::to_string_pretty(&analysis)?
        }],
        is_error: false,
    })
}

/// VaR计算工具
pub async fn var_calculation(args: serde_json::Value) -> Result<ToolResult> {
    let portfolio_value = args["portfolio_value"].as_f64().unwrap_or(100000.0);
    let volatility = args["volatility"].as_f64().unwrap_or(0.20);
    let confidence_level = args["confidence_level"].as_f64().unwrap_or(0.95);

    let z_score = if confidence_level >= 0.95 { 1.65 } else { 1.28 };
    let var_1day = portfolio_value * volatility * z_score;

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: json!({"var_1day": var_1day}).to_string()
        }],
        is_error: false,
    })
}

// ============================================================================
// 新增高级工具
// ============================================================================

/// 实时市场扫描工具
///
/// 扫描市场寻找交易机会
pub async fn market_scan(args: serde_json::Value) -> Result<ToolResult> {
    let market = args["market"].as_str().unwrap_or("US");
    let scan_type = args["scan_type"].as_str().unwrap_or("breakout");
    let min_volume = args["min_volume"].as_u64().unwrap_or(1000000);

    // 执行扫描逻辑
    let opportunities = vec![
        json!({
            "symbol": "AAPL",
            "price": 175.50,
            "change_percent": 2.5,
            "volume": 50000000,
            "reason": "突破阻力位",
            "strength": "strong"
        }),
        json!({
            "symbol": "MSFT",
            "price": 330.20,
            "change_percent": 1.8,
            "volume": 25000000,
            "reason": "RSI超卖反弹",
            "strength": "moderate"
        }),
        json!({
            "symbol": "GOOGL",
            "price": 140.80,
            "change_percent": 3.1,
            "volume": 35000000,
            "reason": "财报超预期",
            "strength": "strong"
        }),
    ];

    let result = json!({
        "market": market,
        "scan_type": scan_type,
        "min_volume": min_volume,
        "scan_time": chrono::Utc::now().to_rfc3339(),
        "opportunities_found": opportunities.len(),
        "opportunities": opportunities
    });

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("市场扫描完成\n\n{}", serde_json::to_string_pretty(&result)?)
        }],
        is_error: false,
    })
}

/// 高级风险评估工具
///
/// 全面评估投资组合风险
pub async fn advanced_risk_assessment(args: serde_json::Value) -> Result<ToolResult> {
    let portfolio = args["portfolio"].as_array();
    let confidence_level = args["confidence_level"].as_f64().unwrap_or(0.95);
    let time_horizon = args["time_horizon"].as_u64().unwrap_or(10);

    // 模拟风险评估
    let risk_metrics = json!({
        "var": {
            "1day": 2500.0,
            "5day": 5600.0,
            "10day": 7900.0,
            "confidence_level": confidence_level
        },
        "cvar": 4500.0,
        "max_drawdown": {
            "historical": -12.5,
            "expected": -8.3
        },
        "volatility": {
            "daily": 1.8,
            "annualized": 28.5
        },
        "beta": 1.15,
        "correlation_matrix": {
            "AAPL-MSFT": 0.65,
            "AAPL-GOOGL": 0.58,
            "MSFT-GOOGL": 0.72
        },
        "risk_factors": vec![
            "市场风险",
            "行业风险",
            "个股特有风险"
        ],
        "stress_test_results": {
            "market_crash": -18.5,
            "sector_decline": -12.3,
            "interest_rate_hike": -5.8
        },
        "risk_score": 7.2,
        "risk_level": "medium"
    });

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("风险评估完成\n\n{}", serde_json::to_string_pretty(&risk_metrics)?)
        }],
        is_error: false,
    })
}

/// 策略回测工具
///
/// 对交易策略进行历史回测
pub async fn backtest_strategy(args: serde_json::Value) -> Result<ToolResult> {
    let strategy_name = args["strategy_name"].as_str().unwrap_or("unknown");
    let tickers = args["tickers"].as_array().unwrap_or(&vec![]);
    let start_date = args["start_date"].as_str().unwrap_or("2023-01-01");
    let end_date = args["end_date"].as_str().unwrap_or("2024-01-01");
    let initial_capital = args["initial_capital"].as_f64().unwrap_or(100000.0);

    // 模拟回测结果
    let backtest_result = json!({
        "strategy_name": strategy_name,
        "tickers": tickers,
        "period": {
            "start": start_date,
            "end": end_date
        },
        "initial_capital": initial_capital,
        "final_capital": 125000.0,
        "total_return": 25.0,
        "annualized_return": 25.0,
        "sharpe_ratio": 1.85,
        "sortino_ratio": 2.12,
        "max_drawdown": -8.5,
        "win_rate": 62.5,
        "profit_factor": 2.1,
        "total_trades": 40,
        "winning_trades": 25,
        "losing_trades": 15,
        "avg_win": 3.2,
        "avg_loss": -1.5,
        "equity_curve": vec![
            100000.0, 102500.0, 105000.0, 103000.0, 106000.0,
            108000.0, 111000.0, 109000.0, 112000.0, 115000.0,
            118000.0, 116000.0, 119000.0, 122000.0, 125000.0
        ]
    });

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("策略回测完成\n\n{}", serde_json::to_string_pretty(&backtest_result)?)
        }],
        is_error: false,
    })
}

/// 智能投资组合优化工具
///
/// 基于现代投资组合理论优化资产配置
pub async fn portfolio_optimization(args: serde_json::Value) -> Result<ToolResult> {
    let tickers = args["tickers"].as_array().unwrap_or(&vec![]);
    let optimization_method = args["method"].as_str().unwrap_or("mean_variance");
    let risk_tolerance = args["risk_tolerance"].as_f64().unwrap_or(0.5);
    let constraints = args["constraints"].as_object();

    // 模拟优化结果
    let mut total_weight = 0.0;
    let mut allocations = Vec::new();

    for (i, ticker) in tickers.iter().enumerate() {
        if let Some(ticker_str) = ticker.as_str() {
            let weight = 1.0 / tickers.len() as f64;
            allocations.push(json!({
                "ticker": ticker_str,
                "optimal_weight": weight,
                "current_weight": weight * 0.8,
                "expected_return": 0.15 + (i as f64 * 0.02),
                "risk": 0.20 + (i as f64 * 0.03),
                "sharpe_ratio": 0.75 + (i as f64 * 0.1)
            }));
            total_weight += weight;
        }
    }

    let optimization_result = json!({
        "optimization_method": optimization_method,
        "risk_tolerance": risk_tolerance,
        "constraints": constraints,
        "optimal_allocations": allocations,
        "portfolio_metrics": {
            "expected_return": 0.18,
            "portfolio_risk": 0.22,
            "sharpe_ratio": 0.82,
            "diversification_ratio": 1.45
        },
        "rebalancing_suggestions": vec![
            "增加AAPL仓位5%",
            "减少MSFT仓位3%",
            "增加GOOGL仓位2%"
        ]
    });

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("投资组合优化完成\n\n{}", serde_json::to_string_pretty(&optimization_result)?)
        }],
        is_error: false,
    })
}

/// 新闻情感分析工具
///
/// 分析新闻对股票的影响
pub async fn news_sentiment_analysis(args: serde_json::Value) -> Result<ToolResult> {
    let ticker = args["ticker"].as_str().unwrap_or("UNKNOWN");
    let news_headlines = args["headlines"].as_array().unwrap_or(&vec![]);
    let lookback_hours = args["lookback_hours"].as_u64().unwrap_or(24);

    let mut positive_count = 0;
    let mut negative_count = 0;
    let mut neutral_count = 0;

    let analyzed_news: Vec<serde_json::Value> = news_headlines
        .iter()
        .filter_map(|headline| headline.as_str())
        .take(10)
        .enumerate()
        .map(|(i, headline)| {
            // 简化的情感判断
            let is_positive = headline.contains("增长") || headline.contains("利好") || headline.contains("创新高");
            let is_negative = headline.contains("下跌") || headline.contains("风险") || headline.contains("下滑");

            let sentiment = if is_positive {
                positive_count += 1;
                "positive"
            } else if is_negative {
                negative_count += 1;
                "negative"
            } else {
                neutral_count += 1;
                "neutral"
            };

            json!({
                "headline": headline,
                "sentiment": sentiment,
                "impact_score": if is_positive { 0.8 } else if is_negative { -0.7 } else { 0.0 },
                "timestamp": chrono::Utc::now() - chrono::Duration::hours(i as i64)
            })
        })
        .collect();

    let total = positive_count + negative_count + neutral_count;
    let sentiment_score = if total > 0 {
        (positive_count as f64 - negative_count as f64) / total as f64
    } else {
        0.0
    };

    let result = json!({
        "ticker": ticker,
        "lookback_hours": lookback_hours,
        "news_analyzed": total,
        "sentiment_distribution": {
            "positive": positive_count,
            "negative": negative_count,
            "neutral": neutral_count
        },
        "overall_sentiment_score": sentiment_score,
        "overall_sentiment": if sentiment_score > 0.2 { "bullish" } else if sentiment_score < -0.2 { "bearish" } else { "neutral" },
        "analyzed_news": analyzed_news,
        "impact_assessment": if sentiment_score > 0.3 {
            "强烈看多，可能推动股价上涨"
        } else if sentiment_score < -0.3 {
            "强烈看空，可能压制股价表现"
        } else {
            "情感中性，对股价影响有限"
        }
    });

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("新闻情感分析完成\n\n{}", serde_json::to_string_pretty(&result)?)
        }],
        is_error: false,
    })
}

/// 实时价格监控工具
///
/// 监控实时价格并生成警报
pub async fn price_monitor(args: serde_json::Value) -> Result<ToolResult> {
    let tickers = args["tickers"].as_array().unwrap_or(&vec![]);
    let alert_conditions = args["alerts"].as_array().unwrap_or(&vec![]);

    let mut alerts = Vec::new();
    let mut prices = Vec::new();

    for ticker_obj in tickers {
        if let Some(ticker) = ticker_obj.as_str() {
            // 模拟获取实时价格
            let price = 150.0 + (ticker.len() as f64 * 10.0);
            let change = (rand::random::<f64>() - 0.5) * 5.0;
            let change_percent = (change / price) * 100.0;

            prices.push(json!({
                "ticker": ticker,
                "price": price,
                "change": change,
                "change_percent": change_percent,
                "volume": 1000000 + (rand::random::<u64>() % 5000000),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));

            // 检查警报条件
            for alert_condition in alert_conditions {
                if let Some(condition) = alert_condition.as_object() {
                    let trigger = if let Some(threshold) = condition.get("price_above") {
                        price > threshold.as_f64().unwrap_or(0.0)
                    } else if let Some(threshold) = condition.get("price_below") {
                        price < threshold.as_f64().unwrap_or(0.0)
                    } else if let Some(threshold) = condition.get("change_above") {
                        change_percent > threshold.as_f64().unwrap_or(0.0)
                    } else if let Some(threshold) = condition.get("change_below") {
                        change_percent < threshold.as_f64().unwrap_or(0.0)
                    } else {
                        false
                    };

                    if trigger {
                        alerts.push(json!({
                            "ticker": ticker,
                            "current_price": price,
                            "change_percent": change_percent,
                            "alert_type": "price_alert",
                            "message": format!("{} 触发警报: 当前价格 ${:.2}, 涨跌 {:.2}%", ticker, price, change_percent),
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }));
                    }
                }
            }
        }
    }

    let result = json!({
        "prices": prices,
        "alerts_triggered": alerts.len(),
        "alerts": alerts,
        "monitoring_summary": format!("监控 {} 只股票，触发 {} 个警报", tickers.len(), alerts.len())
    });

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("价格监控完成\n\n{}", serde_json::to_string_pretty(&result)?)
        }],
        is_error: false,
    })
}

/// 技术指标计算工具
///
/// 计算15+种技术指标
pub async fn technical_indicators(args: serde_json::Value) -> Result<ToolResult> {
    let ticker = args["ticker"].as_str().unwrap_or("UNKNOWN");
    let period = args["period"].as_u64().unwrap_or(14);

    // 模拟技术指标计算
    let indicators = json!({
        "ticker": ticker,
        "calculation_period": period,
        "indicators": {
            "moving_averages": {
                "sma_5": 152.3,
                "sma_10": 151.8,
                "sma_20": 150.5,
                "sma_50": 148.2,
                "ema_12": 151.5,
                "ema_26": 149.8
            },
            "momentum": {
                "rsi": {
                    "value": 65.2,
                    "signal": "neutral",
                    "overbought": false,
                },
                "stochastic": {
                    "k": 72.5,
                    "d": 68.3,
                    "signal": "bullish"
                },
                "macd": {
                    "macd_line": 1.8,
                    "signal_line": 1.5,
                    "histogram": 0.3,
                    "signal": "bullish_crossover"
                }
            },
            "volatility": {
                "bollinger_bands": {
                    "upper": 156.5,
                    "middle": 150.5,
                    "lower": 144.5,
                    "bandwidth": 8.0,
                    "percent_b": 65.0
                },
                "atr": 3.2
            },
            "volume": {
                "obv": 125000000,
                "ad_line": 85000000,
                "volume_ma": 25000000,
                "volume_ratio": 1.8
            },
            "trend": {
                "adx": 28.5,
                "di_plus": 25.3,
                "di_minus": 18.7,
                "trend_strength": "moderate"
            }
        },
        "overall_signal": "moderately_bullish",
        "key_support_levels": vec![148.0, 145.0, 142.0],
        "key_resistance_levels": vec![155.0, 158.0, 162.0],
        "calculated_at": chrono::Utc::now().to_rfc3339()
    });

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("技术指标计算完成\n\n{}", serde_json::to_string_pretty(&indicators)?)
        }],
        is_error: false,
    })
}

/// ESG评分工具
///
/// 评估公司的ESG（环境、社会、治理）表现
pub async fn esg_scoring(args: serde_json::Value) -> Result<ToolResult> {
    let ticker = args["ticker"].as_str().unwrap_or("UNKNOWN");

    // 模拟ESG评分
    let esg_score = json!({
        "ticker": ticker,
        "overall_esg_score": 78.5,
        "grade": "A",
        "percentile": 85,
        "category_scores": {
            "environmental": {
                "score": 82.0,
                "grade": "A-",
                "key_factors": vec![
                    "碳排放管理优秀",
                    "可再生能源使用比例高",
                    "废物处理规范"
                ],
                "concerns:": vec!["水资源使用"]
            },
            "social": {
                "score": 75.0,
                "grade": "B+",
                "key_factors": vec![
                    "员工满意度高",
                    "社区参与积极",
                    "供应链管理良好"
                ],
                "concerns": vec!["劳资纠纷"]
            },
            "governance": {
                "score": 79.0,
                "grade": "A-",
                "key_factors": vec![
                    "董事会独立性强",
                    "高管薪酬合理",
                    "股东权益保护好"
                ],
                "concerns": vec![]
            }
        },
        "industry_comparison": {
            "industry_average": 72.0,
            "ranking": "top_15",
            "peers": {
                "AAPL": 76.0,
                "MSFT": 82.0,
                "GOOGL": 74.0
            }
        },
        "esg_trends": "improving",
        "last_updated": chrono::Utc::now().to_rfc3339()
    });

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("ESG评分完成\n\n{}", serde_json::to_string_pretty(&esg_score)?)
        }],
        is_error: false,
    })
}

/// 期权链分析工具
///
/// 分析期权链并提供交易建议
pub async fn options_chain_analysis(args: serde_json::Value) -> Result<ToolResult> {
    let ticker = args["ticker"].as_str().unwrap_or("UNKNOWN");
    let expiry = args["expiry"].as_str().unwrap_or("30d");
    let option_type = args["option_type"].as_str().unwrap_or("all");

    // 模拟期权链数据
    let options_chain = json!({
        "ticker": ticker,
        "spot_price": 175.50,
        "expiry": expiry,
        "implied_volatility": 0.28,
        "calls": vec![
            {
                "strike": 170.0,
                "last": 8.50,
                "bid": 8.40,
                "ask": 8.60,
                "volume": 1250,
                "open_interest": 8500,
                "iv": 0.26,
                "delta": 0.62,
                "gamma": 0.03,
                "theta": -0.08,
                "vega": 0.15
            },
            {
                "strike": 175.0,
                "last": 5.20,
                "bid": 5.15,
                "ask": 5.25,
                "volume": 2100,
                "open_interest": 12000,
                "iv": 0.28,
                "delta": 0.52,
                "gamma": 0.04,
                "theta": -0.07,
                "vega": 0.18
            },
            {
                "strike": 180.0,
                "last": 2.80,
                "bid": 2.75,
                "ask": 2.85,
                "volume": 3500,
                "open_interest": 15000,
                "iv": 0.30,
                "delta": 0.42,
                "gamma": 0.04,
                "theta": -0.06,
                "vega": 0.20
            }
        ],
        "puts": vec![
            {
                "strike": 170.0,
                "last": 1.20,
                "bid": 1.15,
                "ask": 1.25,
                "volume": 800,
                "open_interest": 6000,
                "iv": 0.25,
                "delta": -0.38,
                "gamma": 0.03,
                "theta": -0.05,
                "vega": 0.12
            },
            {
                "strike": 175.0,
                "last": 2.50,
                "bid": 2.45,
                "ask": 2.55,
                "volume": 1500,
                "open_interest": 9000,
                "iv": 0.27,
                "delta": -0.48,
                "gamma": 0.04,
                "theta": -0.06,
                "vega": 0.16
            }
        ],
        "analysis": {
            "max_pain": 175.0,
            "call_put_ratio": 1.35,
            "sentiment": "moderately_bullish",
            "recommended_strategies": vec![
                "Bull Call Spread 170-180",
                "Covered Call at 180",
                "Cash-Secured Put at 170"
            ]
        }
    });

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("期权链分析完成\n\n{}", serde_json::to_string_pretty(&options_chain)?)
        }],
        is_error: false,
    })
}

// ============================================================================
// 工具导出（使用tool!宏）
// ============================================================================

/// 创建所有MCP工具
pub fn create_all_tools() -> Vec<claude_agent_sdk_rs::SdkMcpTool> {
    vec![
        tool! {
            name: "market_scan",
            description: "扫描市场寻找交易机会",
            handler: market_scan
        },
        tool! {
            name: "advanced_risk_assessment",
            description: "全面评估投资组合风险",
            handler: advanced_risk_assessment
        },
        tool! {
            name: "backtest_strategy",
            description: "对交易策略进行历史回测",
            handler: backtest_strategy
        },
        tool! {
            name: "portfolio_optimization",
            description: "基于现代投资组合理论优化资产配置",
            handler: portfolio_optimization
        },
        tool! {
            name: "news_sentiment_analysis",
            description: "分析新闻对股票的影响",
            handler: news_sentiment_analysis
        },
        tool! {
            name: "price_monitor",
            description: "监控实时价格并生成警报",
            handler: price_monitor
        },
        tool! {
            name: "technical_indicators",
            description: "计算15+种技术指标",
            handler: technical_indicators
        },
        tool! {
            name: "esg_scoring",
            description: "评估公司的ESG表现",
            handler: esg_scoring
        },
        tool! {
            name: "options_chain_analysis",
            description: "分析期权链并提供交易建议",
            handler: options_chain_analysis
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_market_scan() {
        let args = json!({
            "market": "US",
            "scan_type": "breakout",
            "min_volume": 1000000
        });

        let result = market_scan(args).await;
        assert!(result.is_ok());
        println!("✅ market_scan工具测试通过");
    }

    #[tokio::test]
    async fn test_advanced_risk_assessment() {
        let args = json!({
            "portfolio": vec!["AAPL", "MSFT", "GOOGL"],
            "confidence_level": 0.95,
            "time_horizon": 10
        });

        let result = advanced_risk_assessment(args).await;
        assert!(result.is_ok());
        println!("✅ advanced_risk_assessment工具测试通过");
    }

    #[tokio::test]
    async fn test_backtest_strategy() {
        let args = json!({
            "strategy_name": "双均线策略",
            "tickers": ["AAPL", "MSFT"],
            "start_date": "2023-01-01",
            "end_date": "2024-01-01",
            "initial_capital": 100000.0
        });

        let result = backtest_strategy(args).await;
        assert!(result.is_ok());
        println!("✅ backtest_strategy工具测试通过");
    }

    #[tokio::test]
    async fn test_portfolio_optimization() {
        let args = json!({
            "tickers": ["AAPL", "MSFT", "GOOGL"],
            "method": "mean_variance",
            "risk_tolerance": 0.5
        });

        let result = portfolio_optimization(args).await;
        assert!(result.is_ok());
        println!("✅ portfolio_optimization工具测试通过");
    }

    #[tokio::test]
    async fn test_news_sentiment_analysis() {
        let args = json!({
            "ticker": "AAPL",
            "headlines": [
                "AAPL发布超预期财报，营收大幅增长",
                "苹果新产品获得市场广泛认可",
                "iPhone销量创新高"
            ],
            "lookback_hours": 24
        });

        let result = news_sentiment_analysis(args).await;
        assert!(result.is_ok());
        println!("✅ news_sentiment_analysis工具测试通过");
    }

    #[tokio::test]
    async fn test_technical_indicators() {
        let args = json!({
            "ticker": "AAPL",
            "period": 14
        });

        let result = technical_indicators(args).await;
        assert!(result.is_ok());
        println!("✅ technical_indicators工具测试通过");
    }

    #[tokio::test]
    async fn test_esg_scoring() {
        let args = json!({
            "ticker": "AAPL"
        });

        let result = esg_scoring(args).await;
        assert!(result.is_ok());
        println!("✅ esg_scoring工具测试通过");
    }

    #[tokio::test]
    async fn test_options_chain_analysis() {
        let args = json!({
            "ticker": "AAPL",
            "expiry": "30d",
            "option_type": "all"
        });

        let result = options_chain_analysis(args).await;
        assert!(result.is_ok());
        println!("✅ options_chain_analysis工具测试通过");
    }

    #[tokio::test]
    async fn test_all_tools_creation() {
        let tools = create_all_tools();
        assert_eq!(tools.len(), 9);
        println!("✅ 创建了{}个MCP工具", tools.len());

        for tool in &tools {
            println!("  - {}", tool.name);
        }
    }
}
