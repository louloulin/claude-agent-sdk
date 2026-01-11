//! 增强的错误处理系统
//!
//! 提供详细的错误信息、错误分类、错误恢复机制

use thiserror::Error;

/// InvestIntel统一错误类型
#[derive(Error, Debug)]
pub enum InvestError {
    /// 数据获取错误
    #[error("Data fetch error for {symbol}: {message}")]
    DataFetch {
        symbol: String,
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// 数据解析错误
    #[error("Data parse error: {message}")]
    DataParse {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// 分析错误
    #[error("Analysis error for {symbol}: {message}")]
    Analysis {
        symbol: String,
        message: String,
        details: Option<String>,
    },

    /// 配置错误
    #[error("Configuration error: {message}")]
    Config {
        message: String,
        path: Option<String>,
    },

    /// 持久化错误
    #[error("Persistence error: {message}")]
    Persistence {
        message: String,
        path: Option<String>,
    },

    /// MCP连接错误
    #[error("MCP connection error to {server}: {message}")]
    MCPConnection {
        server: String,
        message: String,
    },

    /// 网络错误
    #[error("Network error: {message}")]
    Network {
        message: String,
        retryable: bool,
    },

    /// 验证错误
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
    },

    /// 业务逻辑错误
    #[error("Business logic error: {message}")]
    Business {
        message: String,
        code: String,
    },

    /// IO错误
    #[error("IO error: {message}")]
    Io {
        message: String,
        path: Option<String>,
    },

    /// 未知错误
    #[error("Unknown error: {message}")]
    Unknown {
        message: String,
    },
}

impl InvestError {
    /// 创建数据获取错误
    pub fn data_fetch(symbol: &str, message: &str, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        InvestError::DataFetch {
            symbol: symbol.to_string(),
            message: message.to_string(),
            source: Box::new(source),
        }
    }

    /// 创建分析错误
    pub fn analysis(symbol: &str, message: &str, details: Option<String>) -> Self {
        InvestError::Analysis {
            symbol: symbol.to_string(),
            message: message.to_string(),
            details,
        }
    }

    /// 创建网络错误
    pub fn network(message: &str, retryable: bool) -> Self {
        InvestError::Network {
            message: message.to_string(),
            retryable,
        }
    }

    /// 创建MCP连接错误
    pub fn mcp_connection(server: &str, message: &str) -> Self {
        InvestError::MCPConnection {
            server: server.to_string(),
            message: message.to_string(),
        }
    }

    /// 判断错误是否可重试
    pub fn is_retryable(&self) -> bool {
        match self {
            InvestError::Network { retryable, .. } => *retryable,
            InvestError::MCPConnection { .. } => true,
            InvestError::DataFetch { .. } => true,
            _ => false,
        }
    }

    /// 获取错误代码
    pub fn error_code(&self) -> &str {
        match self {
            InvestError::DataFetch { .. } => "DATA_FETCH_ERROR",
            InvestError::DataParse { .. } => "DATA_PARSE_ERROR",
            InvestError::Analysis { .. } => "ANALYSIS_ERROR",
            InvestError::Config { .. } => "CONFIG_ERROR",
            InvestError::Persistence { .. } => "PERSISTENCE_ERROR",
            InvestError::MCPConnection { .. } => "MCP_CONNECTION_ERROR",
            InvestError::Network { .. } => "NETWORK_ERROR",
            InvestError::Validation { .. } => "VALIDATION_ERROR",
            InvestError::Business { code, .. } => code,
            InvestError::Io { .. } => "IO_ERROR",
            InvestError::Unknown { .. } => "UNKNOWN_ERROR",
        }
    }

    /// 获取错误详情
    pub fn details(&self) -> Option<String> {
        match self {
            InvestError::Analysis { details, .. } => details.clone(),
            _ => None,
        }
    }

    /// 获取用户友好的错误信息
    pub fn user_friendly_message(&self) -> String {
        match self {
            InvestError::DataFetch { symbol, .. } => {
                format!("无法获取{}的数据，请稍后重试", symbol)
            },
            InvestError::Analysis { symbol, message, .. } => {
                format!("分析{}时出错: {}", symbol, message)
            },
            InvestError::Network { message, .. } => {
                format!("网络错误: {}", message)
            },
            InvestError::MCPConnection { server, .. } => {
                format!("无法连接到MCP服务器: {}", server)
            },
            _ => self.to_string(),
        }
    }
}

/// 错误恢复策略
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// 立即重试
    RetryImmediate,

    /// 延迟重试
    RetryDelayed(std::time::Duration),

    /// 使用备用数据源
    FallbackToAlternative,

    /// 返回缓存值
    UseCachedValue,

    /// 返回默认值
    UseDefaultValue,

