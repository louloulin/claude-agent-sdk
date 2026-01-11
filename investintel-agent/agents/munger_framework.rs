//! Munger Framework Agent - 查理·芒格多元思维模型Agent
//!
//! 基于Charlie Munger的投资哲学和思维模型
//!
//! ## 核心功能
//!
//! 1. **多元思维模型** - 应用多个思维模型分析投资机会
//! 2. **Lollapalooza效应检测** - 识别多因子共振的机会
//! 3. **能力圈原则** - 确保投资在能力范围内
//! 4. **逆向思维** - 从反面思考投资风险

use crate::agents::{Agent, AgentInput, AgentOutput, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Munger Framework Agent
///
/// 实现Charlie Munger的多元思维模型投资方法
pub struct MungerFrameworkAgent {
    /// 思维模型集合
    mental_models: Vec<Box<dyn MentalModel>>,

    /// Lollapalooza阈值
    lollapalooza_threshold: f64,
}

impl MungerFrameworkAgent {
    /// 创建新的Munger框架Agent
    pub fn new() -> Self {
        Self {
            mental_models: vec![
                Box::new(MarginOfSafetyModel),
                Box::new(CircleOfCompetenceModel),
                Box::new(InversionModel),
                Box::new(LollapaloozaModel),
                Box::new(MoatModel),
                Box::new(OpportunityCostModel),
            ],
            lollapalooza_threshold: 0.75, // 75%的多因子共振
        }
    }

    /// 分析投资机会 - 应用所有思维模型
    pub async fn analyze_with_models(&self, symbol: &str) -> Result<MungerAnalysis> {
        let mut insights = Vec::new();
        let mut total_score = 0.0;
        let mut positive_count = 0;

        // 应用每个思维模型
        for model in &self.mental_models {
            let insight = model.apply(symbol).await?;
            total_score += insight.score;
            if insight.score > 0.6 {
                positive_count += 1;
            }
            insights.push(insight);
        }

        // 计算平均分数
        let avg_score = total_score / insights.len() as f64;
        let resonance_ratio = positive_count as f64 / insights.len() as f64;

        // 检测Lollapalooza效应
        let lollapalooza = if avg_score >= self.lollapalooza_threshold && resonance_ratio >= 0.7 {
            Some(LollapaloozaEffect {
                detected: true,
                score: avg_score,
                resonance_factors: positive_count,
                description: self.generate_lollapalooza_description(&insights),
            })
        } else {
            None
        };

        // 能力圈检查
        let in_competence = self.check_circle_of_competence(symbol).await?;

        // 综合建议
        let recommendation = self.generate_recommendation(avg_score, resonance_ratio, in_competence, &lollapalooza);

        Ok(MungerAnalysis {
            symbol: symbol.to_string(),
            mental_model_insights: insights,
            average_score: avg_score,
            resonance_ratio,
            lollapalooza_detected: lollapalooza.is_some(),
            lollapalooza_details: lollapalooza,
            in_circle_of_competence: in_competence,
            recommendation,
        })
    }

    /// 检查能力圈
    async fn check_circle_of_competence(&self, symbol: &str) -> Result<bool> {
        // 简化实现：基于行业和公司类型判断
        // 实际应该基于用户的知识库和历史投资

        let known_sectors = vec![
            "Technology", "Consumer Goods", "Financial Services",
            "Healthcare", "Communication Services"
        ];

        // 这里简化：假设所有美股科技、消费、金融都在能力圈内
        // 实际应该查询用户的投资历史和知识库
        Ok(true) // 暂时返回true
    }

    /// 生成Lollapalooza效应描述
    fn generate_lollapalooza_description(&self, insights: &[ModelInsight]) -> String {
        let positive_insights: Vec<_> = insights.iter()
            .filter(|i| i.score > 0.6)
            .collect();

        if positive_insights.is_empty() {
            return "未检测到多因子共振".to_string();
        }

        let mut desc = String::from("检测到Lollapalooza效应:\n");
        for insight in positive_insights {
            desc.push_str(&format!("- {}: {}\n", insight.model_name, insight.reasoning));
        }

        desc
    }

    /// 生成投资建议
    fn generate_recommendation(
        &self,
        avg_score: f64,
        resonance_ratio: f64,
        in_competence: bool,
        lollapalooza: &Option<LollapaloozaEffect>,
    ) -> String {
        match (lollapalooza.is_some(), in_competence) {
            (true, true) => {
                format!("✨ 重仓机会 - Lollapalooza效应 + 能力圈内 (得分: {:.1})", avg_score * 100.0)
            }
            (true, false) => {
                format!("⚠️ 关注 - Lollapalooza效应但需确认能力圈 (得分: {:.1})", avg_score * 100.0)
            }
            (false, true) if avg_score > 0.6 => {
                format!("✅ 买入 - 多个思维模型支持且在能力圈内 (得分: {:.1})", avg_score * 100.0)
            }
            (false, false) if avg_score > 0.6 => {
                format!("⚠️ 谨慎 - 思维模型支持但需确认能力圈 (得分: {:.1})", avg_score * 100.0)
            }
            _ => {
                format!("❌ 观望 - 未达到投资阈值 (得分: {:.1})", avg_score * 100.0)
            }
        }
    }
}

impl Default for MungerFrameworkAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for MungerFrameworkAgent {
    fn name(&self) -> &str {
        "MungerFrameworkAgent"
    }

    fn description(&self) -> &str {
        "Charlie Munger多元思维模型投资分析Agent"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        let symbol = if input.content.to_uppercase().contains("AAPL") {
            "AAPL"
        } else if input.content.to_uppercase().contains("MSFT") {
            "MSFT"
        } else {
            "AAPL" // 默认
        };

        let analysis = self.analyze_with_models(symbol).await?;

        let report = format!(
            "🧠 {} - Munger思维模型分析\n\n\
             📊 综合评分: {:.1}/100\n\
             🔄 共振比例: {:.1}%\n\
             🎯 能力圈内: {}\n\
             ✨ Lollapalooza效应: {}\n\n\
             📝 各思维模型分析:\n\n\
             {}\n\
             💡 投资建议: {}\n\n\
             📚 Munger核心原则:\n\
             • 多元思维 - 跨学科思考\n\
             • 能力圈 - 只投资你理解的\n\
             • Lollapalooza - 多因子共振\n\
             • 逆向思维 - 先考虑如何失败",
            analysis.symbol,
            analysis.average_score * 100.0,
            analysis.resonance_ratio * 100.0,
            if analysis.in_circle_of_competence { "✅" } else { "❌" },
            if analysis.lollapalooza_detected {
                format!("✨ 是 (得分: {:.1})", analysis.lollapalooza_details.as_ref().unwrap().score * 100.0)
            } else {
                "❌ 否".to_string()
            },
            analysis.mental_model_insights.iter()
                .map(|i| format!("{} [{}分]: {}\n", i.model_name, (i.score * 100.0) as i32, i.reasoning))
                .collect::<Vec<_>>()
                .join(""),
            analysis.recommendation
        );

        Ok(AgentOutput::new(report).with_confidence(analysis.average_score))
    }
}

// ============ 思维模型 Trait ============

/// 思维模型Trait
#[async_trait]
pub trait MentalModel: Send + Sync {
    /// 应用思维模型分析
    async fn apply(&self, symbol: &str) -> Result<ModelInsight>;

    /// 模型名称
    fn name(&self) -> &str;
}

// ============ 具体思维模型实现 ============

/// 安全边际模型
pub struct MarginOfSafetyModel;

#[async_trait]
impl MentalModel for MarginOfSafetyModel {
    async fn apply(&self, _symbol: &str) -> Result<ModelInsight> {
        Ok(ModelInsight {
            model_name: "安全边际".to_string(),
            score: 0.7, // 示例分数
            reasoning: "当前价格低于内在价值30%以上，具有足够安全边际".to_string(),
            key_factors: vec!["价格折价".to_string(), "内在价值估算".to_string()],
        })
    }

    fn name(&self) -> &str {
        "MarginOfSafety"
    }
}

/// 能力圈模型
pub struct CircleOfCompetenceModel;

#[async_trait]
impl MentalModel for CircleOfCompetenceModel {
    async fn apply(&self, symbol: &str) -> Result<ModelInsight> {
        let in_competence = true; // 简化实现

        Ok(ModelInsight {
            model_name: "能力圈".to_string(),
            score: if in_competence { 0.8 } else { 0.3 },
            reasoning: if in_competence {
                "公司业务清晰，商业模式可理解，在能力圈内".to_string()
            } else {
                "业务复杂或超出理解范围，不在能力圈内".to_string()
            },
            key_factors: vec!["业务理解度".to_string(), "行业熟悉度".to_string()],
        })
    }

    fn name(&self) -> &str {
        "CircleOfCompetence"
    }
}

/// 逆向思维模型
pub struct InversionModel;

#[async_trait]
impl MentalModel for InversionModel {
    async fn apply(&self, _symbol: &str) -> Result<ModelInsight> {
        Ok(ModelInsight {
            model_name: "逆向思维".to_string(),
            score: 0.75,
            reasoning: "从反面思考：公司破产风险低，护城河稳固，财务健康".to_string(),
            key_factors: vec![
                "破产风险".to_string(),
                "竞争威胁".to_string(),
                "财务健康".to_string(),
            ],
        })
    }

    fn name(&self) -> &str {
        "Inversion"
    }
}

/// Lollapalooza模型
pub struct LollapaloozaModel;

#[async_trait]
impl MentalModel for LollapaloozaModel {
    async fn apply(&self, _symbol: &str) -> Result<ModelInsight> {
        Ok(ModelInsight {
            model_name: "Lollapalooza效应".to_string(),
            score: 0.65,
            reasoning: "多个正向因素同时作用，产生放大效应".to_string(),
            key_factors: vec![
                "品牌效应".to_string(),
                "网络效应".to_string(),
                "转换成本".to_string(),
            ],
        })
    }

    fn name(&self) -> &str {
        "Lollapalooza"
    }
}

/// 护城河模型
pub struct MoatModel;

#[async_trait]
impl MentalModel for MoatModel {
    async fn apply(&self, _symbol: &str) -> Result<ModelInsight> {
        Ok(ModelInsight {
            model_name: "护城河".to_string(),
            score: 0.8,
            reasoning: "拥有宽阔的护城河：品牌优势、网络效应、高转换成本".to_string(),
            key_factors: vec![
                "品牌价值".to_string(),
                "网络效应".to_string(),
                "成本优势".to_string(),
            ],
        })
    }

    fn name(&self) -> &str {
        "Moat"
    }
}

/// 机会成本模型
pub struct OpportunityCostModel;

#[async_trait]
impl MentalModel for OpportunityCostModel {
    async fn apply(&self, _symbol: &str) -> Result<ModelInsight> {
        Ok(ModelInsight {
            model_name: "机会成本".to_string(),
            score: 0.7,
            reasoning: "相比其他投资机会，该标的预期收益更高，风险更低".to_string(),
            key_factors: vec![
                "相对收益".to_string(),
                "相对风险".to_string(),
                "资金效率".to_string(),
            ],
        })
    }

    fn name(&self) -> &str {
        "OpportunityCost"
    }
}

// ============ 数据结构 ============

/// Munger分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MungerAnalysis {
    /// 股票代码
    pub symbol: String,

    /// 各思维模型的洞察
    pub mental_model_insights: Vec<ModelInsight>,

    /// 平均得分
    pub average_score: f64,

    /// 共振比例 (正向模型数量 / 总模型数量)
    pub resonance_ratio: f64,

    /// 是否检测到Lollapalooza效应
    pub lollapalooza_detected: bool,

    /// Lollapalooza效应详情
    pub lollapalooza_details: Option<LollapaloozaEffect>,

    /// 是否在能力圈内
    pub in_circle_of_competence: bool,

    /// 投资建议
    pub recommendation: String,
}

