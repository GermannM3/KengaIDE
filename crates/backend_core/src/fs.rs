//! Файловая система: чтение, запись, обход дерева.

use std::path::PathBuf;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum FsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path not found: {0}")]
    NotFound(PathBuf),
}

/// Сервис работы с файловой системой.
/// UI не обращается напрямую — только через Backend.
pub struct FsService;

impl FsService {
    pub fn new() -> Self {
        Self
    }

    /// Читает содержимое файла.
    pub fn read_file(&self, path: &std::path::Path) -> Result<String, FsError> {
        std::fs::read_to_string(path).map_err(Into::into)
    }

    /// Записывает содержимое в файл.
    pub fn write_file(&self, path: &std::path::Path, content: &str) -> Result<(), FsError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content).map_err(Into::into)
    }

    /// Возвращает дерево файлов проекта (относительные пути).
    pub fn project_tree(&self, root: &std::path::Path, max_depth: usize) -> Vec<PathBuf> {
        WalkDir::new(root)
            .max_depth(max_depth)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().strip_prefix(root).unwrap_or(e.path()).to_path_buf())
            .collect()
    }
}

impl Default for FsService {
    fn default() -> Self {
        Self::new()
    }
}
