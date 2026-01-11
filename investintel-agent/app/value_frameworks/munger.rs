//! Munger Framework - 多元思维模型
//!
//! 基于Charlie Munger的多元思维模型投资方法实现
//! 包括思维模型、Lollapalooza效应检测和能力圈分析

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Mental Model - 思维模型
// ============================================================================

/// 思维模型trait
#[async_trait::async_trait]
pub trait MentalModel: Send + Sync {
    /// 获取模型名称
    fn name(&self) -> &str;

    /// 获取模型描述
    fn description(&self) -> &str;

    /// 应用思维模型分析
    async fn apply(&self, symbol: &str, data: &serde_json::Value) -> Result<ModelInsight>;
}

/// 模型洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInsight {
    /// 模型名称
    pub name: String,

    /// 洞察强度 (0-1)
    pub strength: f64,

    /// 情绪 (正向/负向/中性)
    pub sentiment: InsightSentiment,

    /// 洞察描述
    pub description: String,

    /// 关键因素
    pub key_factors: Vec<String>,

    /// 评分 (0-100)
    pub score: f64,
}

/// 洞察情绪
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsightSentiment {
    Positive,
    Neutral,
    Negative,
}

// ============================================================================
// Concrete Mental Models - 具体思维模型实现
// ============================================================================

/// 1. 逆向思维模型 (Inversion)
///
/// "告诉我我会死在哪里,这样我就永远不去那里"
/// - Charlie Munger
#[derive(Debug, Clone)]
pub struct InversionModel;

#[async_trait::async_trait]
impl MentalModel for InversionModel {
    fn name(&self) -> &str {
        "逆向思维"
    }

    fn description(&self) -> &str {
        "通过避免愚蠢来获得智慧,先思考如何失败然后避免"
    }

    async fn apply(&self, _symbol: &str, data: &serde_json::Value) -> Result<ModelInsight> {
        // 检查可能导致失败的因素
        let mut risk_factors = Vec::new();
        let mut total_score = 100.0;

        // 1. 高债务风险
        if let Some(debt_ratio) = data.get("debt_to_equity").and_then(|v| v.as_f64()) {
            if debt_ratio > 2.0 {
                risk_factors.push(format!("高债务比率: {:.1}", debt_ratio));
                total_score -= 20.0;
            }
        }

        // 2. 盈利波动
        if let Some(volatility) = data.get("earnings_volatility").and_then(|v| v.as_f64()) {
            if volatility > 0.3 {
                risk_factors.push(format!("盈利波动率高: {:.1}%", volatility * 100.0));
                total_score -= 15.0;
            }
        }

        // 3. 行业衰退
        if let Some(industry_decline) = data.get("industry_declining").and_then(|v| v.as_bool()) {
            if industry_decline {
                risk_factors.push("行业处于衰退期".to_string());
                total_score -= 25.0;
            }
        }

        // 4. 激烈竞争
        if let Some(high_competition) = data.get("intense_competition").and_then(|v| v.as_bool()) {
            if high_competition {
                risk_factors.push("竞争激烈".to_string());
                total_score -= 15.0;
            }
        }

        let strength = if total_score >= 80.0 {
            0.2
        } else if total_score >= 60.0 {
            0.1
        } else {
            0.0
        };

        let sentiment = if total_score >= 70.0 {
            InsightSentiment::Positive
        } else if total_score >= 50.0 {
            InsightSentiment::Neutral
        } else {
            InsightSentiment::Negative
        };

        let description = if risk_factors.is_empty() {
            "未发现明显的失败风险因素".to_string()
        } else {
            format!("需要避免的风险: {}", risk_factors.join(", "))
        };

        Ok(ModelInsight {
            name: self.name().to_string(),
            strength,
            sentiment,
            description,
            key_factors: risk_factors,
            score: total_score.max(0.0),
        })
    }
}

/// 2. 能力圈模型 (Circle of Competence)
///
/// "知道自己不知道什么比聪明更重要"
/// - Charlie Munger
#[derive(Debug, Clone)]
pub struct CircleOfCompetenceModel {
    /// 熟悉的行业
    familiar_industries: Vec<String>,

    /// 熟悉的商业模式
    familiar_business_models: Vec<String>,
}

impl CircleOfCompetenceModel {
    pub fn new(familiar_industries: Vec<String>, familiar_business_models: Vec<String>) -> Self {
        Self {
            familiar_industries,
            familiar_business_models,
        }
    }
}

