//! Типизированные ошибки MCP.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("config: {0}")]
    Config(String),

    #[error("HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON-RPC error: {0}")]
    JsonRpc(String),

    #[error("timeout")]
    Timeout,

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
}
