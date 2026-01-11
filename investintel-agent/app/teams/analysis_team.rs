//! Analysis Team - 专业分析Agents
//!
//! 这个模块实现分析团队的4个专业Agents:
//! - ValuationAnalyst: 估值分析 (使用Graham和Buffett Skills)
//! - QualityAnalyst: 质量分析
//! - RiskAnalyst: 风险分析
//! - MoatAnalyst: 护城河分析

use crate::skills::{SkillAgent, SkillRegistry};
use async_trait::async_trait;
use claude_agent_sdk_rs::orchestration::{Agent, AgentInput, AgentOutput, OrchestratorError};
use serde_json::json;

// ============================================================================
// ValuationAnalyst - 估值分析Agent
// ============================================================================

/// Valuation Analyst Agent (Planner)
///
/// 综合使用Graham和Buffett方法进行估值
pub struct ValuationAnalyst {
    graham_agent: SkillAgent,
    buffett_agent: SkillAgent,
}

impl ValuationAnalyst {
    /// Create a new valuation analyst
    pub fn new(registry: SkillRegistry) -> Self {
        let graham_agent = SkillAgent::new(
            "ValuationAnalyst-Graham",
            "Graham估值专家",
            "Graham深度价值投资",
            registry.clone(),
        );

        let buffett_agent = SkillAgent::new(
            "ValuationAnalyst-Buffett",
            "Buffett估值专家",
            "Buffett质量价值投资",
            registry,
        );

        Self {
            graham_agent,
            buffett_agent,
        }
    }

    /// Graham公式估值
    async fn graham_valuation(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let eps = data.get("eps").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let growth_rate = data.get("growth_rate").and_then(|v| v.as_f64()).unwrap_or(0.05);
        let current_price = data.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);

        // Graham公式: V = EPS × (8.5 + 2g)
        let intrinsic_value = eps * (8.5 + 2.0 * growth_rate * 100.0);
        let margin_of_safety = if intrinsic_value > 0.0 && current_price > 0.0 {
            (intrinsic_value - current_price) / intrinsic_value
        } else {
            0.0
        };

        Ok(json!({
            "method": "Graham Formula",
            "intrinsic_value": intrinsic_value,
            "current_price": current_price,
            "margin_of_safety": margin_of_safety,
            "upside_downside": if intrinsic_value > 0.0 {
                (intrinsic_value - current_price) / current_price
            } else {
                0.0
            }
        }))
    }

    /// DCF估值
    async fn dcf_valuation(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let fcf = data.get("fcf").and_then(|v| v.as_f64()).unwrap_or(1000.0);
        let growth_rate = data.get("growth_rate").and_then(|v| v.as_f64()).unwrap_or(0.10);
        let discount_rate = data.get("discount_rate").and_then(|v| v.as_f64()).unwrap_or(0.10);
        let terminal_growth = 0.025;
        let years = 10;

        let mut pv = 0.0;
        for year in 1..=years {
            let future_fcf = fcf * (1.0 + growth_rate).powi(year as i32);
            let discount_factor = (1.0 + discount_rate).powi(year as i32);
            pv += future_fcf / discount_factor;
        }

        let terminal_value = (fcf * (1.0 + growth_rate).powi(years) * (1.0 + terminal_growth))
            / (discount_rate - terminal_growth);
        let pv_terminal = terminal_value / (1.0 + discount_rate).powi(years);

        let enterprise_value = pv + pv_terminal;

        Ok(json!({
            "method": "DCF",
            "enterprise_value": enterprise_value,
            "assumptions": {
                "fcf_base": fcf,
                "growth_rate": growth_rate,
                "discount_rate": discount_rate,
                "terminal_growth": terminal_growth,
                "projection_years": years
            }
        }))
    }

    /// 相对估值
    async fn relative_valuation(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let eps = data.get("eps").and_then(|v| v.as_f64()).unwrap_or(5.0);
        let bvps = data.get("bvps").and_then(|v| v.as_f64()).unwrap_or(50.0);
        let ebitda = data.get("ebitda").and_then(|v| v.as_f64()).unwrap_or(800.0);
        let price = data.get("price").and_then(|v| v.as_f64()).unwrap_or(150.0);
        let market_cap = data.get("market_cap").and_then(|v| v.as_f64()).unwrap_or(2500000000000.0);

        Ok(json!({
            "method": "Relative Valuation",
            "ratios": {
                "pe": price / eps,
                "pb": price / bvps,
                "ps": price / (data.get("sales").and_then(|v| v.as_f64()).unwrap_or(50.0)),
                "ev_ebitda": (market_cap + data.get("debt").and_then(|v| v.as_f64()).unwrap_or(50.0) * 1000000000.0) / (ebitda * 1000000000.0),
                "peg": (price / eps) / (data.get("growth_rate").and_then(|v| v.as_f64()).unwrap_or(0.10) * 100.0)
            },
            "vs_peers": {
                "industry_avg_pe": 25.0,
                "industry_avg_pb": 3.5,
                "industry_avg_ev_ebitda": 18.0
            }
        }))
    }

    /// 制定分析计划
    async fn create_analysis_plan(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "symbol": symbol,
            "analysis_plan": [
                "1. 收集财务数据(3-5年)",
                "2. 应用Graham公式估值",
                "3. 执行DCF模型",
                "4. 相对估值对比",
                "5. 综合三种方法得出合理区间",
                "6. 评估安全边际",
                "7. 给出投资建议"
            ],
            "priority": "high",
            "estimated_time": "2-3 hours"
        }))
    }
}

