//! GGUF inference через llama.cpp.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use std::num::NonZeroU32;

use crate::config::DEFAULT_CONTEXT_SIZE;
use crate::error::LocalProviderError;
use crate::hardware_detect::cpu_cores;

/// Максимум токенов в одном batch для decode (ограничение llama.cpp; при большем префилле — "Insufficient Space").
const PREFILL_BATCH_SIZE: i32 = 512;

pub struct InferenceEngine {
    backend: Arc<LlamaBackend>,
    model: Arc<LlamaModel>,
}

impl InferenceEngine {
    pub fn load(path: &Path, _n_threads: Option<usize>) -> Result<Self, LocalProviderError> {
        let backend = LlamaBackend::init()
            .map_err(|e| LocalProviderError::ModelLoadFailed(e.to_string()))?;

        let model_params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, path, &model_params)
            .map_err(|e| LocalProviderError::ModelLoadFailed(e.to_string()))?;

        Ok(Self {
            backend: Arc::new(backend),
            model: Arc::new(model),
        })
    }

    pub fn generate(
        &self,
        prompt: &str,
        max_tokens: usize,
    ) -> Result<(String, u32, u64), LocalProviderError> {
        let n_threads = cpu_cores().unwrap_or(4);
        let ctx_size = DEFAULT_CONTEXT_SIZE.min(max_tokens + prompt.len() / 4 + 256);

        let n_ctx = NonZeroU32::new(ctx_size as u32)
            .or(NonZeroU32::new(2048))
            .ok_or_else(|| LocalProviderError::InferenceFailed("Invalid context size".into()))?;

        let n_threads_i32 = n_threads as i32;
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(n_ctx))
            .with_n_threads(n_threads_i32)
            .with_n_threads_batch(n_threads_i32);

        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| LocalProviderError::InferenceFailed(e.to_string()))?;

        let tokens_list = self
            .model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| LocalProviderError::InferenceFailed(e.to_string()))?;

        let mut batch = LlamaBatch::new(PREFILL_BATCH_SIZE as usize, 1);
        let n_tokens = tokens_list.len();
        let last_pos = (n_tokens as i32).saturating_sub(1);

        // Prefill по чанкам, чтобы не превышать лимит batch (иначе "Insufficient Space of 512").
        let mut pos = 0i32;
        for chunk in tokens_list.chunks(PREFILL_BATCH_SIZE as usize) {
            batch.clear();
            for (j, &token) in chunk.iter().enumerate() {
                let p = pos + j as i32;
                let is_last = p == last_pos;
                batch
                    .add(token, p, &[0], is_last)
                    .map_err(|e| LocalProviderError::InferenceFailed(e.to_string()))?;
            }
            ctx.decode(&mut batch)
                .map_err(|e| LocalProviderError::InferenceFailed(e.to_string()))?;
            pos += chunk.len() as i32;
        }

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::dist(1234),
            LlamaSampler::greedy(),
        ]);

        let start = Instant::now();
        let mut output = String::new();
        let mut n_cur = n_tokens as i32;
        let mut tokens_generated: u32 = 0;

        for _ in 0..max_tokens {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);

            if self.model.is_eog_token(token) {
                break;
            }

            let piece = self
                .model
                .token_to_str(token, llama_cpp_2::model::Special::Tokenize)
                .unwrap_or_else(|_| String::new());
            output.push_str(&piece);

            batch.clear();
            batch
                .add(token, n_cur, &[0], true)
                .map_err(|e| LocalProviderError::InferenceFailed(e.to_string()))?;

            n_cur += 1;
            tokens_generated += 1;

            ctx.decode(&mut batch)
                .map_err(|e| LocalProviderError::InferenceFailed(e.to_string()))?;
        }

        let latency_ms = start.elapsed().as_millis() as u64;
        Ok((output, tokens_generated, latency_ms))
    }

    /// Стриминговая генерация: для каждого токена вызывается `on_token`;
    /// при `cancel_requested.load(Ordering::Relaxed) == true` цикл прерывается.
    pub fn generate_stream<F>(
        &self,
        prompt: &str,
        max_tokens: usize,
        cancel_requested: &AtomicBool,
        mut on_token: F,
    ) -> Result<(), LocalProviderError>
    where
        F: FnMut(&str),
    {
        let n_threads = cpu_cores().unwrap_or(4);
        let ctx_size = DEFAULT_CONTEXT_SIZE.min(max_tokens + prompt.len() / 4 + 256);

        let n_ctx = NonZeroU32::new(ctx_size as u32)
            .or(NonZeroU32::new(2048))
            .ok_or_else(|| LocalProviderError::InferenceFailed("Invalid context size".into()))?;

        let n_threads_i32 = n_threads as i32;
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(n_ctx))
            .with_n_threads(n_threads_i32)
            .with_n_threads_batch(n_threads_i32);

        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| LocalProviderError::InferenceFailed(e.to_string()))?;

        let tokens_list = self
            .model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| LocalProviderError::InferenceFailed(e.to_string()))?;

        let mut batch = LlamaBatch::new(PREFILL_BATCH_SIZE as usize, 1);
        let n_tokens = tokens_list.len();
        let last_pos = (n_tokens as i32).saturating_sub(1);

        // Prefill по чанкам (см. generate()).
        let mut pos = 0i32;
        for chunk in tokens_list.chunks(PREFILL_BATCH_SIZE as usize) {
            batch.clear();
            for (j, &token) in chunk.iter().enumerate() {
                let p = pos + j as i32;
                let is_last = p == last_pos;
                batch
                    .add(token, p, &[0], is_last)
                    .map_err(|e| LocalProviderError::InferenceFailed(e.to_string()))?;
            }
            ctx.decode(&mut batch)
                .map_err(|e| LocalProviderError::InferenceFailed(e.to_string()))?;
            pos += chunk.len() as i32;
        }

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::dist(1234),
            LlamaSampler::greedy(),
        ]);

        let mut n_cur = n_tokens as i32;

        for _ in 0..max_tokens {
            if cancel_requested.load(Ordering::Relaxed) {
                break;
            }

            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);

            if self.model.is_eog_token(token) {
                break;
            }

            let piece = self
                .model
                .token_to_str(token, llama_cpp_2::model::Special::Tokenize)
                .unwrap_or_else(|_| String::new());
            on_token(&piece);

            batch.clear();
            batch
                .add(token, n_cur, &[0], true)
                .map_err(|e| LocalProviderError::InferenceFailed(e.to_string()))?;

            n_cur += 1;

            ctx.decode(&mut batch)
                .map_err(|e| LocalProviderError::InferenceFailed(e.to_string()))?;
        }

        Ok(())
    }
}
