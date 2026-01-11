//! Value Investment Agent - 价值投资分析Agent
//!
//! 基于Graham-Buffett-Munger价值投资理念的分析Agent
//!
//! ## 核心功能
//!
//! 1. **Graham深度价值分析** - 安全边际、内在价值计算
//! 2. **Buffett质量价值分析** - ROIC、护城河评估
//! 3. **综合评分** - 多维度价值评估

use crate::agents::{
    Agent, AgentInput, AgentOutput, BuffettAnalysis, GrahamAnalysis, InvestError,
    InvestmentAction, InvestmentRecommendation, Result,
    get_realtime_quote, get_fundamental_data, FundamentalData,
};
use async_trait::async_trait;
use std::collections::HashMap;

/// Value Investment Agent
///
/// 价值投资分析Agent，整合Graham-Buffett-Munger三位一体的价值分析框架
pub struct ValueInvestmentAgent {
    /// Graham公式基础倍数
    graham_base_multiplier: f64,

    /// Graham增长率倍数
    graham_growth_multiplier: f64,

    /// 安全边际要求
    margin_requirement: f64,

    /// Buffett最低ROIC要求
    min_roic: f64,
}

impl ValueInvestmentAgent {
    /// 创建新的价值投资Agent
    pub fn new() -> Self {
        Self {
            graham_base_multiplier: 8.5,
            graham_growth_multiplier: 2.0,
            margin_requirement: 0.3, // 30%安全边际
            min_roic: 0.10,          // 10% ROIC
        }
    }

    /// 设置Graham参数
    pub fn with_graham_params(
        mut self,
        base_multiplier: f64,
        growth_multiplier: f64,
        margin_requirement: f64,
    ) -> Self {
        self.graham_base_multiplier = base_multiplier;
        self.graham_growth_multiplier = growth_multiplier;
        self.margin_requirement = margin_requirement;
        self
    }

    /// 设置Buffett参数
    pub fn with_buffett_params(mut self, min_roic: f64) -> Self {
        self.min_roic = min_roic;
        self
    }

    /// Graham公式: V = EPS × (8.5 + 2g)
    ///
    /// # 参数
    /// - `eps`: 每股收益
    /// - `growth_rate`: 预期增长率 (如0.1表示10%)
    fn calculate_graham_intrinsic_value(&self, eps: f64, growth_rate: f64) -> f64 {
        let multiplier = self.graham_base_multiplier
            + self.graham_growth_multiplier * growth_rate * 100.0;
        eps * multiplier
    }

    /// 计算安全边际
    ///
    /// 安全边际 = (内在价值 - 当前价格) / 内在价值
    fn calculate_margin_of_safety(&self, intrinsic_value: f64, current_price: f64) -> f64 {
        if intrinsic_value <= 0.0 {
            return 0.0;
        }
        (intrinsic_value - current_price) / intrinsic_value
    }

    /// Graham价值分析
    ///
    /// 基于Benjamin Graham的经典价值投资方法，使用真实市场数据
    async fn analyze_graham(&self, symbol: &str) -> Result<GrahamAnalysis> {
        // 从Yahoo Finance获取实时报价
        let quote = get_realtime_quote(symbol).await
            .map_err(|e| InvestError::DataError(format!("获取实时报价失败: {}", e)))?;

        // 获取基本面数据
        let fundamental = get_fundamental_data(symbol).await
            .map_err(|e| InvestError::DataError(format!("获取基本面数据失败: {}", e)))?;

        // 使用真实数据
        let eps = fundamental.eps.unwrap_or(quote.current_price / 25.0); // 如果没有EPS，假设PE=25
        let growth_rate = 0.08; // 默认8%增长率 (后续可从分析师预期获取)
        let current_price = quote.current_price;

        // 计算内在价值
        let intrinsic_value = self.calculate_graham_intrinsic_value(eps, growth_rate);

        // 计算安全边际
        let margin = self.calculate_margin_of_safety(intrinsic_value, current_price);

        // 评估是否符合Graham标准
        let meets_criteria = margin >= self.margin_requirement;

        Ok(GrahamAnalysis {
            intrinsic_value,
            current_price,
            margin_of_safety: margin,
            eps,
            growth_rate,
            meets_criteria,
        })
    }

