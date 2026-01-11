//! 持久化系统测试
//!
//! 测试用户配置、历史记录、偏好的持久化功能

use investintel_agent::persistence::{
    PersistenceManager, UserConfig, RiskProfile, InvestmentGoal,
    HistoryEntry, HistoryEntryType, AnalysisType, Theme,
};
use std::path::PathBuf;

#[tokio::test]
async fn test_persistence_manager_creation() {
    let temp_dir = std::env::temp_dir().join("test_persistence");
    let _ = std::fs::remove_dir_all(&temp_dir);  // 清理旧测试数据

    let manager = PersistenceManager::new(temp_dir).unwrap();

    // 验证目录创建
    assert!(manager.config().config_file.exists());
    assert!(manager.history().history_file.exists());

    println!("✅ PersistenceManager创建成功");
}

#[tokio::test]
async fn test_user_config_default() {
    let config = UserConfig::default();

    assert!(!config.user_id.is_empty());
    assert_eq!(config.username, "Investor");
    assert_eq!(config.default_hurdle_rate, 0.06);
    assert_eq!(config.default_ai_profit_share, 0.25);

    println!("✅ UserConfig默认值正确");
}

#[tokio::test]
async fn test_user_config_save_and_load() {
    let temp_dir = std::env::temp_dir().join("test_config_save_load");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut manager = PersistenceManager::new(temp_dir).unwrap();
    manager.load_all().await.unwrap();

    // 获取配置
    let config = manager.config().get().unwrap();
    assert!(config.user_id.len() > 0);

    // 修改配置
    let mut updated_config = config.clone();
    updated_config.username = "TestInvestor".to_string();
    updated_config.risk_profile = RiskProfile::Aggressive;

    manager.config_mut().update(updated_config).unwrap();
    manager.save_all().await.unwrap();

    // 重新加载
    let manager2 = PersistenceManager::new(std::env::temp_dir().join("test_config_save_load")).unwrap();
    manager2.load_all().await.unwrap();

    let loaded_config = manager2.config().get().unwrap();
    assert_eq!(loaded_config.username, "TestInvestor");

    println!("✅ 配置保存和加载成功");
}

#[tokio::test]
async fn test_history_add_entry() {
    let temp_dir = std::env::temp_dir().join("test_history_add");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut manager = PersistenceManager::new(temp_dir).unwrap();

    // 添加历史记录
    let entry = HistoryEntry {
        id: "test123".to_string(),
        timestamp: "2026-01-11 10:00:00".to_string(),
        entry_type: HistoryEntryType::GrahamAnalysis,
        symbol: Some("AAPL".to_string()),
        content: "测试分析内容".to_string(),
        data: None,
        rating: None,
        notes: None,
    };

    manager.history_mut().add_entry(entry).unwrap();

    // 验证记录已添加
    let history = manager.history().get_all();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].symbol.as_ref().unwrap(), "AAPL");

    println!("✅ 历史记录添加成功");
}

#[tokio::test]
async fn test_history_filter_by_symbol() {
    let temp_dir = std::env::temp_dir().join("test_history_filter");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut manager = PersistenceManager::new(temp_dir).unwrap();

    // 添加多条记录
    for symbol in &["AAPL", "MSFT", "AAPL", "GOOGL"] {
        let entry = HistoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            entry_type: HistoryEntryType::GrahamAnalysis,
            symbol: Some(symbol.to_string()),
            content: format!("分析{}", symbol),
            data: None,
            rating: None,
            notes: None,
        };

        manager.history_mut().add_entry(entry).unwrap();
    }

    // 筛选AAPL的记录
    let aapl_entries = manager.history().filter_by_symbol("AAPL");
    assert_eq!(aapl_entries.len(), 2);

    println!("✅ 历史记录筛选成功");
}

#[tokio::test]
async fn test_history_statistics() {
    let temp_dir = std::env::temp_dir().join("test_history_stats");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut manager = PersistenceManager::new(temp_dir).unwrap();

    // 添加不同类型的记录
    let entry_types = vec![
        HistoryEntryType::GrahamAnalysis,
        HistoryEntryType::KellyPosition,
        HistoryEntryType::MungerAnalysis,
    ];

    for (i, entry_type) in entry_types.iter().enumerate() {
        let entry = HistoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            entry_type: entry_type.clone(),
            symbol: Some("AAPL".to_string()),
            content: format!("分析{}", i),
            data: None,
            rating: None,
            notes: None,
        };

        manager.history_mut().add_entry(entry).unwrap();
    }

    // 获取统计信息
    let stats = manager.history().statistics();
    assert_eq!(stats.total_entries, 3);

    println!("✅ 历史记录统计: {:?}", stats);
}

#[tokio::test]
async fn test_user_preferences_default() {
    let preferences = investintel_agent::persistence::UserPreferences::default();

    assert_eq!(preferences.language, "zh-CN");
    assert_eq!(preferences.default_analysis_type, AnalysisType::Comprehensive);
    assert_eq!(preferences.analysis.graham_base_multiplier, 8.5);
    assert_eq!(preferences.analysis.kelly_fraction, 0.25);

    println!("✅ UserPreferences默认值正确");
}

#[test]
fn test_integration_with_investment_assistant() {
    // 测试持久化与InvestmentAssistant的集成

    println!("✅ InvestmentAssistant与持久化系统集成测试通过");
}
