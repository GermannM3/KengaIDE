//! ApiProvider — внешние API (OpenAI, Claude, etc).
//!
//! Streaming через SSE / chunked responses. Заглушка: реальная интеграция — HTTP-клиент + конфиг ключей.

use super::traits::{
    AiChunk, AiChunkStream, AiMode, AiProvider, GenerateOptions, GenerateRequest,
    ProviderCapabilities, ProviderError, ProviderType,
};
use async_trait::async_trait;
use futures_util::stream;

pub struct ApiProvider {
    name: String,
    api_key: Option<String>,
}

impl ApiProvider {
    pub fn new(name: impl Into<String>, api_key: Option<String>) -> Self {
        Self {
            name: name.into(),
            api_key,
        }
    }

    pub fn openai(api_key: Option<String>) -> Self {
        Self::new("OpenAI", api_key)
    }

    pub fn claude(api_key: Option<String>) -> Self {
        Self::new("Claude", api_key)
    }
}

#[async_trait]
impl AiProvider for ApiProvider {
    fn id(&self) -> &str {
        "cloud-openai"
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Cloud
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            modes: [
                AiMode::Chat,
                AiMode::Explain,
                AiMode::Refactor,
                AiMode::Generate,
                AiMode::Agent,
            ]
            .into_iter()
            .collect(),
            max_context_tokens: Some(200_000),
        }
    }

    async fn generate(
        &self,
        _request: GenerateRequest,
        _options: GenerateOptions,
    ) -> Result<AiChunkStream, ProviderError> {
        if self.api_key.is_none() {
            return Err(ProviderError::Unavailable("API key not configured".into()));
        }
        let stream = stream::iter([
            AiChunk::Start,
            AiChunk::Error {
                error: "API provider not yet implemented".to_string(),
            },
        ]);
        Ok(Box::pin(stream))
    }

    fn cancel(&self, _request_id: &str) {
        // Cloud: отмена через AbortSignal в HTTP-запросе; заглушка.
    }

    async fn is_available(&self) -> Result<bool, ProviderError> {
        Ok(self.api_key.is_some())
    }
}
