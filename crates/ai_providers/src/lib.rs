//! AI Providers — единый интерфейс для локальных и API-провайдеров.
//!
//! Provider не знает контекст IDE — только получает готовый prompt и генерирует ответ.

mod api_provider;
mod traits;

pub use api_provider::ApiProvider;
pub use traits::{
    AiChunk, AiChunkStream, AiMode, AiProvider, AiResponse, EditorContext, GenerateOptions,
    GenerateRequest, ProviderCapabilities, ProviderError, ProviderType,
};
