//! Состояние приложения: Backend, AI Runtime, AiController, провайдеры.

use ai_providers::ApiProvider;
use ai_runtime::{ensure_model_roles_config, AiController, AiRuntime};
use backend_core::{CommandRouter, FsService, ProjectService};
use gigachat_provider::GigaChatProvider;
#[cfg(feature = "local")]
use local_provider::{LocalConfig, LocalProvider, ModelVariant};
use model_manager::ModelManager;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ai_config::load_config;

pub struct AppState {
    pub fs: Arc<FsService>,
    pub project: Arc<RwLock<ProjectService>>,
    pub router: Arc<CommandRouter>,
    pub ai_runtime: Arc<RwLock<AiRuntime>>,
    pub ai_controller: Arc<AiController>,
    #[cfg(feature = "local")]
    pub local_providers: Vec<Arc<LocalProvider>>,
}

impl Default for AppState {
    fn default() -> Self {
        let _ = ensure_model_roles_config();
        let models_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("kengaide")
            .join("models");
        std::fs::create_dir_all(&models_dir).ok();

        let model_manager = Arc::new(ModelManager::new(models_dir));
        let fs_read: Arc<dyn Fn(&Path) -> Result<String, std::io::Error> + Send + Sync> =
            Arc::new(|p| std::fs::read_to_string(p));

        let mut ai_runtime = AiRuntime::new(model_manager, fs_read);

        #[cfg(feature = "local")]
        let local_providers = {
            let mut gigachat_config = LocalConfig::default_config();
            gigachat_config.model_variant = ModelVariant::GigaChat;
            let gigachat = Arc::new(LocalProvider::new(gigachat_config));
            ai_runtime.add_provider(gigachat.clone());

            let mut deepseek_config = LocalConfig::default_config();
            deepseek_config.model_variant = ModelVariant::DeepSeekCoder;
            let deepseek = Arc::new(LocalProvider::new(deepseek_config));
            ai_runtime.add_provider(deepseek.clone());

            vec![gigachat, deepseek]
        };

        if let (Some(client_id), Some(client_secret)) = (
            std::env::var("KENGACHAT_CLIENT_ID").ok(),
            std::env::var("KENGACHAT_CLIENT_SECRET").ok(),
        ) {
            if let Ok(provider) = GigaChatProvider::new(client_id, client_secret) {
                ai_runtime.add_provider(Arc::new(provider));
            }
        }

        ai_runtime.add_provider(Arc::new(ApiProvider::openai(None)));

        let ai_config = load_config();
        let mut added_openai = false;
        for entry in &ai_config.providers {
            if entry.provider_type == "openai" && !added_openai {
                if let Some(key) = &entry.api_key {
                    if !key.is_empty() {
                        ai_runtime.add_provider(Arc::new(ApiProvider::openai(Some(key.clone()))));
                        added_openai = true;
                    }
                }
            }
        }
        if let Some(ref id) = ai_config.active_provider_id {
            ai_runtime.set_preferred_provider(Some(id.clone()));
        }

        let ai_runtime = Arc::new(RwLock::new(ai_runtime));
        let ai_controller = Arc::new(AiController::new(Arc::clone(&ai_runtime)));

        Self {
            fs: Arc::new(FsService::new()),
            project: Arc::new(RwLock::new(ProjectService::new())),
            router: Arc::new(CommandRouter::new()),
            ai_runtime,
            ai_controller,
            #[cfg(feature = "local")]
            local_providers,
        }
    }
}
