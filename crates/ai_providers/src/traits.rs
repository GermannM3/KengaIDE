//! Интерфейс AI-провайдера: только streaming, без полного ответа.
//!
//! Ответ идёт чанками (start → token* → end | error). Отмена через cancel(request_id).

use async_trait::async_trait;
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::pin::Pin;

// ---------------------------------------------------------------------------
// Chunk types (streaming only)
// ---------------------------------------------------------------------------

/// Один чанк потока ответа. Полный ответ не возвращается — только поток чанков.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AiChunk {
    /// Генерация началась.
    Start,
    /// Очередной токен текста.
    Token { value: String },
    /// Генерация завершена успешно.
    End,
    /// Ошибка (провайдер отдаёт её в потоке, не через Result).
    Error { error: String },
}

// ---------------------------------------------------------------------------
// Request & options
// ---------------------------------------------------------------------------

/// Контекст редактора для IDE-режимов (explain, refactor, generate).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditorContext {
    pub path: Option<String>,
    pub content: Option<String>,
    pub selection: Option<String>,
}

/// Запрос на генерацию с обязательным id (для отмены и привязки чанков к запросу).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub id: String,
    pub prompt: String,
    pub context: Option<EditorContext>,
    pub mode: AiMode,
}

/// Опции генерации: температура, лимит токенов.
/// Токен отмены провайдер создаёт сам при generate() и хранит по request.id; cancel(id) отменяет его.
#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
}

// ---------------------------------------------------------------------------
// Modes & capabilities
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiMode {
    Chat,
    Explain,
    Refactor,
    Generate,
    /// Режим агента: план → вызов инструментов → проверка. Меняет файлы.
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    Local,
    Cloud,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub modes: HashSet<AiMode>,
    pub max_context_tokens: Option<usize>,
}

// ---------------------------------------------------------------------------
// Legacy: полный ответ (только для обратной совместимости при миграции, не для нового кода)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub content: String,
    pub tokens_used: Option<u32>,
    pub model: String,
    pub latency_ms: u64,
}

// ---------------------------------------------------------------------------
// Provider trait (streaming + cancel)
// ---------------------------------------------------------------------------

/// Тип потока чанков: только AiChunk, ошибки — через AiChunk::Error.
pub type AiChunkStream = Pin<Box<dyn Stream<Item = AiChunk> + Send>>;

/// Интерфейс AI-провайдера. Только streaming; полный ответ не возвращается.
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Уникальный id провайдера (local, cloud-openai, …).
    fn id(&self) -> &str;

    /// Имя для UI.
    fn name(&self) -> &str;

    /// Локальный или облачный.
    fn provider_type(&self) -> ProviderType;

    /// Возможности (режимы, лимит контекста).
    fn capabilities(&self) -> ProviderCapabilities;

    /// Генерация: возвращает поток чанков. Ошибки до старта — через Result; после старта — через AiChunk::Error.
    async fn generate(
        &self,
        request: GenerateRequest,
        options: GenerateOptions,
    ) -> Result<AiChunkStream, ProviderError>;

    /// Отменить генерацию по id запроса. Должно освобождать ресурсы и останавливать поток.
    fn cancel(&self, request_id: &str);

    /// Доступность (модель загружена, API ключ есть и т.д.).
    async fn is_available(&self) -> Result<bool, ProviderError>;

    /// Model ID для role-based orchestration (E5). Например: gigachat3, deepseek-coder.
    /// None = провайдер не участвует в role-based выборе.
    fn model_id(&self) -> Option<&str> {
        None
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Provider unavailable: {0}")]
    Unavailable(String),
    #[error("Generation failed: {0}")]
    Generation(String),
}
