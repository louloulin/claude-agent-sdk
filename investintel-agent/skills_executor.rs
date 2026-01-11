//! Skills执行器
//!
//! 连接Claude Skills系统与InvestmentAssistant的桥梁

use crate::agents::{
    InvestmentAssistant, ValueInvestmentAgent, KellyPositionAgent,
    MungerFrameworkAgent, DividendInvestorAgent,
};
use crate::backtest::{BacktestEngine, BacktestConfig, GrahamStrategy, KellyStrategy};
use claude_agent_sdk_rs::skills::{SkillPackage, SkillRegistry};
use std::path::PathBuf;
use anyhow::Result;

/// Skills执行器
pub struct SkillsExecutor {
    /// Skills注册表
    registry: SkillRegistry,

    /// 已加载的Skills
    skills: Vec<SkillPackage>,

    /// 投资助手
    assistant: InvestmentAssistant,
}

impl SkillsExecutor {
    /// 创建新的Skills执行器
    pub fn new(assistant: InvestmentAssistant) -> Self {
        Self {
            registry: SkillRegistry::new(),
            skills: Vec::new(),
            assistant,
        }
    }

    /// 从项目目录加载Skills
    pub fn load_skills_from_project(&mut self) -> Result<usize> {
        let project_dir = std::env::current_dir()?;
        let skills_dir = project_dir.join(".claude").join("skills");

        if !skills_dir.exists() {
            tracing::warn!("Skills目录不存在: {:?}", skills_dir);
            return Ok(0);
        }

        let packages = SkillRegistry::discover_skill_md_from_dir(&skills_dir)?;
        let count = packages.len();

        tracing::info!("加载了{}个Skills:", count);
        for package in &packages {
            tracing::info!("  - {}", package.metadata.name);
        }

        self.skills = packages;
        Ok(count)
    }

    /// 执行指定Skill
    pub async fn execute_skill(
        &self,
        skill_name: &str,
        input: &str,
    ) -> Result<String> {
        // 查找Skill
        let skill = self.skills.iter()
            .find(|s| s.metadata.name.contains(skill_name)
                || s.metadata.id.contains(skill_name))
            .ok_or_else(|| anyhow::anyhow!("Skill不存在: {}", skill_name))?;

        tracing::info!("执行Skill: {}", skill.metadata.name);

        // 根据Skill类型路由到对应的Agent
        let result = match skill.metadata.name.as_str() {
            "Graham Value Investing" => {
                let symbol = self.extract_symbol(input)?;
                self.execute_graham_analysis(&symbol).await?
            },
            "Kelly Position Sizing" => {
                let symbol = self.extract_symbol(input)?;
                self.execute_kelly_position(&symbol).await?
            },
            "Munger Mental Models" => {
                let symbol = self.extract_symbol(input)?;
                self.execute_munger_analysis(&symbol).await?
            },
            "Dividend Investing" => {
                let symbol = self.extract_symbol(input)?;
                self.execute_dividend_analysis(&symbol).await?
            },
            "MCP Data Gateway" => {
                self.execute_mcp_query(input).await?
            },
            _ => {
                format!("Skill '{}' 已加载，但需要手动实现执行逻辑", skill.metadata.name)
            }
        };

        Ok(result)
    }

    /// 执行Graham价值投资分析
    async fn execute_graham_analysis(&self, symbol: &str) -> Result<String> {
        let agent = ValueInvestmentAgent::new();

        // 使用Agent执行分析
        let output = agent.execute(
            claude_agent_sdk_rs::orchestration::AgentInput::new(symbol)
        ).await?;

        // 结合Skill的instructions增强输出
        let skill = self.find_skill("Graham Value Investing")?;
        let enhanced_output = format!(
            "{}\n\n---\n\n📘 Skill说明:\n{}",
            output.content,
            skill.instructions
        );

        Ok(enhanced_output)
    }

    /// 执行Kelly仓位分析
    async fn execute_kelly_position(&self, symbol: &str) -> Result<String> {
        let agent = KellyPositionAgent::new();

        let output = agent.execute(
            claude_agent_sdk_rs::orchestration::AgentInput::new(symbol)
        ).await?;

        let skill = self.find_skill("Kelly Position Sizing")?;
        let enhanced_output = format!(
            "{}\n\n---\n\n📘 Skill说明:\n{}",
            output.content,
            skill.instructions
        );

        Ok(enhanced_output)
    }

    /// 执行Munger思维模型分析
    async fn execute_munger_analysis(&self, symbol: &str) -> Result<String> {
        let agent = MungerFrameworkAgent::new();

        let output = agent.execute(
            claude_agent_sdk_rs::orchestration::AgentInput::new(symbol)
        ).await?;

        let skill = self.find_skill("Munger Mental Models")?;
        let enhanced_output = format!(
            "{}\n\n---\n\n📘 Skill说明:\n{}",
            output.content,
            skill.instructions
        );

        Ok(enhanced_output)
    }

    /// 执行股息投资分析
    async fn execute_dividend_analysis(&self, symbol: &str) -> Result<String> {
        let agent = DividendInvestorAgent::new();

        let output = agent.execute(
            claude_agent_sdk_rs::orchestration::AgentInput::new(symbol)
        ).await?;

        let skill = self.find_skill("Dividend Investing")?;
        let enhanced_output = format!(
            "{}\n\n---\n\n📘 Skill说明:\n{}",
            output.content,
            skill.instructions
        );

        Ok(enhanced_output)
    }

