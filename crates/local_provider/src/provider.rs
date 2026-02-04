//! LocalProvider — impl AiProvider для offline GigaChat3 (GGUF, llama.cpp).
//!
//! Streaming token-by-token; отмена через cancel(request_id).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use ai_providers::{
    AiChunk, AiChunkStream, AiMode, AiProvider, GenerateOptions, GenerateRequest,
    ProviderCapabilities, ProviderError, ProviderType,
};
use async_trait::async_trait;

use crate::config::LocalConfig;
use crate::error::LocalProviderError;
use crate::inference::InferenceEngine;
use crate::model_manager::{DownloadProgress, ModelManager};

const CHUNK_CHANNEL_CAP: usize = 64;

pub struct LocalProvider {
    config: LocalConfig,
    model_manager: ModelManager,
    engine: RwLock<Option<Arc<InferenceEngine>>>,
    /// request_id → флаг отмены (проверяется в inference loop). Arc для передачи в cancel() без блокировки.
    active_requests: Arc<RwLock<HashMap<String, Arc<AtomicBool>>>>,
}

impl LocalProvider {
    pub fn new(config: LocalConfig) -> Self {
        let model_manager = ModelManager::new(config.clone());
        Self {
            config,
            model_manager,
            engine: RwLock::new(None),
            active_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_models_dir(path: PathBuf) -> Self {
        let mut config = LocalConfig::default_config();
        config.models_dir = path;
        Self::new(config)
    }

    async fn ensure_engine(&self) -> Result<Arc<InferenceEngine>, ProviderError> {
        {
            let guard = self.engine.read().await;
            if let Some(ref e) = *guard {
                return Ok(Arc::clone(e));
            }
        }

        let mut guard = self.engine.write().await;
        if let Some(ref e) = *guard {
            return Ok(Arc::clone(e));
        }

        let path = self
            .model_manager
            .find_gguf_path()
            .ok_or_else(|| ProviderError::Unavailable("Model not found".to_string()))?;

        self.model_manager
            .verify_integrity(&path)
            .map_err(|e| ProviderError::Unavailable(e.to_string()))?;

        let engine = InferenceEngine::load(&path, self.config.n_threads)
            .map_err(|e| ProviderError::Unavailable(e.to_string()))?;

        let engine = Arc::new(engine);
        *guard = Some(Arc::clone(&engine));
        Ok(engine)
    }

    pub async fn ensure_model<F>(&self, on_progress: F) -> Result<(), LocalProviderError>
    where
        F: FnMut(DownloadProgress) + Send,
    {
        if self.model_manager.is_loaded(self.config.model_variant.model_id()) {
            return Ok(());
        }
        self.model_manager
            .ensure_model_installed(on_progress)
            .await?;
        Ok(())
    }

    pub fn model_size_gb(&self) -> f64 {
        self.config.model_variant.hf_config().size_gb
    }
}

#[async_trait]
impl AiProvider for LocalProvider {
    fn id(&self) -> &str {
        self.config.model_variant.provider_id()
    }

    fn name(&self) -> &str {
        self.config.model_variant.display_name()
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Local
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
            max_context_tokens: Some(crate::config::DEFAULT_CONTEXT_SIZE),
        }
    }

    async fn generate(
        &self,
        request: GenerateRequest,
        options: GenerateOptions,
    ) -> Result<AiChunkStream, ProviderError> {
        let engine = self.ensure_engine().await?;

        let max_tokens = options
            .max_tokens
            .unwrap_or(crate::config::DEFAULT_MAX_TOKENS)
            .min(crate::config::DEFAULT_MAX_TOKENS);

        let cancel_flag = Arc::new(AtomicBool::new(false));
        {
            let mut guard = self.active_requests.write().await;
            guard.insert(request.id.clone(), Arc::clone(&cancel_flag));
        }

        let (tx, mut rx) = mpsc::channel::<AiChunk>(CHUNK_CHANNEL_CAP);
        let prompt = request.prompt.clone();
        let engine_clone = Arc::clone(&engine);
        let cancel_clone = Arc::clone(&cancel_flag);
        let request_id = request.id.clone();

        tokio::task::spawn_blocking(move || {
            let result = engine_clone.generate_stream(
                &prompt,
                max_tokens,
                cancel_clone.as_ref(),
                |piece| {
                    let _ = tx.blocking_send(AiChunk::Token {
                        value: piece.to_string(),
                    });
                },
            );
            if let Err(e) = result {
                let _ = tx.blocking_send(AiChunk::Error {
                    error: e.to_string(),
                });
            } else {
                let _ = tx.blocking_send(AiChunk::End);
            }
        });

        let request_id_guard = request_id.clone();
        let active_guard = Arc::clone(&self.active_requests);
        let stream = async_stream::stream! {
            yield AiChunk::Start;
            while let Some(chunk) = rx.recv().await {
                yield chunk.clone();
                if matches!(chunk, AiChunk::End | AiChunk::Error { .. }) {
                    break;
                }
            }
            let mut g = active_guard.write().await;
            g.remove(&request_id_guard);
        };

        Ok(Box::pin(stream))
    }

    fn cancel(&self, request_id: &str) {
        let active = Arc::clone(&self.active_requests);
        let rid = request_id.to_string();
        tokio::spawn(async move {
            let mut guard = active.write().await;
            if let Some(f) = guard.get(&rid) {
                f.store(true, Ordering::Relaxed);
            }
            guard.remove(&rid);
        });
    }

    async fn is_available(&self) -> Result<bool, ProviderError> {
        Ok(self.model_manager.is_loaded(self.config.model_variant.model_id()))
    }

    fn model_id(&self) -> Option<&str> {
        Some(self.config.model_variant.model_id())
    }
}
