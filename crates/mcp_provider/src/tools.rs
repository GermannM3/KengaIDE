//! MCP Tool Registry: discovery (tools/list) с кэшем TTL 5 мин, вызов (tools/call) по имени mcp::server::tool.

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::client::McpClient;
use crate::config::load_config;
use crate::error::McpError;
use crate::types::{McpConfig, McpToolDescriptor};

const CACHE_TTL_SECS: u64 = 300;

struct CachedEntry {
    at: Instant,
    tools: Vec<McpToolDescriptor>,
}

/// Реестр MCP-инструментов: агрегирует tools по всем серверам, кэш TTL 5 мин.
pub struct McpToolRegistry {
    servers: Vec<(String, McpClient)>,
    cache: RwLock<HashMap<String, CachedEntry>>,
}

impl McpToolRegistry {
    pub fn from_config_file() -> Result<Self, McpError> {
        let config = load_config()?;
        Self::from_config(config)
    }

    pub fn from_config(config: McpConfig) -> Result<Self, McpError> {
        let servers = config
            .mcp_servers
            .into_iter()
            .map(|(key, server_config)| {
                let name = if key.contains('\\') || key.contains('/') || key.ends_with(".json") {
                    std::path::Path::new(&key)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("default")
                        .to_string()
                } else {
                    key
                };
                let client = McpClient::new(&server_config);
                (name, client)
            })
            .collect();
        Ok(Self {
            servers,
            cache: RwLock::new(HashMap::new()),
        })
    }

    /// Все инструменты всех серверов с именами mcp::server::tool. Кэш TTL 5 мин.
    pub async fn list_all_tools(&self) -> Vec<McpToolDescriptor> {
        let mut out = Vec::new();
        let ttl = Duration::from_secs(CACHE_TTL_SECS);
        for (server_name, client) in &self.servers {
            let need_refresh = {
                let guard = self.cache.read().ok();
                guard
                    .as_ref()
                    .and_then(|g| g.get(server_name))
                    .map(|e| e.at.elapsed() > ttl)
                    .unwrap_or(true)
            };
            if need_refresh {
                match client.list_tools(server_name).await {
                    Ok(tools) => {
                        tracing::debug!(server = %server_name, count = tools.len(), "MCP tool discovery");
                        if let Ok(mut w) = self.cache.write() {
                            w.insert(
                                server_name.clone(),
                                CachedEntry {
                                    at: Instant::now(),
                                    tools: tools.clone(),
                                },
                            );
                        }
                        out.extend(tools);
                    }
                    Err(e) => {
                        tracing::debug!(server = %server_name, error = %e, "MCP list_tools failed");
                        if let Ok(guard) = self.cache.read() {
                            if let Some(entry) = guard.get(server_name) {
                                out.extend(entry.tools.clone());
                            }
                        }
                    }
                }
            } else if let Ok(guard) = self.cache.read() {
                if let Some(entry) = guard.get(server_name) {
                    out.extend(entry.tools.clone());
                }
            }
        }
        out
    }

    /// Вызов инструмента по имени mcp::server::tool. Возвращает результат или ошибку (для tool_result).
    pub async fn call_tool(
        &self,
        namespaced_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let (server_name, tool_name) = parse_mcp_tool_name(namespaced_name)
            .ok_or_else(|| McpError::JsonRpc(format!("invalid MCP tool name: {}", namespaced_name)))?;
        let looks_like_path = server_name.contains('\\') || server_name.contains('/') || server_name.ends_with(".json");
        let client = self
            .servers
            .iter()
            .find(|(name, _)| name == server_name)
            .map(|(_, c)| c)
            .ok_or_else(|| {
                let msg = if looks_like_path {
                    format!(
                        "MCP server not found. In mcp.json the key in mcpServers must be a short name (e.g. \"context7\"), not a file path. Example: \"mcpServers\": {{ \"myserver\": {{ \"url\": \"http://...\" }} }}. Got key: {}",
                        server_name
                    )
                } else {
                    format!("MCP server not found: {}. Check that the server name in mcp.json mcpServers matches the name in the tool call.", server_name)
                };
                McpError::JsonRpc(msg)
            })?;
        tracing::debug!(server = %server_name, tool = %tool_name, "MCP tool call");
        let result = client.call_tool(tool_name, arguments).await;
        if let Err(ref e) = result {
            tracing::debug!(server = %server_name, tool = %tool_name, error = %e, "MCP tool call failed");
        }
        result
    }

    /// Есть ли хотя бы один сервер.
    pub fn has_servers(&self) -> bool {
        !self.servers.is_empty()
    }

    /// Клиенты по имени сервера (для вызова).
    pub fn get_client(&self, server_name: &str) -> Option<&McpClient> {
        self.servers
            .iter()
            .find(|(name, _)| name == server_name)
            .map(|(_, c)| c)
    }
}

/// Парсит "mcp::server::tool" в ("server", "tool").
pub fn parse_mcp_tool_name(name: &str) -> Option<(&str, &str)> {
    let prefix = "mcp::";
    let name = name.strip_prefix(prefix)?.trim();
    let mut parts = name.splitn(2, "::");
    let server = parts.next()?;
    let tool = parts.next()?;
    if server.is_empty() || tool.is_empty() {
        return None;
    }
    Some((server, tool))
}

#[cfg(test)]
mod tests {
    use super::{parse_mcp_tool_name, McpToolRegistry};
    use crate::types::McpConfig;

    #[test]
    fn test_parse_mcp_tool_name() {
        assert_eq!(
            parse_mcp_tool_name("mcp::context7::query-docs"),
            Some(("context7", "query-docs"))
        );
        assert_eq!(
            parse_mcp_tool_name("mcp::context7::resolve-library-id"),
            Some(("context7", "resolve-library-id"))
        );
        assert_eq!(parse_mcp_tool_name("create_file"), None);
        assert_eq!(parse_mcp_tool_name("mcp::context7"), None);
        assert_eq!(parse_mcp_tool_name("mcp::::tool"), None);
    }

    #[tokio::test]
    async fn test_mcp_registry_empty_config() {
        let config = McpConfig::default();
        let reg = McpToolRegistry::from_config(config).expect("default config is valid");
        assert!(!reg.has_servers());
        let tools = reg.list_all_tools().await;
        assert!(tools.is_empty());
    }
}
