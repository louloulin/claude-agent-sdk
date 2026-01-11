//! Trading Team - 交易执行团队
//!
//! 这个模块实现交易团队的3个专业Agents:
//! - ExecutionAgent: 订单生成和执行策略
//! - PositionSizer: Kelly准则仓位优化
//! - OrderRouter: 订单路由和券商选择

use crate::skills::{SkillAgent, SkillRegistry};
use async_trait::async_trait;
use claude_agent_sdk_rs::orchestration::{Agent, AgentInput, AgentOutput, OrchestratorError};
use serde_json::json;

// ============================================================================
// ExecutionAgent - 执行Agent
// ============================================================================

/// Execution Agent
///
/// 负责订单生成、执行策略和滑点控制
pub struct ExecutionAgent {
    base_agent: SkillAgent,
}

impl ExecutionAgent {
    /// Create a new execution agent
    pub fn new(registry: SkillRegistry) -> Self {
        let base_agent = SkillAgent::new(
            "ExecutionAgent",
            "交易执行专家:订单生成、执行策略、滑点控制",
            "Kelly准则仓位管理",
            registry,
        );

        Self { base_agent }
    }

    /// 生成订单
    async fn generate_order(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let symbol = data.get("symbol").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
        let action = data.get("action").and_then(|v| v.as_str()).unwrap_or("buy");
        let quantity = data.get("quantity").and_then(|v| v.as_f64()).unwrap_or(100.0);
        let order_type = data.get("order_type").and_then(|v| v.as_str()).unwrap_or("market");

        let limit_price = if order_type == "limit" {
            data.get("limit_price").and_then(|v| v.as_f64())
        } else {
            None
        };

        Ok(json!({
            "order": {
                "symbol": symbol,
                "action": action,
                "quantity": quantity,
                "order_type": order_type,
                "limit_price": limit_price,
                "time_in_force": "DAY",
                "status": "generated"
            }
        }))
    }

    /// 执行策略
    async fn determine_strategy(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let market_cap = data.get("market_cap")
            .and_then(|v| v.as_f64())
            .unwrap_or(1_000_000_000.0);
        let volume = data.get("avg_volume")
            .and_then(|v| v.as_f64())
            .unwrap_or(10_000_000.0);

        // 根据市值和成交量确定执行策略
        let strategy = if market_cap > 100_000_000_000.0 && volume > 50_000_000.0 {
            "twap" // 时间加权平均价格
        } else if market_cap > 10_000_000_000.0 {
            "vwap" // 成交量加权平均价格
        } else {
            "market" // 市价单
        };

        Ok(json!({
            "execution_strategy": strategy,
            "reason": format!(
                "Market cap: {:.1}B, Volume: {:.1}M",
                market_cap / 1_000_000_000.0,
                volume / 1_000_000.0
            )
        }))
    }

    /// 滑点控制
    async fn estimate_slippage(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let quantity = data.get("quantity").and_then(|v| v.as_f64()).unwrap_or(100.0);
        let avg_volume = data.get("avg_volume")
            .and_then(|v| v.as_f64())
            .unwrap_or(10_000_000.0);

        // 估算滑点:数量越大,相对成交量越大,滑点越大
        let volume_ratio = quantity / avg_volume;
        let estimated_slippage = volume_ratio * 0.01; // 简化模型

        Ok(json!({
            "slippage_estimation": {
                "volume_ratio": volume_ratio,
                "estimated_slippage_bps": estimated_slippage * 10000.0,
                "estimated_slippage_percent": estimated_slippage * 100.0
            },
            "recommendation": if estimated_slippage > 0.005 {
                "split_order"
            } else {
                "single_order"
            }
        }))
    }
}

#[async_trait]
impl Agent for ExecutionAgent {
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

