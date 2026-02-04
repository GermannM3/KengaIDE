//! Маршрутизация команд от UI к соответствующим обработчикам.
//!
//! Backend не генерирует текст — только валидирует и передаёт в AI Runtime.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Типы запросов от UI к AI Runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AiRequest {
    Chat { message: String },
    Explain { path: String, selection: Option<String> },
    Refactor { path: String, selection: String, instruction: String },
    Generate { path: String, prompt: String },
    /// Агент: план + вызов инструментов (create_file, read_file, list_files, update_file).
    Agent { message: String },
}

/// Ответ AI (сырой текст, UI форматирует сам).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub content: String,
    pub done: bool,
}

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("No project opened")]
    NoProject,
}

/// Маршрутизатор команд.
/// Валидирует запросы, добавляет контекст, передаёт в AI Runtime.
pub struct CommandRouter;

impl CommandRouter {
    pub fn new() -> Self {
        Self
    }

    /// Валидирует запрос перед передачей в AI Runtime.
    pub fn validate(&self, request: &AiRequest) -> Result<(), RouterError> {
        match request {
            AiRequest::Chat { message } => {
                if message.trim().is_empty() {
                    return Err(RouterError::Validation("Message cannot be empty".into()));
                }
            }
            AiRequest::Explain { path, .. } => {
                if path.trim().is_empty() {
                    return Err(RouterError::Validation("Path cannot be empty".into()));
                }
            }
            AiRequest::Refactor { path, selection, .. } => {
                if path.trim().is_empty() || selection.trim().is_empty() {
                    return Err(RouterError::Validation("Path and selection required".into()));
                }
            }
            AiRequest::Generate { path, prompt } => {
                if path.trim().is_empty() || prompt.trim().is_empty() {
                    return Err(RouterError::Validation("Path and prompt required".into()));
                }
            }
            AiRequest::Agent { message } => {
                if message.trim().is_empty() {
                    return Err(RouterError::Validation("Agent message cannot be empty".into()));
                }
            }
        }
        Ok(())
    }
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new()
    }
}
