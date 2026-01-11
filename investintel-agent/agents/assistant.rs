//! Investment Assistant - 投资智能助手主协调Agent
//!
//! 整合所有投资分析Agents，提供统一的投资建议接口
//!
//! ## 核心功能
//!
//! 1. **统一协调** - 协调ValueInvestment、PortfolioManager、TradingAdvisor
//! 2. **对话式交互** - 自然语言交互，理解用户投资需求
//! 3. **综合建议** - 整合多个分析维度，给出综合投资建议
//! 4. **Skills集成** - 支持Claude Skills系统，可动态调用投资分析Skills

use crate::agents::{
    DividendInvestorAgent, KellyPositionAgent, MungerFrameworkAgent, PortfolioManagerAgent, Result, TradingAdvisorAgent, ValueInvestmentAgent,
};
use claude_agent_sdk_rs::orchestration::{
    Agent, AgentInput, AgentOutput, ParallelOrchestrator, SequentialOrchestrator,
};
use claude_agent_sdk_rs::skills::{SkillRegistry, SkillPackage};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;

/// Investment Assistant - 投资智能助手
///
/// 主协调Agent，整合所有投资分析功能 (Graham-Buffett-Munger三位一体)
/// 支持Claude Skills系统，可动态加载和调用投资分析Skills
pub struct InvestmentAssistant {
    /// 价值投资分析Agent (Graham-Buffett)
    value_agent: ValueInvestmentAgent,

    /// 组合管理Agent
    portfolio_agent: PortfolioManagerAgent,

    /// 交易建议Agent
    trading_agent: TradingAdvisorAgent,

    /// 股息投资Agent
    dividend_agent: DividendInvestorAgent,

    /// Kelly仓位管理Agent
    kelly_agent: KellyPositionAgent,

    /// Munger框架Agent (多元思维模型)
    munger_agent: MungerFrameworkAgent,

    /// Skills注册表（用于动态加载Skills）
    skills_registry: Arc<SkillRegistry>,

    /// 已加载的Skills包
    loaded_skills: Vec<SkillPackage>,
}

impl InvestmentAssistant {
    /// 创建新的投资助手
    pub fn new() -> Self {
        Self {
            value_agent: ValueInvestmentAgent::new(),
            portfolio_agent: PortfolioManagerAgent::new(),
            trading_agent: TradingAdvisorAgent::new(),
            dividend_agent: DividendInvestorAgent::new(),
            kelly_agent: KellyPositionAgent::new(),
            munger_agent: MungerFrameworkAgent::new(),
            skills_registry: Arc::new(SkillRegistry::new()),
            loaded_skills: Vec::new(),
        }
    }

    /// 创建新的投资助手并加载Skills
    pub async fn with_skills() -> Result<Self, anyhow::Error> {
        let mut assistant = Self::new();
        assistant.load_skills().await?;
        Ok(assistant)
    }

    /// 从指定目录加载Skills
    pub async fn load_skills_from_dir<P: Into<PathBuf>>(&mut self, dir: P) -> Result<(), anyhow::Error> {
        let packages = SkillRegistry::discover_skill_md_from_dir(dir.into())?;
        self.loaded_skills.extend(packages);
        Ok(())
    }

    /// 加载项目Skills（从.claude/skills/）
    pub async fn load_skills(&mut self) -> Result<(), anyhow::Error> {
        let project_dir = std::env::current_dir()?;
        let skills_dir = project_dir.join(".claude").join("skills");

        if skills_dir.exists() {
            let packages = SkillRegistry::discover_skill_md_from_dir(&skills_dir)?;
            tracing::info!("加载了 {} 个Skills", packages.len());

            for package in &packages {
                tracing::info!("  - {} ({})", package.metadata.name, package.metadata.description);
            }

            self.loaded_skills = packages;
        }

        Ok(())
    }

    /// 获取已加载的Skills列表
    pub fn list_skills(&self) -> Vec<String> {
        self.loaded_skills.iter()
            .map(|p| p.metadata.name.clone())
            .collect()
    }

