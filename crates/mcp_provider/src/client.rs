//! MCP HTTP/JSON-RPC клиент: POST, timeout 30s, retry 1.
//! Методы: call, list_tools, call_tool.

use std::time::Duration;

use crate::error::McpError;
use crate::types::{McpServerConfig, McpToolDescriptor};

/// JSON-RPC запрос (2.0).
#[derive(serde::Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    method: String,
    params: serde_json::Value,
    id: u64,
}

/// JSON-RPC ответ: result или error.
#[derive(serde::Deserialize)]
struct JsonRpcResponse {
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(serde::Deserialize)]
struct JsonRpcError {
    message: String,
}

/// Клиент к одному MCP-серверу.
pub struct McpClient {
    url: String,
    headers: std::collections::HashMap<String, String>,
    request_id: std::sync::atomic::AtomicU64,
}

impl McpClient {
    pub fn new(config: &McpServerConfig) -> Self {
        Self {
            url: config.url.clone(),
            headers: config.headers.clone(),
            request_id: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Вызов метода MCP. Timeout 30s, при ошибке — один повтор.
    pub async fn call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let id = self
            .request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let body = JsonRpcRequest {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
            id,
        };
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(McpError::Http)?;

        let mut req = client.post(&self.url).json(&body);
        for (k, v) in &self.headers {
            req = req.header(k.as_str(), v.as_str());
        }

        let resp = req.send().await;
        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(mcp_url = %self.url, error = %e, "MCP request failed, retrying once");
                let mut req2 = client.post(&self.url).json(&body);
                for (k, v) in &self.headers {
                    req2 = req2.header(k.as_str(), v.as_str());
                }
                req2.send().await.map_err(McpError::Http)?
            }
        };

        let status = resp.status();
        let text = resp.text().await.map_err(McpError::Http)?;
        if !status.is_success() {
            return Err(McpError::JsonRpc(format!(
                "HTTP {}: {}",
                status,
                text.chars().take(200).collect::<String>()
            )));
        }

        let parsed: JsonRpcResponse = serde_json::from_str(&text)
            .map_err(|e| McpError::JsonRpc(format!("parse response: {}", e)))?;
        if let Some(err) = parsed.error {
            return Err(McpError::JsonRpc(err.message));
        }
        parsed
            .result
            .ok_or_else(|| McpError::JsonRpc("missing result".to_string()))
    }

    /// Список инструментов сервера (MCP tools/list). Ответ: { "tools": [ { "name", "description?", "inputSchema"? } ] }.
    pub async fn list_tools(&self, server_name: &str) -> Result<Vec<McpToolDescriptor>, McpError> {
        let result = self.call("tools/list", serde_json::json!({})).await?;
        let arr = result
            .get("tools")
            .and_then(|v| v.as_array())
            .ok_or_else(|| McpError::JsonRpc("tools/list: missing tools array".to_string()))?;
        let mut out = Vec::with_capacity(arr.len());
        for item in arr {
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| McpError::JsonRpc("tools/list: tool missing name".to_string()))?
                .to_string();
            let description = item.get("description").and_then(|v| v.as_str()).map(String::from);
            let input_schema = item
                .get("inputSchema")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            out.push(McpToolDescriptor {
                server: server_name.to_string(),
                name,
                description,
                input_schema,
            });
        }
        Ok(out)
    }

    /// Вызов инструмента (MCP tools/call). Параметры: name, arguments.
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": arguments
        });
        let result = self.call("tools/call", params).await?;
        Ok(result)
    }
}