        let order = self.generate_order(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Order generation failed: {}", e)))?;

        let strategy = self.determine_strategy(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Strategy determination failed: {}", e)))?;

        let slippage = self.estimate_slippage(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Slippage estimation failed: {}", e)))?;

        let combined = json!({
            "symbol": symbol,
            "execution_plan": {
                "order": order,
                "strategy": strategy,
                "slippage": slippage
            }
        });

        let content = format!(
            "执行计划生成:\n{}\n",
            serde_json::to_string_pretty(&combined).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(combined)
            .with_confidence(0.90)
            .with_metadata("agent_type", "execution_agent")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// PositionSizer - 仓位管理Agent
// ============================================================================

/// Position Sizer Agent
///
/// 使用Kelly准则进行仓位优化和风险调整
pub struct PositionSizer {
    kelly_agent: SkillAgent,
}

impl PositionSizer {
    /// Create a new position sizer
    pub fn new(registry: SkillRegistry) -> Self {
        let kelly_agent = SkillAgent::new(
            "PositionSizer-Kelly",
            "Kelly仓位管理",
            "Kelly准则仓位管理",
            registry,
        );

        Self { kelly_agent }
    }

    /// Kelly准则计算
    async fn calculate_kelly(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let expected_return = data.get("expected_return")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.10);
        let variance = data.get("variance")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.04);

        // Kelly公式: f* = μ / σ²
        let kelly_fraction = if variance > 0.0 {
            expected_return / variance
        } else {
            0.0
        };

        // 应用实际限制
        let full_kelly = kelly_fraction.min(1.0).max(0.0);
        let half_kelly = full_kelly * 0.5;
        let quarter_kelly = full_kelly * 0.25;

        Ok(json!({
            "kelly_analysis": {
                "expected_return": expected_return,
                "variance": variance,
                "full_kelly": full_kelly,
                "half_kelly": half_kelly,
                "quarter_kelly": quarter_kelly
            },
            "recommendation": "Use quarter Kelly for safety"
        }))
    }

    /// 仓位优化
    async fn optimize_position(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let portfolio_value = data.get("portfolio_value")
            .and_then(|v| v.as_f64())
            .unwrap_or(1_000_000.0);
        let kelly_fraction = data.get("kelly_fraction")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.20);

        // 考虑现有仓位和相关性的优化
        let existing_positions = data.get("existing_positions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(5);

        let concentration_adjustment = if existing_positions > 10 {
            0.5
        } else if existing_positions > 5 {
            0.7
        } else {
            1.0
        };

        let optimal_fraction = kelly_fraction * concentration_adjustment;
        let position_value = portfolio_value * optimal_fraction;

        Ok(json!({
            "position_optimization": {
                "portfolio_value": portfolio_value,
                "base_kelly": kelly_fraction,
                "concentration_adjustment": concentration_adjustment,
                "optimal_fraction": optimal_fraction,
                "position_value": position_value
            }
        }))
    }

    /// 风险调整
    async fn adjust_risk(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let base_position = data.get("base_position")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.20);

        let volatility = data.get("volatility")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.25);

        // 根据波动率调整
        let volatility_adjustment = if volatility > 0.40 {
            0.5
        } else if volatility > 0.30 {
            0.7
        } else {
            1.0
        };

        let adjusted_position = base_position * volatility_adjustment;

        Ok(json!({
            "risk_adjustment": {
                "base_position": base_position,
                "volatility": volatility,
                "volatility_adjustment": volatility_adjustment,
                "adjusted_position": adjusted_position
            }
        }))
    }
}

#[async_trait]
impl Agent for PositionSizer {
    fn name(&self) -> &str {
        "PositionSizer"
    }