    /// Buffett质量价值分析
    ///
    /// 基于Warren Buffett的质量价值投资方法，使用真实市场数据
    async fn analyze_buffett(&self, symbol: &str) -> Result<BuffettAnalysis> {
        // 获取实时报价
        let quote = get_realtime_quote(symbol).await
            .map_err(|e| InvestError::DataError(format!("获取实时报价失败: {}", e)))?;

        // 获取基本面数据
        let fundamental = get_fundamental_data(symbol).await
            .map_err(|e| InvestError::DataError(format!("获取基本面数据失败: {}", e)))?;

        // 使用真实数据
        let roic = fundamental.roic.unwrap_or(0.12) / 100.0; // 转换为小数
        let roe = fundamental.roe.unwrap_or(0.15) / 100.0; // 转换为小数
        let current_price = quote.current_price;

        // 简化DCF估值 (假设FCF为EPS的80%)
        let eps = fundamental.eps.unwrap_or(quote.current_price / 25.0);
        let fcf = eps * 0.8;
        let growth_rate = 0.08; // 默认8%增长
        let discount_rate = 0.10; // 10%折现率
        let terminal_growth = 0.025; // 2.5%永续增长

        let intrinsic_value = self.calculate_simple_dcf(fcf, growth_rate, discount_rate, terminal_growth);
        let fair_price = intrinsic_value * 0.9; // 公允价格为内在价值的90%

        // 护城河评分 (基于ROIC和ROE)
        let moat_score = if roic > 0.15 && roe > 0.15 {
            3 // Wide moat - 宽护城河
        } else if roic > 0.10 && roe > 0.12 {
            2 // Narrow moat - 窄护城河
        } else {
            1 // No moat - 无护城河
        };

        let meets_criteria = roic >= self.min_roic && moat_score >= 2;

        Ok(BuffettAnalysis {
            roic,
            roe,
            moat_score,
            intrinsic_value,
            fair_price,
            meets_criteria,
        })
    }

    /// 简化的DCF估值
    ///
    /// V = FCF × (1+g) / (r-g)  [永续增长模型]
    fn calculate_simple_dcf(
        &self,
        fcf: f64,
        growth_rate: f64,
        discount_rate: f64,
        terminal_growth: f64,
    ) -> f64 {
        if discount_rate <= terminal_growth {
            return 0.0;
        }

        let terminal_fcf = fcf * (1.0 + growth_rate) * (1.0 + terminal_growth);
        let terminal_value = terminal_fcf / (discount_rate - terminal_growth);

        terminal_value
    }

    /// 综合价值分析
    ///
    /// 整合Graham和Buffett的分析结果，给出综合评分
    async fn comprehensive_analysis(&self, symbol: &str) -> Result<InvestmentRecommendation> {
        // 并行执行Graham和Buffett分析
        let (graham_result, buffett_result) =
            tokio::try_join!(self.analyze_graham(symbol), self.analyze_buffett(symbol))?;

        // 综合评分逻辑
        let mut score = 0.0;
        let mut reasons = Vec::new();

        // Graham安全边际得分 (0-40分)
        let graham_score = if graham_result.margin_of_safety >= 0.4 {
            40.0
        } else if graham_result.margin_of_safety >= 0.3 {
            35.0
        } else if graham_result.margin_of_safety >= 0.2 {
            25.0
        } else if graham_result.margin_of_safety >= 0.1 {
            15.0
        } else {
            0.0
        };
        score += graham_score;

        if graham_result.margin_of_safety >= 0.3 {
            reasons.push(format!(
                "Graham安全边际{:.1}% (优秀)",
                graham_result.margin_of_safety * 100.0
            ));
        }

        // Buffett质量得分 (0-40分)
        let buffett_score = if buffett_result.roic >= 0.20 {
            40.0
        } else if buffett_result.roic >= 0.15 {
            35.0
        } else if buffett_result.roic >= 0.10 {
            25.0
        } else {
            10.0
        };
        score += buffett_score;

        if buffett_result.roic >= 0.15 {
            reasons.push(format!("Buffett ROIC {:.1}% (优秀)", buffett_result.roic * 100.0));
        }

        // 护城河得分 (0-20分)
        let moat_score = match buffett_result.moat_score {
            3 => 20.0,
            2 => 15.0,
            1 => 5.0,
            _ => 0.0,
        };
        score += moat_score;

        if buffett_result.moat_score >= 2 {
            reasons.push(format!(
                "护城河评分 {}/3 (良好)",
                buffett_result.moat_score
            ));
        }

        // 归一化到0-1
        let normalized_score = score / 100.0;

        // 确定投资建议
        let action = InvestmentAction::from_score(normalized_score);
        let confidence = normalized_score;

        // 目标价位: 取Graham和Buffett估值的平均值
        let target_price = Some(
            (graham_result.intrinsic_value + buffett_result.intrinsic_value) / 2.0,
        );

        let reasoning = if reasons.is_empty() {
            "未达到价值投资标准".to_string()
        } else {
            reasons.join("; ")
        };

        Ok(InvestmentRecommendation {
            symbol: symbol.to_string(),
            action,
            confidence,
            reasoning,
            target_price,
            current_price: Some(graham_result.current_price),
            margin_of_safety: Some(graham_result.margin_of_safety),
            risk_level: if normalized_score > 0.7 {
                1 // 低风险
            } else if normalized_score > 0.4 {
                2 // 中等风险
            } else {
                3 // 高风险
            },
            metadata: serde_json::json!({
                "graham_analysis": graham_result,
                "buffett_analysis": buffett_result,
                "score": score,
                "normalized_score": normalized_score,
            }),
        })
    }
}