#[async_trait::async_trait]
impl MentalModel for CircleOfCompetenceModel {
    fn name(&self) -> &str {
        "能力圈"
    }

    fn description(&self) -> &str {
        "只投资在能力圈内的企业,避免超出理解范围"
    }

    async fn apply(&self, symbol: &str, data: &serde_json::Value) -> Result<ModelInsight> {
        let industry = data
            .get("industry")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let business_model = data
            .get("business_model")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        // 检查是否在能力圈内
        let in_industry_circle = self
            .familiar_industries
            .iter()
            .any(|i| industry.to_lowercase().contains(&i.to_lowercase()));

        let in_model_circle = self
            .familiar_business_models
            .iter()
            .any(|m| business_model.to_lowercase().contains(&m.to_lowercase()));

        let in_circle = in_industry_circle || in_model_circle;

        let strength = if in_circle { 0.9 } else { 0.1 };

        let sentiment = if in_circle {
            InsightSentiment::Positive
        } else {
            InsightSentiment::Negative
        };

        let description = if in_circle {
            format!(
                "在能力圈内: 行业={}, 商业模式={}",
                industry, business_model
            )
        } else {
            format!(
                "超出能力圈: 行业={}, 商业模式={}",
                industry, business_model
            )
        };

        let score = if in_circle { 90.0 } else { 30.0 };

        let key_factors = vec![
            format!("行业: {}", industry),
            format!("商业模式: {}", business_model),
            format!("在能力圈内: {}", in_circle),
        ];

        Ok(ModelInsight {
            name: self.name().to_string(),
            strength,
            sentiment,
            description,
            key_factors,
            score,
        })
    }
}

/// 3. 安全边际模型 (Margin of Safety)
#[derive(Debug, Clone)]
pub struct MarginOfSafetyModel;

#[async_trait::async_trait]
impl MentalModel for MarginOfSafetyModel {
    fn name(&self) -> &str {
        "安全边际"
    }

    fn description(&self) -> &str {
        "留出安全边际以应对不确定性和错误"
    }

    async fn apply(&self, _symbol: &str, data: &serde_json::Value) -> Result<ModelInsight> {
        let margin = data
            .get("margin_of_safety")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let (strength, sentiment, description, score) = if margin >= 0.40 {
            (
                0.95,
                InsightSentiment::Positive,
                format!("极佳的安全边际: {:.1}%", margin * 100.0),
                95.0,
            )
        } else if margin >= 0.30 {
            (
                0.85,
                InsightSentiment::Positive,
                format!("良好的安全边际: {:.1}%", margin * 100.0),
                85.0,
            )
        } else if margin >= 0.20 {
            (
                0.6,
                InsightSentiment::Neutral,
                format!("适度的安全边际: {:.1}%", margin * 100.0),
                60.0,
            )
        } else if margin >= 0.10 {
            (
                0.3,
                InsightSentiment::Neutral,
                format!("较低的安全边际: {:.1}%", margin * 100.0),
                30.0,
            )
        } else {
            (
                0.0,
                InsightSentiment::Negative,
                format!("不足的安全边际: {:.1}%", margin * 100.0),
                10.0,
            )
        };

        Ok(ModelInsight {
            name: self.name().to_string(),
            strength,
            sentiment,
            description,
            key_factors: vec![format!("安全边际: {:.1}%", margin * 100.0)],
            score,
        })
    }
}

/// 4. 护城河模型 (Moat)
#[derive(Debug, Clone)]
pub struct MoatModel;

#[async_trait::async_trait]
impl MentalModel for MoatModel {
    fn name(&self) -> &str {
        "护城河"
    }

    fn description(&self) -> &str {
        "寻找具有持久竞争优势的企业"
    }

    async fn apply(&self, _symbol: &str, data: &serde_json::Value) -> Result<ModelInsight> {
        let mut score = 0.0;
        let mut factors = Vec::new();

        // 品牌价值
        if data.get("strong_brand").and_then(|v| v.as_bool()).unwrap_or(false) {
            score += 25.0;
            factors.push("强大品牌".to_string());
        }

        // 成本优势
        if data
            .get("cost_advantage")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            score += 25.0;
            factors.push("成本优势".to_string());
        }

