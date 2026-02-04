//! Чтение конфига MCP из ~/.kengaide/mcp.json (Windows: %USERPROFILE%\.kengaide\mcp.json).

use std::path::PathBuf;

use crate::error::McpError;
use crate::types::McpConfig;

/// Путь к файлу конфига MCP.
pub fn config_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    #[cfg(windows)]
    let path = home.join(".kengaide").join("mcp.json");
    #[cfg(not(windows))]
    let path = home.join(".kengaide").join("mcp.json");
    Some(path)
}

/// Загружает конфиг MCP. Если файла нет — возвращает Ok(McpConfig с пустым mcp_servers).
pub fn load_config() -> Result<McpConfig, McpError> {
    let path = match config_path() {
        Some(p) => p,
        None => return Ok(McpConfig::default()),
    };
    if !path.exists() {
        return Ok(McpConfig::default());
    }
    let s = std::fs::read_to_string(&path)
        .map_err(|e| McpError::Config(format!("read {}: {}", path.display(), e)))?;
    let config: McpConfig = serde_json::from_str(&s)
        .map_err(|e| McpError::Config(format!("parse mcp.json: {}", e)))?;
    Ok(config)
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            mcp_servers: std::collections::HashMap::new(),
        }
    }
}
