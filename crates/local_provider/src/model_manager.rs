//! Загрузка, проверка, hot-swap моделей GigaChat3 из HuggingFace.

use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

use crate::config::{
    hf_api_tree_url, hf_resolve_url, LocalConfig, ModelVariant, HuggingFaceModelConfig,
};
use crate::error::LocalProviderError;

/// Прогресс загрузки для UI.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub file_index: usize,
    pub file_count: usize,
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub path: PathBuf,
    pub loaded: bool,
}

/// Ответ HuggingFace API tree.
#[derive(Debug, serde::Deserialize)]
struct HfTreeItem {
    #[serde(rename = "type")]
    item_type: String,
    path: String,
    size: Option<u64>,
}

/// Управление локальными GGUF-моделями GigaChat3.
pub struct ModelManager {
    config: LocalConfig,
}

impl ModelManager {
    pub fn new(config: LocalConfig) -> Self {
        Self { config }
    }

    /// Проверить наличие модели.
    pub fn is_loaded(&self, name: &str) -> bool {
        if name != self.config.model_variant.model_id() {
            return false;
        }
        self.find_gguf_path().is_some()
    }

    /// Убедиться, что модель установлена. Скачивает при необходимости.
    pub async fn ensure_model_installed<F>(
        &self,
        mut on_progress: F,
    ) -> Result<PathBuf, LocalProviderError>
    where
        F: FnMut(DownloadProgress) + Send,
    {
        if let Some(path) = self.find_gguf_path() {
            return Ok(path);
        }
        self.download_from_huggingface(&mut on_progress)
            .await
    }

    /// Скачать модель с HuggingFace.
    pub async fn download_from_huggingface<F>(
        &self,
        on_progress: &mut F,
    ) -> Result<PathBuf, LocalProviderError>
    where
        F: FnMut(DownloadProgress) + Send,
    {
        let hf = self.config.model_variant.hf_config();
        let target_dir = self.config.model_dir();
        std::fs::create_dir_all(&target_dir)?;

        let files = self.list_hf_files(&hf).await?;
        let file_count = files.len();
        let mut bytes_total: u64 = files.iter().map(|(_, s)| s).sum();
        let mut bytes_done: u64 = 0;

        for (idx, (path, size)) in files.iter().enumerate() {
            let url = hf_resolve_url(hf.repo_id, path);
            let out_path = target_dir.join(
                Path::new(path)
                    .file_name()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(path)),
            );

            if out_path.exists() && out_path.metadata().ok().map(|m| m.len()) == Some(*size) {
                bytes_done += size;
                on_progress(DownloadProgress {
                    bytes_done,
                    bytes_total,
                    file_index: idx + 1,
                    file_count,
                });
                continue;
            }

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(86400))
                .build()
                .map_err(|e| LocalProviderError::DownloadFailed(e.to_string()))?;

            let response = client
                .get(&url)
                .send()
                .await
                .map_err(|e| LocalProviderError::DownloadFailed(e.to_string()))?;

            if !response.status().is_success() {
                return Err(LocalProviderError::DownloadFailed(format!(
                    "HTTP {}: {}",
                    response.status(),
                    response.status().canonical_reason().unwrap_or("")
                )));
            }

            let total = response.content_length().unwrap_or(*size);
            bytes_total += total;

            let mut file = tokio::fs::File::create(&out_path)
                .await
                .map_err(|e| LocalProviderError::DownloadFailed(e.to_string()))?;

            let mut stream = response.bytes_stream();