        // 转换成本
        if data
            .get("high_switching_cost")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            score += 20.0;
            factors.push("高转换成本".to_string());
        }

        // 网络效应
        if data
            .get("network_effects")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            score += 30.0;
            factors.push("网络效应".to_string());
        }

        let strength = score / 100.0;

        let sentiment = if score >= 70.0 {
            InsightSentiment::Positive
        } else if score >= 40.0 {
            InsightSentiment::Neutral
        } else {
            InsightSentiment::Negative
        };

        let description = if factors.is_empty() {
            "无明显竞争优势".to_string()
        } else {
            format!("竞争优势: {}", factors.join(", "))
        };

        Ok(ModelInsight {
            name: self.name().to_string(),
            strength,
            sentiment,
            description,
            key_factors: factors,
            score,
        })
    }
}

/// 5. 机会成本模型 (Opportunity Cost)
#[derive(Debug, Clone)]
pub struct OpportunityCostModel;

#[async_trait::async_trait]
impl MentalModel for OpportunityCostModel {
    fn name(&self) -> &str {
        "机会成本"
    }

    fn description(&self) &str {
        "考虑最佳替代方案的收益"
    }

    async fn apply(&self, _symbol: &str, data: &serde_json::Value) -> Result<ModelInsight> {
        let expected_return = data
            .get("expected_return")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let alternative_return = data
            .get("alternative_return")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.08); // 默认8%市场回报

        let excess_return = expected_return - alternative_return;

        let (strength, sentiment, description, score) = if excess_return > 0.05 {
            (
                0.9,
                InsightSentiment::Positive,
                format!(
                    "显著优于替代方案: 超额收益 {:.1}%",
                    excess_return * 100.0
                ),
                90.0,
            )
        } else if excess_return > 0.0 {
            (
                0.6,
                InsightSentiment::Positive,
                format!(
                    "略优于替代方案: 超额收益 {:.1}%",
                    excess_return * 100.0
                ),
                60.0,
            )
        } else if excess_return > -0.02 {
            (
                0.4,
                InsightSentiment::Neutral,
                format!(
                    "接近替代方案: 差异 {:.1}%",
                    excess_return * 100.0
                ),
                40.0,
            )
        } else {
            (
                0.1,
                InsightSentiment::Negative,
                format!(
                    "劣于替代方案: 机会成本 {:.1}%",
                    (-excess_return) * 100.0
                ),
                20.0,
            )
        };

        Ok(ModelInsight {
            name: self.name().to_string(),
            strength,
            sentiment,
            description,
            key_factors: vec![
                format!("预期收益: {:.1}%", expected_return * 100.0),
                format!("替代收益: {:.1}%", alternative_return * 100.0),
                format!("超额收益: {:.1}%", excess_return * 100.0),
            ],
            score,
        })
    }
}

/// 6. 复利模型 (Compound Interest)
#[derive(Debug, Clone)]
pub struct CompoundInterestModel;

#[async_trait::async_trait]
impl MentalModel for CompoundInterestModel {
    fn name(&self) -> &str {
        "复利效应"
    }

    fn description(&self) -> &str {
        "理解复利的力量,长期持有优质企业"
    }

    async fn apply(&self, _symbol: &str, data: &serde_json::Value) -> Result<ModelInsight> {
        let expected_return = data
            .get("expected_return")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.10);

        let holding_period = data
            .get("holding_period_years")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as f64;

        // 计算复利效果
        let compound_multiplier = (1.0 + expected_return).powf(holding_period);

        let (strength, sentiment, description, score) = if expected_return >= 0.15
            && holding_period >= 10.0
        {
            (
                0.95,
                InsightSentiment::Positive,
                format!(
                    "优秀的复利潜力: {:.2}年内增长{:.1}倍",
                    holding_period, compound_multiplier
                ),
                95.0,
            )
        } else if expected_return >= 0.10 && holding_period >= 5.0 {
            (
                0.8,
                InsightSentiment::Positive,
                format!(
                    "良好的复利潜力: {:.2}年内增长{:.1}倍",
                    holding_period, compound_multiplier
                ),
                80.0,
            )
        } else {
            (
                0.5,
                InsightSentiment::Neutral,
                format!(
                    "一般的复利潜力: {:.2}年内增长{:.1}倍",
                    holding_period, compound_multiplier
                ),
                50.0,
            )
        };

