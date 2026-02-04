//! Ошибки LocalProvider.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LocalProviderError {
    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("model integrity check failed: {0}")]
    IntegrityCheckFailed(String),

    #[error("download failed: {0}")]
    DownloadFailed(String),

    #[error("inference failed: {0}")]
    InferenceFailed(String),

    #[error("model load failed: {0}")]
    ModelLoadFailed(String),

    #[error("hardware detection failed: {0}")]
    HardwareDetectionFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
