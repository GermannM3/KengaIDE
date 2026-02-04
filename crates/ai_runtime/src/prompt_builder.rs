//! PromptBuilder pipeline.
//!
//! Последовательность шагов: format_context → system_prompt → assemble.

use ai_providers::AiMode;
use context_manager::Context;

use crate::error::AiRuntimeError;

/// Pipeline для сборки промпта.
///
/// Шаги:
/// 1. format_context — контекст в строку
/// 2. system_prompt — системный промпт по режиму
/// 3. assemble — финальная сборка
pub struct PromptBuilder;

impl PromptBuilder {
    /// Собирает промпт из контекста и ввода пользователя.
    pub fn build(
        mode: AiMode,
        context: &Context,
        user_input: &str,
    ) -> Result<String, AiRuntimeError> {
        let formatted_context = Self::format_context(context)?;
        let system_prompt = Self::system_prompt(mode)?;
        let prompt = Self::assemble(system_prompt, user_input, &formatted_context)?;
        Ok(prompt)
    }

    /// Шаг 1: контекст в строку.
    fn format_context(context: &Context) -> Result<String, AiRuntimeError> {
        let mut parts = Vec::new();

        if let Some(ref f) = context.current_file {
            parts.push(format!("File: {}\n```\n{}\n```", f.path.display(), f.content));
        }
        if let Some(ref s) = context.selection {
            parts.push(format!("Selection:\n```\n{}\n```", s));
        }
        if !context.project_tree.is_empty() {
            let tree: Vec<String> = context
                .project_tree
                .iter()
                .map(|p| p.display().to_string())
                .collect();
            parts.push(format!("Project tree:\n{}", tree.join("\n")));
        }
        for f in &context.extra_files {
            parts.push(format!("File: {}\n```\n{}\n```", f.path.display(), f.content));
        }

        let result = if parts.is_empty() {
            "(no context)".into()
        } else {
            parts.join("\n\n")
        };

        Ok(result)
    }

    /// Шаг 2: системный промпт по режиму.
    fn system_prompt(mode: AiMode) -> Result<&'static str, AiRuntimeError> {
        let prompt = match mode {
            AiMode::Chat => "You are a helpful coding assistant.",
            AiMode::Explain => "Explain the code clearly. Focus on logic and structure.",
            AiMode::Refactor => "Refactor the selected code according to the instruction.",
            AiMode::Generate => "Generate code according to the prompt.",
            AiMode::Agent => crate::agent::AGENT_SYSTEM_PROMPT,
        };
        Ok(prompt)
    }

    /// Шаг 3: сборка финального промпта.
    fn assemble(
        system: &str,
        user_input: &str,
        context: &str,
    ) -> Result<String, AiRuntimeError> {
        let prompt = format!(
            "{}\n\nContext:\n{}\n\nUser: {}",
            system, context, user_input
        );
        Ok(prompt)
    }
}