    fn description(&self) -> &str {
        "Kelly仓位管理专家:仓位优化、风险调整"
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        let symbol = input.metadata.get("symbol")
            .unwrap_or(&input.content)
            .clone();

        let data = &input.context;

        let kelly = self.calculate_kelly(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Kelly calculation failed: {}", e)))?;

        let optimization = self.optimize_position(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Position optimization failed: {}", e)))?;

        let risk_adj = self.adjust_risk(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Risk adjustment failed: {}", e)))?;

        let combined = json!({
            "symbol": symbol,
            "position_sizing": {
                "kelly": kelly,
                "optimization": optimization,
                "risk_adjustment": risk_adj
            },
            "final_recommendation": "综合Kelly、集中度和波动率后的仓位建议"
        });

        let content = format!(
            "仓位管理:\n{}\n",
            serde_json::to_string_pretty(&combined).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(combined)
            .with_confidence(0.88)
            .with_metadata("agent_type", "position_sizer")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// OrderRouter - 订单路由Agent
// ============================================================================

/// Order Router Agent
///
/// 负责订单路由和券商选择
pub struct OrderRouter {
    base_agent: SkillAgent,
}

impl OrderRouter {
    /// Create a new order router
    pub fn new(registry: SkillRegistry) -> Self {
        let base_agent = SkillAgent::new(
            "OrderRouter",
            "订单路由专家:券商选择、执行确认",
            "Kelly准则仓位管理",
            registry,
        );

        Self { base_agent }
    }

    /// 选择券商
    async fn select_broker(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let market = data.get("market").and_then(|v| v.as_str()).unwrap_or("US");
        let order_size = data.get("order_value")
            .and_then(|v| v.as_f64())
            .unwrap_or(10_000.0);

        // 根据市场和订单大小选择券商
        let broker = match market {
            "CN" => {
                if order_size > 1_000_000.0 {
                    "qmt-broker-mcp" // QMT适合大额
                } else {
                    "xtp-broker-mcp"  // XTP适合中小额
                }
            }
            "US" => {
                if order_size > 100_000.0 {
                    "interactive-brokers-mcp" // IB适合大额
                } else {
                    "td-ameritrade-mcp"  // TD适合中小额
                }
            }
            "CRYPTO" => {
                "binance-trading-mcp"
            }
            _ => "default-broker"
        };

        Ok(json!({
            "broker_selection": {
                "market": market,
                "order_size": order_size,
                "selected_broker": broker,
                "reason": "基于市场和订单大小的最优选择"
            }
        }))
    }

    /// 路由订单
    async fn route_order(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let broker = data.get("broker").and_then(|v| v.as_str()).unwrap_or("unknown");
        let symbol = data.get("symbol").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");

        Ok(json!({
            "order_routing": {
                "broker": broker,
                "symbol": symbol,
                "route": format!("{} -> {} -> {}", symbol, "gateway", broker, "exchange"),
                "estimated_latency_ms": 50,
                "routing_confidence": 0.95
            }
        }))
    }

    /// 执行确认
    async fn confirm_execution(&self, data: &serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        Ok(json!({
            "execution_confirmation": {
                "order_id": format!("ORD-{}", uuid::Uuid::new_v4()),
                "status": "submitted",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "estimated_fill_time_ms": 200
            }
        }))
    }
}

#[async_trait]
impl Agent for OrderRouter {
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

        let broker = self.select_broker(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Broker selection failed: {}", e)))?;

        let routing = self.route_order(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Order routing failed: {}", e)))?;

        let confirmation = self.confirm_execution(data).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(format!("Execution confirmation failed: {}", e)))?;

        let combined = json!({
            "symbol": symbol,
            "order_routing": {
                "broker_selection": broker,
                "routing": routing,
                "confirmation": confirmation
            }
        });

        let content = format!(
            "订单路由:\n{}\n",
            serde_json::to_string_pretty(&combined).unwrap()
        );

        Ok(AgentOutput::new(content)
            .with_data(combined)
            .with_confidence(0.92)
            .with_metadata("agent_type", "order_router")
            .with_metadata("symbol", &symbol))
    }
}

// ============================================================================
// Trading Team - 团队管理
// ============================================================================

/// Trading Team
///
/// 管理交易团队的所有Agents
pub struct TradingTeam {
    execution: ExecutionAgent,
    position_sizer: PositionSizer,
    order_router: OrderRouter,
}

impl TradingTeam {
    /// Create a new trading team
    pub async fn new() -> Result<Self, anyhow::Error> {
        let registry = crate::skills::SkillRegistry::from_dir(
            std::path::PathBuf::from(".claude/skills")
        ).await?;

        Ok(Self {
            execution: ExecutionAgent::new(registry.clone()),
            position_sizer: PositionSizer::new(registry.clone()),
            order_router: OrderRouter::new(registry),
        })
    }

    /// Get all team members as Agent trait objects
    pub fn get_agents(&self) -> Vec<Box<dyn Agent + Send + Sync>> {
        vec![
            Box::new(self.execution.clone()) as Box<dyn Agent + Send + Sync>,
            Box::new(self.position_sizer.clone()) as Box<dyn Agent + Send + Sync>,
            Box::new(self.order_router.clone()) as Box<dyn Agent + Send + Sync>,
        ]
    }

    /// Get execution agent
    pub fn execution(&self) -> &ExecutionAgent {
        &self.execution
    }

    /// Get position sizer
    pub fn position_sizer(&self) -> &PositionSizer {
        &self.position_sizer
    }

    /// Get order router
    pub fn order_router(&self) -> &OrderRouter {
        &self.order_router
    }
}

// ============================================================================
// Clone implementations
// ============================================================================

impl Clone for ExecutionAgent {
    fn clone(&self) -> Self {
        Self {
            base_agent: self.base_agent.clone(),
        }
    }
}

impl Clone for PositionSizer {
    fn clone(&self) -> Self {
        Self {
            kelly_agent: self.kelly_agent.clone(),
        }
    }
}

impl Clone for OrderRouter {
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
    async fn test_trading_team_creation() {
        let team = TradingTeam::new().await;
        assert!(team.is_ok());

        let team = team.unwrap();
        let agents = team.get_agents();

        assert_eq!(agents.len(), 3);
        assert_eq!(agents[0].name(), "ExecutionAgent");
        assert_eq!(agents[1].name(), "PositionSizer");
        assert_eq!(agents[2].name(), "OrderRouter");
    }

    #[tokio::test]
    async fn test_position_sizer() {
        let registry = crate::skills::SkillRegistry::from_dir(
            std::path::PathBuf::from(".claude/skills")
        ).await.unwrap();

        let agent = PositionSizer::new(registry);

        let input_data = json!({
            "symbol": "AAPL",
            "expected_return": 0.15,
            "variance": 0.0625,
            "portfolio_value": 100000.0
        });

        let input = AgentInput::new("Calculate position")
            .with_context(input_data);

        let output = agent.execute(input).await.unwrap();

        assert!(output.is_successful());
        assert!(output.content.contains("仓位管理"));
    }
}
