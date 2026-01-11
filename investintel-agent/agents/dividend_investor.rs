//! Dividend Investor Agent - 股息投资Agent
//!
//! 基于股息投资策略的Agent，帮助普通人获得稳定被动收入
//!
//! ## 核心功能
//!
//! 1. **股息收益率分析** - 计算和分析股息收益率
//! 2. **股息安全性评估** - 评估股息支付能力
//! 3. **股息成长性分析** - 评估股息增长历史
//! 4. **月度股息收入规划** - 计算预期月度收入

use crate::agents::{
    Agent, AgentInput, AgentOutput, Result, get_dividend_info,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Dividend Investor Agent
///
/// 专注于股息投资的Agent，帮助投资者建立稳定的被动收入流
pub struct DividendInvestorAgent {
    /// 最低股息收益率要求
    min_dividend_yield: f64,

    /// 股息增长率要求
    min_dividend_growth: f64,

    /// 派息比率安全上限
    max_payout_ratio: f64,
}

impl DividendInvestorAgent {
    /// 创建新的股息投资Agent
    pub fn new() -> Self {
        Self {
            min_dividend_yield: 0.03,      // 3%最低收益率
            min_dividend_growth: 0.05,      // 5%增长率
            max_payout_ratio: 0.70,        // 70%派息率上限
        }
    }

    /// 设置参数
    pub fn with_params(
        mut self,
        min_yield: f64,
        min_growth: f64,
        max_payout: f64,
    ) -> Self {
        self.min_dividend_yield = min_yield;
        self.min_dividend_growth = min_growth;
        self.max_payout_ratio = max_payout;
        self
    }

    /// 分析股息股票 - 使用真实市场数据
    async fn analyze_dividend_stock(&self, symbol: &str) -> Result<DividendAnalysis> {
        // 从data层获取股息数据
        let dividend_data = get_dividend_info(symbol).await
            .map_err(|e| anyhow::anyhow!("获取股息数据失败: {}", e))?;

        let current_price = dividend_data.current_price;
        let annual_dividend = dividend_data.annual_dividend;
        let dividend_yield = dividend_data.dividend_yield;
        let payout_ratio = dividend_data.payout_ratio;
        let dividend_growth_rate = dividend_data.growth_rate;
        let years_of_growth = dividend_data.consecutive_years;
        let monthly_income = dividend_data.monthly_income;

        // 股息安全性评分
        let safety_score = self.calculate_safety_score(
            payout_ratio,
            dividend_growth_rate,
        );

        // 股息吸引力评分
        let attractiveness_score = self.calculate_attractiveness_score(
            dividend_yield,
            dividend_growth_rate,
            safety_score,
        );

        // 评估
        let recommendation = if dividend_yield >= self.min_dividend_yield
            && payout_ratio <= self.max_payout_ratio
            && safety_score >= 3
        {
            "优秀股息股 - 可考虑买入"
        } else if dividend_yield >= self.min_dividend_yield {
            "一般股息股 - 观察为主"
        } else {
            "不符合股息投资标准"
        };

        Ok(DividendAnalysis {
            symbol: symbol.to_string(),
            current_price,
            annual_dividend,
            dividend_yield,
            payout_ratio,
            dividend_growth_rate,
            years_of_consecutive_growth: years_of_growth,
            monthly_income,
            safety_score,
            attractiveness_score,
            recommendation: recommendation.to_string(),
        })
    }

    /// 计算股息安全性评分 (1-5分)
    ///
    /// 基于派息率和增长率
    fn calculate_safety_score(&self, payout_ratio: f64, growth_rate: f64) -> u8 {
        let mut score = 0;

        // 派息率评分 (0-2分)
        if payout_ratio <= 0.50 {
            score += 2; // 非常安全
        } else if payout_ratio <= 0.70 {
            score += 1; // 安全
        }

        // 增长率评分 (0-2分)
        if growth_rate >= 0.10 {
            score += 2; // 强劲增长
        } else if growth_rate >= 0.05 {
            score += 1; // 稳定增长
        }

        // 稳定性评分 (0-1分)
        if growth_rate > 0.0 && payout_ratio < 0.70 {
            score += 1; // 正向增长且保守派息
        }

        score.min(5)
    }

    /// 计算股息吸引力评分 (1-5分)
    ///
    /// 综合收益率、增长率和安全性
    fn calculate_attractiveness_score(
        &self,
        yield_: f64,
        growth_rate: f64,
        safety_score: u8,
    ) -> u8 {
        let mut score = 0;

        // 收益率评分 (0-2分)
        if yield_ >= 0.05 {
            score += 2; // 高收益
        } else if yield_ >= 0.03 {
            score += 1; // 中等收益
        }

        // 增长率评分 (0-1分)
        if growth_rate >= 0.08 {
            score += 1;
        }

        // 安全性加分 (0-2分)
        score += (safety_score / 3).min(2) as u8;

        score.min(5)
    }

    /// 计算股息复利效果
    ///
    /// 展示股息再投资的威力
    fn calculate_dividend_compounding(
        &self,
        initial_investment: f64,
        dividend_yield: f64,
        years: u32,
    ) -> DividendCompoundingResult {
        let reinvested_dividends = initial_investment
            * (1.0 + dividend_yield).powi(years as i32) - initial_investment;

        let total_value = initial_investment + reinvested_dividends;
        let total_return = total_value / initial_investment - 1.0;

        DividendCompoundingResult {
            initial_investment,
            years,
            dividend_yield,
            reinvested_dividends,
            total_value,
            total_return,
        }
    }
}

impl Default for DividendInvestorAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for DividendInvestorAgent {
    fn name(&self) -> &str {
        "DividendInvestorAgent"
    }

    fn description(&self) -> &str {
        "股息投资Agent，帮助建立稳定的被动收入流"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        let symbol = input.content.trim();

        if symbol.is_empty() {
            return Ok(AgentOutput::new(
                "请提供股票代码进行分析。\
                例如: \"分析AAPL的股息投资价值\""
            ));
        }

        // 分析股息股票
        let analysis = self.analyze_dividend_stock(symbol).await?;

        // 计算复利效果
        let compounding = self.calculate_dividend_compounding(
            10000.0,  // 假设投资$10,000
            analysis.dividend_yield,
            10,       // 10年
        );

        // 格式化输出
        let content = format!(
            "💰 {} 股息投资分析报告\n\n\
             📊 基本信息:\n\
             当前价格: ${:.2}\n\
             年度股息: ${:.2}\n\
             股息收益率: {:.2}%\n\
             月度收入: ${:.2}\n\n\
             📈 股息成长:\n\
             年增长率: {:.1}%\n\
             连续增长: {}年\n\
             派息比率: {:.1}%\n\n\
             🛡️ 安全性评分: {}/5\n\
             ⭐ 吸引力评分: {}/5\n\n\
             💡 投资建议: {}\n\n\
             📊 复利效果 (投资$10,000，{}年):\n\
             再投资股息: ${:.2}\n\
             总价值: ${:.2}\n\
             总回报: {:.1}%\n\n\
             🎓 股息投资要点:\n\
             • 股息提供稳定现金流\n\
             • 复利效应威力巨大\n\
             • 关注股息支付能力\n\
             • 优先选择股息贵族(连续25年+增长)",
            symbol,
            analysis.current_price,
            analysis.annual_dividend,
            analysis.dividend_yield * 100.0,
            analysis.monthly_income,
            analysis.dividend_growth_rate * 100.0,
            analysis.years_of_consecutive_growth,
            analysis.payout_ratio * 100.0,
            analysis.safety_score,
            analysis.attractiveness_score,
            analysis.recommendation,
            compounding.years,
            compounding.reinvested_dividends,
            compounding.total_value,
            compounding.total_return * 100.0
        );

        Ok(AgentOutput::new(content)
            .with_data(serde_json::to_value(analysis)?)
            .with_confidence(0.85))
    }
}

/// 股息分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividendAnalysis {
    /// 股票代码
    pub symbol: String,

    /// 当前价格
    pub current_price: f64,

    /// 年度股息
    pub annual_dividend: f64,

    /// 股息收益率
    pub dividend_yield: f64,

    /// 派息比率
    pub payout_ratio: f64,

    /// 股息增长率
    pub dividend_growth_rate: f64,

    /// 连续增长年数
    pub years_of_consecutive_growth: u32,

    /// 月度收入
    pub monthly_income: f64,

    /// 安全性评分 (1-5)
    pub safety_score: u8,

    /// 吸引力评分 (1-5)
    pub attractiveness_score: u8,

    /// 投资建议
    pub recommendation: String,
}

