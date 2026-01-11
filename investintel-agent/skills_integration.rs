//! Skills与InvestmentOrchestrator深度集成
//!
//! 基于Claude Skills最佳实践，实现Skills系统与Agent编排系统的无缝集成
//!
//! ## 核心功能
//!
//! 1. **自动Skills发现** - 从.claude/skills/目录自动发现并加载Skills
//! 2. **智能Skills路由** - 根据用户请求自动路由到合适的Skill
//! 3. **Progressive Disclosure** - 支持Skills的多级内容加载
//! 4. **Skills编排** - 多个Skills协同工作
//! 5. **缓存优化** - 智能缓存已加载的Skills内容

use crate::orchestration::{InvestmentOrchestrator, AnalysisType, OrchestrationConfig, OrchestrationResult};
use crate::skills_executor::SkillsExecutor;
use claude_agent_sdk_rs::skills::{SkillRegistry, SkillPackage};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Skills集成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsIntegrationConfig {
    /// Skills目录路径
    pub skills_dir: PathBuf,

    /// 是否启用缓存
    pub enable_cache: bool,

    /// 缓存TTL（秒）
    pub cache_ttl: u64,

    /// 是否启用Progressive Disclosure
    pub enable_progressive_disclosure: bool,

    /// 最大并发Skills执行数
    pub max_concurrent_skills: usize,
}

impl Default for SkillsIntegrationConfig {
    fn default() -> Self {
        Self {
            skills_dir: PathBuf::from(".claude/skills"),
            enable_cache: true,
            cache_ttl: 3600, // 1小时
            enable_progressive_disclosure: true,
            max_concurrent_skills: 5,
        }
    }
}

/// Skills缓存项
#[derive(Debug, Clone)]
struct SkillsCacheItem {
    /// Skill包
    package: SkillPackage,

    /// 缓存时间
    cached_at: std::time::Instant,

    /// 访问次数
    access_count: usize,
}

/// Skills集成系统
///
/// 整合Skills系统和InvestmentOrchestrator，提供统一的投资分析能力
pub struct SkillsIntegrationSystem {
    /// Skills注册表
    registry: Arc<SkillRegistry>,

    /// 已加载的Skills
    skills: Arc<RwLock<HashMap<String, SkillsCacheItem>>>,

    /// Investment编排器
    orchestrator: InvestmentOrchestrator,

    /// Skills执行器
    skills_executor: SkillsExecutor,

    /// 配置
    config: SkillsIntegrationConfig,
}

impl SkillsIntegrationSystem {
    /// 创建新的Skills集成系统
    pub async fn new() -> Result<Self> {
        Self::with_config(SkillsIntegrationConfig::default()).await
    }

    /// 使用自定义配置创建
    pub async fn with_config(config: SkillsIntegrationConfig) -> Result<Self> {
        // 创建Skills注册表
        let registry = Arc::new(SkillRegistry::new());

        // 加载Skills
        let skills = Arc::new(RwLock::new(HashMap::new()));

        // 创建编排器和执行器
        let orchestrator = InvestmentOrchestrator::new();
        let skills_executor = SkillsExecutor::new();

        let mut system = Self {
            registry,
            skills,
            orchestrator,
            skills_executor,
            config,
        };

        // 自动发现并加载Skills
        system.discover_and_load_skills().await?;

        Ok(system)
    }

    /// 发现并加载Skills
    async fn discover_and_load_skills(&self) -> Result<()> {
        if !self.config.skills_dir.exists() {
            tracing::warn!("Skills目录不存在: {:?}", self.config.skills_dir);
            return Ok(());
        }

        // 发现Skills
        let packages = SkillRegistry::discover_skill_md_from_dir(&self.config.skills_dir)?;

        tracing::info!("发现 {} 个Skills", packages.len());

        // 缓存Skills
        let mut skills = self.skills.write().await;
        for package in packages {
            let skill_name = package.metadata.name.clone();

            tracing::info!("  - {} ({})", skill_name, package.metadata.description);

            skills.insert(
                skill_name.clone(),
                SkillsCacheItem {
                    package,
                    cached_at: std::time::Instant::now(),
                    access_count: 0,
                },
            );
        }

        Ok(())
    }

