//! Trading Advisor Agent - 交易建议Agent
//!
//! 提供交易时机和执行建议
//!
//! ## 核心功能
//!
//! 1. **入场时机分析** - 判断是否适合买入/卖出
//! 2. **仓位建议** - 根据Kelly准则和风险偏好建议仓位
//! 3. **风险提示** - 提醒潜在风险

use crate::agents::{Agent, AgentInput, AgentOutput, InvestmentAction, Result};
use async_trait::async_trait;

/// Trading Advisor Agent
///
/// 交易建议Agent，提供交易时机和仓位建议
pub struct TradingAdvisorAgent {
    /// 最大单仓位
    max_position_size: f64,

    /// 风险系数
    risk_coefficient: f64,
}

impl TradingAdvisorAgent {
    /// 创建新的交易建议Agent
    pub fn new() -> Self {
        Self {
            max_position_size: 0.25, // 单个仓位最大25%
            risk_coefficient: 0.5,   // 风险系数
        }
    }

    /// 设置最大仓位
    pub fn with_max_position(mut self, max_size: f64) -> Self {
        self.max_position_size = max_size;
        self
    }

    /// 设置风险系数
    pub fn with_risk_coefficient(mut self, coefficient: f64) -> Self {
        self.risk_coefficient = coefficient;
        self
    }

    /// 分析交易建议
    async fn analyze_trade(
        &self,
        symbol: &str,
        action: &InvestmentAction,
        current_price: f64,
        confidence: f64,
    ) -> Result<TradingAdvice> {
        // 计算建议仓位 (基于置信度和风险系数)
        let base_position = self.calculate_position_size(confidence);

        // 限制最大仓位
        let suggested_position = base_position.min(self.max_position_size);

        // 计算风险等级
        let risk_level = self.assess_risk_level(symbol, confidence).await?;

        // 生成建议说明
        let explanation = match action {
            InvestmentAction::StrongBuy => {
                format!(
                    "强烈买入信号，置信度高。建议仓位 {:.1}%",
                    suggested_position * 100.0
                )
            }
            InvestmentAction::Buy => {
                format!(
                    "买入信号。建议仓位 {:.1}%，分批建仓降低风险",
                    suggested_position * 100.0
                )
            }
            InvestmentAction::Hold => "继续持有，观察市场变化".to_string(),
            InvestmentAction::Sell => "考虑减仓或止盈".to_string(),
            InvestmentAction::StrongSell => "建议卖出，控制风险".to_string(),
        };

        // 止损止盈建议
        let (stop_loss, take_profit) = self.calculate_stop_levels(
            current_price,
            action,
        );

        Ok(TradingAdvice {
            symbol: symbol.to_string(),
            action: action.clone(),
            suggested_position,
            explanation,
            risk_level,
            stop_loss,
            take_profit,
            timing_advice: self.get_timing_advice(action).await?,
        })
    }

    /// 计算仓位大小
    ///
    /// 基于置信度和风险系数
    fn calculate_position_size(&self, confidence: f64) -> f64 {
        // 简化的仓位计算
        // 仓位 = 置信度 × 风险系数
        confidence * self.risk_coefficient
    }

    /// 评估风险等级
    async fn assess_risk_level(&self, _symbol: &str, confidence: f64) -> Result<u8> {
        // TODO: 实际实现应该考虑更多因素
        // 这里简化为基于置信度
        let risk = if confidence > 0.8 {
            1 // 低风险
        } else if confidence > 0.5 {
            2 // 中等风险
        } else {
            3 // 高风险
        };

        Ok(risk)
    }

    /// 计算止损止盈价位
    fn calculate_stop_levels(
        &self,
        current_price: f64,
        action: &InvestmentAction,
    ) -> (Option<f64>, Option<f64>) {
        match action {
            InvestmentAction::StrongBuy | InvestmentAction::Buy => {
                // 买入: 止损-8%，止盈+20%
                let stop_loss = Some(current_price * 0.92);
                let take_profit = Some(current_price * 1.20);
                (stop_loss, take_profit)
            }
            InvestmentAction::Sell | InvestmentAction::StrongSell => {
                // 卖出: 不适用
                (None, None)
            }
            InvestmentAction::Hold => (None, None),
        }
    }