impl Default for ValueInvestmentAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for ValueInvestmentAgent {
    fn name(&self) -> &str {
        "ValueInvestmentAgent"
    }

    fn description(&self) -> &str {
        "价值投资分析Agent，基于Graham-Buffett-Munger框架进行价值评估"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 解析输入获取股票代码
        let symbol = input.content.trim();

        if symbol.is_empty() {
            return Err(InvestError::InvalidInput(
                "股票代码不能为空".to_string(),
            )
            .into());
        }

        // 执行综合价值分析
        let recommendation = self.comprehensive_analysis(symbol).await?;

        // 格式化输出
        let content = format!(
            "📊 {} 价值投资分析报告\n\n\
             💡 投资建议: {}\n\
             ✅ 置信度: {:.1}%\n\
             📈 目标价位: ${:.2}\n\
             💰 当前价格: ${:.2}\n\
             🛡️ 安全边际: {:.1}%\n\
             ⚠️ 风险等级: {}/3\n\n\
             📝 分析理由:\n\
             {}\n\n\
             📊 详细分析:\n\
             - Graham内在价值: ${:.2}\n\
             - Buffett内在价值: ${:.2}\n\
             - ROIC: {:.1}%\n\
             - 护城河: {}/3",
            symbol,
            recommendation.action.display_name(),
            recommendation.confidence * 100.0,
            recommendation.target_price.unwrap_or(0.0),
            recommendation.current_price.unwrap_or(0.0),
            recommendation.margin_of_safety.unwrap_or(0.0) * 100.0,
            recommendation.risk_level,
            recommendation.reasoning,
            recommendation
                .metadata
                .get("graham_analysis")
                .and_then(|v| v.get("intrinsic_value"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            recommendation
                .metadata
                .get("buffett_analysis")
                .and_then(|v| v.get("intrinsic_value"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            recommendation
                .metadata
                .get("buffett_analysis")
                .and_then(|v| v.get("roic"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
                * 100.0,
            recommendation
                .metadata
                .get("buffett_analysis")
                .and_then(|v| v.get("moat_score"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        );

        Ok(AgentOutput::new(content)
            .with_data(serde_json::to_value(recommendation)?)
            .with_confidence(recommendation.confidence))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_value_investment_agent() {
        let agent = ValueInvestmentAgent::new();

        let result = agent
            .execute(AgentInput::new("AAPL"))
            .await
            .unwrap();

        assert!(!result.content.is_empty());
        assert!(result.confidence > 0.0);

        // 验证输出包含关键信息
        assert!(result.content.contains("价值投资分析报告"));
        assert!(result.content.contains("投资建议"));
        assert!(result.content.contains("置信度"));
    }

    #[test]
    fn test_graham_formula() {
        let agent = ValueInvestmentAgent::new();

        // V = EPS × (8.5 + 2g)
        // EPS = $10, g = 10%
        // V = 10 × (8.5 + 2×0.1×100) = 10 × (8.5 + 20) = 10 × 28.5 = $285
        let intrinsic_value = agent.calculate_graham_intrinsic_value(10.0, 0.1);
        assert_eq!(intrinsic_value, 285.0);
    }

    #[test]
    fn test_margin_of_safety() {
        let agent = ValueInvestmentAgent::new();

        // 内在价值 $100, 当前价格 $70
        // 安全边际 = (100 - 70) / 100 = 30%
        let margin = agent.calculate_margin_of_safety(100.0, 70.0);
        assert_eq!(margin, 0.3);

        // 内在价值 $100, 当前价格 $120 (高估)
        // 安全边际 = (100 - 120) / 100 = -20%
        let margin_negative = agent.calculate_margin_of_safety(100.0, 120.0);
        assert_eq!(margin_negative, -0.2);
    }

    #[test]
    fn test_simple_dcf() {
        let agent = ValueInvestmentAgent::new();

        // FCF = $100, g = 10%, r = 10%, terminal_g = 2.5%
        let value = agent.calculate_simple_dcf(100.0, 0.10, 0.10, 0.025);
        // V = 100 × 1.1 × 1.025 / (0.10 - 0.025) = 112.75 / 0.075 = $1503.33
        assert!(value > 1500.0);
        assert!(value < 1510.0);
    }
}