/// 股息复利结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividendCompoundingResult {
    /// 初始投资
    pub initial_investment: f64,

    /// 年数
    pub years: u32,

    /// 股息收益率
    pub dividend_yield: f64,

    /// 再投资股息收益
    pub reinvested_dividends: f64,

    /// 总价值
    pub total_value: f64,

    /// 总回报率
    pub total_return: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dividend_investor_agent() {
        let agent = DividendInvestorAgent::new();

        let result = agent
            .execute(AgentInput::new("AAPL"))
            .await
            .unwrap();

        assert!(!result.content.is_empty());
        assert!(result.content.contains("股息投资分析报告"));
        assert!(result.content.contains("复利效果"));
    }

    #[test]
    fn test_safety_score() {
        let agent = DividendInvestorAgent::new();

        // 高安全性: 低派息率+高增长
        let score1 = agent.calculate_safety_score(0.50, 0.10);
        assert_eq!(score1, 5);

        // 中等安全性
        let score2 = agent.calculate_safety_score(0.65, 0.06);
        assert!(score2 >= 2 && score2 <= 4);

        // 低安全性: 高派息率
        let score3 = agent.calculate_safety_score(0.80, 0.02);
        assert!(score3 <= 2);
    }

    #[test]
    fn test_dividend_compounding() {
        let agent = DividendInvestorAgent::new();

        // $10,000, 4%收益率, 10年
        let result = agent.calculate_dividend_compounding(10000.0, 0.04, 10);

        // 复利效应: 10000 * (1.04)^10 - 10000
        let expected_reinvested = 10000.0 * 1.04_f64.powi(10) - 10000.0;

        assert!((result.reinvested_dividends - expected_reinvested).abs() < 1.0);
        assert!(result.total_return > 0.45); // >45%总回报
        assert_eq!(result.initial_investment, 10000.0);
    }
}