    /// 智能分析 - 根据请求自动选择最佳分析方法
    ///
    /// 这个方法整合了Skills系统和Orchestration系统
    pub async fn smart_analyze(
        &self,
        symbol: &str,
        user_request: &str,
    ) -> Result<String> {
        // 1. 解析用户意图
        let intent = self.parse_intent(user_request);

        // 2. 根据意图选择分析方法
        match intent.analysis_type {
            SmartAnalysisType::Skill => {
                // 使用Skills系统
                self.execute_skill_analysis(symbol, intent.skill_name.unwrap()).await
            }
            SmartAnalysisType::Orchestration => {
                // 使用Orchestration系统
                self.execute_orchestration_analysis(symbol, intent.orchestration_type.unwrap()).await
            }
            SmartAnalysisType::Hybrid => {
                // 混合模式：先Skill后Orchestration
                self.execute_hybrid_analysis(symbol, intent).await
            }
        }
    }

    /// 执行Skill分析
    async fn execute_skill_analysis(&self, symbol: &str, skill_name: String) -> Result<String> {
        let skills = self.skills.read().await;

        // 查找Skill
        let skill_item = skills.get(&skill_name)
            .ok_or_else(|| anyhow::anyhow!("Skill not found: {}", skill_name))?;

        // 使用SkillsExecutor执行
        let result = self.skills_executor.execute_skill(
            &skill_name,
            &format!("分析 {}", symbol)
        ).await?;

        Ok(result)
    }

    /// 执行Orchestration分析
    async fn execute_orchestration_analysis(
        &self,
        symbol: &str,
        analysis_type: AnalysisType,
    ) -> Result<String> {
        let config = OrchestrationConfig::default();
        let result = self.orchestrator.analyze(symbol, analysis_type, config).await?;

        Ok(format!(
            "{}\n\n执行时间: {}ms\n置信度: {:.1}%",
            result.recommendation,
            result.execution_time_ms,
            result.confidence * 100.0
        ))
    }

    /// 执行混合分析
    async fn execute_hybrid_analysis(
        &self,
        symbol: &str,
        intent: ParsedIntent,
    ) -> Result<String> {
        let mut results = Vec::new();

        // 1. 先执行Skill分析
        if let Some(skill_name) = intent.skill_name {
            let skill_result = self.execute_skill_analysis(symbol, skill_name).await?;
            results.push(("Skill分析".to_string(), skill_result));
        }

        // 2. 再执行Orchestration分析
        if let Some(analysis_type) = intent.orchestration_type {
            let orch_result = self.execute_orchestration_analysis(symbol, analysis_type).await?;
            results.push(("编排分析".to_string(), orch_result));
        }

        // 3. 综合结果
        Ok(format!(
            "🎯 混合分析结果 - {}\n\n{}",
            symbol,
            results.iter()
                .map(|(name, result)| format!("### {}\n\n{}", name, result))
                .collect::<Vec<_>>()
                .join("\n\n")
        ))
    }

