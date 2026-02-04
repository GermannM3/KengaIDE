//! Обработка ошибок AI Runtime.
//!
//! Все ошибки проходят через thiserror. Без unwrap, без panic.

use ai_providers::ProviderError;
use thiserror::Error;

/// Ошибки AI Runtime.
#[derive(Error, Debug)]
pub enum AiRuntimeError {
    #[error("no provider available for this mode")]
    NoProvider,

    #[error("provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("context error: {0}")]
    Context(#[from] context_manager::ContextError),

    #[error("prompt build failed: {0}")]
    PromptBuild(String),

    #[error("response post-processing failed: {0}")]
    PostProcess(String),
}
