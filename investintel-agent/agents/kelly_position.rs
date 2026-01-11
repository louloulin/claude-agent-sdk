//! Kelly Position Management Agent - Kelly仓位管理Agent
//!
//! 基于Kelly准则的科学仓位管理，优化资金配置
//!
//! ## 核心功能
//!
//! 1. **Kelly公式计算** - 科学计算最优仓位比例
//! 2. **风险调整** - 考虑市场波动和不确定性
//! 3. **多资产组合** - 跨多个投资标的的Kelly配置
//! 4. **实用建议** - 保守Kelly和分批建仓策略

use crate::agents::{
    Agent, AgentInput, AgentOutput, Result, InvestmentRecommendation,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Kelly Position Management Agent
///
/// 基于Kelly准则的科学仓位管理Agent
///
/// ## Kelly公式
///
/// f* = (bp - q) / b
///
/// 其中:
/// - f* = 最优投资比例
/// - b = 盈亏比 (平均盈利/平均亏损)
/// - p = 胜率 (盈利概率)
/// - q = 败率 (1 - p)
///
/// ## 实用调整
///
/// 实际投资中建议使用 **Fractional Kelly** (半Kelly或1/4 Kelly)
/// 以降低风险，应对市场不确定性
pub struct KellyPositionAgent {
    /// Kelly保守系数 (0.25 = 1/4 Kelly, 0.5 = 半Kelly, 1.0 = 完整Kelly)
    kelly_fraction: f64,

    /// 最小仓位比例 (避免过度分散)
    min_position: f64,

    /// 最大仓位比例 (单标的)
    max_position: f64,

    /// 组合最大总仓位
    max_total_exposure: f64,
}

impl KellyPositionAgent {
    /// 创建新的Kelly仓位管理Agent
    pub fn new() -> Self {
        Self {
            kelly_fraction: 0.25, // 默认1/4 Kelly (保守策略)
            min_position: 0.02,   // 最小2%仓位
            max_position: 0.25,   // 最大25%仓位
            max_total_exposure: 1.0, // 最大100%总仓位
        }
    }

    /// 设置Kelly参数
    pub fn with_kelly_params(
        mut self,
        fraction: f64,
        min_position: f64,
        max_position: f64,
    ) -> Self {
        self.kelly_fraction = fraction;
        self.min_position = min_position;
        self.max_position = max_position;
        self
    }

    /// 计算完整Kelly公式
    ///
    /// f* = (bp - q) / b
    ///
    /// # 参数
    /// - `win_rate`: 胜率 (0.0 - 1.0)
    /// - `avg_win`: 平均盈利金额
    /// - `avg_loss`: 平均亏损金额 (正数)
    fn calculate_kelly(&self, win_rate: f64, avg_win: f64, avg_loss: f64) -> f64 {
        if avg_loss <= 0.0 || win_rate <= 0.0 || win_rate >= 1.0 {
            return 0.0;
        }

        let b = avg_win / avg_loss; // 盈亏比
        let p = win_rate; // 胜率
        let q = 1.0 - p; // 败率

        let kelly = (b * p - q) / b;

        // Kelly不能为负数
        kelly.max(0.0)
    }

    /// 应用保守Kelly系数
    ///
    /// 实际投资中建议使用Fractional Kelly降低风险
    fn apply_fractional_kelly(&self, kelly: f64) -> f64 {
        let adjusted = kelly * self.kelly_fraction;

        // 限制在合理范围内
        adjusted.clamp(self.min_position, self.max_position)
    }

    /// 基于预期收益和波动率计算Kelly
    ///
    /// 简化版本：使用预期收益率和波动率替代胜率和盈亏比
    ///
    /// # 参数
    /// - `expected_return`: 预期年化收益率 (如0.15表示15%)
    /// - `volatility`: 年化波动率 (如0.20表示20%)
    fn calculate_kelly_from_return_volatility(
        &self,
        expected_return: f64,
        volatility: f64,
    ) -> f64 {
        if volatility <= 0.0 {
            return 0.0;
        }

        // 简化Kelly: f = μ / σ²
        let kelly = expected_return / (volatility * volatility);

        // 应用保守系数
        self.apply_fractional_kelly(kelly)
    }

    /// 计算投资组合的Kelly配置
    ///
    /// 基于多个投资标的的预期收益、波动率和相关性
    fn calculate_portfolio_kelly(
        &self,
        opportunities: Vec<KellyOpportunity>,
    ) -> PortfolioKellyAllocation {
        let mut allocations = Vec::new();
        let mut total_kelly = 0.0;

        for opp in opportunities {
            // 计算每个标的的Kelly
            let kelly = self.calculate_kelly_from_return_volatility(
                opp.expected_return,
                opp.volatility,
            );

            // 调整后的Kelly
            let adjusted_kelly = self.apply_fractional_kelly(kelly);

            total_kelly += adjusted_kelly;

            allocations.push(KellyPositionAllocation {
                symbol: opp.symbol,
                raw_kelly: kelly,
                adjusted_kelly,
                expected_return: opp.expected_return,
                volatility: opp.volatility,
                risk_score: opp.volatility / opp.expected_return, // 风险收益比
            });
        }

        // 归一化到最大总仓位
        let scaling_factor = if total_kelly > self.max_total_exposure {
            self.max_total_exposure / total_kelly
        } else {
            1.0
        };

        for allocation in &mut allocations {
            allocation.final_allocation = allocation.adjusted_kelly * scaling_factor;
        }

        PortfolioKellyAllocation {
            allocations,
            total_raw_kelly: total_kelly,
            total_adjusted_kelly: total_kelly * scaling_factor,
            scaling_factor,
        }
    }

    /// 生成仓位建议
    fn generate_position_recommendation(
        &self,
        allocation: &PortfolioKellyAllocation,
    ) -> String {
        let mut report = String::from("📊 Kelly仓位管理建议\n\n");

        report.push_str(&format!(
            "Kelly策略: {} Kelly\n",
            if self.kelly_fraction == 0.25 {
                "1/4"
            } else if self.kelly_fraction == 0.5 {
                "半"
            } else {
                format!("{}", self.kelly_fraction)
            }
        ));

        report.push_str(&format!("总建议仓位: {:.1}%\n\n", allocation.total_adjusted_kelly * 100.0));

        report.push_str("📈 各标的仓位配置:\n");

        for alloc in &allocation.allocations {
            if alloc.final_allocation > 0.0 {
                report.push_str(&format!(
                    "\n{}:\n\
                     ├─ 建议仓位: {:.1}%\n\
                     ├─ 预期收益: {:.1}%\n\
                     ├─ 波动率: {:.1}%\n\
                     └─ 风险等级: {}\n",
                    alloc.symbol,
                    alloc.final_allocation * 100.0,
                    alloc.expected_return * 100.0,
                    alloc.volatility * 100.0,
                    if alloc.risk_score < 1.0 {
                        "低"
                    } else if alloc.risk_score < 2.0 {
                        "中"
                    } else {
                        "高"
                    }
                ));
            }
        }

        report.push_str("\n💡 使用建议:\n");
        report.push_str("• Kelly公式提供的是理论最优值，实际应结合个人风险承受能力\n");
        report.push_str("• 建议使用1/4 Kelly或半Kelly，保留安全边际\n");
        report.push_str("• 分批建仓，降低市场择时风险\n");
        report.push_str("• 定期再平衡，保持仓位比例\n");

        report
    }

    /// 分析单个标的的Kelly仓位
    async fn analyze_single_position(
        &self,
        symbol: &str,
        expected_return: f64,
        volatility: f64,
    ) -> Result<SinglePositionAnalysis> {
        let kelly = self.calculate_kelly_from_return_volatility(expected_return, volatility);

        let risk_level = if volatility < 0.15 {
            1
        } else if volatility < 0.25 {
            2
        } else {
            3
        };

        let confidence = if expected_return > volatility * 1.5 {
            0.8
        } else if expected_return > volatility {
            0.6
        } else {
            0.4
        };

        Ok(SinglePositionAnalysis {
            symbol: symbol.to_string(),
            kelly_percentage: kelly * 100.0,
            expected_return: expected_return * 100.0,
            volatility: volatility * 100.0,
            risk_level,
            confidence,
            recommendation: self.generate_position_recommendation_text(kelly, risk_level),
        })
    }

    /// 生成仓位建议文本
    fn generate_position_recommendation_text(&self, kelly: f64, risk_level: u8) -> String {
        if kelly < 0.02 {
            "不建议建仓 - 风险收益比不佳".to_string()
        } else if kelly < 0.05 {
            format!("小仓位试探 - 建议仓位{:.1}%", kelly * 100.0)
        } else if kelly < 0.15 {
            format!("适中仓位 - 建议仓位{:.1}%", kelly * 100.0)
        } else {
            format!("较大仓位机会 - 建议仓位{:.1}%, 建议分批建仓", kelly * 100.0)
        }
    }
}

impl Default for KellyPositionAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for KellyPositionAgent {
    fn name(&self) -> &str {
        "KellyPositionAgent"
    }

    fn description(&self) -> &str {
        "基于Kelly准则的科学仓位管理Agent"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 解析输入
        let content = input.content.to_lowercase();

        if content.contains("portfolio") || content.contains("组合") {
            // 组合Kelly分析
            let opportunities = vec![
                KellyOpportunity {
                    symbol: "AAPL".to_string(),
                    expected_return: 0.15,
                    volatility: 0.22,
                },
                KellyOpportunity {
                    symbol: "MSFT".to_string(),
                    expected_return: 0.14,
                    volatility: 0.20,
                },
                KellyOpportunity {
                    symbol: "GOOGL".to_string(),
                    expected_return: 0.12,
                    volatility: 0.25,
                },
            ];

            let allocation = self.calculate_portfolio_kelly(opportunities);
            let recommendation = self.generate_position_recommendation(&allocation);

            Ok(AgentOutput::new(recommendation).with_confidence(0.85))
        } else {
            // 单标的分析 (假设AAPL)
            let symbol = if content.contains("aapl") {
                "AAPL"
            } else if content.contains("msft") {
                "MSFT"
            } else {
                "AAPL"
            };

            let analysis = self
                .analyze_single_position(symbol, 0.15, 0.22)
                .await?;

            let report = format!(
                "📊 {} Kelly仓位分析\n\n\
                 建议仓位: {:.2}%\n\
                 预期收益: {:.1}%\n\
                 波动率: {:.1}%\n\
                 风险等级: {}/3\n\
                 置信度: {:.0}%\n\
                 建议: {}\n\n\
                 💡 Kelly公式: f = μ / σ²\n\
                 其中 μ 为预期收益，σ 为波动率\n\n\
                 ⚠️ 风险提示:\n\
                 • 当前使用{} Kelly (保守策略)\n\
                 • 实际建仓建议分批进行\n\
                 • 密切关注市场变化，及时调整",
                analysis.symbol,
                analysis.kelly_percentage,
                analysis.expected_return,
                analysis.volatility,
                analysis.risk_level,
                analysis.confidence * 100.0,
                analysis.recommendation,
                if self.kelly_fraction == 0.25 {
                    "1/4"
                } else if self.kelly_fraction == 0.5 {
                    "半"
                } else {
                    "完整"
                }
            );

            Ok(AgentOutput::new(report).with_confidence(analysis.confidence))
        }
    }
}

/// Kelly机会
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KellyOpportunity {
    pub symbol: String,
    pub expected_return: f64,
    pub volatility: f64,
}

/// 单标的仓位分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinglePositionAnalysis {
    pub symbol: String,
    pub kelly_percentage: f64,
    pub expected_return: f64,
    pub volatility: f64,
    pub risk_level: u8,
    pub confidence: f64,
    pub recommendation: String,
}