    /// 获取时机建议
    async fn get_timing_advice(&self, action: &InvestmentAction) -> Result<String> {
        let advice = match action {
            InvestmentAction::StrongBuy => "建议尽快买入，可分2-3批建仓".to_string(),
            InvestmentAction::Buy => "可以考虑逢低买入，不要追高".to_string(),
            InvestmentAction::Hold => "持有观察，等待更明确的信号".to_string(),
            InvestmentAction::Sell => "建议逐步减仓，不要一次性清仓".to_string(),
            InvestmentAction::StrongSell => "建议尽快卖出，控制风险".to_string(),
        };

        Ok(advice)
    }
}

impl Default for TradingAdvisorAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for TradingAdvisorAgent {
    fn name(&self) -> &str {
        "TradingAdvisorAgent"
    }

    fn description(&self) -> &str {
        "交易建议Agent，提供交易时机和仓位建议"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 从context中解析参数
        let symbol = input.content.trim().to_string();
        let action: InvestmentAction = serde_json::from_value(
            input.context.get("action").unwrap_or(&serde_json::json!("Hold")).clone()
        )
            .unwrap_or(InvestmentAction::Hold);
        let current_price = input.context.get("current_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0);
        let confidence = input.context.get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);

        // 分析交易建议
        let advice = self.analyze_trade(&symbol, &action, current_price, confidence).await?;

        // 格式化输出
        let content = format!(
            "💼 交易建议: {}\n\n\
             📊 建议仓位: {:.1}%\n\
             📝 说明: {}\n\
             ⚠️ 风险等级: {}/3\n\
             🛑 止损价位: ${:.2}\n\
             🎯 止盈价位: ${:.2}\n\
             ⏰ 时机建议: {}",
            action.display_name(),
            advice.suggested_position * 100.0,
            advice.explanation,
            advice.risk_level,
            advice.stop_loss.unwrap_or(0.0),
            advice.take_profit.unwrap_or(0.0),
            advice.timing_advice
        );

        Ok(AgentOutput::new(content).with_data(serde_json::to_value(advice)?))
    }
}

/// 交易建议
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TradingAdvice {
    /// 股票代码
    pub symbol: String,

    /// 操作建议
    pub action: InvestmentAction,

    /// 建议仓位 (0-1)
    pub suggested_position: f64,

    /// 建议说明
    pub explanation: String,

    /// 风险等级 (1-3)
    pub risk_level: u8,

    /// 止损价位
    pub stop_loss: Option<f64>,

    /// 止盈价位
    pub take_profit: Option<f64>,

    /// 时机建议
    pub timing_advice: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trading_advisor_agent() {
        let agent = TradingAdvisorAgent::new();

        let input = AgentInput::new("AAPL")
            .with_context(serde_json::json!({
                "action": "Buy",
                "current_price": 150.0,
                "confidence": 0.8
            }));

        let result = agent.execute(input).await.unwrap();

        assert!(!result.content.is_empty());
        assert!(result.content.contains("交易建议"));
    }

    #[test]
    fn test_calculate_position_size() {
        let agent = TradingAdvisorAgent::new();

        // 置信度80%，风险系数0.5 -> 仓位40%
        let position = agent.calculate_position_size(0.8);
        assert_eq!(position, 0.4);
    }

    #[test]
    fn test_calculate_stop_levels() {
        let agent = TradingAdvisorAgent::new();

        // 买入: 当前价$100
        let (stop_loss, take_profit) = agent.calculate_stop_levels(
            100.0,
            &InvestmentAction::Buy
        );

        assert_eq!(stop_loss, Some(92.0));  // -8%
        assert_eq!(take_profit, Some(120.0)); // +20%
    }
}