    /// 根据名称查找Skill
    pub fn find_skill(&self, name: &str) -> Option<&SkillPackage> {
        self.loaded_skills.iter()
            .find(|p| p.metadata.id == name || p.metadata.name == name)
    }

    /// 分析股票投资价值
    pub async fn analyze_stock(&self, symbol: &str) -> Result<StockAnalysis> {
        // 1. 价值分析
        let value_output = self
            .value_agent
            .execute(AgentInput::new(symbol))
            .await?;

        // 2. 交易建议
        let trading_output = self
            .trading_agent
            .execute(
                AgentInput::new(symbol).with_context(
                    value_output.data.clone()
                )
            )
            .await?;

        Ok(StockAnalysis {
            symbol: symbol.to_string(),
            value_analysis: value_output.content,
            trading_advice: trading_output.content,
            data: value_output.data,
        })
    }

    /// 管理投资组合
    pub async fn manage_portfolio(&self, portfolio: serde_json::Value) -> Result<String> {
        let output = self
            .portfolio_agent
            .execute(
                AgentInput::new("管理组合").with_context(portfolio)
            )
            .await?;

        Ok(output.content)
    }

    /// 分析股息股票
    pub async fn analyze_dividend_stock(&self, symbol: &str) -> Result<String> {
        let output = self
            .dividend_agent
            .execute(AgentInput::new(symbol))
            .await?;

        Ok(output.content)
    }

    /// 交互式投资咨询
    pub async fn chat(&self, user_message: &str) -> Result<String> {
        // 解析用户意图
        let intent = self.parse_intent(user_message);

        match intent.as_str() {
            "munger_analysis" => {
                // Munger框架分析
                let output = self
                    .munger_agent
                    .execute(AgentInput::new(user_message))
                    .await?;
                Ok(output.content)
            }
            "kelly_position" => {
                // Kelly仓位分析
                let output = self
                    .kelly_agent
                    .execute(AgentInput::new(user_message))
                    .await?;
                Ok(output.content)
            }
            "dividend_analysis" => {
                // 提取股票代码
                let symbol = self.extract_symbol(user_message);
                let analysis = self.analyze_dividend_stock(&symbol).await?;

                Ok(analysis)
            }
            "analyze_stock" => {
                // 提取股票代码
                let symbol = self.extract_symbol(user_message);
                let analysis = self.analyze_stock(&symbol).await?;

                Ok(format!(
                    "📊 {} 价值分析:\n\n{}\n\n💡 交易建议:\n\n{}",
                    analysis.symbol, analysis.value_analysis, analysis.trading_advice
                ))
            }
            "portfolio_help" => {
                Ok("请提供您的投资组合信息(JSON格式)，我将为您分析配置情况。".to_string())
            }
            "general_advice" => {
                Ok(self.general_investment_advice().await?)
            }
            _ => Ok("抱歉，我还没有理解您的需求。您可以:\n\
                    1. 询问股票价值，如\"分析AAPL\"\n\
                    2. 询问股息投资，如\"股息分析AAPL\"或\"AAPL的股息怎么样\"\n\
                    3. Munger框架分析，如\"Munger分析AAPL\"或\"思维模型分析\"\n\
                    4. Kelly仓位分析，如\"Kelly仓位建议\"或\"仓位分析\"\n\
                    5. 管理投资组合\n\
                    6. 咨询一般投资建议".to_string()),
        }
    }

