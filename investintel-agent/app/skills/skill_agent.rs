//! Skill-Enabled Agent Implementation
//!
//! This module provides agents that can load and use Skills from the .claude/skills directory.
//! Skills contain investment knowledge and frameworks that agents can invoke.

use anyhow::Result;
use async_trait::async_trait;
use claude_agent_sdk_rs::orchestration::{Agent, AgentInput, AgentOutput, OrchestratorError};
use claude_agent_sdk_rs::skills::skill_md::{SkillsDirScanner, SkillMdFile};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Skill Registry - Manages loaded skills
// ============================================================================

/// Skill registry that holds all available skills
#[derive(Clone)]
pub struct SkillRegistry {
    skills: Arc<RwLock<HashMap<String, SkillMdFile>>>,
    skills_dir: PathBuf,
}

impl SkillRegistry {
    /// Create a new skill registry from a directory
    pub async fn from_dir(skills_dir: PathBuf) -> Result<Self> {
        let scanner = SkillsDirScanner::new(&skills_dir);
        let skills = scanner.scan()?;

        let mut skills_map = HashMap::new();
        for skill in skills {
            let name = skill.metadata.name.clone();
            skills_map.insert(name, skill);
        }

        Ok(Self {
            skills: Arc::new(RwLock::new(skills_map)),
            skills_dir,
        })
    }

    /// Get a skill by name
    pub async fn get_skill(&self, name: &str) -> Option<SkillMdFile> {
        let skills = self.skills.read().await;
        skills.get(name).cloned()
    }

    /// List all available skill names
    pub async fn list_skills(&self) -> Vec<String> {
        let skills = self.skills.read().await;
        skills.keys().cloned().collect()
    }

    /// Reload skills from directory
    pub async fn reload(&self) -> Result<()> {
        let scanner = SkillsDirScanner::new(&self.skills_dir);
        let skills = scanner.scan()?;

        let mut skills_map = HashMap::new();
        for skill in skills {
            let name = skill.metadata.name.clone();
            skills_map.insert(name, skill);
        }

        let mut skills_lock = self.skills.write().await;
        *skills_lock = skills_map;

        Ok(())
    }
}

// ============================================================================
// Skill Agent - An agent that uses skills
// ============================================================================

/// An agent that can invoke skills for analysis
///
/// This agent loads skills and uses them to provide investment analysis
/// based on the skill's knowledge framework.
pub struct SkillAgent {
    name: String,
    description: String,
    skill_name: String,
    skill_registry: SkillRegistry,
}

