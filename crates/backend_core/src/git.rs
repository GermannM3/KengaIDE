//! Git: статус, diff, базовая интеграция.

use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("Not a git repository")]
    NotRepo,
}

/// Сервис работы с Git.
pub struct GitService;

impl GitService {
    pub fn new() -> Self {
        Self
    }

    /// Проверяет, является ли путь git-репозиторием.
    pub fn is_repo(&self, path: &Path) -> bool {
        path.join(".git").exists()
    }

    /// Возвращает статус файла (modified, untracked, etc).
    pub fn file_status(&self, repo_path: &Path, file_path: &Path) -> Result<Option<String>, GitError> {
        let repo = git2::Repository::open(repo_path).map_err(|_| GitError::NotRepo)?;
        let statuses = repo.statuses(None)?;
        let path_str = file_path.to_string_lossy();
        for entry in statuses.iter() {
            if entry.path().map(|p| p == path_str.as_ref()) == Some(true) {
                let status = format!("{:?}", entry.status());
                return Ok(Some(status));
            }
        }
        Ok(None)
    }
}

impl Default for GitService {
    fn default() -> Self {
        Self::new()
    }
}