    /// 解析用户意图
    fn parse_intent(&self, message: &str) -> String {
        let message_lower = message.to_lowercase();

        // Munger框架分析相关
        if message_lower.contains("munger") || message_lower.contains("思维模型") || message_lower.contains("lollapalooza") {
            "munger_analysis".to_string()
        }
        // Kelly仓位管理相关
        else if message_lower.contains("仓位") || message_lower.contains("kelly") || message_lower.contains("position") {
            "kelly_position".to_string()
        }
        // 股息投资相关
        else if message_lower.contains("股息") || message_lower.contains("dividend") {
            "dividend_analysis".to_string()
        }
        // 价值分析
        else if message_lower.contains("分析") || message_lower.contains("analyze") {
            "analyze_stock".to_string()
        } else if message_lower.contains("组合") || message_lower.contains("portfolio") {
            "portfolio_help".to_string()
        } else if message_lower.contains("建议") || message_lower.contains("advice") {
            "general_advice".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// 提取股票代码
    fn extract_symbol(&self, message: &str) -> String {
        // 简化实现：提取大写字母序列
        let words: Vec<&str> = message.split_whitespace().collect();
        for word in words {
            if word.len() >= 2 && word.len() <= 5 && word.chars().all(|c| c.is_ascii_uppercase()) {
                return word.to_string();
            }
        }

        // 如果没找到，返回默认
        "AAPL".to_string()
    }

    /// 一般投资建议
    async fn general_investment_advice(&self) -> Result<String> {
        let advice = r#"
🎯 投资智能助手 - 核心投资原则

基于Graham-Buffett-Munger三位一体价值投资理念:

1️⃣ **安全边际** (Graham)
   - 买入价格必须低于内在价值
   - 至少30%的安全边际
   - 为错误和不确定性留出缓冲

2️⃣ **质量优先** (Buffett)
   - 买入优秀企业
   - ROIC > 10%
   - 宽护城优势
   - 持有5-10年

3️⃣ **多元思维** (Munger)
   - 跨学科思考，应用多个思维模型
   - 能力圈原则：只投资你理解的
   - 逆向思维：先考虑如何失败
   - Lollapalooza效应：寻找多因子共振

4️⃣ **长期持有**
   - 频繁交易增加成本
   - 复利需要时间
   - 耐心是最大的美德

5️⃣ **股息收入**
   - 稳定的被动收入来源
   - 优先股息贵族(连续25年+增长)
   - 关注股息安全性，不只是收益率
   - 再投资加速复利效应

6️⃣ **科学仓位管理** (Kelly准则)
   - 基于Kelly公式计算最优仓位
   - 使用1/4 Kelly或半Kelly降低风险
   - 分批建仓，降低择时风险
   - 定期再平衡，保持仓位比例

💡 使用建议:
- 价值分析: "分析AAPL"
- 股息分析: "股息分析AAPL" 或 "AAPL的股息怎么样"
- Munger分析: "Munger分析AAPL" 或 "思维模型分析"
- Kelly仓位: "Kelly仓位建议" 或 "仓位分析"
- 管理组合: "分析我的组合"
- 更多建议: 继续提问！
        "#;

        Ok(advice.to_string())
    }
}

impl Default for InvestmentAssistant {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for InvestmentAssistant {
    fn name(&self) -> &str {
        "InvestmentAssistant"
    }

    fn description(&self) -> &str {
        "AI投资智能助手，提供价值投资分析和组合管理建议"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 调用chat方法处理用户输入
        let response = self.chat(&input.content).await?;

        Ok(AgentOutput::new(response).with_confidence(0.85))
    }
}

/// 股票分析结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StockAnalysis {
    /// 股票代码
    pub symbol: String,

    /// 价值分析
    pub value_analysis: String,

    /// 交易建议
    pub trading_advice: String,

    /// 原始数据
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_investment_assistant() {
        let assistant = InvestmentAssistant::new();

        // 测试股票分析
        let result = assistant
            .chat("分析AAPL")
            .await
            .unwrap();

        assert!(!result.is_empty());
        assert!(result.contains("AAPL"));
    }

    #[test]
    fn test_parse_intent() {
        let assistant = InvestmentAssistant::new();

        assert_eq!(assistant.parse_intent("分析AAPL"), "analyze_stock");
        assert_eq!(assistant.parse_intent("我的组合怎么样"), "portfolio_help");
        assert_eq!(assistant.parse_intent("有什么建议"), "general_advice");
    }

    #[test]
    fn test_extract_symbol() {
        let assistant = InvestmentAssistant::new();

        assert_eq!(assistant.extract_symbol("分析AAPL"), "AAPL");
        assert_eq!(assistant.extract_symbol("MSFT怎么样"), "MSFT");
        assert_eq!(assistant.extract_symbol("随便说说"), "AAPL"); // 默认值
    }
}