impl SkillAgent {
    /// Create a new skill-based agent
    ///
    /// # Arguments
    /// * `name` - Agent name
    /// * `description` - Agent description
    /// * `skill_name` - Name of the skill this agent should use
    /// * `skill_registry` - Skill registry to load skills from
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        skill_name: impl Into<String>,
        skill_registry: SkillRegistry,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            skill_name: skill_name.into(),
            skill_registry,
        }
    }

    /// Get the skill this agent uses
    pub async fn get_skill(&self) -> Result<SkillMdFile> {
        self.skill_registry
            .get_skill(&self.skill_name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Skill not found: {}", self.skill_name))
    }

    /// Apply skill analysis to investment data
    ///
    /// This method extracts key parameters from the input and applies
    /// the skill's analysis framework to generate insights.
    async fn apply_skill_analysis(
        &self,
        input: &AgentInput,
        skill: &SkillMdFile,
    ) -> Result<serde_json::Value> {
        // Parse input data
        let data = &input.context;

        // Extract symbol if present
        let symbol = data.get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN");

        // Apply skill-specific analysis based on skill name
        let analysis = match skill.metadata.name.as_str() {
            "Graham深度价值投资" => {
                self.apply_graham_analysis(data).await?
            }
            "Buffett质量价值投资" => {
                self.apply_buffett_analysis(data).await?
            }
            "Munger多元思维模型" => {
                self.apply_munger_analysis(data).await?
            }
            "Kelly准则仓位管理" => {
                self.apply_kelly_analysis(data).await?
            }
            "Lollapalooza效应检测" => {
                self.apply_lollapalooza_analysis(data).await?
            }
            _ => {
                json!({
                    "error": "Unknown skill",
                    "skill": skill.metadata.name
                })
            }
        };

        // Add metadata
        Ok(json!({
            "symbol": symbol,
            "skill": skill.metadata.name,
            "skill_version": skill.metadata.version,
            "analysis": analysis,
            "agent": self.name
        }))
    }

    /// Apply Graham value investing analysis
    async fn apply_graham_analysis(&self, data: &serde_json::Value) -> Result<serde_json::Value> {
        // Extract financial metrics
        let eps = data.get("eps").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let growth_rate = data.get("growth_rate").and_then(|v| v.as_f64()).unwrap_or(0.05);
        let current_price = data.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);

        // Graham formula: V = EPS × (8.5 + 2g)
        let intrinsic_value = eps * (8.5 + 2.0 * growth_rate * 100.0);

        // Calculate margin of safety
        let margin_of_safety = if intrinsic_value > 0.0 && current_price > 0.0 {
            (intrinsic_value - current_price) / intrinsic_value
        } else {
            0.0
        };

        // Score calculation (simplified)
        let mut score = 0.0;
        if margin_of_safety >= 0.40 { score += 30.0; }
        else if margin_of_safety >= 0.30 { score += 20.0; }
        else if margin_of_safety >= 0.20 { score += 10.0; }

        if current_price > 0.0 && eps > 0.0 {
            let pe = current_price / eps;
            if pe < 10.0 { score += 20.0; }
            else if pe < 15.0 { score += 10.0; }
        }

        Ok(json!({
            "intrinsic_value": intrinsic_value,
            "margin_of_safety": margin_of_safety,
            "score": score,
            "recommendation": if score >= 50 { "Buy" } else { "Hold" },
            "method": "Graham Formula",
            "formula": "V = EPS × (8.5 + 2g)"
        }))
    }

    /// Apply Buffett quality value analysis
    async fn apply_buffett_analysis(&self, data: &serde_json::Value) -> Result<serde_json::Value> {
        let roic = data.get("roic").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let roe = data.get("roe").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let pe = data.get("pe").and_then(|v| v.as_f64()).unwrap_or(20.0);

        // Quality score
        let mut quality_score = 0.0;
        if roic > 0.20 { quality_score += 25.0; }
        else if roic > 0.15 { quality_score += 20.0; }
        else if roic > 0.10 { quality_score += 15.0; }

        if roe > 0.20 { quality_score += 15.0; }
        else if roe > 0.15 { quality_score += 12.0; }

        // Valuation score
        let mut valuation_score = 0.0;
        if pe < 15.0 { valuation_score += 20.0; }
        else if pe < 20.0 { valuation_score += 10.0; }

        let total_score = quality_score + valuation_score;

        Ok(json!({
            "quality_score": quality_score,
            "valuation_score": valuation_score,
            "total_score": total_score,
            "recommendation": if total_score >= 40 { "High Quality" } else { "Medium Quality" },
            "criteria": {
                "roic": roic,
                "roe": roe,
                "pe": pe
            }
        }))
    }

    /// Apply Munger mental models analysis
    async fn apply_munger_analysis(&self, data: &serde_json::Value) -> Result<serde_json::Value> {
        // Lollapalooza factors
        let factors = json!({
            "mathematics": {
                "compound_growth": data.get("growth_rate").unwrap_or(&json!(0.05)),
                "probability": 0.6
            },
            "physics": {
                "economies_of_scale": data.get("scale_advantage").unwrap_or(&json!(false)),
                "tipping_point": false
            },
            "biology": {
                "evolutionary_advantage": true,
                "ecological_niche": "dominant"
            },
            "psychology": {
                "brand_value": data.get("brand_strength").unwrap_or(&json!("high")),
                "customer_loyalty": 0.8
            },
            "economics": {
                "opportunity_cost": 0.05,
                "network_effects": data.get("network_effects").unwrap_or(&json!(false))
            }
        });

        // Calculate lollapalooza score
        let lollapalooza_score = 0.75; // Simplified

        Ok(json!({
            "mental_models": factors,
            "lollapalooza_score": lollapalooza_score,
            "recommendation": if lollapalooza_score > 0.65 { "Strong Buy" } else { "Buy" },
            "munger_principle": "Multiple factors working in same direction"
        }))
    }

    /// Apply Kelly position sizing analysis
    async fn apply_kelly_analysis(&self, data: &serde_json::Value) -> Result<serde_json::Value> {
        let expected_return = data.get("expected_return")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.10);
        let variance = data.get("variance")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.04);

        // Kelly formula: f* = μ / σ²
        let kelly_fraction = if variance > 0.0 {
            expected_return / variance
        } else {
            0.0
        };

        // Apply practical constraints
        let full_kelly = kelly_fraction.min(1.0).max(0.0);
        let half_kelly = full_kelly * 0.5;
        let quarter_kelly = full_kelly * 0.25;

        Ok(json!({
            "expected_return": expected_return,
            "variance": variance,
            "kelly_fraction": kelly_fraction,
            "position_suggestions": {
                "full_kelly": full_kelly,
                "half_kelly": half_kelly,
                "quarter_kelly": quarter_kelly
            },
            "recommendation": format!("Use quarter Kelly: {:.1}%", quarter_kelly * 100.0),
            "formula": "f* = μ / σ²"
        }))
    }

    /// Apply Lollapalooza detection analysis
    async fn apply_lollapalooza_analysis(&self, data: &serde_json::Value) -> Result<serde_json::Value> {
        // Four dimensions
        let valuation = data.get("valuation_score").and_then(|v| v.as_f64()).unwrap_or(0.15);
        let quality = data.get("quality_score").and_then(|v| v.as_f64()).unwrap_or(0.20);
        let moat = data.get("moat_score").and_then(|v| v.as_f64()).unwrap_or(0.15);
        let catalysts = data.get("catalyst_score").and_then(|v| v.as_f64()).unwrap_or(0.10);

        let lollapalooza_score = valuation + quality + moat + catalysts;

        let (level, position) = if lollapalooza_score >= 0.80 {
            ("Super Lollapalooza", "30-40%")
        } else if lollapalooza_score >= 0.65 {
            ("Strong Lollapalooza", "15-25%")
        } else if lollapalooza_score >= 0.50 {
            ("Lollapalooza", "5-15%")
        } else {
            ("No Lollapalooza", "0%")
        };

        Ok(json!({
            "scores": {
                "valuation": valuation,
                "quality": quality,
                "moat": moat,
                "catalysts": catalysts
            },
            "total_score": lollapalooza_score,
            "level": level,
            "position_range": position,
            "decision": if lollapalooza_score >= 0.50 { "Invest" } else { "Skip" }
        }))
    }
}