        Ok(ModelInsight {
            name: self.name().to_string(),
            strength,
            sentiment,
            description,
            key_factors: vec![
                format!("年化收益: {:.1}%", expected_return * 100.0),
                format!("持有年限: {:.1}年", holding_period),
                format!("复利倍数: {:.1}x", compound_multiplier),
            ],
            score,
        })
    }
}

// ============================================================================
// Lollapalooza Detector - Lollapalooza效应检测器
// ============================================================================

/// Lollapalooza效应
///
/// 多个正向因素同时出现,产生放大效应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lollapalooza {
    /// 是否检测到
    pub detected: bool,

    /// 综合评分 (0-1)
    pub score: f64,

    /// 强因子数量
    pub strong_factors: usize,

    /// 贡献因子列表
    pub contributing_factors: Vec<String>,

    /// 推理说明
    pub reasoning: String,

    /// 放大倍数
    pub amplification: f64,
}

/// Lollapalooza效应检测器
///
/// 检测多个思维模型/因子同时出现的放大效应
#[derive(Debug, Clone)]
pub struct LollapaloozaDetector {
    /// 检测阈值 (默认0.8,即需要强因子)
    pub strong_threshold: f64,

    /// 最小因子数量 (默认3个)
    pub min_factors: usize,
}

impl Default for LollapaloozaDetector {
    fn default() -> Self {
        Self {
            strong_threshold: 0.8,
            min_factors: 3,
        }
    }
}

impl LollapaloozaDetector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.strong_threshold = threshold;
        self
    }

    pub fn with_min_factors(mut self, min_factors: usize) -> Self {
        self.min_factors = min_factors;
        self
    }

    /// 检测Lollapalooza效应
    ///
    /// Lollapalooza = 多个思维模型/因子同时出现,产生放大效应
    pub async fn detect(&self, insights: &[ModelInsight]) -> Result<Option<Lollapalooza>> {
        // 统计强正向因子
        let strong_positive: Vec<_> = insights
            .iter()
            .filter(|i| {
                i.strength >= self.strong_threshold
                    && i.sentiment == InsightSentiment::Positive
            })
            .collect();

        if strong_positive.len() >= self.min_factors {
            // 检测到Lollapalooza效应
            let combined_score = self.calculate_lollapalooza_score(insights);

            let contributing_factors = strong_positive
                .iter()
                .map(|i| i.name.clone())
                .collect();

            let amplification = 1.0 + (strong_positive.len() as f64 * 0.1);

            Ok(Some(Lollapalooza {
                detected: true,
                score: combined_score,
                strong_factors: strong_positive.len(),
                contributing_factors,
                reasoning: format!(
                    "Lollapalooza效应: {}个强因子共振,综合评分{:.2},放大{:.1}倍",
                    strong_positive.len(),
                    combined_score,
                    amplification
                ),
                amplification,
            }))
        } else {
            Ok(None)
        }
    }

    /// 计算Lollapalooza综合评分
    fn calculate_lollapalooza_score(&self, insights: &[ModelInsight]) -> f64 {
        let strong_insights: Vec<_> = insights
            .iter()
            .filter(|i| i.strength >= self.strong_threshold)
            .collect();

        if strong_insights.is_empty() {
            return 0.0;
        }

        // 计算平均强度
        let avg_strength = strong_insights.iter().map(|i| i.strength).sum::<f64>()
            / strong_insights.len() as f64;

        // Lollapalooza效应有放大作用
        let amplification = 1.0 + (strong_insights.len() as f64 * 0.1);

        avg_strength * amplification
    }

    /// 评估Lollapalooza投资机会级别
    pub fn evaluate_opportunity_level(&self, lollapalooza: &Lollapalooza) -> &'static str {
        if lollapalooza.score >= 0.95 && lollapalooza.strong_factors >= 5 {
            "绝佳机会 - 多因素超级共振"
        } else if lollapalooza.score >= 0.85 && lollapalooza.strong_factors >= 4 {
            "优秀机会 - 强力多因子共振"
        } else if lollapalooza.score >= 0.75 && lollapalooza.strong_factors >= 3 {
            "良好机会 - 多因子共振"
        } else {
            "一般机会 - 基本共振"
        }
    }
}

// ============================================================================
// Circle of Competence - 能力圈
// ============================================================================

/// 能力圈
#[derive(Debug, Clone)]
pub struct CircleOfCompetence {
    /// 熟悉的行业
    familiar_industries: Vec<String>,

