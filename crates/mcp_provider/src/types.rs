//! Типы MCP: конфиг, контекст.

use serde::{Deserialize, Serialize};

/// Конфиг одного MCP-сервера (формат как в Cursor).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpServerConfig {
    pub url: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
}

/// Конфиг mcp.json: mcpServers — имя сервера -> конфиг.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpConfig {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: std::collections::HashMap<String, McpServerConfig>,
}

/// Один фрагмент контекста от MCP (сервер + текст).
#[derive(Debug, Clone, Serialize)]
pub struct McpContextChunk {
    pub server: String,
    pub content: String,
}

/// Дескриптор инструмента MCP (из tools/list).
#[derive(Debug, Clone, Serialize)]
pub struct McpToolDescriptor {
    pub server: String,
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

impl McpToolDescriptor {
    /// Имя для вызова агентом: mcp::server::tool.
    pub fn namespaced_name(&self) -> String {
        format!("mcp::{}::{}", self.server, self.name)
    }
}
