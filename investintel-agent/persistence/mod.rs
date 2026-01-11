//! 用户配置持久化模块
//!
//! 提供用户配置、历史记录、偏好的持久化存储功能

pub mod config;
pub mod history;
pub mod preferences;

pub use config::{UserConfig, UserConfigManager};
pub use history::{AnalysisHistory, HistoryEntry, HistoryManager};
pub use preferences::{UserPreferences, PreferencesManager};

use std::path::PathBuf;
use anyhow::Result;

/// 持久化管理器 - 统一管理所有持久化功能
pub struct PersistenceManager {
    /// 用户目录
    user_dir: PathBuf,

    /// 用户配置管理器
    config_manager: UserConfigManager,

    /// 历史记录管理器
    history_manager: HistoryManager,

    /// 偏好管理器
    preferences_manager: PreferencesManager,
}

impl PersistenceManager {
    /// 创建新的持久化管理器
    pub fn new(base_dir: PathBuf) -> Result<Self> {
        let user_dir = base_dir.join(".investintel");
        std::fs::create_dir_all(&user_dir)?;

        let config_dir = user_dir.join("config");
        let history_dir = user_dir.join("history");
        let preferences_dir = user_dir.join("preferences");

        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(&history_dir)?;
        std::fs::create_dir_all(&preferences_dir)?;

        Ok(Self {
            user_dir,
            config_manager: UserConfigManager::new(config_dir)?,
            history_manager: HistoryManager::new(history_dir)?,
            preferences_manager: PreferencesManager::new(preferences_dir)?,
        })
    }

    /// 从默认位置创建
    pub fn default() -> Result<Self> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))?;
        Self::new(PathBuf::from(home))
    }

    /// 获取用户配置管理器
    pub fn config(&self) -> &UserConfigManager {
        &self.config_manager
    }

    /// 获取可变用户配置管理器
    pub fn config_mut(&mut self) -> &mut UserConfigManager {
        &mut self.config_manager
    }

    /// 获取历史记录管理器
    pub fn history(&self) -> &HistoryManager {
        &self.history_manager
    }

    /// 获取可变历史记录管理器
    pub fn history_mut(&mut self) -> &mut HistoryManager {
        &mut self.history_manager
    }

    /// 获取偏好管理器
    pub fn preferences(&self) -> &PreferencesManager {
        &self.preferences_manager
    }

    /// 获取可变偏好管理器
    pub fn preferences_mut(&mut self) -> &mut PreferencesManager {
        &mut self.preferences_manager
    }

    /// 保存所有数据
    pub async fn save_all(&self) -> Result<()> {
        self.config_manager.save_if_modified()?;
        self.preferences_manager.save_if_modified()?;
        Ok(())
    }

    /// 加载所有数据
    pub async fn load_all(&self) -> Result<()> {
        self.config_manager.load()?;
        self.preferences_manager.load()?;
        Ok(())
    }
}
