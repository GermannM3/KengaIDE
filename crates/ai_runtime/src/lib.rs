//! AI Runtime v1.1 — оркестрация AI для IDE.
//!
//! Pipeline: PromptBuilder → ProviderSelector → generate (streaming) → emit chunks.

mod agent;
mod controller;
mod error;
mod orchestration;
mod prompt_builder;
mod provider_selector;
mod runtime;
mod streaming;

pub use orchestration::{TaskRole, load_model_roles, ensure_model_roles_config};
pub use agent::{
    build_agent_system_prompt, run_agent_loop, AgentProgress, AgentProgressEmitter,
    AGENT_SYSTEM_PROMPT,
};
pub use ai_providers::AiResponse;
pub use controller::{AiController, ChunkEmitter};
pub use error::AiRuntimeError;
pub use runtime::AiRuntime;
