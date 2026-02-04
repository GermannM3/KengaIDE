//! Конфигурация: GigaChat3 из HuggingFace, лимиты, директории.

use std::path::PathBuf;

/// Вариант модели: 10B для десктопа (~10 ГБ) или 702B для high-end (~170+ ГБ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModelVariant {
    /// GigaChat3-10B-A1.8B — десктоп, ~10 ГБ, универсальная RU.
    #[default]
    GigaChat,
    /// DeepSeek-Coder 6.7B Instruct — coding-first, ~4 ГБ.
    DeepSeekCoder,
    /// SmolLM2-1.7B-Instruct — лёгкая, ~1.2 ГБ, быстрая.
    SmolLM2,
    /// GigaChat3-702B-A36B — полная модель, ~170+ ГБ.
    Full,
}

/// HuggingFace репозиторий и квантизация.
#[derive(Debug, Clone)]
pub struct HuggingFaceModelConfig {
    pub repo_id: &'static str,
    pub quant: &'static str,
    pub file_pattern: &'static str,
    pub size_gb: f64,
}

impl ModelVariant {
    pub fn hf_config(&self) -> HuggingFaceModelConfig {
        match self {
            ModelVariant::GigaChat => HuggingFaceModelConfig {
                repo_id: "ubergarm/GigaChat3-10B-A1.8B-GGUF",
                quant: "Q8_0",
                file_pattern: "GigaChat3-10B-A1.8B-Q8_0.gguf",
                size_gb: 10.6,
            },
            ModelVariant::DeepSeekCoder => HuggingFaceModelConfig {
                repo_id: "TheBloke/deepseek-coder-6.7B-instruct-GGUF",
                quant: "Q4_K_M",
                file_pattern: "deepseek-coder-6.7b-instruct.Q4_K_M.gguf",
                size_gb: 4.0,
            },
            ModelVariant::SmolLM2 => HuggingFaceModelConfig {
                repo_id: "HuggingFaceTB/SmolLM2-1.7B-Instruct-GGUF",
                quant: "q4_k_m",
                file_pattern: "smollm2-1.7b-instruct-q4_k_m.gguf",
                size_gb: 1.2,
            },
            ModelVariant::Full => HuggingFaceModelConfig {
                repo_id: "bartowski/ai-sage_GigaChat3-702B-A36B-preview-GGUF",
                quant: "IQ2_XXS",
                file_pattern: "ai-sage_GigaChat3-702B-A36B-preview-IQ2_XXS",
                size_gb: 170.0,
            },
        }
    }

    pub fn model_id(&self) -> &'static str {
        match self {
            ModelVariant::GigaChat => "gigachat3",
            ModelVariant::DeepSeekCoder => "deepseek-coder",
            ModelVariant::SmolLM2 => "smollm2",
            ModelVariant::Full => "gigachat3-702b",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ModelVariant::GigaChat => "GigaChat3 10B",
            ModelVariant::DeepSeekCoder => "DeepSeek-Coder 6.7B",
            ModelVariant::SmolLM2 => "SmolLM2 1.7B",
            ModelVariant::Full => "GigaChat3 702B",
        }
    }

    pub fn provider_id(&self) -> &'static str {
        match self {
            ModelVariant::GigaChat => "local-gigachat",
            ModelVariant::DeepSeekCoder => "local-deepseek",
            ModelVariant::SmolLM2 => "local-smollm2",
            ModelVariant::Full => "local-gigachat-702b",
        }
    }
}

pub const DEFAULT_CONTEXT_SIZE: usize = 8192;
pub const DEFAULT_MAX_TOKENS: usize = 2048;

/// Базовый URL для скачивания с HuggingFace.
pub fn hf_resolve_url(repo_id: &str, path: &str) -> String {
    format!("https://huggingface.co/{repo_id}/resolve/main/{path}")
}

/// API для списка файлов.
pub fn hf_api_tree_url(repo_id: &str, path: &str) -> String {
    if path.is_empty() {
        format!("https://huggingface.co/api/models/{repo_id}/tree/main")
    } else {
        let encoded = path.replace('/', "%2F");
        format!("https://huggingface.co/api/models/{repo_id}/tree/main/{encoded}")
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct LocalConfig {
    pub models_dir: PathBuf,
    pub context_size: usize,
    pub max_tokens: usize,
    pub n_threads: Option<usize>,
    pub model_variant: ModelVariant,
}

impl LocalConfig {
    pub fn default_models_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("kengaide")
            .join("models")
    }

    /// Директория модели.
    pub fn model_dir(&self) -> PathBuf {
        let subdir = match self.model_variant {
            ModelVariant::GigaChat => "gigachat3-10b-a18b",
            ModelVariant::DeepSeekCoder => "deepseek-coder-6.7b",
            ModelVariant::SmolLM2 => "smollm2-1.7b",
            ModelVariant::Full => "gigachat3-702b-a36b",
        };
        self.models_dir.join(subdir)
    }

    /// Для обратной совместимости.
    #[allow(dead_code)]
    pub fn gigachat3_dir(&self) -> PathBuf {
        self.model_dir()
    }

    pub fn default_config() -> Self {
        Self {
            models_dir: Self::default_models_dir(),
            context_size: DEFAULT_CONTEXT_SIZE,
            max_tokens: DEFAULT_MAX_TOKENS,
            n_threads: None,
            model_variant: ModelVariant::GigaChat,
        }
    }
}
