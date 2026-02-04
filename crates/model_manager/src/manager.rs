//! Управление моделями: список, загрузка, проверка ресурсов.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelManagerError {
    #[error("Model not found: {0}")]
    NotFound(String),
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub required_ram_gb: u32,
    pub supports_gpu: bool,
}

/// Model Manager. Заглушка: реальная логика — загрузка, кэш, проверка RAM/GPU.
pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new(models_dir: PathBuf) -> Self {
        Self { models_dir }
    }

    /// Список доступных (загруженных) моделей.
    pub fn list_available(&self) -> Vec<ModelInfo> {
        // Заглушка: пустой список.
        let _ = &self.models_dir;
        Vec::new()
    }

    /// Проверка, загружена ли модель.
    pub fn is_loaded(&self, model_id: &str) -> bool {
        let _ = model_id;
        false
    }

    /// Запуск загрузки модели. Заглушка.
    pub async fn download(&self, _model_id: &str) -> Result<(), ModelManagerError> {
        Err(ModelManagerError::DownloadFailed(
            "Download not implemented".into(),
        ))
    }

    /// Путь к директории моделей.
    pub fn models_dir(&self) -> &std::path::Path {
        &self.models_dir
    }
}