/// Kelly仓位配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KellyPositionAllocation {
    pub symbol: String,
    pub raw_kelly: f64,
    pub adjusted_kelly: f64,
    pub expected_return: f64,
    pub volatility: f64,
    pub risk_score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_allocation: Option<f64>,
}

/// 组合Kelly配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioKellyAllocation {
    pub allocations: Vec<KellyPositionAllocation>,
    pub total_raw_kelly: f64,
    pub total_adjusted_kelly: f64,
    pub scaling_factor: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kelly_calculation() {
        let agent = KellyPositionAgent::new();

        // 测试基础Kelly计算
        let kelly = agent.calculate_kelly(0.6, 100.0, 50.0);
        assert!((kelly - 0.2).abs() < 0.01); // (2*0.6 - 0.4) / 2 = 0.2

        // 测试负Kelly情况
        let kelly_negative = agent.calculate_kelly(0.4, 100.0, 100.0);
        assert!(kelly_negative >= 0.0);
    }

    #[test]
    fn test_fractional_kelly() {
        let agent = KellyPositionAgent::with_kelly_params(
            KellyPositionAgent::new(),
            0.25, // 1/4 Kelly
            0.02,
            0.25,
        );

        let kelly = 0.20; // 20% raw Kelly
        let adjusted = agent.apply_fractional_kelly(kelly);

        // 1/4 Kelly应该是5%
        assert!((adjusted - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_kelly_from_volatility() {
        let agent = KellyPositionAgent::new();

        // 预期15%收益，20%波动率
        let kelly = agent.calculate_kelly_from_return_volatility(0.15, 0.20);

        // Kelly = 0.15 / (0.20^2) = 3.75
        // 但会被限制在max_position (0.25)
        assert!(kelly <= 0.25);
    }

    #[test]
    fn test_portfolio_allocation() {
        let agent = KellyPositionAgent::new();

        let opportunities = vec![
            KellyOpportunity {
                symbol: "AAPL".to_string(),
                expected_return: 0.15,
                volatility: 0.22,
            },
            KellyOpportunity {
                symbol: "MSFT".to_string(),
                expected_return: 0.14,
                volatility: 0.20,
            },
        ];

        let allocation = agent.calculate_portfolio_kelly(opportunities);

        assert!(!allocation.allocations.is_empty());
        assert!(allocation.total_adjusted_kelly <= 1.0);
    }
}
