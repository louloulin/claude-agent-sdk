//! 历史记录管理
//!
//! 存储用户的分析历史和投资决策记录

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::PathBuf;
use std::io::BufWriter;
use anyhow::{Context, Result};

/// 历史记录管理器
pub struct HistoryManager {
    /// 历史记录目录
    pub history_dir: PathBuf,

    /// 历史记录文件
    pub history_file: PathBuf,

    /// 历史记录
    history: AnalysisHistory,
}

/// 分析历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisHistory {
    /// 所有历史记录
    pub entries: Vec<HistoryEntry>,

    /// 最大保存数量
    pub max_entries: usize,
}

/// 历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 记录ID
    pub id: String,

    /// 时间戳
    pub timestamp: String,

    /// 类型
    pub entry_type: HistoryEntryType,

    /// 股票代码
    pub symbol: Option<String>,

    /// 分析内容
    pub content: String,

    /// 分析结果数据
    pub data: Option<serde_json::Value>,

    /// 用户评分 (1-5)
    pub rating: Option<u8>,

    /// 用户备注
    pub notes: Option<String>,
}

/// 历史记录类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryEntryType {
    /// Graham价值分析
    GrahamAnalysis,

    /// Kelly仓位分析
    KellyPosition,

    /// Munger思维模型分析
    MungerAnalysis,

    /// 股息投资分析
    DividendAnalysis,

    /// 综合分析
    ComprehensiveAnalysis,

    /// 投资决策
    InvestmentDecision,

    /// 其他
    Other,
}

impl HistoryManager {
    /// 创建新的历史记录管理器
    pub fn new(history_dir: PathBuf) -> Result<Self> {
        let history_file = history_dir.join("analysis_history.json");

        let history = if history_file.exists() {
            let content = fs::read_to_string(&history_file)
                .context("Failed to read history file")?;

            serde_json::from_str(&content)
                .context("Failed to parse history file")?
        } else {
            AnalysisHistory {
                entries: vec![],
                max_entries: 1000,  // 最多保存1000条记录
            }
        };

        Ok(Self {
            history_dir,
            history_file,
            history,
        })
    }

    /// 添加历史记录
    pub fn add_entry(&mut self, entry: HistoryEntry) -> Result<()> {
        let mut entry = entry;

        // 如果没有ID，生成一个
        if entry.id.is_empty() {
            entry.id = uuid::Uuid::new_v4().to_string();
        }

        // 如果没有时间戳，添加当前时间
        if entry.timestamp.is_empty() {
            entry.timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        }

        self.history.entries.push(entry);

        // 限制记录数量
        if self.history.entries.len() > self.history.max_entries {
            // 删除最旧的记录
            self.history.entries.remove(0);
        }

        self.save()?;

        Ok(())
    }

    /// 添加Graham分析记录
    pub fn add_graham_analysis(
        &mut self,
        symbol: &str,
        content: &str,
        data: serde_json::Value,
    ) -> Result<()> {
        let entry = HistoryEntry {
            id: String::new(),
            timestamp: String::new(),
            entry_type: HistoryEntryType::GrahamAnalysis,
            symbol: Some(symbol.to_string()),
            content: content.to_string(),
            data: Some(data),
            rating: None,
            notes: None,
        };

        self.add_entry(entry)
    }

    /// 添加Kelly分析记录
    pub fn add_kelly_analysis(
        &mut self,
        symbol: &str,
        content: &str,
        data: serde_json::Value,
    ) -> Result<()> {
        let entry = HistoryEntry {
            id: String::new(),
            timestamp: String::new(),
            entry_type: HistoryEntryType::KellyPosition,
            symbol: Some(symbol.to_string()),
            content: content.to_string(),
            data: Some(data),
            rating: None,
            notes: None,
        };

        self.add_entry(entry)
    }

    /// 获取所有历史记录
    pub fn get_all(&self) -> &[HistoryEntry] {
        &self.history.entries
    }

    /// 按股票代码筛选
    pub fn filter_by_symbol(&self, symbol: &str) -> Vec<&HistoryEntry> {
        self.history.entries
            .iter()
            .filter(|e| e.symbol.as_ref().map(|s| s == symbol).unwrap_or(false))
            .collect()
    }

    /// 按类型筛选
    pub fn filter_by_type(&self, entry_type: &HistoryEntryType) -> Vec<&HistoryEntry> {
        self.history.entries
            .iter()
            .filter(|e| &e.entry_type == entry_type)
            .collect()
    }

    /// 获取最近N条记录
    pub fn get_recent(&self, n: usize) -> &[HistoryEntry] {
        let len = self.history.entries.len();
        if n >= len {
            &self.history.entries
        } else {
            &self.history.entries[len - n..]
        }
    }

    /// 评分历史记录
    pub fn rate_entry(&mut self, id: &str, rating: u8) -> Result<()> {
        if let Some(entry) = self.history.entries.iter_mut().find(|e| e.id == id) {
            entry.rating = Some(rating.min(5));  // 最大5分
            self.save()?;
        }
        Ok(())
    }

    /// 添加备注
    pub fn add_notes(&mut self, id: &str, notes: String) -> Result<()> {
        if let Some(entry) = self.history.entries.iter_mut().find(|e| e.id == id) {
            entry.notes = Some(notes);
            self.save()?;
        }
        Ok(())
    }

    /// 清空历史记录
    pub fn clear(&mut self) -> Result<()> {
        self.history.entries.clear();
        self.save()?;
        Ok(())
    }

    /// 保存历史记录
    fn save(&self) -> Result<()> {
        let file = File::create(&self.history_file)
            .context("Failed to create history file")?;

        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.history)
            .context("Failed to write history file")?;

        Ok(())
    }

    /// 获取统计信息
    pub fn statistics(&self) -> HistoryStatistics {
        let total = self.history.entries.len();

        let mut type_counts = std::collections::HashMap::new();
        let mut symbol_counts = std::collections::HashMap::new();

        for entry in &self.history.entries {
            *type_counts.entry(format!("{:?}", entry.entry_type)).or_insert(0) += 1;

            if let Some(symbol) = &entry.symbol {
                *symbol_counts.entry(symbol.clone()).or_insert(0) += 1;
            }
        }

        HistoryStatistics {
            total_entries: total,
            type_counts,
            symbol_counts,
        }
    }
}

/// 历史统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStatistics {
    /// 总记录数
    pub total_entries: usize,

    /// 按类型统计
    pub type_counts: std::collections::HashMap<String, usize>,

    /// 按股票统计
    pub symbol_counts: std::collections::HashMap<String, usize>,
}
