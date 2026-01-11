//! Integrated Value Investing Framework
//!
//! Graham-Buffett-Munger三位一体综合决策框架
//! 整合三大投资大师的价值投资理念

use anyhow::Result;
use chrono::Duration;
use serde::{Deserialize, Serialize};

use super::{
    graham::{GrahamAnalysis, GrahamFramework},
    buffett::{BuffettAnalysis, BuffettFramework},
    munger::{MungerAnalysis, MungerFramework},
};

// ============================================================================
// Investment Action - 投资行动
// ============================================================================

/// 投资行动
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestmentAction {
    /// 重仓买入 (30-50%仓位)
    HeavyBuy,
    /// 买入 (15-25%仓位)
    Buy,
    /// 小仓位买入 (5-10%仓位)
    SmallBuy,
    /// 持有/观望
    Hold,
    /// 卖出
    Sell,
    /// 强烈卖出
    StrongSell,
}

impl InvestmentAction {
    /// 获取行动描述
    pub fn description(&self) -> &'static str {
        match self {
            InvestmentAction::HeavyBuy => "重仓买入 - 绝佳机会",
            InvestmentAction::Buy => "买入 - 良好机会",
            InvestmentAction::SmallBuy => "小仓位买入 - 谨慎参与",
            InvestmentAction::Hold => "持有/观望 - 等待更好机会",
            InvestmentAction::Sell => "卖出 - 减少暴露",
            InvestmentAction::StrongSell => "强烈卖出 - 避免损失",
        }
    }

    /// 获取建议仓位范围
    pub fn position_range(&self) -> (f64, f64) {
        match self {
            InvestmentAction::HeavyBuy => (0.30, 0.50),
            InvestmentAction::Buy => (0.15, 0.25),
            InvestmentAction::SmallBuy => (0.05, 0.10),
            InvestmentAction::Hold => (0.0, 0.0),
            InvestmentAction::Sell => (0.0, 0.0),
            InvestmentAction::StrongSell => (0.0, 0.0),
        }
    }
}

// ============================================================================
// Comprehensive Decision - 综合决策
// ============================================================================

/// 综合投资决策
///
/// 基于Graham-Buffett-Munger三位一体的完整投资决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveDecision {
    /// 股票代码
    pub symbol: String,

    /// 投资行动
    pub action: InvestmentAction,

    /// 置信度 (0-1)
    pub confidence: f64,

    /// 仓位建议范围
    pub position_size_range: (f64, f64),

    /// 决策推理
    pub reasoning: String,

    /// Graham分析
    pub graham_analysis: GrahamAnalysis,

    /// Buffett分析
    pub buffett_analysis: BuffettAnalysis,

    /// Munger分析
    pub munger_analysis: MungerAnalysis,

    /// 预期收益率
    pub expected_return: Option<f64>,

    /// 建议持有期限
    pub time_horizon: Duration,

    /// 综合评分 (0-100)
    pub score: f64,

    /// 决策时间戳
    pub decision_timestamp: chrono::DateTime<chrono::Utc>,
}

impl ComprehensiveDecision {
    /// 创建新的综合决策
    pub fn new(
        symbol: String,
        action: InvestmentAction,
        confidence: f64,
        reasoning: String,
        graham_analysis: GrahamAnalysis,
        buffett_analysis: BuffettAnalysis,
        munger_analysis: MungerAnalysis,
        expected_return: Option<f64>,
    ) -> Self {
        let position_size_range = action.position_range();

        // 根据行动确定持有期限
        let time_horizon = match action {
            InvestmentAction::HeavyBuy => Duration::days(365 * 5),  // 5年
            InvestmentAction::Buy => Duration::days(365 * 3),       // 3年
            InvestmentAction::SmallBuy => Duration::days(365 * 2),  // 2年
            _ => Duration::zero(),
        };

        // 计算综合评分
        let score = Self::calculate_comprehensive_score(
            &graham_analysis,
            &buffett_analysis,
            &munger_analysis,
        );

        Self {
            symbol,
            action,
            confidence,
            position_size_range,
            reasoning,
            graham_analysis,
            buffett_analysis,
            munger_analysis,
            expected_return,
            time_horizon,
            score,
            decision_timestamp: chrono::Utc::now(),
        }
    }