    /// 熟悉的商业模式
    familiar_business_models: Vec<String>,
}

impl CircleOfCompetence {
    pub fn new(familiar_industries: Vec<String>, familiar_business_models: Vec<String>) -> Self {
        Self {
            familiar_industries,
            familiar_business_models,
        }
    }

    /// 检查是否在能力圈内
    pub async fn check(&self, industry: &str, business_model: &str) -> bool {
        let in_industry = self
            .familiar_industries
            .iter()
            .any(|i| industry.to_lowercase().contains(&i.to_lowercase()));

        let in_model = self
            .familiar_business_models
            .iter()
            .any(|m| business_model.to_lowercase().contains(&m.to_lowercase()));

        in_industry || in_model
    }

    /// 扩展能力圈
    pub fn expand_competence(&mut self, industry: String, business_model: String) {
        if !self.familiar_industries.contains(&industry) {
            self.familiar_industries.push(industry);
        }
        if !self.familiar_business_models.contains(&business_model) {
            self.familiar_business_models.push(business_model);
        }
    }

    /// 获取能力圈覆盖度
    pub fn coverage_ratio(&self, industry: &str, business_model: &str) -> f64 {
        let mut covered = 0;
        let total = 2;

        if self
            .familiar_industries
            .iter()
            .any(|i| industry.to_lowercase().contains(&i.to_lowercase()))
        {
            covered += 1;
        }

        if self
            .familiar_business_models
            .iter()
            .any(|m| business_model.to_lowercase().contains(&m.to_lowercase()))
        {
            covered += 1;
        }

        covered as f64 / total as f64
    }
}

// ============================================================================
// Munger Analysis - Munger分析结果
// ============================================================================

/// Munger分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MungerAnalysis {
    /// 股票代码
    pub symbol: String,

    /// 所有思维模型的洞察
    pub mental_model_insights: Vec<ModelInsight>,

    /// 是否检测到Lollapalooza效应
    pub lollapalooza_detected: bool,

    /// Lollapalooza评分
    pub lollapalooza_score: f64,

    /// Lollapalooza详细信息
    pub lollapalooza_details: Option<Lollapalooza>,

    /// 是否在能力圈内
    pub in_circle_of_competence: bool,

    /// 能力圈覆盖度
    pub competence_coverage: f64,

    /// 投资建议
    pub recommendation: String,

    /// 综合评分 (0-100)
    pub score: f64,
}

impl MungerAnalysis {
    pub fn new(
        symbol: String,
        mental_model_insights: Vec<ModelInsight>,
        lollapalooza_details: Option<Lollapalooza>,
        in_circle_of_competence: bool,
        competence_coverage: f64,
    ) -> Self {
        let lollapalooza_detected = lollapalooza_details.is_some();
        let lollapalooza_score = lollapalooza_details
            .as_ref()
            .map(|l| l.score)
            .unwrap_or(0.0);

        // 计算综合评分
        let avg_insight_score = if mental_model_insights.is_empty() {
            0.0
        } else {
            mental_model_insights.iter().map(|i| i.score).sum::<f64>()
                / mental_model_insights.len() as f64
        };

        let lollapalooza_bonus = if lollapalooza_detected {
            20.0
        } else {
            0.0
        };

        let competence_bonus = competence_coverage * 10.0;

        let score = avg_insight_score * 0.6 + lollapalooza_bonus + competence_bonus;

        // 生成建议
        let recommendation = if lollapalooza_detected && in_circle_of_competence {
            "重仓机会 - Lollapalooza效应 + 能力圈内".to_string()
        } else if lollapalooza_detected {
            "关注 - Lollapalooza效应但需确认能力圈".to_string()
        } else if in_circle_of_competence && score > 60.0 {
            "考虑 - 能力圈内且评分良好".to_string()
        } else {
            "观察 - 未达到Lollapalooza阈值".to_string()
        };

        Self {
            symbol,
            mental_model_insights,
            lollapalooza_detected,
            lollapalooza_score,
            lollapalooza_details,
            in_circle_of_competence,
            competence_coverage,
            recommendation,
            score: score.min(100.0),
        }
    }
}

// ============================================================================
// Munger Framework - Munger框架整合
// ============================================================================

