//! 日志系统
//!
//! 提供结构化日志记录、性能追踪、审计日志功能

use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use tracing::{info, warn, error, debug};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Registry,
};

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// 日志管理器
pub struct LogManager {
    /// 日志目录
    log_dir: PathBuf,

    /// 是否启用控制台输出
    console_output: bool,

    /// 是否启用文件输出
    file_output: bool,

    /// 日志级别
    log_level: LogLevel,
}

impl LogManager {
    /// 创建新的日志管理器
    pub fn new(log_dir: PathBuf) -> Self {
        Self {
            log_dir,
            console_output: true,
            file_output: true,
            log_level: LogLevel::Info,
        }
    }

    /// 设置日志级别
    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }

    /// 禁用控制台输出
    pub fn without_console(mut self) -> Self {
        self.console_output = false;
        self
    }

    /// 禁用文件输出
    pub fn without_file(mut self) -> Self {
        self.file_output = false;
        self
    }

    /// 初始化日志系统
    pub fn init(self) -> Result<(), anyhow::Error> {
        // 创建日志目录
        std::fs::create_dir_all(&self.log_dir)?;

        let log_level = match self.log_level {
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warning => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        };

        let builder = Registry::default();

        // 添加控制台输出
        if self.console_output {
            let console_layer = fmt::layer()
                .with_span_events(FmtSpan::CLOSE)
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false);

            builder.with(console_layer).init();
        }

        // 添加文件输出
        if self.file_output {
            let file_appender = rolling::daily(&self.log_dir, "investintel.log");
            let (non_blocking_appender, _guard) = non_blocking(file_appender);

            let file_layer = fmt::layer()
                .with_writer(non_blocking_appender)
                .with_span_events(FmtSpan::CLOSE)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false);

            builder.with(file_layer).init();
        }

        Ok(())
    }
}

/// 操作日志记录器
pub struct OperationLogger {
    /// 审计日志文件
    audit_log: PathBuf,
}

impl OperationLogger {
    /// 创建新的操作日志记录器
    pub fn new(log_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&log_dir).unwrap();

        let audit_log = log_dir.join("audit.log");

        Self {
            audit_log,
        }
    }

    /// 记录操作
    pub fn log_operation(&self, operation: Operation) -> Result<(), anyhow::Error> {
        let log_entry = OperationLogEntry {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            operation,
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.audit_log)?;

        writeln!(file, "{}", serde_json::to_string(&log_entry)?)?;

        Ok(())
    }

    /// 记录股票分析操作
    pub fn log_stock_analysis(
        &self,
        symbol: &str,
        analysis_type: &str,
        result: &str,
    ) -> Result<(), anyhow::Error> {
        self.log_operation(Operation::StockAnalysis {
            symbol: symbol.to_string(),
            analysis_type: analysis_type.to_string(),
            result: result.to_string(),
        })
    }

    /// 记录交易建议
    pub fn log_trading_advice(
        &self,
        symbol: &str,
        action: &str,
        confidence: f64,
    ) -> Result<(), anyhow::Error> {
        self.log_operation(Operation::TradingAdvice {
            symbol: symbol.to_string(),
            action: action.to_string(),
            confidence,
        })
    }

    /// 记录配置变更
    pub fn log_config_change(
        &self,
        key: &str,
        old_value: &str,
        new_value: &str,
    ) -> Result<(), anyhow::Error> {
        self.log_operation(Operation::ConfigChange {
            key: key.to_string(),
            old_value: old_value.to_string(),
            new_value: new_value.to_string(),
        })
    }
}

/// 操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Operation {
    /// 股票分析
    StockAnalysis {
        symbol: String,
        analysis_type: String,
        result: String,
    },

    /// 交易建议
    TradingAdvice {
        symbol: String,
        action: String,
        confidence: f64,
    },

    /// 配置变更
    ConfigChange {
        key: String,
        old_value: String,
        new_value: String,
    },

    /// 数据获取
    DataFetch {
        source: String,
        symbol: String,
        success: bool,
        duration_ms: u64,
    },

    /// 用户登录
    UserLogin {
        user_id: String,
        success: bool,
    },

    /// 组合变更
    PortfolioChange {
        action: String,
        symbol: String,
        shares: f64,
        price: f64,
    },
}

/// 操作日志条目
#[derive(Debug, Serialize)]
struct OperationLogEntry {
    timestamp: String,
    operation: Operation,
}

/// 性能追踪器
pub struct PerformanceTracker {
    /// 开始时间
    start_time: std::time::Instant,

    /// 操作名称
    operation: String,

    /// 元数据
    metadata: serde_json::Value,
}

impl PerformanceTracker {
    /// 开始追踪
    pub fn start(operation: &str) -> Self {
        info!("开始操作: {}", operation);

        Self {
            start_time: std::time::Instant::now(),
            operation: operation.to_string(),
            metadata: serde_json::json!({}),
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// 完成追踪
    pub fn finish(self) {
        let duration = self.start_time.elapsed();

        info!(
            operation = %self.operation,
            duration_ms = duration.as_millis(),
            metadata = %self.metadata,
            "操作完成"
        );
    }

    /// 完成追踪并返回结果
    pub fn finish_with<T>(self, result: T) -> T {
        self.finish();
        result
    }
}

/// 异步性能追踪器
#[derive(Clone)]
pub struct AsyncPerformanceTracker {
    operation: String,
    metadata: serde_json::Value,
}

impl AsyncPerformanceTracker {
    /// 开始追踪
    pub fn start(operation: &str) -> Self {
        debug!("开始异步操作: {}", operation);

        Self {
            operation: operation.to_string(),
            metadata: serde_json::json!({}),
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// 完成追踪
    pub fn finish(self, duration: std::time::Duration) {
        debug!(
            operation = %self.operation,
            duration_ms = duration.as_millis(),
            metadata = %self.metadata,
            "异步操作完成"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_tracker() {
        let tracker = PerformanceTracker::start("test_operation")
            .with_metadata(serde_json::json!({"key": "value"}));

        std::thread::sleep(std::time::Duration::from_millis(100));
        tracker.finish();
    }

    #[test]
    fn test_operation_logger() {
        let log_dir = std::env::temp_dir().join("test_logs");
        let logger = OperationLogger::new(log_dir);

        logger.log_stock_analysis("AAPL", "Graham", "Buy").unwrap();
        logger.log_trading_advice("MSFT", "Hold", 0.75).unwrap();
        logger.log_config_change("theme", "Dark", "Light").unwrap();
    }
}