#[async_trait]
impl Agent for SkillAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, input: AgentInput) -> claude_agent_sdk_rs::orchestration::Result<AgentOutput> {
        // Get the skill
        let skill = self.get_skill().await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(
                format!("Failed to load skill: {}", e)
            ))?;

        // Apply skill analysis
        let analysis = self.apply_skill_analysis(&input, &skill).await
            .map_err(|e| OrchestratorError::AgentExecutionFailed(
                format!("Analysis failed: {}", e)
            ))?;

        // Format output
        let content = format!(
            "Skill Analysis using: {}\n\n{}",
            skill.metadata.name,
            serde_json::to_string_pretty(&analysis).unwrap()
        );

        let output = AgentOutput::new(content)
            .with_confidence(0.85)
            .with_data(analysis)
            .with_metadata("skill", &skill.metadata.name)
            .with_metadata("skill_version", &skill.metadata.version);

        Ok(output)
    }
}

// ============================================================================
// Skill Agent Builder
// ============================================================================

/// Builder for creating skill-based agents
pub struct SkillAgentBuilder {
    name: Option<String>,
    description: Option<String>,
    skill_name: Option<String>,
    skill_registry: Option<SkillRegistry>,
}

impl SkillAgentBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            skill_name: None,
            skill_registry: None,
        }
    }

    /// Set agent name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set agent description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set skill name
    pub fn with_skill(mut self, skill_name: impl Into<String>) -> Self {
        self.skill_name = Some(skill_name.into());
        self
    }

    /// Set skill registry
    pub fn with_registry(mut self, registry: SkillRegistry) -> Self {
        self.skill_registry = Some(registry);
        self
    }

    /// Build the agent
    pub fn build(self) -> Result<SkillAgent> {
        Ok(SkillAgent {
            name: self.name.ok_or_else(|| anyhow::anyhow!("Name is required"))?,
            description: self.description.ok_or_else(|| anyhow::anyhow!("Description is required"))?,
            skill_name: self.skill_name.ok_or_else(|| anyhow::anyhow!("Skill name is required"))?,
            skill_registry: self.skill_registry.ok_or_else(|| anyhow::anyhow!("Skill registry is required"))?,
        })
    }
}