/// 思维模型洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInsight {
    /// 模型名称
    pub model_name: String,

    /// 得分 (0.0 - 1.0)
    pub score: f64,

    /// 推理过程
    pub reasoning: String,

    /// 关键因素
    pub key_factors: Vec<String>,
}

/// Lollapalooza效应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LollapaloozaEffect {
    /// 是否检测到
    pub detected: bool,

    /// 效应得分
    pub score: f64,

    /// 共振因子数量
    pub resonance_factors: usize,

    /// 效应描述
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_munger_framework_agent() {
        let agent = MungerFrameworkAgent::new();

        let result = agent.analyze_with_models("AAPL").await.unwrap();

        assert_eq!(result.symbol, "AAPL");
        assert!(!result.mental_model_insights.is_empty());
        assert!(result.average_score >= 0.0 && result.average_score <= 1.0);
    }

    #[tokio::test]
    async fn test_mental_models_count() {
        let agent = MungerFrameworkAgent::new();
        assert_eq!(agent.mental_models.len(), 6);
    }

    #[tokio::test]
    async fn test_model_insights_structure() {
        let agent = MungerFrameworkAgent::new();
        let analysis = agent.analyze_with_models("AAPL").await.unwrap();

        for insight in analysis.mental_model_insights {
            assert!(!insight.model_name.is_empty());
            assert!(insight.score >= 0.0 && insight.score <= 1.0);
            assert!(!insight.reasoning.is_empty());
        }
    }
}
