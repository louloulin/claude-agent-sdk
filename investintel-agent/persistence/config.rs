//! 用户配置管理
//!
//! 存储用户的基本配置信息

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};

/// 用户配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// 用户ID
    pub user_id: String,

    /// 用户名
    pub username: String,

    /// 邮箱
    pub email: Option<String>,

    /// 默认基准收益率 (巴菲特: 6%)
    pub default_hurdle_rate: f64,

    /// 默认AI利润分成 (巴菲特: 25%)
    pub default_ai_profit_share: f64,

    /// 风险偏好
    pub risk_profile: RiskProfile,

    /// 投资目标
    pub investment_goals: Vec<InvestmentGoal>,

    /// 创建时间
    pub created_at: String,

    /// 最后更新时间
    pub updated_at: String,
}

/// 风险偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskProfile {
    /// 保守
    Conservative,
    /// 平衡
    Moderate,
    /// 进取
    Aggressive,
}

/// 投资目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentGoal {
    /// 目标名称
    pub name: String,

    /// 目标描述
    pub description: String,

    /// 目标金额
    pub target_amount: f64,

    /// 目标日期
    pub target_date: String,

    /// 优先级
    pub priority: u8,
}

impl Default for UserConfig {
    fn default() -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            user_id: uuid::Uuid::new_v4().to_string(),
            username: "Investor".to_string(),
            email: None,
            default_hurdle_rate: 0.06,  // 6%
            default_ai_profit_share: 0.25,  // 25%
            risk_profile: RiskProfile::Moderate,
            investment_goals: vec![],
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// 用户配置管理器
pub struct UserConfigManager {
    /// 配置目录
    pub config_dir: PathBuf,

    /// 配置文件路径
    pub config_file: PathBuf,

    /// 当前配置
    config: Option<UserConfig>,

    /// 是否已修改
    modified: bool,
}

impl UserConfigManager {
    /// 创建新的配置管理器
    pub fn new(config_dir: PathBuf) -> Result<Self> {
        let config_file = config_dir.join("user_config.json");
        Ok(Self {
            config_dir,
            config_file,
            config: None,
            modified: false,
        })
    }

    /// 加载配置
    pub fn load(&mut self) -> Result<()> {
        if self.config_file.exists() {
            let content = fs::read_to_string(&self.config_file)
                .context("Failed to read config file")?;

            let config: UserConfig = serde_json::from_str(&content)
                .context("Failed to parse config file")?;

            self.config = Some(config);
            self.modified = false;
        } else {
            // 创建默认配置
            self.config = Some(UserConfig::default());
            self.modified = true;
            self.save()?;
        }

        Ok(())
    }

    /// 保存配置
    pub fn save(&mut self) -> Result<()> {
        if let Some(config) = &self.config {
            // 更新时间戳
            let mut config = config.clone();
            config.updated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

            let content = serde_json::to_string_pretty(&config)
                .context("Failed to serialize config")?;

            fs::write(&self.config_file, content)
                .context("Failed to write config file")?;

            self.modified = false;
        }
        Ok(())
    }

    /// 如果已修改则保存
    pub fn save_if_modified(&mut self) -> Result<()> {
        if self.modified {
            self.save()?;
        }
        Ok(())
    }

    /// 获取配置
    pub fn get(&self) -> Option<&UserConfig> {
        self.config.as_ref()
    }

    /// 更新配置
    pub fn update(&mut self, config: UserConfig) -> Result<()> {
        self.config = Some(config);
        self.modified = true;
        Ok(())
    }

    /// 更新风险偏好
    pub fn update_risk_profile(&mut self, risk_profile: RiskProfile) -> Result<()> {
        if let Some(config) = &mut self.config {
            config.risk_profile = risk_profile;
            self.modified = true;
        }
        Ok(())
    }

    /// 添加投资目标
    pub fn add_investment_goal(&mut self, goal: InvestmentGoal) -> Result<()> {
        if let Some(config) = &mut self.config {
            config.investment_goals.push(goal);
            self.modified = true;
        }
        Ok(())
    }

    /// 是否已修改
    pub fn is_modified(&self) -> bool {
        self.modified
    }
}