    /// 计算综合评分
    ///
    /// 综合三位大师的分析结果
    fn calculate_comprehensive_score(
        graham: &GrahamAnalysis,
        buffett: &BuffettAnalysis,
        munger: &MungerAnalysis,
    ) -> f64 {
        // Graham评分权重30%
        let graham_weight = 0.30;
        let graham_score = graham.score;

        // Buffett评分权重40%
        let buffett_weight = 0.40;
        let buffett_score = buffett.score;

        // Munger评分权重30%
        let munger_weight = 0.30;
        let munger_score = munger.score;

        // 加权平均
        let base_score = (graham_score * graham_weight)
            + (buffett_score * buffett_weight)
            + (munger_score * munger_weight);

        // Lollapalooza效应额外加分
        let lollapalooza_bonus = if munger.lollapalooza_detected {
            10.0
        } else {
            0.0
        };

        // 能力圈内加分
        let competence_bonus = if munger.in_circle_of_competence {
            5.0
        } else {
            0.0
        };

        let total_score = base_score + lollapalooza_bonus + competence_bonus;

        total_score.min(100.0)
    }

    /// 生成决策摘要
    pub fn summary(&self) -> String {
        format!(
            "股票: {}\n行动: {}\n置信度: {:.1}%\n综合评分: {:.1}\n理由: {}",
            self.symbol,
            self.action.description(),
            self.confidence * 100.0,
            self.score,
            self.reasoning
        )
    }
}

// ============================================================================
// Value Investing Framework - 价值投资框架
// ============================================================================

/// 价值投资框架
///
/// Graham-Buffett-Munger三位一体综合决策框架
///
/// 这是Plan6的核心创新点,业界最完整的AI价值投资实现
#[derive(Debug, Clone)]
pub struct ValueInvestingFramework {
    /// Graham框架
    pub graham: GrahamFramework,

    /// Buffett框架
    pub buffett: BuffettFramework,

    /// Munger框架
    pub munger: MungerFramework,
}

impl ValueInvestingFramework {
    /// 创建新的价值投资框架
    ///
    /// # 参数
    /// - `familiar_industries`: 熟悉的行业列表
    /// - `familiar_business_models`: 熟悉的商业模式列表
    pub fn new(
        familiar_industries: Vec<String>,
        familiar_business_models: Vec<String>,
    ) -> Self {
        Self {
            graham: GrahamFramework::new(),
            buffett: BuffettFramework::new(),
            munger: MungerFramework::new(familiar_industries, familiar_business_models),
        }
    }

    /// 综合分析
    ///
    /// 整合Graham、Buffett、Munger三位大师的分析方法
    ///
    /// # 参数
    /// - `symbol`: 股票代码
    /// - `data`: 综合分析数据
    ///
    /// # 返回
    /// 综合投资决策
    ///
    /// # 分析流程
    /// 1. 并行执行三位分析
    /// 2. 综合决策逻辑
    /// 3. 生成投资建议
    pub async fn comprehensive_analysis(
        &self,
        symbol: &str,
        data: &serde_json::Value,
    ) -> Result<ComprehensiveDecision> {
        // 并行执行三位分析
        let (graham_result, buffett_result, munger_result) = tokio::try_join!(
            self.analyze_graham(symbol, data),
            self.analyze_buffett(symbol, data),
            self.analyze_munger(symbol, data)
        )?;

        // 综合决策逻辑
        let decision = self.make_decision(
            symbol.to_string(),
            &graham_result,
            &buffett_result,
            &munger_result,
        );

        Ok(decision)
    }

