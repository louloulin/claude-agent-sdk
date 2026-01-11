//! 用户偏好管理
//!
//! 存储用户的界面偏好、分析偏好等

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};

/// 用户偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// 界面语言
    pub language: String,

    /// 主题
    pub theme: Theme,

    /// 默认分析类型
    pub default_analysis_type: AnalysisType,

    /// 显示设置
    pub display: DisplayPreferences,

    /// 分析偏好
    pub analysis: AnalysisPreferences,

    /// 通知设置
    pub notifications: NotificationPreferences,

    /// 数据偏好
    pub data: DataPreferences,
}

/// 主题设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

/// 分析类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    Graham,
    Kelly,
    Munger,
    Dividend,
    Comprehensive,
}

/// 显示偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayPreferences {
    /// 显示货币
    pub currency: String,

    /// 显示日期格式
    pub date_format: String,

    /// 显示数字格式
    pub number_format: String,

    /// 每页显示数量
    pub page_size: usize,

    /// 是否显示图表
    pub show_charts: bool,

    /// 是否显示详细信息
    pub show_details: bool,
}

/// 分析偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPreferences {
    /// 默认Graham系数
    pub graham_base_multiplier: f64,

    /// 默认Graham增长率系数
    pub graham_growth_multiplier: f64,

    /// 默认Kelly分数
    pub kelly_fraction: f64,

    /// 默认安全边际阈值
    pub margin_of_safety_threshold: f64,

    /// 是否使用Lollapalooza效应检测
    pub enable_lollapalooza: bool,

    /// 是否使用逆向思维
    pub enable_inversion: bool,
}

/// 通知偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// 是否启用通知
    pub enabled: bool,

    /// 价格提醒阈值
    pub price_alert_threshold: f64,

    /// 是否发送邮件通知
    pub email_notifications: bool,

    /// 是否发送推送通知
    pub push_notifications: bool,
}

/// 数据偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPreferences {
    /// 数据源优先级
    pub data_source_priority: Vec<String>,

    /// 缓存TTL (秒)
    pub cache_ttl: u64,

    /// 是否自动刷新数据
    pub auto_refresh: bool,

    /// 刷新间隔 (秒)
    pub refresh_interval: u64,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(),
            theme: Theme::Auto,
            default_analysis_type: AnalysisType::Comprehensive,
            display: DisplayPreferences::default(),
            analysis: AnalysisPreferences::default(),
            notifications: NotificationPreferences::default(),
            data: DataPreferences::default(),
        }
    }
}

impl Default for DisplayPreferences {
    fn default() -> Self {
        Self {
            currency: "USD".to_string(),
            date_format: "%Y-%m-%d".to_string(),
            number_format: "#,##0.00".to_string(),
            page_size: 20,
            show_charts: true,
            show_details: true,
        }
    }
}

impl Default for AnalysisPreferences {
    fn default() -> Self {
        Self {
            graham_base_multiplier: 8.5,
            graham_growth_multiplier: 2.0,
            kelly_fraction: 0.25,  // 1/4 Kelly
            margin_of_safety_threshold: 0.30,  // 30%
            enable_lollapalooza: true,
            enable_inversion: true,
        }
    }
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            enabled: true,
            price_alert_threshold: 0.05,  // 5%
            email_notifications: false,
            push_notifications: false,
        }
    }
}

impl Default for DataPreferences {
    fn default() -> Self {
        Self {
            data_source_priority: vec![
                "mcp".to_string(),
                "yahoo".to_string(),
                "alpha_vantage".to_string(),
            ],
            cache_ttl: 60,
            auto_refresh: false,
            refresh_interval: 300,  // 5分钟
        }
    }
}

/// 用户偏好管理器
pub struct PreferencesManager {
    /// 偏好目录
    preferences_dir: PathBuf,

    /// 偏好文件
    preferences_file: PathBuf,

    /// 当前偏好
    preferences: Option<UserPreferences>,

    /// 是否已修改
    modified: bool,
}

impl PreferencesManager {
    /// 创建新的偏好管理器
    pub fn new(preferences_dir: PathBuf) -> Result<Self> {
        let preferences_file = preferences_dir.join("user_preferences.json");

        Ok(Self {
            preferences_dir,
            preferences_file,
            preferences: None,
            modified: false,
        })
    }

    /// 加载偏好
    pub fn load(&mut self) -> Result<()> {
        if self.preferences_file.exists() {
            let content = fs::read_to_string(&self.preferences_file)
                .context("Failed to read preferences file")?;

            let preferences: UserPreferences = serde_json::from_str(&content)
                .context("Failed to parse preferences file")?;

            self.preferences = Some(preferences);
            self.modified = false;
        } else {
            // 创建默认偏好
            self.preferences = Some(UserPreferences::default());
            self.modified = true;
            self.save()?;
        }

        Ok(())
    }

    /// 保存偏好
    pub fn save(&mut self) -> Result<()> {
        if let Some(preferences) = &self.preferences {
            let content = serde_json::to_string_pretty(preferences)
                .context("Failed to serialize preferences")?;

            fs::write(&self.preferences_file, content)
                .context("Failed to write preferences file")?;

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

    /// 获取偏好
    pub fn get(&self) -> Option<&UserPreferences> {
        self.preferences.as_ref()
    }

    /// 更新偏好
    pub fn update(&mut self, preferences: UserPreferences) -> Result<()> {
        self.preferences = Some(preferences);
        self.modified = true;
        Ok(())
    }

    /// 更新分析偏好
    pub fn update_analysis_preferences(
        &mut self,
        analysis: AnalysisPreferences,
    ) -> Result<()> {
        if let Some(preferences) = &mut self.preferences {
            preferences.analysis = analysis;
            self.modified = true;
        }
        Ok(())
    }

    /// 是否已修改
    pub fn is_modified(&self) -> bool {
        self.modified
    }
}