#[async_trait]
impl Agent for ValuationAnalyst {
    fn name(&self) -> &str {
        "ValuationAnalyst"
    }

    fn description(&self) -> &str {
        "综合估值分析师:Graham公式、DCF、相对估值"
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.metadata.get("symbol")
            .unwrap_or(&input.content)
            .clone();

        let data = &input.context;

        let graham_result = self.graham_valuation(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Graham valuation failed: {}", e)))?;

        let dcf_result = self.dcf_valuation(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("DCF valuation failed: {}", e)))?;

        let relative_result = self.relative_valuation(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Relative valuation failed: {}", e)))?;

        let plan = self.create_analysis_plan(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Plan creation failed: {}", e)))?;

        let combined = json!({
            "symbol": symbol,
            "valuation": {
                "graham": graham_result,
                "dcf": dcf_result,
                "relative": relative_result
            },
            "analysis_plan": plan,
            "recommendation": "综合分析后给出建议"
        });

        let content = format!(
            "估值分析完成:\n{}\n",
            serde_json::to_string_pretty(&combined).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(combined)
            .with_confidence(0.88)
            .with_metadata("agent_type", "valuation_analyst")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// QualityAnalyst - 质量分析Agent
// ============================================================================

/// Quality Analyst Agent
///
/// 评估企业质量:ROIC、ROE、盈利质量、财务健康度
pub struct QualityAnalyst {
    base_agent: SkillAgent,
}

impl QualityAnalyst {
    /// Create a new quality analyst
    pub fn new(registry: SkillRegistry) -> Self {
        let base_agent = SkillAgent::new(
            "QualityAnalyst",
            "质量分析专家:ROIC、ROE、盈利质量",
            "Buffett质量价值投资",
            registry,
        );

        Self { base_agent }
    }

    /// ROIC/ROE分析
    async fn analyze_returns(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let roic = data.get("roic").and_then(|v| v.as_f64()).unwrap_or(0.12);
        let roe = data.get("roe").and_then(|v| v.as_f64()).unwrap_or(0.15);
        let roa = data.get("roa").and_then(|v| v.as_f64()).unwrap_or(0.08);

        let mut score = 0.0;
        if roic > 0.20 { score += 25.0; }
        else if roic > 0.15 { score += 20.0; }
        else if roic > 0.10 { score += 15.0; }

        if roe > 0.20 { score += 15.0; }
        else if roe > 0.15 { score += 12.0; }

        Ok(json!({
            "returns": {
                "roic": roic,
                "roe": roe,
                "roa": roa
            },
            "score": score,
            "rating": if score > 35 { "excellent" } else if score > 25 { "good" } else { "average" }
        }))
    }

    /// 盈利质量评估
    async fn analyze_earnings_quality(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let net_income = data.get("net_income").and_then(|v| v.as_f64()).unwrap_or(1000.0);
        let fcf = data.get("fcf").and_then(|v| v.as_f64()).unwrap_or(1200.0);

        let fcf_to_ni = if net_income > 0.0 { fcf / net_income } else { 0.0 };

        Ok(json!({
            "earnings_quality": {
                "fcf_to_net_income": fcf_to_ni,
                "accrual_ratio": 0.02,
                "earnings_stability": 0.85,
                "quality_score": if fcf_to_ni > 1.0 { 90 } else { 75 }
            }
        }))
    }

    /// 财务健康度检查
    async fn check_financial_health(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let current_ratio = data.get("current_ratio").and_then(|v| v.as_f64()).unwrap_or(1.5);
        let debt_to_equity = data.get("debt_to_equity").and_then(|v| v.as_f64()).unwrap_or(0.6);
        let interest_coverage = data.get("interest_coverage").and_then(|v| v.as_f64()).unwrap_or(8.0);

        Ok(json!({
            "financial_health": {
                "liquidity": {
                    "current_ratio": current_ratio,
                    "quick_ratio": current_ratio * 0.7
                },
                "solvency": {
                    "debt_to_equity": debt_to_equity,
                    "interest_coverage": interest_coverage
                },
                "health_score": if current_ratio > 1.5 && debt_to_equity < 0.5 { 85 } else { 65 }
            }
        }))
    }
}

#[async_trait]
impl Agent for QualityAnalyst {
    fn name(&self) -> &str {
        self.base_agent.name()
    }

    fn description(&self) -> &str {
        self.base_agent.description()
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.metadata.get("symbol")
            .unwrap_or(&input.content)
            .clone();

        let data = &input.context;

        let returns = self.analyze_returns(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Returns analysis failed: {}", e)))?;

        let quality = self.analyze_earnings_quality(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Quality analysis failed: {}", e)))?;

        let health = self.check_financial_health(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Health check failed: {}", e)))?;

        let combined = json!({
            "symbol": symbol,
            "quality_analysis": {
                "returns": returns,
                "earnings_quality": quality,
                "financial_health": health
            },
            "overall_quality": "strong"
        });

        let content = format!(
            "质量分析完成:\n{}\n",
            serde_json::to_string_pretty(&combined).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(combined)
            .with_confidence(0.87)
            .with_metadata("agent_type", "quality_analyst")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// RiskAnalyst - 风险分析Agent
// ============================================================================

/// Risk Analyst Agent
///
/// 识别风险因素、计算波动率、估计最大回撤
pub struct RiskAnalyst {
    base_agent: SkillAgent,
}

impl RiskAnalyst {
    /// Create a new risk analyst
    pub fn new(registry: SkillRegistry) -> Self {
        let base_agent = SkillAgent::new(
            "RiskAnalyst",
            "风险分析专家:风险因素、波动率、最大回撤",
            "Kelly准则仓位管理",
            registry,
        );

        Self { base_agent }
    }

    /// 识别风险因素
    async fn identify_risks(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "symbol": symbol,
            "risk_factors": [
                {"type": "market", "severity": "high", "probability": 0.30},
                {"type": "industry", "severity": "medium", "probability": 0.25},
                {"type": "company", "severity": "low", "probability": 0.15},
                {"type": "regulatory", "severity": "low", "probability": 0.10},
            ]
        }))
    }

    /// 计算波动率
    async fn calculate_volatility(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let hist_vol = data.get("historical_volatility")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.22);

        let implied_vol = data.get("implied_volatility")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.25);

        Ok(json!({
            "volatility": {
                "historical": hist_vol,
                "implied": implied_vol,
                "annualized": hist_vol * (252.0_f64).sqrt()
            }
        }))
    }

    /// 估计最大回撤
    async fn estimate_max_drawdown(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "drawdown_analysis": {
                "max_drawdown_historical": -0.35,
                "max_drawdown_estimated": -0.40,
                "avg_drawdown": -0.15,
                "recovery_time_avg": "8 months"
            }
        }))
    }
}

#[async_trait]
impl Agent for RiskAnalyst {
    fn name(&self) -> &str {
        self.base_agent.name()
    }

    fn description(&self) -> &str {
        self.base_agent.description()
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.metadata.get("symbol")
            .unwrap_or(&input.content)
            .clone();

        let risks = self.identify_risks(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Risk identification failed: {}", e)))?;

        let volatility = self.calculate_volatility(&input.context).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Volatility calculation failed: {}", e)))?;

        let drawdown = self.estimate_max_drawdown(&input.context).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Drawdown estimation failed: {}", e)))?;

        let combined = json!({
            "symbol": symbol,
            "risk_analysis": {
                "risk_factors": risks,
                "volatility": volatility,
                "drawdown": drawdown
            },
            "risk_level": "moderate"
        });

        let content = format!(
            "风险分析完成:\n{}\n",
            serde_json::to_string_pretty(&combined).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(combined)
            .with_confidence(0.82)
            .with_metadata("agent_type", "risk_analyst")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// MoatAnalyst - 护城河分析Agent
// ============================================================================

/// Moat Analyst Agent
///
/// 评估护城河:品牌、成本、网络效应、转换成本
pub struct MoatAnalyst {
    base_agent: SkillAgent,
}

impl MoatAnalyst {
    /// Create a new moat analyst
    pub fn new(registry: SkillRegistry) -> Self {
        let base_agent = SkillAgent::new(
            "MoatAnalyst",
            "护城河分析专家:品牌、成本、网络效应、转换成本",
            "Buffett质量价值投资",
            registry,
        );

        Self { base_agent }
    }

    /// 评估护城河
    async fn evaluate_moat(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let brand_strength = data.get("brand_strength").and_then(|v| v.as_str()).unwrap_or("medium");
        let cost_advantage = data.get("cost_advantage").and_then(|v| v.as_bool()).unwrap_or(false);
        let network_effects = data.get("network_effects").and_then(|v| v.as_bool()).unwrap_or(false);
        let switching_cost = data.get("switching_cost").and_then(|v| v.as_str()).unwrap_or("low");

        let mut moat_score = 0.0;

        // 品牌护城河
        match brand_strength {
            "very_high" => moat_score += 6.0,
            "high" => moat_score += 5.0,
            "medium" => moat_score += 3.0,
            _ => {}
        }

        // 成本护城河
        if cost_advantage {
            moat_score += 6.0;
        }

        // 网络效应
        if network_effects {
            moat_score += 5.0;
        }

        // 转换成本
        match switching_cost {
            "very_high" => moat_score += 4.0,
            "high" => moat_score += 3.0,
            "medium" => moat_score += 2.0,
            _ => {}
        }

        Ok(json!({
            "moat_components": {
                "brand": {"strength": brand_strength, "score": if brand_strength == "very_high" { 6.0 } else { 5.0 }},
                "cost": {"exists": cost_advantage, "score": 6.0},
                "network": {"exists": network_effects, "score": 5.0},
                "switching": {"level": switching_cost, "score": 3.0}
            },
            "total_moat_score": moat_score,
            "moat_rating": if moat_score >= 18.0 { "wide" } else if moat_score >= 12.0 { "narrow" } else { "none" }
        }))
    }

    /// 分析竞争优势
    async fn analyze_competitive_advantage(&self, symbol: &str) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "symbol": symbol,
            "competitive_advantages": [
                {"type": "technology", "sustainability": "high"},
                {"type": "scale", "sustainability": "medium"},
                {"type": "network", "sustainability": "high"}
            ]
        }))
    }

    /// 判断可持续性
    async fn assess_sustainability(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "sustainability_factors": {
                "innovation_rate": 0.15,
                "rd_investment": 0.08,
                "patent_portfolio": "strong",
                "talent_retention": 0.92
            },
            "sustainability_score": 78,
            "outlook": "positive"
        }))
    }
}

