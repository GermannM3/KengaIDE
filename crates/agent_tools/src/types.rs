//! Типы вызова инструмента и результата.

use serde::{Deserialize, Serialize};

/// Вызов инструмента (имя + аргументы в виде JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Результат выполнения инструмента.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
        }
    }

    pub fn err(output: impl Into<String>) -> Self {
        Self {
            success: false,
            output: output.into(),
        }
    }
}

/// Структурированные ошибки apply_patch. Агент видит код и может повторить с исправленным контекстом.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum PatchError {
    /// Блок `before` не найден в файле (0 вхождений).
    BeforeBlockNotFound { path: String, detail: String },
    /// Блок `before` встречается больше одного раза — патч неоднозначен.
    AmbiguousPatch { path: String, count: u32 },
    /// Файл не найден.
    FileNotFound { path: String },
    /// Ошибка ввода-вывода при чтении/записи.
    IoError { path: String, detail: String },
}

impl PatchError {
    /// Сообщение для агента (в output).
    pub fn message_for_agent(&self) -> String {
        match self {
            PatchError::BeforeBlockNotFound { path, detail } => {
                format!("BeforeBlockNotFound: path={} detail={}", path, detail)
            }
            PatchError::AmbiguousPatch { path, count } => {
                format!("AmbiguousPatch: path={} before_occurrences={}", path, count)
            }
            PatchError::FileNotFound { path } => format!("FileNotFound: path={}", path),
            PatchError::IoError { path, detail } => format!("IOError: path={} detail={}", path, detail),
        }
    }
}