    /// 无法恢复
    Abort,
}

/// 错误处理器
pub struct ErrorHandler {
    /// 最大重试次数
    max_retries: usize,

    /// 重试延迟
    retry_delay: std::time::Duration,

    /// 是否使用缓存
    use_cache: bool,
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: std::time::Duration::from_secs(1),
            use_cache: true,
        }
    }
}

impl ErrorHandler {
    /// 创建新的错误处理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 设置重试延迟
    pub fn with_retry_delay(mut self, delay: std::time::Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// 处理错误并返回恢复策略
    pub fn handle(&self, error: &InvestError, attempt: usize) -> RecoveryStrategy {
        // 如果超过最大重试次数，放弃
        if attempt >= self.max_retries {
            return RecoveryStrategy::Abort;
        }

        // 根据错误类型返回恢复策略
        match error {
            InvestError::Network { retryable: true, .. } => {
                RecoveryStrategy::RetryDelayed(self.retry_delay)
            },
            InvestError::DataFetch { .. } if self.use_cache => {
                RecoveryStrategy::UseCachedValue
            },
            InvestError::MCPConnection { .. } => {
                RecoveryStrategy::RetryDelayed(self.retry_delay * 2)
            },
            _ => RecoveryStrategy::Abort,
        }
    }

    /// 执行带重试的操作
    pub async fn retry<F, T, E>(&self, mut operation: F) -> Result<T, InvestError>
    where
        F: FnMut() -> Result<T, InvestError>,
        E: Into<InvestError>,
    {
        let mut attempt = 0;

        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    let strategy = self.handle(&error, attempt);

                    match strategy {
                        RecoveryStrategy::RetryImmediate => {
                            attempt += 1;
                            continue;
                        },
                        RecoveryStrategy::RetryDelayed(delay) => {
                            tokio::time::sleep(delay).await;
                            attempt += 1;
                            continue;
                        },
                        RecoveryStrategy::Abort => {
                            return Err(error);
                        },
                        _ => {
                            return Err(error);
                        },
                    }
                },
            }
        }
    }
}

/// 错误日志记录器
pub struct ErrorLogger {
    /// 错误日志文件路径
    log_file: Option<std::path::PathBuf>,
}

impl ErrorLogger {
    /// 创建新的错误日志记录器
    pub fn new() -> Self {
        Self {
            log_file: None,
        }
    }

    /// 设置日志文件
    pub fn with_log_file(mut self, path: std::path::PathBuf) -> Self {
        self.log_file = Some(path);
        self
    }

    /// 记录错误
    pub fn log_error(&self, error: &InvestError) {
        let log_entry = ErrorLogEntry {
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            error_code: error.error_code().to_string(),
            message: error.to_string(),
            user_friendly: error.user_friendly_message(),
            retryable: error.is_retryable(),
        };

        // 打印到stderr
        eprintln!("[ERROR] {}", serde_json::to_string(&log_entry).unwrap());

        // 如果设置了日志文件，写入文件
        if let Some(log_file) = &self.log_file {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
            {
                use std::io::Write;
                let _ = writeln!(file, "{}", serde_json::to_string(&log_entry).unwrap());
            }
        }
    }
}

/// 错误日志条目
#[derive(Debug, serde::Serialize)]
struct ErrorLogEntry {
    timestamp: String,
    error_code: String,
    message: String,
    user_friendly: String,
    retryable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryable() {
        let error = InvestError::network("Connection refused", true);
        assert!(error.is_retryable());

        let error = InvestError::analysis("AAPL", "Invalid data", None);
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_error_code() {
        let error = InvestError::network("Connection refused", true);
        assert_eq!(error.error_code(), "NETWORK_ERROR");

        let error = InvestError::analysis("AAPL", "Invalid data", None);
        assert_eq!(error.error_code(), "ANALYSIS_ERROR");
    }

    #[test]
    fn test_user_friendly_message() {
        let error = InvestError::analysis("AAPL", "Invalid PE ratio", None);
        let msg = error.user_friendly_message();
        assert!(msg.contains("AAPL"));
        assert!(msg.contains("Invalid PE ratio"));
    }

    #[test]
    fn test_recovery_strategy() {
        let handler = ErrorHandler::new().with_max_retries(3);

        // 可重试的错误
        let error = InvestError::network("Connection refused", true);
        let strategy = handler.handle(&error, 0);
        assert!(matches!(strategy, RecoveryStrategy::RetryDelayed(_)));

        // 不可重试的错误
        let error = InvestError::analysis("AAPL", "Invalid data", None);
        let strategy = handler.handle(&error, 0);
        assert!(matches!(strategy, RecoveryStrategy::Abort));
    }
}