    /// Graham分析
    async fn analyze_graham(
        &self,
        symbol: &str,
        data: &serde_json::Value,
    ) -> Result<GrahamAnalysis> {
        let eps = data
            .get("eps")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("缺少eps数据"))?;

        let growth_rate = data
            .get("growth_rate")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("缺少growth_rate数据"))?;

        let current_price = data
            .get("current_price")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("缺少current_price数据"))?;

        let current_assets = data.get("current_assets").and_then(|v| v.as_f64());
        let total_liabilities = data.get("total_liabilities").and_then(|v| v.as_f64());
        let shares_outstanding = data.get("shares_outstanding").and_then(|v| v.as_f64());

        self.graham
            .analyze(
                symbol.to_string(),
                eps,
                growth_rate,
                current_price,
                current_assets,
                total_liabilities,
                shares_outstanding,
            )
            .await
    }

    /// Buffett分析
    async fn analyze_buffett(
        &self,
        symbol: &str,
        data: &serde_json::Value,
    ) -> Result<BuffettAnalysis> {
        let roic = data
            .get("roic")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("缺少roic数据"))?;

        let free_cash_flow = data
            .get("free_cash_flow")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("缺少free_cash_flow数据"))?;

        let growth_rate = data
            .get("growth_rate")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("缺少growth_rate数据"))?;

        let discount_rate = data
            .get("discount_rate")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("缺少discount_rate数据"))?;

        let current_price = data
            .get("current_price")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("缺少current_price数据"))?;

        let has_strong_brand = data.get("strong_brand").and_then(|v| v.as_bool()).unwrap_or(false);
        let has_cost_advantage = data.get("cost_advantage").and_then(|v| v.as_bool()).unwrap_or(false);
        let has_high_switching_cost = data.get("high_switching_cost").and_then(|v| v.as_bool()).unwrap_or(false);
        let has_network_effects = data.get("network_effects").and_then(|v| v.as_bool()).unwrap_or(false);

        let capital_allocation_score = data.get("capital_allocation_score").and_then(|v| v.as_f64()).unwrap_or(50.0);
        let shareholder_returns = data.get("shareholder_returns").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let transparency_score = data.get("transparency_score").and_then(|v| v.as_f64()).unwrap_or(50.0);
        let insider_ownership = data.get("insider_ownership").and_then(|v| v.as_f64()).unwrap_or(0.0);

        self.buffett
            .analyze(
                symbol.to_string(),
                roic,
                free_cash_flow,
                growth_rate,
                discount_rate,
                current_price,
                has_strong_brand,
                has_cost_advantage,
                has_high_switching_cost,
                has_network_effects,
                capital_allocation_score,
                shareholder_returns,
                transparency_score,
                insider_ownership,
            )
            .await
    }

    /// Munger分析
    async fn analyze_munger(
        &self,
        symbol: &str,
        data: &serde_json::Value,
    ) -> Result<MungerAnalysis> {
        self.munger.analyze(symbol.to_string(), data).await
    }

    /// 综合决策逻辑
    ///
    /// 基于三位大师的分析结果做出最终投资决策
    fn make_decision(
        &self,
        symbol: String,
        graham: &GrahamAnalysis,
        buffett: &BuffettAnalysis,
        munger: &MungerAnalysis,
    ) -> ComprehensiveDecision {
        // 判断是否满足三位一体共振条件
        let resonance = graham.margin_of_safety > 0.30
            && buffett.roic > 0.10
            && buffett.moat_score >= crate::value_frameworks::buffett::MoatScore::Wide
            && munger.lollapalooza_detected
            && munger.in_circle_of_competence;

        let (action, confidence, reasoning, expected_return) = if resonance {
            // 三位一体共振: 最强信号
            (
                InvestmentAction::HeavyBuy,
                0.98,
                "三位一体共振 + Lollapalooza效应 + 能力圈内".to_string(),
                Some(0.25),
            )
        } else if graham.margin_of_safety > 0.25 && buffett.roic > 0.08 {
            // Graham-Buffett双重确认
            (
                InvestmentAction::Buy,
                0.85,
                "Graham-Buffett双重确认".to_string(),
                Some(0.18),
            )
        } else if graham.margin_of_safety > 0.15 {
            // 仅Graham信号
            (
                InvestmentAction::SmallBuy,
                0.65,
                "Graham安全边际,但质量尚需确认".to_string(),
                Some(0.12),
            )
        } else if buffett.meets_buffett_criteria {
            // 仅Buffett信号
            (
                InvestmentAction::SmallBuy,
                0.60,
                "Buffett优质企业,但安全边际不足".to_string(),
                Some(0.15),
            )
        } else {
            // 观望
            (
                InvestmentAction::Hold,
                0.40,
                "等待更好的价格或确认质量".to_string(),
                None,
            )
        };

        ComprehensiveDecision::new(
            symbol,
            action,
            confidence,
            reasoning,
            graham.clone(),
            buffett.clone(),
            munger.clone(),
            expected_return,
        )
    }

    /// 批量分析
    ///
    /// 分析多只股票并返回排序后的结果
    pub async fn batch_analyze(
        &self,
        stocks: Vec<serde_json::Value>,
    ) -> Result<Vec<ComprehensiveDecision>> {
        let mut decisions = Vec::new();

        for stock_data in stocks {
            let symbol = stock_data
                .get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");

            let decision = self.comprehensive_analysis(symbol, &stock_data).await?;
            decisions.push(decision);
        }

        // 按综合评分降序排序
        decisions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        Ok(decisions)
    }

    /// 筛选投资机会
    ///
    /// 根据投资行动筛选股票
    pub fn filter_opportunities(
        &self,
        decisions: &[ComprehensiveDecision],
        min_action: InvestmentAction,
    ) -> Vec<&ComprehensiveDecision> {
        decisions
            .iter()
            .filter(|d| {
                // 判断是否满足最小行动要求
                match min_action {
                    InvestmentAction::HeavyBuy => {
                        matches!(d.action, InvestmentAction::HeavyBuy | InvestmentAction::Buy | InvestmentAction::SmallBuy)
                    }
                    InvestmentAction::Buy => {
                        matches!(d.action, InvestmentAction::Buy | InvestmentAction::SmallBuy)
                    }
                    InvestmentAction::SmallBuy => {
                        matches!(d.action, InvestmentAction::SmallBuy)
                    }
                    InvestmentAction::Hold => true,
                    _ => false,
                }
            })
            .collect()
    }

    /// 生成投资组合建议
    ///
    /// 基于Munger的集中投资理念,推荐最佳投资组合
    pub async fn generate_portfolio_recommendation(
        &self,
        decisions: Vec<ComprehensiveDecision>,
        max_positions: usize,
    ) -> Result<Vec<PortfolioRecommendation>> {
        // 筛选出买入级别的股票
        let buy_opportunities: Vec<_> = decisions
            .into_iter()
            .filter(|d| {
                matches!(
                    d.action,
                    InvestmentAction::HeavyBuy | InvestmentAction::Buy | InvestmentAction::SmallBuy
                )
            })
            .take(max_positions)
            .collect();

        if buy_opportunities.is_empty() {
            return Ok(vec![]);
        }

        // Munger集中投资策略:前几大持仓重仓
        let total = buy_opportunities.len() as f64;
        let weights = vec![0.40, 0.30, 0.20, 0.10, 0.05];

        let mut recommendations = Vec::new();

        for (i, decision) in buy_opportunities.iter().enumerate() {
            let weight = if i < weights.len() {
                weights[i]
            } else {
                0.05 / total
            };

            recommendations.push(PortfolioRecommendation {
                symbol: decision.symbol.clone(),
                weight,
                action: decision.action.clone(),
                confidence: decision.confidence,
                reasoning: decision.reasoning.clone(),
                expected_return: decision.expected_return,
                score: decision.score,
            });
        }

        Ok(recommendations)
    }
}

