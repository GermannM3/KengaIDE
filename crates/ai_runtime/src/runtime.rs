//! AI Runtime: оркестрация провайдеров, контекста, промптов.

use ai_providers::{AiChunk, AiMode, AiProvider, EditorContext, GenerateOptions, GenerateRequest};
use backend_core::command_router::AiRequest;
use context_manager::{Context, ContextBuilder, ContextLimits};
use futures_util::StreamExt;
use model_manager::ModelManager;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

use ai_providers::AiResponse;

use crate::error::AiRuntimeError;
use crate::prompt_builder::PromptBuilder;
use crate::provider_selector::ProviderSelector;

/// AI Runtime. Управляет провайдерами, контекстом, вызывает generate.
pub struct AiRuntime {
    providers: Vec<Arc<dyn AiProvider>>,
    preferred_provider_id: Option<String>,
    #[allow(dead_code)]
    model_manager: Arc<ModelManager>,
    context_limits: ContextLimits,
    #[allow(dead_code)]
    fs_read: Arc<dyn Fn(&Path) -> Result<String, std::io::Error> + Send + Sync>,
}

/// Публичный доступ к дереву проекта (для контроллера).
pub fn get_project_tree(root: &Path) -> Vec<std::path::PathBuf> {
    walkdir::WalkDir::new(root)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.path().strip_prefix(root).ok().map(|p| p.to_path_buf()))
        .collect()
}

impl AiRuntime {
    pub fn new(
        model_manager: Arc<ModelManager>,
        fs_read: Arc<dyn Fn(&Path) -> Result<String, std::io::Error> + Send + Sync>,
    ) -> Self {
        Self {
            providers: Vec::new(),
            preferred_provider_id: None,
            model_manager,
            context_limits: ContextLimits::default(),
            fs_read,
        }
    }

    pub fn add_provider(&mut self, provider: Arc<dyn AiProvider>) {
        self.providers.push(provider);
    }

    pub fn set_preferred_provider(&mut self, id: Option<String>) {
        self.preferred_provider_id = id;
    }

    pub fn preferred_provider_id(&self) -> Option<&str> {
        self.preferred_provider_id.as_deref()
    }

    pub fn providers(&self) -> &[Arc<dyn AiProvider>] {
        &self.providers
    }

    pub fn context_limits(&self) -> &ContextLimits {
        &self.context_limits
    }

    /// Обрабатывает запрос от Backend (legacy: собирает stream в один ответ; для streaming используйте AiController).
    pub async fn handle_request(
        &self,
        request: AiRequest,
        project_root: Option<&Path>,
        current_file: Option<(std::path::PathBuf, String)>,
        selection: Option<&str>,
    ) -> Result<AiResponse, AiRuntimeError> {
        let (mode, user_input) = Self::extract_mode_and_input(&request);
        let editor_ctx = editor_context_from_request(&request, current_file.as_ref(), selection);
        let context = self.build_context(project_root, current_file, selection)?;
        let prompt = PromptBuilder::build(mode, &context, &user_input)?;

        let selection = ProviderSelector::select(
            &self.providers,
            mode,
            &user_input,
            self.preferred_provider_id.as_deref(),
            project_root,
        )
        .await?;
        let provider = selection.provider;
        let request_id = Uuid::new_v4().to_string();
        let gen_request = GenerateRequest {
            id: request_id,
            prompt,
            context: Some(editor_ctx),
            mode,
        };
        let options = GenerateOptions {
            temperature: None,
            max_tokens: None,
        };

        let mut stream = provider
            .generate(gen_request, options)
            .await
            .map_err(AiRuntimeError::from)?;
        let mut content = String::new();
        let mut done = false;
        while let Some(chunk) = stream.next().await {
            match chunk {
                AiChunk::Token { value } => content.push_str(&value),
                AiChunk::End => {
                    done = true;
                    break;
                }
                AiChunk::Error { error } => return Err(AiRuntimeError::Provider(ai_providers::ProviderError::Generation(error))),
                AiChunk::Start => {}
            }
        }
        if !done && content.is_empty() {
            content = "(no response)".to_string();
        }
        Ok(AiResponse {
            content,
            tokens_used: None,
            model: provider.name().to_string(),
            latency_ms: 0,
        })
    }

    fn extract_mode_and_input(request: &AiRequest) -> (AiMode, String) {
        match request {
            AiRequest::Chat { message } => (AiMode::Chat, message.clone()),
            AiRequest::Explain {
                selection: sel, ..
            } => {
                let input = sel
                    .clone()
                    .unwrap_or_else(|| "Explain this code".to_string());
                (AiMode::Explain, input)
            }
            AiRequest::Refactor { instruction, .. } => (AiMode::Refactor, instruction.clone()),
            AiRequest::Generate { prompt, .. } => (AiMode::Generate, prompt.clone()),
            AiRequest::Agent { message } => (AiMode::Agent, message.clone()),
        }
    }

    fn build_context(
        &self,
        project_root: Option<&Path>,
        current_file: Option<(std::path::PathBuf, String)>,
        selection: Option<&str>,
    ) -> Result<Context, AiRuntimeError> {
        let mut builder = ContextBuilder::new(self.context_limits.clone());

        if let Some((path, content)) = current_file {
            builder = builder.current_file(path, content);
        }
        if let Some(s) = selection {
            builder = builder.selection(s.to_string());
        }
        if let Some(root) = project_root {
            let tree = get_project_tree(root);
            builder = builder.project_tree(tree);
        }

        builder.build().map_err(Into::into)
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