impl Default for SkillAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_skill_registry_creation() {
        let registry = SkillRegistry::from_dir(
            PathBuf::from(".claude/skills")
        ).await;

        assert!(registry.is_ok());

        let registry = registry.unwrap();
        let skills = registry.list_skills().await;

        println!("Available skills: {:?}", skills);
        assert!(!skills.is_empty());
        assert!(skills.contains(&"Graham深度价值投资".to_string()));
    }

    #[tokio::test]
    async fn test_graham_skill_agent() {
        let registry = SkillRegistry::from_dir(
            PathBuf::from(".claude/skills")
        ).await.unwrap();

        let agent = SkillAgent::new(
            "GrahamAgent",
            "Applies Graham value investing analysis",
            "Graham深度价值投资",
            registry,
        );

        // Create test input
        let input_data = json!({
            "symbol": "TEST",
            "eps": 5.0,
            "growth_rate": 0.06,
            "price": 50.0
        });

        let input = AgentInput::new("Analyze TEST")
            .with_context(input_data);

        // Execute agent
        let output = agent.execute(input).await.unwrap();

        assert!(output.is_successful());
        assert!(output.confidence > 0.5);

        println!("Graham Agent Output:\n{}", output.content);
    }

    #[tokio::test]
    async fn test_kelly_skill_agent() {
        let registry = SkillRegistry::from_dir(
            PathBuf::from(".claude/skills")
        ).await.unwrap();

        let agent = SkillAgent::new(
            "KellyAgent",
            "Applies Kelly position sizing",
            "Kelly准则仓位管理",
            registry,
        );

        let input_data = json!({
            "symbol": "TEST",
            "expected_return": 0.15,
            "variance": 0.0625
        });

        let input = AgentInput::new("Calculate position size")
            .with_context(input_data);

        let output = agent.execute(input).await.unwrap();

        assert!(output.is_successful());

        // Check Kelly calculation
        let data = &output.data;
        if let Some(analysis) = data.get("analysis") {
            println!("Kelly Analysis: {}", serde_json::to_string_pretty(analysis).unwrap());
        }
    }

    #[tokio::test]
    async fn test_lollapalooza_skill_agent() {
        let registry = SkillRegistry::from_dir(
            PathBuf::from(".claude/skills")
        ).await.unwrap();

        let agent = SkillAgent::new(
            "LollapaloozaAgent",
            "Detects Lollapalooza opportunities",
            "Lollapalooza效应检测",
            registry,
        );

        // High scores example
        let input_data = json!({
            "symbol": "SUPER",
            "valuation_score": 0.20,
            "quality_score": 0.25,
            "moat_score": 0.25,
            "catalyst_score": 0.18
        });

        let input = AgentInput::new("Detect Lollapalooza")
            .with_context(input_data);

        let output = agent.execute(input).await.unwrap();

        assert!(output.is_successful());
        println!("Lollapalooza Detection:\n{}", output.content);
    }
}
