//! Управление проектами: открытие, текущий путь, метаданные.

use std::path::PathBuf;
use thiserror::Error;

use crate::workspace::ensure_workspace_dir;

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("Project not opened")]
    NotOpened,
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Состояние открытого проекта.
#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub name: String,
}

/// Сервис управления проектами.
pub struct ProjectService {
    current: Option<Project>,
}

impl ProjectService {
    pub fn new() -> Self {
        Self { current: None }
    }

    /// Открывает проект по пути.
    /// Создаёт/проверяет `.kengaide/` и config.json при открытии.
    pub fn open(&mut self, root: PathBuf) -> Result<&Project, ProjectError> {
        if !root.exists() || !root.is_dir() {
            return Err(ProjectError::InvalidPath(root.to_string_lossy().into()));
        }
        ensure_workspace_dir(&root).map_err(|e| {
            ProjectError::InvalidPath(format!("Workspace init failed: {}", e))
        })?;
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string();
        self.current = Some(Project { root, name });
        Ok(self.current.as_ref().unwrap())
    }

    /// Закрывает текущий проект.
    pub fn close(&mut self) {
        self.current = None;
    }

    /// Возвращает текущий проект.
    pub fn current(&self) -> Option<&Project> {
        self.current.as_ref()
    }
}

impl Default for ProjectService {
    fn default() -> Self {
        Self::new()
    }
}
