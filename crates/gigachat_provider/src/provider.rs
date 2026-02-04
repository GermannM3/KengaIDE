//! GigaChat Provider — impl AiProvider (cloud, streaming).

use std::sync::Arc;

use ai_providers::{
    AiChunk, AiChunkStream, AiMode, AiProvider, GenerateOptions, GenerateRequest,
    ProviderCapabilities, ProviderError, ProviderType,
};
use async_trait::async_trait;

use crate::auth::AuthManager;
use crate::client::GigaChatClient;
use crate::error::GigaChatError;
use crate::models::GigaChatModel;

const SYSTEM_PROMPT: &str = "You are a helpful coding assistant. Respond concisely and accurately.";

pub struct GigaChatProvider {
    client: Arc<GigaChatClient>,
    model_name: String,
}

impl GigaChatProvider {
    pub fn new(client_id: String, client_secret: String) -> Result<Self, GigaChatError> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| GigaChatError::Http(e.to_string()))?;

        let auth = AuthManager::new(client_id, client_secret, http_client.clone());
        let client = GigaChatClient::new(auth, http_client);

        Ok(Self {
            client: Arc::new(client),
            model_name: GigaChatModel::GigaChatUltra.as_str().to_string(),
        })
    }
}

#[async_trait]
impl AiProvider for GigaChatProvider {
    fn id(&self) -> &str {
        "cloud-gigachat"
    }

    fn name(&self) -> &str {
        "GigaChat Ultra"
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Cloud
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            modes: std::collections::HashSet::from([
                AiMode::Chat,
                AiMode::Explain,
                AiMode::Refactor,
                AiMode::Generate,
                AiMode::Agent,
            ]),
            max_context_tokens: Some(128_000),
        }
    }

    async fn generate(
        &self,
        request: GenerateRequest,
        _options: GenerateOptions,
    ) -> Result<AiChunkStream, ProviderError> {
        let client = Arc::clone(&self.client);
        let prompt = request.prompt.clone();

        let s = async_stream::stream! {
            yield AiChunk::Start;
            match client.chat(SYSTEM_PROMPT, &prompt).await {
                Ok((content, _tokens_used)) => {
                    if !content.is_empty() {
                        yield AiChunk::Token { value: content };
                    }
                    yield AiChunk::End;
                }
                Err(e) => {
                    yield AiChunk::Error {
                        error: e.to_string(),
                    };
                }
            }
        };
        Ok(Box::pin(s))
    }

    fn cancel(&self, _request_id: &str) {
        // Cloud: отмена через прерывание HTTP-запроса при необходимости; заглушка.
    }

    async fn is_available(&self) -> Result<bool, ProviderError> {
        self.client
            .healthcheck()
            .await
            .map_err(|e| ProviderError::Unavailable(e.to_string()))
    }
}