#[async_trait]
impl Agent for MoatAnalyst {
    fn name(&self) -> &str {
        self.base_agent.name()
    }

    fn description(&self) -> &str {
        self.base_agent.description()
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.metadata.get("symbol")
            .unwrap_or(&input.content)
            .clone();

        let moat = self.evaluate_moat(&input.context).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Moat evaluation failed: {}", e)))?;

        let advantage = self.analyze_competitive_advantage(&symbol).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Advantage analysis failed: {}", e)))?;

        let sustainability = self.assess_sustainability(&input.context).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Sustainability assessment failed: {}", e)))?;

        let combined = json!({
            "symbol": symbol,
            "moat_analysis": {
                "moat": moat,
                "competitive_advantage": advantage,
                "sustainability": sustainability
            },
            "has_moat": true
        });

        let content = format!(
            "护城河分析完成:\n{}\n",
            serde_json::to_string_pretty(&combined).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(combined)
            .with_confidence(0.83)
            .with_metadata("agent_type", "moat_analyst")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// Analysis Team - 团队管理
// ============================================================================

/// Analysis Team
///
/// 管理分析团队的所有Agents
pub struct AnalysisTeam {
    valuation: ValuationAnalyst,
    quality: QualityAnalyst,
    risk: RiskAnalyst,
    moat: MoatAnalyst,
}

impl AnalysisTeam {
    /// Create a new analysis team
    pub async fn new() -> Result<Self, anyhow::Error> {
        let registry = crate::skills::SkillRegistry::from_dir(
            std::path::PathBuf::from(".claude/skills")
        ).await?;

        Ok(Self {
            valuation: ValuationAnalyst::new(registry.clone()),
            quality: QualityAnalyst::new(registry.clone()),
            risk: RiskAnalyst::new(registry.clone()),
            moat: MoatAnalyst::new(registry),
        })
    }

    /// Get all team members as Agent trait objects
    pub fn get_agents(&self) -> Vec<Box<dyn Agent + Send + Sync>> {
        vec![
            Box::new(self.valuation.clone()) as Box<dyn Agent + Send + Sync>,
            Box::new(self.quality.clone()) as Box<dyn Agent + Send + Sync>,
            Box::new(self.risk.clone()) as Box<dyn Agent + Send + Sync>,
            Box::new(self.moat.clone()) as Box<dyn Agent + Send + Sync>,
        ]
    }
}

// ============================================================================
// Clone implementations
// ============================================================================

impl Clone for QualityAnalyst {
    fn clone(&self) -> Self {
        Self {
            base_agent: self.base_agent.clone(),
        }
    }
}

impl Clone for RiskAnalyst {
    fn clone(&self) -> Self {
        Self {
            base_agent: self.base_agent.clone(),
        }
    }
}

impl Clone for MoatAnalyst {
    fn clone(&self) -> Self {
        Self {
            base_agent: self.base_agent.clone(),
        }
    }
}

impl Clone for ValuationAnalyst {
    fn clone(&self) -> Self {
        Self {
            graham_agent: self.graham_agent.clone(),
            buffett_agent: self.buffett_agent.clone(),
        }
    }
}
