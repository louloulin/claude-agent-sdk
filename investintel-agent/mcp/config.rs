//! MCP配置

use serde::{Deserialize, Serialize};

/// MCP配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    /// 服务器URL
    pub url: Option<String>,

    /// API密钥
    pub api_key: Option<String>,

    /// 密钥（用于加密API）
    pub secret: Option<String>,

    /// 连接超时（秒）
    pub timeout: Option<u64>,

    /// 最大重试次数
    pub max_retries: Option<u32>,

    /// 启用缓存
    pub enable_cache: Option<bool>,

    /// 自定义参数
    #[serde(flatten)]
    pub custom: serde_json::Value,
}

impl Default for MCPConfig {
    fn default() -> Self {
        Self {
            url: None,
            api_key: None,
            secret: None,
            timeout: Some(30),
            max_retries: Some(3),
            enable_cache: Some(true),
            custom: serde_json::json!({}),
        }
    }
}

impl MCPConfig {
    /// 创建新的配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置URL
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// 设置API密钥
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// 设置密钥
    pub fn with_secret(mut self, secret: impl Into<String>) -> Self {
        self.secret = Some(secret.into());
        self
    }

    /// 设置超时
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// 启用/禁用缓存
    pub fn with_cache(mut self, enable: bool) -> Self {
        self.enable_cache = Some(enable);
        self
    }

    /// 添加自定义参数
    pub fn with_custom(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        let key = key.into();
        if let Some(obj) = self.custom.as_object_mut() {
            obj.insert(key, value);
        } else {
            self.custom = serde_json::json!({ key: value });
        }
        self
    }

    /// 从环境变量加载配置
    pub fn from_env(prefix: &str) -> Self {
        let mut config = Self::default();

        // 从环境变量读取配置
        if let Ok(url) = std::env::var(format!("{}_URL", prefix)) {
            config.url = Some(url);
        }

        if let Ok(api_key) = std::env::var(format!("{}_API_KEY", prefix)) {
            config.api_key = Some(api_key);
        }

        if let Ok(secret) = std::env::var(format!("{}_SECRET", prefix)) {
            config.secret = Some(secret);
        }

        config
    }

    /// 验证配置
    pub fn validate(&self) -> anyhow::Result<()> {
        // 检查URL格式
        if let Some(url) = &self.url {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                anyhow::bail!("Invalid URL format: {}", url);
            }
        }

        // 检查超时值
        if let Some(timeout) = self.timeout {
            if timeout == 0 || timeout > 300 {
                anyhow::bail!("Invalid timeout: {}", timeout);
            }
        }

        // 检查重试次数
        if let Some(max_retries) = self.max_retries {
            if max_retries > 10 {
                anyhow::bail!("Invalid max_retries: {}", max_retries);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = MCPConfig::new()
            .with_url("https://example.com")
            .with_api_key("test-key")
            .with_timeout(60)
            .with_max_retries(5);

        assert_eq!(config.url, Some("https://example.com".to_string()));
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout, Some(60));
        assert_eq!(config.max_retries, Some(5));
    }

    #[test]
    fn test_config_validation() {
        let config = MCPConfig::new()
            .with_url("https://example.com")
            .with_timeout(30);

        assert!(config.validate().is_ok());

        // Invalid URL
        let bad_config = MCPConfig::new()
            .with_url("invalid-url");
        assert!(bad_config.validate().is_err());

        // Invalid timeout
        let bad_config = MCPConfig::new()
            .with_timeout(0);
        assert!(bad_config.validate().is_err());
    }

    #[test]
    fn test_config_from_env() {
        // 设置环境变量
        std::env::set_var("TEST_MCP_URL", "https://test.com");
        std::env::set_var("TEST_MCP_API_KEY", "test-key");

        let config = MCPConfig::from_env("TEST_MCP");

        assert_eq!(config.url, Some("https://test.com".to_string()));
        assert_eq!(config.api_key, Some("test-key".to_string()));

        // 清理环境变量
        std::env::remove_var("TEST_MCP_URL");
        std::env::remove_var("TEST_MCP_API_KEY");
    }

    #[test]
    fn test_config_serialization() {
        let config = MCPConfig::new()
            .with_url("https://example.com")
            .with_api_key("test-key");

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("https://example.com"));
        assert!(json.contains("test-key"));
    }
}
