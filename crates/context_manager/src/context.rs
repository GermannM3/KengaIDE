//! Контекст запроса: файлы, выделение, дерево.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("Context limit exceeded: {0}")]
    LimitExceeded(String),
}

/// Лимиты контекста (в символах, грубая оценка токенов ~4 chars/token).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLimits {
    pub max_chars: usize,
    pub max_files: usize,
}

impl Default for ContextLimits {
    fn default() -> Self {
        Self {
            max_chars: 100_000, // ~25k tokens
            max_files: 50,
        }
    }
}

/// Собранный контекст для AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// Текущий файл (путь + содержимое).
    pub current_file: Option<FileContext>,
    /// Выделенный фрагмент.
    pub selection: Option<String>,
    /// Дерево проекта (список путей).
    pub project_tree: Vec<PathBuf>,
    /// Дополнительные файлы по запросу.
    pub extra_files: Vec<FileContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    pub path: PathBuf,
    pub content: String,
}

/// Построитель контекста с учётом лимитов.
pub struct ContextBuilder {
    limits: ContextLimits,
    current_file: Option<FileContext>,
    selection: Option<String>,
    project_tree: Vec<PathBuf>,
    extra_files: Vec<FileContext>,
}

impl ContextBuilder {
    pub fn new(limits: ContextLimits) -> Self {
        Self {
            limits,
            current_file: None,
            selection: None,
            project_tree: Vec::new(),
            extra_files: Vec::new(),
        }
    }

    pub fn current_file(mut self, path: PathBuf, content: String) -> Self {
        self.current_file = Some(FileContext { path, content });
        self
    }

    pub fn selection(mut self, text: String) -> Self {
        self.selection = Some(text);
        self
    }

    pub fn project_tree(mut self, paths: Vec<PathBuf>) -> Self {
        self.project_tree = paths.into_iter().take(self.limits.max_files).collect();
        self
    }

    pub fn add_file(mut self, path: PathBuf, content: String) -> Self {
        if self.extra_files.len() < self.limits.max_files {
            self.extra_files.push(FileContext { path, content });
        }
        self
    }

    /// Собирает контекст и проверяет лимиты.
    pub fn build(self) -> Result<Context, ContextError> {
        let total_chars: usize = self
            .current_file
            .as_ref()
            .map(|f| f.content.len())
            .unwrap_or(0)
            + self.selection.as_ref().map(|s| s.len()).unwrap_or(0)
            + self.extra_files.iter().map(|f| f.content.len()).sum::<usize>();

        if total_chars > self.limits.max_chars {
            return Err(ContextError::LimitExceeded(format!(
                "{} > {}",
                total_chars, self.limits.max_chars
            )));
        }

        Ok(Context {
            current_file: self.current_file,
            selection: self.selection,
            project_tree: self.project_tree,
            extra_files: self.extra_files,
        })
    }
}