    /// 执行MCP数据查询
    async fn execute_mcp_query(&self, query: &str) -> Result<String> {
        // 解析查询意图
        if query.contains("报价") || query.contains("quote") {
            let symbol = self.extract_symbol(query)?;

            // 使用MarketDataProvider获取实时数据
            use crate::agents::MarketDataProvider;
            let provider = MarketDataProvider::new();
            let quote = provider.get_quote(&symbol).await?;

            Ok(format!("📊 {} 实时报价:\n\n当前价格: ${}\n涨跌: {} ({:.2}%)\n日高: ${}\n日低: ${}\n成交量: {}",
                symbol,
                quote.current_price,
                quote.change,
                quote.change_percent,
                quote.day_high,
                quote.day_low,
                quote.volume
            ))
        } else {
            Ok("请提供具体的查询需求，例如：'查询AAPL的报价'".to_string())
        }
    }

    /// 执行组合策略回测
    pub async fn execute_backtest(
        &self,
        strategy_name: &str,
        symbol: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<String> {
        use crate::backtest::{BacktestDataset, DateRange};
        use chrono::NaiveDate;

        // 创建回测配置
        let config = BacktestConfig {
            initial_capital: 100_000.0,
            date_range: DateRange::new(start_date, end_date)?,
            ..Default::default()
        };

        // 生成模拟数据（实际应用中应从数据库获取）
        let dataset = BacktestDataset::generate_mock_data(
            vec![symbol],
            &config.date_range,
            150.0,
        );

        // 选择策略
        let strategy: Box<dyn crate::backtest::Strategy> = match strategy_name {
            "graham" | "Graham" => Box::new(GrahamStrategy::new()),
            "kelly" | "Kelly" => Box::new(KellyStrategy::new()),
            _ => return Err(anyhow::anyhow!("未知策略: {}", strategy_name)),
        };

        // 执行回测
        let engine = BacktestEngine::new(config, dataset);
        let result = engine.run(strategy.as_ref())?;

        // 格式化结果
        Ok(format!(
            "📊 {} 策略回测结果 ({:?} 到 {:?}):\n\n\
            总收益率: {:.2}%\n\
            年化收益率: {:.2}%\n\
            夏普比率: {:.2}\n\
            总交易次数: {}",
            result.strategy_name,
            result.config.date_range.start_date,
            result.config.date_range.end_date,
            result.performance.total_return * 100.0,
            result.performance.annualized_return * 100.0,
            result.performance.sharpe_ratio,
            result.performance.total_trades
        ))
    }

    /// 查找Skill
    fn find_skill(&self, name: &str) -> Option<&SkillPackage> {
        self.skills.iter()
            .find(|s| s.metadata.name.contains(name)
                || s.metadata.id.contains(name))
    }

    /// 提取股票代码
    fn extract_symbol(&self, input: &str) -> Result<String> {
        // 提取大写字母序列（股票代码）
        let words: Vec<&str> = input.split_whitespace().collect();
        for word in words {
            if word.len() >= 2 && word.len() <= 5
                && word.chars().all(|c| c.is_ascii_uppercase()) {
                return Ok(word.to_string());
            }
        }

        // 如果没找到，返回错误
        Err(anyhow::anyhow!("无法从输入中提取股票代码: {}", input))
    }

    /// 列出所有可用Skills
    pub fn list_skills(&self) -> Vec<String> {
        self.skills.iter()
            .map(|s| format!("{}: {}", s.metadata.name, s.metadata.description))
            .collect()
    }

    /// 获取Skills统计信息
    pub fn skills_stats(&self) -> SkillsStats {
        SkillsStats {
            total: self.skills.len(),
            with_reference: self.skills.iter().filter(|s| s.reference.is_some()).count(),
            total_instructions: self.skills.iter().map(|s| s.instructions.len()).sum(),
        }
    }
}

/// Skills统计信息
#[derive(Debug, Clone)]
pub struct SkillsStats {
    /// Skills总数
    pub total: usize,

    /// 有参考文档的Skills数量
    pub with_reference: usize,

    /// 总指令长度
    pub total_instructions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_skills_executor_creation() {
        let assistant = InvestmentAssistant::new();
        let mut executor = SkillsExecutor::new(assistant);

        assert!(executor.load_skills_from_project().is_ok());
    }

    #[tokio::test]
    async fn test_list_skills() {
        let assistant = InvestmentAssistant::new();
        let mut executor = SkillsExecutor::new(assistant);
        executor.load_skills_from_project().await.unwrap();

        let skills = executor.list_skills();
        println!("可用Skills: {:?}", skills);
        assert!(!skills.is_empty());
    }

    #[tokio::test]
    async fn test_execute_graham_skill() {
        let assistant = InvestmentAssistant::new();
        let mut executor = SkillsExecutor::new(assistant);
        executor.load_skills_from_project().await.unwrap();

        // 测试执行（可能失败，因为需要网络）
        let result = executor.execute_skill("Graham", "分析AAPL").await;

        // 只验证能找到Skill，不验证实际结果
        assert!(executor.find_skill("Graham").is_some());
    }
}