/// Munger框架
///
/// 整合多元思维模型、Lollapalooza检测和能力圈的完整框架
#[derive(Debug, Clone)]
pub struct MungerFramework {
    /// 思维模型集合
    mental_models: Vec<Box<dyn MentalModel>>,

    /// Lollapalooza检测器
    lollapalooza_detector: LollapaloozaDetector,

    /// 能力圈
    circle_of_competence: CircleOfCompetence,
}

impl MungerFramework {
    pub fn new(familiar_industries: Vec<String>, familiar_business_models: Vec<String>) -> Self {
        // 创建默认的思维模型集合
        let mental_models: Vec<Box<dyn MentalModel>> = vec![
            Box::new(InversionModel),
            Box::new(CircleOfCompetenceModel::new(
                familiar_industries.clone(),
                familiar_business_models.clone(),
            )),
            Box::new(MarginOfSafetyModel),
            Box::new(MoatModel),
            Box::new(OpportunityCostModel),
            Box::new(CompoundInterestModel),
        ];

        Self {
            mental_models,
            lollapalooza_detector: LollapaloozaDetector::new(),
            circle_of_competence: CircleOfCompetence::new(
                familiar_industries,
                familiar_business_models,
            ),
        }
    }

    /// 执行完整的Munger分析
    pub async fn analyze(
        &self,
        symbol: String,
        data: &serde_json::Value,
    ) -> Result<MungerAnalysis> {
        // 1. 应用所有思维模型
        let mut insights = Vec::new();
        for model in &self.mental_models {
            let insight = model.apply(&symbol, data).await?;
            insights.push(insight);
        }

        // 2. 检测Lollapalooza效应
        let lollapalooza_details = self.lollapalooza_detector.detect(&insights).await?;

        // 3. 检查能力圈
        let industry = data
            .get("industry")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let business_model = data
            .get("business_model")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        let in_circle = self.circle_of_competence.check(industry, business_model).await;
        let competence_coverage = self.circle_of_competence.coverage_ratio(industry, business_model);

        // 4. 创建分析结果
        let analysis = MungerAnalysis::new(
            symbol,
            insights,
            lollapalooza_details,
            in_circle,
            competence_coverage,
        );

        Ok(analysis)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_mental_models() {
        let inversion = InversionModel;
        let data = json!({
            "debt_to_equity": 1.5,
            "earnings_volatility": 0.2,
            "industry_declining": false,
            "intense_competition": false
        });

        let insight = inversion.apply("TEST", &data).await.unwrap();
        assert_eq!(insight.name, "逆向思维");
        assert!(insight.score > 50.0);
    }

    #[tokio::test]
    async fn test_lollapalooza_detector() {
        let detector = LollapaloozaDetector::new();

        let insights = vec![
            ModelInsight {
                name: "模型1".to_string(),
                strength: 0.9,
                sentiment: InsightSentiment::Positive,
                description: "优秀".to_string(),
                key_factors: vec![],
                score: 90.0,
            },
            ModelInsight {
                name: "模型2".to_string(),
                strength: 0.85,
                sentiment: InsightSentiment::Positive,
                description: "良好".to_string(),
                key_factors: vec![],
                score: 85.0,
            },
            ModelInsight {
                name: "模型3".to_string(),
                strength: 0.8,
                sentiment: InsightSentiment::Positive,
                description: "不错".to_string(),
                key_factors: vec![],
                score: 80.0,
            },
        ];

        let lollapalooza = detector.detect(&insights).await.unwrap();
        assert!(lollapalooza.is_some());
        assert_eq!(lollapalooza.as_ref().unwrap().strong_factors, 3);
    }

    #[tokio::test]
    async fn test_munger_framework() {
        let framework = MungerFramework::new(
            vec!["Technology".to_string(), "Finance".to_string()],
            vec!["SaaS".to_string(), "Platform".to_string()],
        );

        let data = json!({
            "industry": "Technology",
            "business_model": "SaaS",
            "margin_of_safety": 0.35,
            "expected_return": 0.15,
            "holding_period_years": 10,
            "strong_brand": true,
            "network_effects": true,
            "debt_to_equity": 0.5,
            "earnings_volatility": 0.1,
            "industry_declining": false,
            "intense_competition": false
        });

        let analysis = framework.analyze("AAPL".to_string(), &data).await.unwrap();
        assert_eq!(analysis.symbol, "AAPL");
        assert!(analysis.score > 0.0);
    }
}
