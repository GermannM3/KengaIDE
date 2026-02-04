//! Offline LocalProvider для GigaChat3 (GGUF, llama.cpp).
//!
//! Модель: ai-sage/GigaChat3-702B-A36B-preview (HuggingFace, MIT).
//! По умолчанию — GigaChat3-10B-A1.8B для десктопа (~10 ГБ).

mod config;
mod error;
pub mod hardware_detect;
mod inference;
mod model_manager;
mod provider;
mod tokenizer;

pub use config::{LocalConfig, ModelVariant};
pub use error::LocalProviderError;
pub use model_manager::{DownloadProgress, ModelManager};
pub use provider::LocalProvider;
