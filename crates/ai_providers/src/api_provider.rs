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
    id: String,
    name: String,
    api_key: Option<String>,
    #[allow(dead_code)]
    base_url: Option<String>,
}

impl ApiProvider {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        api_key: Option<String>,
        base_url: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            api_key,
            base_url,
        }
    }

    pub fn openai(api_key: Option<String>) -> Self {
        Self::new(
            "cloud-openai",
            "OpenAI",
            api_key,
            Some("https://api.openai.com/v1".to_string()),
        )
    }

    pub fn openai_with_id(id: impl Into<String>, api_key: Option<String>) -> Self {
        Self::new(
            id,
            "OpenAI",
            api_key,
            Some("https://api.openai.com/v1".to_string()),
        )
    }

    pub fn kimi(api_key: Option<String>) -> Self {
        Self::new(
            "cloud-kimi",
            "Kimi (Moonshot)",
            api_key,
            Some("https://api.moonshot.cn/v1".to_string()),
        )
    }

    pub fn kimi_with_id(id: impl Into<String>, api_key: Option<String>) -> Self {
        Self::new(id, "Kimi (Moonshot)", api_key, Some("https://api.moonshot.cn/v1".to_string()))
    }

    pub fn mistral(api_key: Option<String>) -> Self {
        Self::new(
            "cloud-mistral",
            "Mistral AI",
            api_key,
            Some("https://api.mistral.ai/v1".to_string()),
        )
    }

    pub fn mistral_with_id(id: impl Into<String>, api_key: Option<String>) -> Self {
        Self::new(id, "Mistral AI", api_key, Some("https://api.mistral.ai/v1".to_string()))
    }

    pub fn custom(id: impl Into<String>, name: impl Into<String>, api_key: String, base_url: String) -> Self {
        Self::new(id, name, Some(api_key), Some(base_url))
    }
}

#[async_trait]
impl AiProvider for ApiProvider {
    fn id(&self) -> &str {
        &self.id
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
