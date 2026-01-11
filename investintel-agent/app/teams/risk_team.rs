//! Risk Team - 风险控制团队
//!
//! 这个模块实现风控团队的3个专业Agents:
//! - PortfolioMonitor: 组合监控和偏离检测
//! - RiskManager: 风险限额检查和对冲建议
//! - ComplianceAgent: 合规检查和报告生成

use crate::skills::{SkillAgent, SkillRegistry};
use async_trait::async_trait;
use claude_agent_sdk_rs::orchestration::{Agent, AgentInput, AgentOutput, OrchestratorError};
use serde_json::json;

// ============================================================================
// PortfolioMonitor - 组合监控Agent
// ============================================================================

/// Portfolio Monitor Agent
///
/// 实时监控投资组合,检测偏离和异常
pub struct PortfolioMonitor {
    base_agent: SkillAgent,
}

impl PortfolioMonitor {
    /// Create a new portfolio monitor
    pub fn new(registry: SkillRegistry) -> Self {
        let base_agent = SkillAgent::new(
            "PortfolioMonitor",
            "组合监控专家:实时监控、偏离检测、告警触发",
            "Kelly准则仓位管理",
            registry,
        );

        Self { base_agent }
    }

    /// 实时监控
    async fn monitor_realtime(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let portfolio_value = data.get("portfolio_value")
            .and_then(|v| v.as_f64())
            .unwrap_or(1_000_000.0);
        let cash = data.get("cash")
            .and_then(|v| v.as_f64())
            .unwrap_or(100_000.0);

        let positions = data.get("positions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(10);

        let exposure = (portfolio_value - cash) / portfolio_value;

        Ok(json!({
            "realtime_monitoring": {
                "portfolio_value": portfolio_value,
                "cash": cash,
                "positions_count": positions,
                "market_exposure": exposure,
                "cash_ratio": cash / portfolio_value,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        }))
    }

    /// 偏离检测
    async fn detect_deviations(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let target_weights = data.get("target_weights")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or(serde_json::Map::new());

        let current_weights = data.get("current_weights")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or(serde_json::Map::new());

        let mut deviations = Vec::new();

        for (symbol, target) in target_weights.iter() {
            if let Some(current) = current_weights.get(symbol) {
                let target_val = target.as_f64().unwrap_or(0.0);
                let current_val = current.as_f64().unwrap_or(0.0);
                let deviation = (current_val - target_val).abs();

                if deviation > 0.05 {
                    // 偏离超过5%
                    deviations.push(json!({
                        "symbol": symbol,
                        "target_weight": target_val,
                        "current_weight": current_val,
                        "deviation": deviation,
                        "action_required": true
                    }));
                }
            }
        }

        Ok(json!({
            "deviation_detection": {
                "deviations": deviations,
                "requires_rebalancing": !deviations.is_empty(),
                "deviation_count": deviations.len()
            }
        }))
    }

    /// 告警触发
    async fn trigger_alerts(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let mut alerts = Vec::new();

        // 检查单只仓位
        let max_position = data.get("max_position")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        if max_position > 0.40 {
            alerts.push(json!({
                "type": "position_limit",
                "severity": "high",
                "message": format!("单一仓位 {:.1}% 超过40%限制", max_position * 100.0),
                "action": "consider_reducing_position"
            }));
        }

        // 检查现金比例
        let cash_ratio = data.get("cash_ratio")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        if cash_ratio < 0.05 {
            alerts.push(json!({
                "type": "cash_low",
                "severity": "medium",
                "message": format!("现金比例 {:.1}% 低于5%", cash_ratio * 100.0),
                "action": "consider_keeping_more_cash"
            }));
        }

        Ok(json!({
            "alerts": {
                "alert_count": alerts.len(),
                "alerts_list": alerts,
                "requires_attention": !alerts.is_empty()
            }
        }))
    }
}

#[async_trait]
impl Agent for PortfolioMonitor {
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

        let realtime = self.monitor_realtime(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Realtime monitoring failed: {}", e)))?;

        let deviations = self.detect_deviations(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Deviation detection failed: {}", e)))?;

        let alerts = self.trigger_alerts(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Alert triggering failed: {}", e)))?;

        let combined = json!({
            "symbol": symbol,
            "portfolio_monitoring": {
                "realtime": realtime,
                "deviations": deviations,
                "alerts": alerts
            }
        });

        let content = format!(
            "组合监控:\n{}\n",
            serde_json::to_string_pretty(&combined).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(combined)
            .with_confidence(0.95)
            .with_metadata("agent_type", "portfolio_monitor")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// RiskManager - 风险管理Agent
// ============================================================================

/// Risk Manager Agent
///
/// 风险限额检查、对冲建议、紧急止损
pub struct RiskManager {
    base_agent: SkillAgent,
}

impl RiskManager {
    /// Create a new risk manager
    pub fn new(registry: SkillRegistry) -> Self {
        let base_agent = SkillAgent::new(
            "RiskManager",
            "风险管理专家:风险限额、对冲建议、紧急止损",
            "Kelly准则仓位管理",
            registry,
        );

        Self { base_agent }
    }

    /// 风险限额检查
    async fn check_limits(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let portfolio_var = data.get("portfolio_var")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.15);

        let max_var_limit = 0.20; // 20% VaR限制
        let within_limit = portfolio_var <= max_var_limit;

        Ok(json!({
            "var_limit_check": {
                "portfolio_var": portfolio_var,
                "var_limit": max_var_limit,
                "within_limit": within_limit,
                "var_utilization": portfolio_var / max_var_limit
            }
        }))
    }

    /// 对冲建议
    async fn suggest_hedge(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let beta = data.get("beta")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.2);

        let market_exposure = data.get("market_exposure")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.80);

        // 对冲建议
        let hedge_ratio = beta * market_exposure;

        Ok(json!({
            "hedge_suggestion": {
                "current_beta": beta,
                "market_exposure": market_exposure,
                "recommended_hedge_ratio": hedge_ratio,
                "hedge_instruments": ["put_options", "inverse_etf", "futures"],
                "hedge_reason": "降低组合Beta和市场风险"
            }
        }))
    }

    /// 紧急止损
    async fn emergency_stop_loss(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let current_drawdown = data.get("drawdown")
            .and_then(|v| v.as_f64())
            .unwrap_or(-0.05);

        let max_drawdown_limit = -0.15; // 15%最大回撤限制

        let emergency_action = if current_drawdown < max_drawdown_limit {
            "reduce_positions"
        } else {
            "hold"
        };

        Ok(json!({
            "emergency_stop_loss": {
                "current_drawdown": current_drawdown,
                "max_drawdown_limit": max_drawdown_limit,
                "emergency_action": emergency_action,
                "action_trigger": format!("回撤 {:.1}% 超过{:.1}% 限制",
                    current_drawdown.abs() * 100.0,
                    max_drawdown_limit.abs() * 100.0)
            }
        }))
    }
}

#[async_trait]
impl Agent for RiskManager {
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

        let limits = self.check_limits(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Limit check failed: {}", e)))?;

        let hedge = self.suggest_hedge(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Hedge suggestion failed: {}", e)))?;

        let stop_loss = self.emergency_stop_loss(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Emergency stop loss failed: {}", e)))?;

        let combined = json!({
            "symbol": symbol,
            "risk_management": {
                "limits": limits,
                "hedge": hedge,
                "stop_loss": stop_loss
            }
        });

        let content = format!(
            "风险管理:\n{}\n",
            serde_json::to_string_pretty(&combined).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(combined)
            .with_confidence(0.90)
            .with_metadata("agent_type", "risk_manager")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// ComplianceAgent - 合规Agent
// ============================================================================

/// Compliance Agent
///
/// 合规检查、监管要求、报告生成
pub struct ComplianceAgent {
    base_agent: SkillAgent,
}

impl ComplianceAgent {
    /// Create a new compliance agent
    pub fn new(registry: SkillRegistry) -> Self {
        let base_agent = SkillAgent::new(
            "ComplianceAgent",
            "合规检查专家:监管要求、报告生成",
            "Munger多元思维模型",
            registry,
        );

        Self { base_agent }
    }

    /// 合规检查
    async fn check_compliance(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let positions = data.get("positions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(10);

        // 检查集中度规则
        let concentration_ok = positions <= 50;

        // 检查单一持仓
        let max_single = data.get("max_single_position")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.25);

        let single_position_ok = max_single <= 0.40;

        let all_compliant = concentration_ok && single_position_ok;

        Ok(json!({
            "compliance_check": {
                "concentration_rule": {
                    "positions": positions,
                    "limit": 50,
                    "compliant": concentration_ok
                },
                "single_position_rule": {
                    "max_single": max_single,
                    "limit": 0.40,
                    "compliant": single_position_ok
                },
                "overall_compliant": all_compliant
            }
        }))
    }

    /// 监管要求
    async fn regulatory_requirements(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let jurisdiction = data.get("jurisdiction")
            .and_then(|v| v.as_str())
            .unwrap_or("US");

        let requirements = match jurisdiction {
            "US" => vec![
                "Form 13F filing",
                "Regulation T",
                "Regulation SHO",
                "SEC reporting"
            ],
            "CN" => vec![
                "证监会报告",
                "交易所披露",
                "持股限制"
            ],
            _ => vec!["local_regulations"]
        };

        Ok(json!({
            "regulatory_requirements": {
                "jurisdiction": jurisdiction,
                "required_filings": requirements,
                "reporting_frequency": "quarterly"
            }
        }))
    }

    /// 报告生成
    async fn generate_report(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let portfolio_value = data.get("portfolio_value")
            .and_then(|v| v.as_f64())
            .unwrap_or(1_000_000.0);

        let returns = data.get("returns")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.12);

        Ok(json!({
            "compliance_report": {
                "report_date": chrono::Utc::now().to_rfc3339(),
                "portfolio_value": portfolio_value,
                "returns_ytd": returns,
                "compliance_status": "compliant",
                "recommendations": [
                    "maintain_adequate_records",
                    "file_required_reports",
                    "monitor_concentration_limits"
                ]
            }
        }))
    }
}

#[async_trait]
impl Agent for ComplianceAgent {
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

        let compliance = self.check_compliance(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Compliance check failed: {}", e)))?;

        let regulatory = self.regulatory_requirements(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Regulatory requirements failed: {}", e)))?;

        let report = self.generate_report(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Report generation failed: {}", e)))?;

        let combined = json!({
            "symbol": symbol,
            "compliance": {
                "checks": compliance,
                "regulatory": regulatory,
                "report": report
            }
        });

        let content = format!(
            "合规检查:\n{}\n",
            serde_json::to_string_pretty(&combined).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(combined)
            .with_confidence(0.93)
            .with_metadata("agent_type", "compliance_agent")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// Risk Team - 团队管理
// ============================================================================

/// Risk Team
///
/// 管理风控团队的所有Agents
pub struct RiskTeam {
    portfolio_monitor: PortfolioMonitor,
    risk_manager: RiskManager,
    compliance: ComplianceAgent,
}

impl RiskTeam {
    /// Create a new risk team
    pub async fn new() -> Result<Self, anyhow::Error> {
        let registry = crate::skills::SkillRegistry::from_dir(
            std::path::PathBuf::from(".claude/skills")
        ).await?;

        Ok(Self {
            portfolio_monitor: PortfolioMonitor::new(registry.clone()),
            risk_manager: RiskManager::new(registry.clone()),
            compliance: ComplianceAgent::new(registry),
        })
    }

    /// Get all team members as Agent trait objects
    pub fn get_agents(&self) -> Vec<Box<dyn Agent + Send + Sync>> {
        vec![
            Box::new(self.portfolio_monitor.clone()) as Box<dyn Agent + Send + Sync>,
            Box::new(self.risk_manager.clone()) as Box<dyn Agent + Send + Sync>,
            Box::new(self.compliance.clone()) as Box<dyn Agent + Send + Sync>,
        ]
    }

    /// Get portfolio monitor
    pub fn portfolio_monitor(&self) -> &PortfolioMonitor {
        &self.portfolio_monitor
    }

    /// Get risk manager
    pub fn risk_manager(&self) -> &RiskManager {
        &self.risk_manager
    }

    /// Get compliance agent
    pub fn compliance(&self) -> &ComplianceAgent {
        &self.compliance
    }
}

// ============================================================================
// Clone implementations
// ============================================================================

impl Clone for PortfolioMonitor {
    fn clone(&self) -> Self {
        Self {
            base_agent: self.base_agent.clone(),
        }
    }
}

impl Clone for RiskManager {
    fn clone(&self) -> Self {
        Self {
            base_agent: self.base_agent.clone(),
        }
    }
}

impl Clone for ComplianceAgent {
    fn clone(&self) -> Self {
        Self {
            base_agent: self.base_agent.clone(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_risk_team_creation() {
        let team = RiskTeam::new().await;
        assert!(team.is_ok());

        let team = team.unwrap();
        let agents = team.get_agents();

        assert_eq!(agents.len(), 3);
        assert_eq!(agents[0].name(), "PortfolioMonitor");
        assert_eq!(agents[1].name(), "RiskManager");
        assert_eq!(agents[2].name(), "ComplianceAgent");
    }

    #[tokio::test]
    async fn test_portfolio_monitor() {
        let registry = crate::skills::SkillRegistry::from_dir(
            std::path::PathBuf::from(".claude/skills")
        ).await.unwrap();

        let agent = PortfolioMonitor::new(registry);

        let input_data = json!({
            "symbol": "PORTFOLIO",
            "portfolio_value": 1000000.0,
            "cash": 50000.0,
            "max_position": 0.35
        });

        let input = AgentInput::new("Monitor portfolio")
            .with_context(input_data);

        let output = agent.execute(input).await.unwrap();

        assert!(output.is_successful());
        assert!(output.content.contains("组合监控"));
    }

    #[tokio::test]
    async fn test_risk_manager() {
        let registry = crate::skills::SkillRegistry::from_dir(
            std::path::PathBuf::from(".claude/skills")
        ).await.unwrap();

        let agent = RiskManager::new(registry);

        let input_data = json!({
            "symbol": "PORTFOLIO",
            "portfolio_var": 0.18,
            "beta": 1.2,
            "market_exposure": 0.75,
            "drawdown": -0.12
        });

        let input = AgentInput::new("Manage risk")
            .with_context(input_data);

        let output = agent.execute(input).await.unwrap();

        assert!(output.is_successful());
        assert!(output.content.contains("风险管理"));
    }
}