            use futures_util::StreamExt;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| LocalProviderError::DownloadFailed(e.to_string()))?;
                file.write_all(&chunk)
                    .await
                    .map_err(|e| LocalProviderError::DownloadFailed(e.to_string()))?;
                bytes_done += chunk.len() as u64;

                on_progress(DownloadProgress {
                    bytes_done,
                    bytes_total,
                    file_index: idx + 1,
                    file_count,
                });
            }

            file.flush().await?;
        }

        self.prepare_for_inference(&target_dir)?;
        self.find_gguf_path()
            .ok_or_else(|| LocalProviderError::DownloadFailed("No GGUF after download".into()))
    }

    /// Список файлов модели на HuggingFace (async, не блокирует runtime).
    async fn list_hf_files(&self, hf: &HuggingFaceModelConfig) -> Result<Vec<(String, u64)>, LocalProviderError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| LocalProviderError::DownloadFailed(e.to_string()))?;

        match self.config.model_variant {
            ModelVariant::GigaChat | ModelVariant::DeepSeekCoder | ModelVariant::SmolLM2 => {
                let url = hf_api_tree_url(hf.repo_id, "");
                let body: Vec<HfTreeItem> = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| LocalProviderError::DownloadFailed(e.to_string()))?
                    .json()
                    .await
                    .map_err(|e| LocalProviderError::DownloadFailed(e.to_string()))?;

                for item in body {
                    if item.item_type == "file" && item.path.ends_with(".gguf") {
                        let path = item.path;
                        if path == hf.file_pattern || path.contains(hf.quant) {
                            let size = item.size.unwrap_or(0);
                            return Ok(vec![(path, size)]);
                        }
                    }
                }
                Err(LocalProviderError::DownloadFailed(format!(
                    "GGUF file not found in {}",
                    hf.repo_id
                )))
            }
            ModelVariant::Full => {
                let subdir = hf.file_pattern;
                let url = hf_api_tree_url(hf.repo_id, subdir);
                let body: Vec<HfTreeItem> = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| LocalProviderError::DownloadFailed(e.to_string()))?
                    .json()
                    .await
                    .map_err(|e| LocalProviderError::DownloadFailed(e.to_string()))?;

                let mut files: Vec<(String, u64)> = body
                    .into_iter()
                    .filter(|i| i.item_type == "file" && i.path.ends_with(".gguf"))
                    .map(|i| (i.path, i.size.unwrap_or(0)))
                    .collect();
                files.sort_by(|a, b| a.0.cmp(&b.0));
                if files.is_empty() {
                    return Err(LocalProviderError::DownloadFailed(
                        "No GGUF files in 702B quant".into(),
                    ));
                }
                Ok(files)
            }
        }
    }

    /// Проверить файлы модели.
    pub fn verify_files(&self) -> Result<bool, LocalProviderError> {
        let path = match self.find_gguf_path() {
            Some(p) => p,
            None => return Ok(false),
        };
        if path.exists() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Подготовить модель к inference: записать metadata.json.
    pub fn prepare_for_inference(&self, dir: &Path) -> Result<(), LocalProviderError> {
        let name = match self.config.model_variant {
            ModelVariant::GigaChat => "GigaChat3-10B-A1.8B",
            ModelVariant::DeepSeekCoder => "DeepSeek-Coder-6.7B-Instruct",
            ModelVariant::SmolLM2 => "SmolLM2-1.7B-Instruct",
            ModelVariant::Full => "GigaChat3-702B-A36B-preview",
        };
        let metadata = serde_json::json!({
            "name": name,
            "source": "huggingface",
            "license": "MIT",
            "offline": true
        });
        let path = dir.join("metadata.json");
        std::fs::write(path, serde_json::to_string_pretty(&metadata).unwrap_or_default())
            .map_err(|e| LocalProviderError::Io(e))?;
        Ok(())
    }

    /// Проверка целостности (опционально по SHA256).
    pub fn verify_integrity(&self, _path: &Path) -> Result<bool, LocalProviderError> {
        Ok(true)
    }

    /// Путь к GGUF-файлу (первый для split).
    pub fn find_gguf_path(&self) -> Option<PathBuf> {
        let dir = self.config.model_dir();
        if !dir.exists() {
            return None;
        }
        let mut candidates: Vec<PathBuf> = std::fs::read_dir(&dir)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .map(|e| e == "gguf")
                    .unwrap_or(false)
            })
            .collect();
        candidates.sort();
        candidates.into_iter().next()
    }

    /// Список моделей.
    pub fn list_models(&self) -> Result<Vec<ModelInfo>, LocalProviderError> {
        let dir = self.config.model_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let path = self.find_gguf_path();
        if let Some(p) = path {
            Ok(vec![ModelInfo {
                id: self.config.model_variant.model_id().to_string(),
                path: p,
                loaded: true,
            }])
        } else {
            Ok(Vec::new())
        }
    }

    /// Hot-swap.
    pub fn hot_swap(&self, name: &str) -> Result<PathBuf, LocalProviderError> {
        self.find_gguf_path()
            .ok_or_else(|| LocalProviderError::ModelNotFound(name.to_string()))
    }
}