/// 投资组合建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRecommendation {
    pub symbol: String,
    pub weight: f64,
    pub action: InvestmentAction,
    pub confidence: f64,
    pub reasoning: String,
    pub expected_return: Option<f64>,
    pub score: f64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_value_investing_framework() {
        let framework = ValueInvestingFramework::new(
            vec!["Technology".to_string(), "Finance".to_string()],
            vec!["SaaS".to_string(), "Platform".to_string()],
        );

        // 准备测试数据 (三位一体共振的完美案例)
        let data = json!({
            "symbol": "PERFECT",
            // Graham数据
            "eps": 8.0,
            "growth_rate": 0.12,
            "current_price": 100.0,
            "current_assets": 500_000_000.0,
            "total_liabilities": 100_000_000.0,
            "shares_outstanding": 10_000_000.0,
            // Buffett数据
            "roic": 0.18,
            "free_cash_flow": 2_000_000_000.0,
            "discount_rate": 0.10,
            "strong_brand": true,
            "cost_advantage": true,
            "high_switching_cost": true,
            "network_effects": true,
            "capital_allocation_score": 90.0,
            "shareholder_returns": 0.20,
            "transparency_score": 85.0,
            "insider_ownership": 0.10,
            // Munger数据
            "industry": "Technology",
            "business_model": "Platform",
            "margin_of_safety": 0.40,
            "expected_return": 0.20,
            "holding_period_years": 10,
            "alternative_return": 0.08
        });

        let decision = framework
            .comprehensive_analysis("PERFECT", &data)
            .await
            .unwrap();

        assert_eq!(decision.symbol, "PERFECT");
        assert!(decision.score > 70.0);
        assert!(decision.confidence > 0.6);
    }

    #[tokio::test]
    async fn test_batch_analyze() {
        let framework = ValueInvestingFramework::new(
            vec!["Technology".to_string()],
            vec!["SaaS".to_string()],
        );

        let stocks = vec![
            json!({
                "symbol": "STOCK1",
                "eps": 5.0,
                "growth_rate": 0.10,
                "current_price": 80.0,
                "roic": 0.15,
                "free_cash_flow": 1_000_000_000.0,
                "discount_rate": 0.10,
                "industry": "Technology",
                "business_model": "SaaS",
                "margin_of_safety": 0.30,
                "expected_return": 0.15,
                "holding_period_years": 5,
                "strong_brand": true,
                "network_effects": true,
                "capital_allocation_score": 80.0,
                "shareholder_returns": 0.12,
                "transparency_score": 75.0,
                "insider_ownership": 0.08,
                "cost_advantage": true,
                "high_switching_cost": false,
                "alternative_return": 0.08
            }),
            json!({
                "symbol": "STOCK2",
                "eps": 3.0,
                "growth_rate": 0.08,
                "current_price": 50.0,
                "roic": 0.12,
                "free_cash_flow": 500_000_000.0,
                "discount_rate": 0.10,
                "industry": "Technology",
                "business_model": "SaaS",
                "margin_of_safety": 0.25,
                "expected_return": 0.12,
                "holding_period_years": 3,
                "strong_brand": false,
                "network_effects": true,
                "capital_allocation_score": 70.0,
                "shareholder_returns": 0.10,
                "transparency_score": 70.0,
                "insider_ownership": 0.05,
                "cost_advantage": false,
                "high_switching_cost": false,
                "alternative_return": 0.08
            }),
        ];

        let decisions = framework.batch_analyze(stocks).await.unwrap();

        assert_eq!(decisions.len(), 2);
        // 应该按评分排序
        assert!(decisions[0].score >= decisions[1].score);
    }

    #[test]
    fn test_investment_action() {
        let action = InvestmentAction::HeavyBuy;
        assert_eq!(action.description(), "重仓买入 - 绝佳机会");

        let (min, max) = action.position_range();
        assert_eq!(min, 0.30);
        assert_eq!(max, 0.50);
    }
}
