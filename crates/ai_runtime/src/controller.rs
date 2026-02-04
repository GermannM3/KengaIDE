//! AiController — единый streaming-pipeline: запуск генерации, эмит чанков, отмена.
//!
//! UI не ждёт полного ответа; получает чанки по событиям. Отмена через cancel(request_id).

use ai_providers::{AiChunk, AiMode, AiProvider, EditorContext, GenerateOptions, GenerateRequest};
use backend_core::{append_log, command_router::AiRequest};
use context_manager::{Context, ContextBuilder, ContextLimits};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::AiRuntimeError;
use crate::orchestration::TaskRole;
use crate::prompt_builder::PromptBuilder;
use crate::provider_selector::ProviderSelector;
use crate::runtime::get_project_tree;

/// Результат run_stream: request_id + метаданные для UI.
#[derive(Debug, Clone)]
pub struct RunStreamResult {
    pub request_id: String,
    pub role: TaskRole,
    pub model_id: String,
}

/// Эмиттер чанков: (request_id, chunk) → UI (например через Tauri events).
pub type ChunkEmitter = Arc<dyn Fn(&str, &AiChunk) + Send + Sync>;

/// Контроллер: активные запросы, run_stream, cancel.
pub struct AiController {
    runtime: Arc<RwLock<crate::runtime::AiRuntime>>,
    /// request_id → провайдер, который обрабатывает этот запрос (для cancel).
    active_requests: Arc<RwLock<HashMap<String, Arc<dyn AiProvider>>>>,
}

impl AiController {
    pub fn new(runtime: Arc<RwLock<crate::runtime::AiRuntime>>) -> Self {
        Self {
            runtime,
            active_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Запускает streaming-генерацию: строит контекст и промпт, выбирает провайдера,
    /// вызывает provider.generate(), эмитит каждый чанк через emitter.
    /// Возвращает request_id сразу после старта; поток чанков идёт асинхронно.
    pub async fn run_stream(
        &self,
        request: AiRequest,
        project_root: Option<&Path>,
        current_file: Option<(std::path::PathBuf, String)>,
        selection: Option<&str>,
        options: GenerateOptions,
        emitter: ChunkEmitter,
    ) -> Result<RunStreamResult, AiRuntimeError> {
        let (mode, user_input) = Self::extract_mode_and_input(&request);
        let context_limits = {
            let guard = self.runtime.read().await;
            guard.context_limits().clone()
        };
        let context = self.build_context(
            project_root,
            current_file.as_ref(),
            selection,
            &context_limits,
        )?;
        let prompt = PromptBuilder::build(mode, &context, &user_input)?;

        let (provider, role, model_id) = {
            let guard = self.runtime.read().await;
            let sel = ProviderSelector::select(
                guard.providers(),
                mode,
                &user_input,
                guard.preferred_provider_id(),
                project_root,
            )
            .await?;
            let log_line = format!(
                "invoke mode={} role={} model={} policy={:?}",
                format!("{:?}", mode),
                sel.role.as_str(),
                sel.model_id,
                sel.policy_source
            );
            append_log(project_root, "runtime.log", &log_line);
            (sel.provider, sel.role, sel.model_id)
        };
        let request_id = Uuid::new_v4().to_string();

        let editor_ctx = Self::editor_context_from_request(&request, current_file.as_ref(), selection);
        let gen_request = GenerateRequest {
            id: request_id.clone(),
            prompt,
            context: Some(editor_ctx),
            mode,
        };

        let stream = provider
            .generate(gen_request, options)
            .await
            .map_err(Into::<AiRuntimeError>::into)?;

        {
            let mut guard = self.active_requests.write().await;
            guard.insert(request_id.clone(), Arc::clone(&provider));
        }

        let emitter_clone = Arc::clone(&emitter);
        let active = Arc::clone(&self.active_requests);
        let rid = request_id.clone();
        tokio::spawn(async move {
            let mut stream = stream;
            while let Some(chunk) = stream.next().await {
                emitter_clone(&rid, &chunk);
                if matches!(chunk, AiChunk::End | AiChunk::Error { .. }) {
                    break;
                }
            }
            let mut guard = active.write().await;
            guard.remove(&rid);
        });

        Ok(RunStreamResult {
            request_id,
            role,
            model_id,
        })
    }

    /// Отменяет генерацию по request_id: вызывает provider.cancel(), убирает из active_requests.
    pub async fn cancel(&self, request_id: &str) {
        let provider = {
            let mut guard = self.active_requests.write().await;
            guard.remove(request_id)
        };
        if let Some(p) = provider {
            p.cancel(request_id);
        }
    }

    fn extract_mode_and_input(request: &AiRequest) -> (AiMode, String) {
        match request {
            AiRequest::Chat { message } => (AiMode::Chat, message.clone()),
            AiRequest::Explain { selection: sel, .. } => {
                let input = sel.clone().unwrap_or_else(|| "Explain this code".to_string());
                (AiMode::Explain, input)
            }
            AiRequest::Refactor { instruction, .. } => (AiMode::Refactor, instruction.clone()),
            AiRequest::Generate { prompt, .. } => (AiMode::Generate, prompt.clone()),
            AiRequest::Agent { message } => (AiMode::Agent, message.clone()),
        }
    }

    fn editor_context_from_request(
        request: &AiRequest,
        current_file: Option<&(std::path::PathBuf, String)>,
        selection: Option<&str>,
    ) -> EditorContext {
        let (path, content) = match request {
            AiRequest::Chat { .. } | AiRequest::Agent { .. } => (None, None),
            AiRequest::Explain { path, .. } | AiRequest::Refactor { path, .. } | AiRequest::Generate { path, .. } => {
                current_file
                    .map(|(p, c)| (Some(p.display().to_string()), Some(c.clone())))
                    .unwrap_or((Some(path.clone()), None))
            }
        };
        EditorContext {
            path,
            content,
            selection: selection.map(String::from),
        }
    }

    fn build_context(
        &self,
        project_root: Option<&Path>,
        current_file: Option<&(std::path::PathBuf, String)>,
        selection: Option<&str>,
        context_limits: &ContextLimits,
    ) -> Result<Context, AiRuntimeError> {
        let mut builder = ContextBuilder::new(context_limits.clone());
        if let Some((path, content)) = current_file {
            builder = builder.current_file(path.clone(), content.clone());
        }
        if let Some(s) = selection {
            builder = builder.selection(s.to_string());
        }
        if let Some(root) = project_root {
            builder = builder.project_tree(get_project_tree(root));
        }
        builder.build().map_err(Into::into)
    }
}
