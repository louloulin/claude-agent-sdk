//! MCP Client - MCP协议客户端实现
//!
//! 负责与MCP服务器通信，执行工具调用

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::config::MCPConfig;

/// MCP客户端
pub struct MCPClient {
    /// 服务器名称
    name: String,

    /// 服务器URL
    url: Option<String>,

    /// API密钥
    api_key: Option<String>,

    /// 密钥（用于加密API）
    secret: Option<String>,

    /// 客户端配置
    config: MCPConfig,
}

impl MCPClient {
    /// 创建新的MCP客户端
    pub async fn new(name: &str, config: MCPConfig) -> Result<Self> {
        let client = Self {
            name: name.to_string(),
            url: config.url.clone(),
            api_key: config.api_key.clone(),
            secret: config.secret.clone(),
            config,
        };

        // 验证连接
        client.ping().await
            .context(format!("Failed to connect to MCP server: {}", name))?;

        Ok(client)
    }

    /// 调用MCP工具
    pub async fn call_tool(&self, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        // 构建MCP请求
        let request = MCPRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": tool_name,
                "arguments": arguments,
            }),
        };

        // 发送请求到MCP服务器
        let response = self.send_request(request).await?;

        // 解析响应
        if let Some(error) = response.error {
            anyhow::bail!("MCP tool call failed: {:?}", error);
        }

        response.result.context("Empty response from MCP server")
    }

    /// 列出可用工具
    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        let request = MCPRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "tools/list".to_string(),
            params: serde_json::Value::Null,
        };

        let response = self.send_request(request).await?;

        if let Some(error) = response.error {
            anyhow::bail!("Failed to list tools: {:?}", error);
        }

        let tools_response: ToolsListResponse = serde_json::from_value(
            response.result.context("Empty response")?
        )?;

        Ok(tools_response.tools)
    }

    /// Ping服务器
    pub async fn ping(&self) -> Result<()> {
        let request = MCPRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "ping".to_string(),
            params: serde_json::Value::Null,
        };

        let response = self.send_request(request).await?;

        if let Some(error) = response.error {
            anyhow::bail!("Ping failed: {:?}", error);
        }

        Ok(())
    }

    /// 发送JSON-RPC请求
    async fn send_request(&self, request: MCPRequest) -> Result<MCPResponse> {
        // 如果有URL，使用HTTP客户端
        if let Some(url) = &self.url {
            return self.send_http_request(url, request).await;
        }

        // 否则，使用stdio（进程间通信）
        self.send_stdio_request(request).await
    }

    /// 通过HTTP发送请求
    async fn send_http_request(&self, url: &str, request: MCPRequest) -> Result<MCPResponse> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;

        let mut http_request = http_client.post(url)
            .header("Content-Type", "application/json")
            .json(&request);

        // 添加认证头
        if let Some(api_key) = &self.api_key {
            http_request = http_request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = http_request
            .send()
            .await
            .context("Failed to send HTTP request")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP request failed with status: {}", response.status());
        }

        let mcp_response: MCPResponse = response
            .json()
            .await
            .context("Failed to parse response")?;

        Ok(mcp_response)
    }

    /// 通过stdio发送请求（进程间通信）
    async fn send_stdio_request(&self, request: MCPRequest) -> Result<MCPResponse> {
        // 对于stdio通信，需要启动子进程
        // 这里简化实现，实际使用时需要完整实现

        tracing::warn!("stdio communication not fully implemented, using mock response");

        // 模拟响应（实际使用时需要真实实现）
        Ok(MCPResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({
                "status": "ok",
                "message": "Mock response for stdio communication"
            })),
            error: None,
        })
    }
}

/// MCP请求（JSON-RPC格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MCPRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

/// MCP响应（JSON-RPC格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MCPResponse {
    jsonrpc: String,
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<MCPError>,
}

/// MCP错误
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MCPError {
    code: i32,
    message: String,
    data: Option<serde_json::Value>,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// 工具名称
    pub name: String,

    /// 工具描述
    pub description: String,

    /// 输入JSON Schema
    pub input_schema: serde_json::Value,
}

/// 工具列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolsListResponse {
    tools: Vec<Tool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_request_serialization() {
        let request = MCPRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "tools/list".to_string(),
            params: serde_json::Value::Null,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"tools/list\""));
    }

    #[test]
    fn test_mcp_response_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"status": "ok"},
            "error": null
        }"#;

        let response: MCPResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, 1);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }
}
