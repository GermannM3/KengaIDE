//! MCP Context Provider: получение контекста по запросу для подмешивания в agent prompt.

use crate::client::McpClient;
use crate::config::load_config;
use crate::error::McpError;
use crate::types::{McpConfig, McpContextChunk};

/// Провайдер контекста от MCP-серверов. Если конфига нет или серверы недоступны — тихо возвращает пустой список.
pub struct McpContextProvider {
    servers: Vec<(String, McpClient)>,
}

impl McpContextProvider {
    /// Создаёт провайдер из конфига ~/.kengaide/mcp.json. Если файла нет — пустой провайдер.
    pub fn from_config_file() -> Result<Self, McpError> {
        let config = load_config()?;
        Self::from_config(config)
    }

    /// Создаёт провайдер из готового конфига.
    pub fn from_config(config: McpConfig) -> Result<Self, McpError> {
        let servers = config
            .mcp_servers
            .into_iter()
            .map(|(name, server_config)| {
                let client = McpClient::new(&server_config);
                (name, client)
            })
            .collect();
        Ok(Self { servers })
    }

    /// Запрашивает контекст у всех серверов по query. Ошибки логируются, недоступные серверы пропускаются.
    /// Конвенция: метод "context/query" с params {"query": query}. Ответ: result — строка или объект с полем content.
    pub async fn fetch_context(&self, query: &str) -> Vec<McpContextChunk> {
        let mut out = Vec::new();
        let params = serde_json::json!({ "query": query });
        for (server_name, client) in &self.servers {
            match client.call("context/query", params.clone()).await {
                Ok(result) => {
                    let content = extract_content(&result);
                    if !content.is_empty() {
                        out.push(McpContextChunk {
                            server: server_name.clone(),
                            content,
                        });
                    }
                }
                Err(e) => {
                    tracing::debug!(server = %server_name, error = %e, "MCP context fetch skipped");
                }
            }
        }
        out
    }
}

/// Извлекает текст из result: если строка — вернуть; если объект с полем "content" — вернуть content; иначе пусто.
fn extract_content(result: &serde_json::Value) -> String {
    match result {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(m) => m
            .get("content")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default(),
        _ => String::new(),
    }
}
