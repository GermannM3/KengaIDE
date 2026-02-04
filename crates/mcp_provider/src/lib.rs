//! MCP (Model Context Protocol) — источник контекста и инструментов для AI-агента.
//!
//! Конфиг: ~/.kengaide/mcp.json (формат как в Cursor).
//! MCP не заменяет LLM — усиливает контекст перед генерацией.

mod client;
mod config;
mod context;
mod error;
mod tools;
mod types;

pub use config::{config_path, load_config};
pub use context::McpContextProvider;
pub use error::McpError;
pub use tools::{parse_mcp_tool_name, McpToolRegistry};
pub use types::{McpConfig, McpContextChunk, McpServerConfig, McpToolDescriptor};