    /// 解析用户意图
    fn parse_intent(&self, user_request: &str) -> ParsedIntent {
        let request_lower = user_request.to_lowercase();

        // 关键词匹配
        let (analysis_type, skill_name, orchestration_type) = if request_lower.contains("graham")
            || request_lower.contains("价值投资")
            || request_lower.contains("内在价值")
            || request_lower.contains("安全边际") {
            (SmartAnalysisType::Skill, Some("graham-value-investing".to_string()), None)
        } else if request_lower.contains("kelly")
            || request_lower.contains("仓位")
            || request_lower.contains("position") {
            (SmartAnalysisType::Skill, Some("kelly-position".to_string()), None)
        } else if request_lower.contains("munger")
            || request_lower.contains("思维模型")
            || request_lower.contains("lollapalooza") {
            (SmartAnalysisType::Skill, Some("munger-mental-models".to_string()), None)
        } else if request_lower.contains("股息")
            || request_lower.contains("dividend") {
            (SmartAnalysisType::Orchestration, None, Some(AnalysisType::Dividend))
        } else if request_lower.contains("深度分析")
            || request_lower.contains("deep")
            || request_lower.contains("综合") {
            (SmartAnalysisType::Orchestration, None, Some(AnalysisType::Deep))
        } else if request_lower.contains("完整分析")
            || request_lower.contains("full") {
            (SmartAnalysisType::Orchestration, None, Some(AnalysisType::Full))
        } else {
            // 默认：快速价值分析
            (SmartAnalysisType::Orchestration, None, Some(AnalysisType::QuickValue))
        };

        ParsedIntent {
            analysis_type,
            skill_name,
            orchestration_type,
        }
    }

    /// 获取已加载的Skills列表
    pub async fn list_skills(&self) -> Vec<String> {
        let skills = self.skills.read().await;
        skills.keys().cloned().collect()
    }

    /// 获取Skill详细信息
    pub async fn get_skill_info(&self, skill_name: &str) -> Result<SkillInfo> {
        let skills = self.skills.read().await;

        let skill_item = skills.get(skill_name)
            .ok_or_else(|| anyhow::anyhow!("Skill not found: {}", skill_name))?;

        Ok(SkillInfo {
            name: skill_item.package.metadata.name.clone(),
            description: skill_item.package.metadata.description.clone(),
            cached_at: skill_item.cached_at,
            access_count: skill_item.access_count,
        })
    }

    /// 清理过期缓存
    pub async fn cleanup_cache(&self) {
        let mut skills = self.skills.write().await;
        let now = std::time::Instant::now();

        skills.retain(|_, item| {
            now.duration_since(item.cached_at).as_secs() < self.config.cache_ttl
        });
    }
}

impl Default for SkillsIntegrationSystem {
    fn default() -> Self {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                Self::new().await.unwrap()
            })
    }
}

/// 解析的意图
#[derive(Debug, Clone)]
struct ParsedIntent {
    /// 分析类型
    analysis_type: SmartAnalysisType,

    /// Skill名称（如果使用Skill）
    skill_name: Option<String>,

    /// Orchestration类型（如果使用Orchestration）
    orchestration_type: Option<AnalysisType>,
}

/// 智能分析类型
#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub enum SmartAnalysisType {
    /// 使用Skill
    Skill,

    /// 使用Orchestration
    Orchestration,

    /// 混合模式
    Hybrid,
}

/// Skill信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    /// Skill名称
    pub name: String,

    /// Skill描述
    pub description: String,

    /// 缓存时间
    pub cached_at: std::time::Instant,

    /// 访问次数
    pub access_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_skills_integration_system_creation() {
        let system = SkillsIntegrationSystem::new().await.unwrap();
        let skills = system.list_skills().await;

        // 验证Skills被正确加载
        assert!(!skills.is_empty());
    }

    #[tokio::test]
    async fn test_parse_intent() {
        let system = SkillsIntegrationSystem::new().await.unwrap();

        // 测试Graham分析
        let intent = system.parse_intent("使用Graham方法分析AAPL");
        assert_eq!(intent.analysis_type, SmartAnalysisType::Skill);
        assert_eq!(intent.skill_name, Some("graham-value-investing".to_string()));

        // 测试深度分析
        let intent = system.parse_intent("对MSFT进行深度分析");
        assert_eq!(intent.analysis_type, SmartAnalysisType::Orchestration);
        assert_eq!(intent.orchestration_type, Some(AnalysisType::Deep));
    }

    #[tokio::test]
    async fn test_smart_analyze() {
        let system = SkillsIntegrationSystem::new().await.unwrap();

        // 测试快速分析
        let result = system.smart_analyze("AAPL", "分析AAPL").await;
        assert!(result.is_ok());
    }
}
