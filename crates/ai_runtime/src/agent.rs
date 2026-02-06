//! Agent loop: план → вызов инструментов (local + MCP) → проверка.
//!
//! Только режим Agent подключает tools и этот цикл.

use agent_tools::{ToolCall, ToolExecutor};
use ai_providers::{AiChunk, AiMode, AiProvider, EditorContext, GenerateOptions, GenerateRequest};
use backend_core::{
    append_audit_event, append_log, current_environment, finish_session_meta, save_session_meta,
    AuditEvent, AuditSessionMeta,
};
use futures_util::StreamExt;
use mcp_provider::{McpContextProvider, McpToolDescriptor, McpToolRegistry};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use crate::error::AiRuntimeError;
use crate::provider_selector::ProviderSelector;

fn emit_session_end(project_root: Option<&Path>, session_id: &str, status: &str) {
    append_audit_event(
        project_root,
        session_id,
        &AuditEvent::SessionEnd {
            status: status.to_string(),
        },
    );
    finish_session_meta(project_root, session_id, status);
}

/// Максимум символов MCP-контекста (~25% от типичного лимита контекста).
const MCP_CONTEXT_MAX_CHARS: usize = 6000;
/// Максимум вызовов инструментов на одно сообщение пользователя.
const MAX_TOOL_CALLS_PER_MESSAGE: usize = 8;
/// Максимум токенов за сессию (приблизительно: ~4 символа на токен).
const MAX_TOKENS_PER_SESSION: usize = 32_000;
/// Максимум времени работы агента (мс). 10 мин.
const MAX_TIME_MS: u64 = 600_000;
/// Сколько одинаковых ошибок подряд = зацикливание.
const SAME_ERROR_THRESHOLD: usize = 3;

/// База системного промпта без списка инструментов (список строится динамически).
const AGENT_SYSTEM_PROMPT_BASE: &str = r#"You are an IDE agent. Your goal is to MODIFY THE PROJECT using tools. NEVER output code as plain text — ALWAYS use tool_call.

Rules:
1. Your FIRST response MUST be a tool_call (list_files, read_file, create_file, or apply_patch).
2. When user asks to create/implement/add/generate — use tools immediately. No explanations, no code in text.
3. For new projects: create_project first, then MUST implement with create_file/apply_patch. Never stop with just skeleton.
4. For existing files: use ONLY apply_patch. Never overwrite a file wholly.
5. After each tool result, send the next tool_call until done.

"#;

/// Строит полный системный промпт с локальными и MCP-инструментами.
pub fn build_agent_system_prompt(mcp_tools: &[McpToolDescriptor]) -> String {
    let mut s = AGENT_SYSTEM_PROMPT_BASE.to_string();
    s.push_str("You have access to the following tools:\n");
    s.push_str("- create_project(template: string, name?: string) — create skeleton only. After this you MUST use create_file/apply_patch to add the actual implementation. Never stop with just hello world.\n");
    s.push_str("- list_files(path?: string) — list directory contents\n");
    s.push_str("- read_file(path: string) — read file content\n");
    s.push_str("- create_file(path: string, content: string) — create new file (or overwrite only when creating from scratch)\n");
    s.push_str("- apply_patch(path: string, before: string, after: string) — apply contextual diff: exact \"before\" block is replaced by \"after\" (once). Use for all edits to existing files.\n");
    for t in mcp_tools {
        let desc = t
            .description
            .as_deref()
            .unwrap_or("MCP tool")
            .replace('\n', " ");
        let short = if desc.len() > 120 {
            format!("{}...", &desc[..117])
        } else {
            desc
        };
        s.push_str(&format!("- {} — {}\n", t.namespaced_name(), short));
    }
    s.push_str(r#"
Output format for tool calls (use exactly this):
```tool_call
{"name": "create_project", "arguments": {"template": "rust", "name": "my-app"}}
```
or:
```tool_call
{"name": "create_file", "arguments": {"path": "relative/path", "content": "..."}}
```
or for edits:
```tool_call
{"name": "apply_patch", "arguments": {"path": "relative/path", "before": "exact text to find", "after": "replacement"}}
```
or for MCP tools (use exact name mcp::server::tool):
```tool_call
{"name": "mcp::server::tool", "arguments": {...}}
```

If the task is unclear, ask ONE clarifying question.
Otherwise, proceed immediately."#);
    s
}

/// Системный промпт для режима Agent без MCP tools (для совместимости).
pub const AGENT_SYSTEM_PROMPT: &str = r#"You are an IDE agent, not a chat assistant.

Your goal is to MODIFY THE PROJECT, not to explain code.

You may receive additional context from external knowledge providers (MCP). Use it to improve accuracy and decisions, but never blindly trust it.

You may call external tools provided via MCP when helpful. MCP tools return factual or structured data. Always reason after receiving tool results before answering the user.

Rules:
1. NEVER output code as plain text if a tool can be used.
2. NEVER call the same tool with the same arguments twice. Use the result you already have. After list_files, proceed to read_file or apply_patch; do not list again.
3. When a user asks to "create", "implement", "add", or "generate":
   - You MUST create or modify files using tools.
4. For creating a NEW PROJECT: use create_project(template, name?) first. Then you MUST implement the user's request with create_file/apply_patch. create_project creates only a skeleton (hello world). NEVER stop after create_project — always add the real code.
5. First, create a PLAN.
6. Then EXECUTE the plan step by step using tools.
7. After each tool call, reassess the state.
8. Stop only when the task is fully completed.

You MUST NEVER overwrite a file wholly. For existing files use ONLY apply_patch.
If a patch fails (BeforeBlockNotFound, AmbiguousPatch, FileNotFound, IOError) — fix the context and try again.

You have access to the following tools:
- create_project(template: string, name?: string) — create new project from template (empty|rust|python|node). Use ONLY this for new projects. If path exists, returns success — use list_files/read_file to work with it.
- list_files(path?: string) — list directory contents
- read_file(path: string) — read file content
- create_file(path: string, content: string) — create new file (or overwrite only when creating from scratch)
- apply_patch(path: string, before: string, after: string) — apply contextual diff: exact "before" block is replaced by "after" (once). Use for all edits to existing files.

Output format for tool calls (use exactly this):
```tool_call
{"name": "create_project", "arguments": {"template": "rust", "name": "my-app"}}
```
or:
```tool_call
{"name": "create_file", "arguments": {"path": "relative/path", "content": "..."}}
```
or for edits:
```tool_call
{"name": "apply_patch", "arguments": {"path": "relative/path", "before": "exact text to find", "after": "replacement"}}
```

If the task is unclear, ask ONE clarifying question.
Otherwise, proceed immediately."#;

const TOOL_CALL_MARKER: &str = "```tool_call";
const TOOL_CALL_END: &str = "```";

/// Извлекает первый полный JSON-объект из строки (пробуем парсить от первой `{` до каждой `}`).
fn extract_json_object(s: &str) -> Option<&str> {
    let start = s.find('{')?;
    let rest = &s[start..];
    let bytes = rest.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'}' {
            let slice = std::str::from_utf8(&bytes[..=i]).ok()?;
            if serde_json::from_str::<ToolCall>(slice).is_ok() {
                return Some(slice);
            }
        }
    }
    None
}

/// Парсит из ответа модели один вызов инструмента (первый найденный).
/// Формат: ```tool_call\n{ "name": "...", "arguments": {...} }\n``` или без закрывающего ```.
pub fn parse_tool_call(response: &str) -> Option<ToolCall> {
    let start = response.find(TOOL_CALL_MARKER)?;
    let after_marker = response[start + TOOL_CALL_MARKER.len()..].trim_start();
    let json_str = if let Some(end_pos) = after_marker.find(TOOL_CALL_END) {
        after_marker[..end_pos].trim()
    } else {
        extract_json_object(after_marker)?.trim()
    };
    let call: ToolCall = serde_json::from_str(json_str).ok()?;
    Some(call)
}

/// Прогресс агента для UI (session_started, model_selected, thinking, tool_call, tool_result, patch events, done).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentProgress {
    /// Сессия начата (E6 audit).
    SessionStarted { session_id: String },
    /// Выбрана модель по роли (E5).
    ModelSelected { role: String, model_id: String },
    /// Модель генерирует ответ (ход мыслей).
    Thinking,
    ToolCall { name: String, path: Option<String> },
    ToolResult { success: bool, output: String },
    /// Начало применения патча (apply_patch).
    PatchApplyStarted { path: String },
    /// Патч применён успешно.
    PatchApplySuccess { path: String },
    /// Детали патча для rollback (path, before, after).
    PatchApplied { path: String, before: String, after: String },
    /// Ошибка применения патча.
    PatchApplyError { path: String, message: String },
    Done { message: String },
}

/// Эмиттер прогресса агента (для Tauri events).
pub type AgentProgressEmitter = Arc<dyn Fn(AgentProgress) + Send + Sync>;

/// Запускает цикл агента: generate → parse tool_call → execute → feed back, пока есть вызовы.
/// По завершении эмитит Done { message }.
/// Формирует блок MCP-контекста для вставки в промпт. Обрезает по MCP_CONTEXT_MAX_CHARS.
fn format_mcp_context_block(chunks: &[mcp_provider::McpContextChunk]) -> String {
    if chunks.is_empty() {
        return String::new();
    }
    let mut s = String::from("\n\nAdditional context from external knowledge systems:\n");
    let mut total = s.len();
    for c in chunks {
        let line = format!("- [{}]: {}\n", c.server, c.content);
        if total + line.len() > MCP_CONTEXT_MAX_CHARS {
            let rest = MCP_CONTEXT_MAX_CHARS.saturating_sub(total);
            if rest > 20 {
                s.push_str(&line.chars().take(rest).collect::<String>());
                s.push_str("...\n");
            }
            break;
        }
        total += line.len();
        s.push_str(&line);
    }
    s
}

pub async fn run_agent_loop(
    providers: &[Arc<dyn AiProvider>],
    project_root: &Path,
    user_message: &str,
    emitter: AgentProgressEmitter,
    max_turns: usize,
    preferred_provider_id: Option<&str>,
) -> Result<String, AiRuntimeError> {
    let session_id = Uuid::new_v4().to_string();
    let project_root_opt = Some(project_root);

    let selection = ProviderSelector::select(
        providers,
        AiMode::Agent,
        user_message,
        preferred_provider_id,
        Some(project_root),
    )
    .await?;
    let provider = selection.provider;
    let provider_id = provider.id().to_string();
    let role_str = selection.role.as_str().to_string();
    let model_id = selection.model_id.clone();

    append_audit_event(
        project_root_opt,
        &session_id,
        &AuditEvent::SessionStart {
            session_id: session_id.clone(),
            mode: "agent".to_string(),
            task: user_message.trim().to_string(),
            policy: "default".to_string(),
            environment: Some(current_environment()),
        },
    );
    save_session_meta(
        project_root_opt,
        &AuditSessionMeta {
            session_id: session_id.clone(),
            started_at: chrono::Utc::now().to_rfc3339(),
            ended_at: None,
            mode: "agent".to_string(),
            task: user_message.trim().to_string(),
            status: "running".to_string(),
            policy: "default".to_string(),
            environment: Some(current_environment()),
        },
    );
    append_audit_event(
        project_root_opt,
        &session_id,
        &AuditEvent::TaskClassified {
            role: role_str.clone(),
            reason: "TaskClassifier (rule-based)".to_string(),
        },
    );
    append_audit_event(
        project_root_opt,
        &session_id,
        &AuditEvent::ModelSelected {
            role: role_str.clone(),
            model: model_id.clone(),
            provider: provider_id.clone(),
        },
    );

    emitter(AgentProgress::SessionStarted {
        session_id: session_id.clone(),
    });
    emitter(AgentProgress::ModelSelected {
        role: role_str,
        model_id: model_id.clone(),
    });
    append_log(
        Some(project_root),
        "agent.log",
        &format!(
            "agent_start role={} model={}",
            selection.role.as_str(),
            selection.model_id
        ),
    );
    let executor = ToolExecutor::new(project_root.to_path_buf());

    let mcp_block = match McpContextProvider::from_config_file() {
        Ok(mcp) => {
            let chunks = mcp.fetch_context(user_message).await;
            format_mcp_context_block(&chunks)
        }
        Err(e) => {
            tracing::debug!(error = %e, "MCP config load failed, agent continues without MCP");
            String::new()
        }
    };

    let (mcp_tools, mcp_registry) = match McpToolRegistry::from_config_file() {
        Ok(reg) if reg.has_servers() => {
            let tools = reg.list_all_tools().await;
            (tools, Some(reg))
        }
        _ => (Vec::new(), None),
    };
    let system_prompt = build_agent_system_prompt(&mcp_tools);

    let mut conversation = format!(
        "{}{}\n\nUser: {}\n\nAssistant: ",
        system_prompt,
        mcp_block,
        user_message.trim()
    );
    let mut turn = 0;
    let mut tool_calls_in_run: usize = 0;
    let mut last_tool_call: Option<(String, serde_json::Value)> = None;
    let start_time = Instant::now();
    let mut total_tokens_approx: usize = 0;
    let mut last_errors: Vec<String> = Vec::with_capacity(SAME_ERROR_THRESHOLD);

    append_log(Some(project_root), "agent.log", &format!("agent_start user_msg_len={}", user_message.len()));

    loop {
        if turn >= max_turns {
            append_log(Some(project_root), "agent.log", "guardrail: max_turns");
            emit_session_end(project_root_opt, &session_id, "aborted");
            emitter(AgentProgress::Done {
                message: "Агент остановлен: достигнут лимит шагов. Попробуй переформулировать задачу.".to_string(),
            });
            break Ok(String::new());
        }
        if start_time.elapsed().as_millis() as u64 > MAX_TIME_MS {
            append_log(Some(project_root), "agent.log", "guardrail: max_time");
            emit_session_end(project_root_opt, &session_id, "aborted");
            emitter(AgentProgress::Done {
                message: "Агент остановлен: превышено время работы. Попробуй разбить задачу на части.".to_string(),
            });
            break Ok(String::new());
        }
        if total_tokens_approx > MAX_TOKENS_PER_SESSION {
            append_log(Some(project_root), "agent.log", "guardrail: max_tokens");
            emit_session_end(project_root_opt, &session_id, "aborted");
            emitter(AgentProgress::Done {
                message: "Агент остановлен: превышен лимит токенов. Попробуй переформулировать задачу.".to_string(),
            });
            break Ok(String::new());
        }
        turn += 1;
        emitter(AgentProgress::Thinking);

        append_audit_event(
            project_root_opt,
            &session_id,
            &AuditEvent::PromptSent {
                tokens: Some(conversation.len() / 4),
            },
        );

        let request_id = Uuid::new_v4().to_string();
        let gen_request = GenerateRequest {
            id: request_id.clone(),
            prompt: conversation.clone(),
            context: Some(EditorContext::default()),
            mode: AiMode::Agent,
        };
        let options = GenerateOptions {
            temperature: Some(0.3),
            max_tokens: Some(4096),
        };

        let mut stream = provider
            .generate(gen_request, options)
            .await
            .map_err(AiRuntimeError::from)?;

        let mut response = String::new();
        while let Some(chunk) = stream.next().await {
            match chunk {
                AiChunk::Token { value } => response.push_str(&value),
                AiChunk::End => break,
                AiChunk::Error { error } => {
                    emit_session_end(project_root_opt, &session_id, "error");
                    append_audit_event(
                        project_root_opt,
                        &session_id,
                        &AuditEvent::Error {
                            message: error.clone(),
                        },
                    );
                    return Err(AiRuntimeError::Provider(ai_providers::ProviderError::Generation(
                        error,
                    )));
                }
                AiChunk::Start => {}
            }
        }

        let response = response.trim().to_string();
        total_tokens_approx += response.len() / 4;

        if response.is_empty() {
            append_log(Some(project_root), "agent.log", "guardrail: empty_response");
            emit_session_end(project_root_opt, &session_id, "error");
            emitter(AgentProgress::Done {
                message: "Агент остановлен: модель вернула пустой ответ. Попробуй переформулировать задачу или сменить провайдер.".to_string(),
            });
            break Ok(String::new());
        }

        if let Some(call) = parse_tool_call(&response) {
            if tool_calls_in_run >= MAX_TOOL_CALLS_PER_MESSAGE {
                emit_session_end(project_root_opt, &session_id, "aborted");
                emitter(AgentProgress::Done {
                    message: "Достигнут лимит вызовов инструментов (8). Остановка.".to_string(),
                });
                break Ok(String::new());
            }
            tool_calls_in_run += 1;

            let path = call
                .arguments
                .get("path")
                .and_then(|v| v.as_str())
                .map(String::from);
            append_log(
                Some(project_root),
                "agent.log",
                &format!(
                    "tool_call {} {}",
                    call.name,
                    path.as_deref().map(|p| format!("path={}", p)).unwrap_or_default()
                ),
            );
            append_audit_event(
                project_root_opt,
                &session_id,
                &AuditEvent::ToolCall {
                    tool: call.name.clone(),
                    path: path.clone(),
                },
            );
            emitter(AgentProgress::ToolCall {
                name: call.name.clone(),
                path: path.clone(),
            });

            let is_repeat = last_tool_call
                .as_ref()
                .map(|(n, a)| n == &call.name && a == &call.arguments)
                .unwrap_or(false);
            last_tool_call = Some((call.name.clone(), call.arguments.clone()));

            if is_repeat {
                append_log(Some(project_root), "agent.log", "guardrail: repeated_tool_call");
                emit_session_end(project_root_opt, &session_id, "aborted");
                emitter(AgentProgress::Done {
                    message: "Агент остановлен: повторяющиеся действия. Попробуй переформулировать задачу.".to_string(),
                });
                break Ok(String::new());
            }

            let (success, output) = if call.name.starts_with("mcp::") {
                match &mcp_registry {
                    Some(reg) => {
                        match reg.call_tool(&call.name, call.arguments.clone()).await {
                            Ok(val) => {
                                let out = if val.is_string() {
                                    val.as_str().unwrap_or("").to_string()
                                } else if let Some(c) = val.get("content") {
                                    c.as_str()
                                        .map(String::from)
                                        .unwrap_or_else(|| c.to_string())
                                } else {
                                    val.to_string()
                                };
                                (true, out)
                            }
                            Err(e) => {
                                tracing::debug!(tool = %call.name, error = %e, "MCP tool call failed");
                                (false, format!("MCP error: {}", e))
                            }
                        }
                    }
                    None => (false, "MCP not configured".to_string()),
                }
            } else {
                if call.name == "apply_patch" {
                    if let Some(ref p) = path {
                        emitter(AgentProgress::PatchApplyStarted { path: p.clone() });
                    }
                }
                let result = executor.run(&call);
                if call.name == "apply_patch" {
                    if let Some(ref p) = path {
                        if result.success {
                            append_audit_event(
                                project_root_opt,
                                &session_id,
                                &AuditEvent::PatchApplied {
                                    path: p.clone(),
                                    hash: None,
                                },
                            );
                            emitter(AgentProgress::PatchApplySuccess { path: p.clone() });
                            if let (Some(before), Some(after)) = (
                                call.arguments.get("before").and_then(|v| v.as_str()),
                                call.arguments.get("after").and_then(|v| v.as_str()),
                            ) {
                                emitter(AgentProgress::PatchApplied {
                                    path: p.clone(),
                                    before: before.to_string(),
                                    after: after.to_string(),
                                });
                            }
                        } else {
                            emitter(AgentProgress::PatchApplyError {
                                path: p.clone(),
                                message: result.output.clone(),
                            });
                        }
                    }
                }
                (result.success, result.output)
            };

            if !success {
                last_errors.push(output.clone());
                if last_errors.len() > SAME_ERROR_THRESHOLD {
                    last_errors.remove(0);
                }
                if last_errors.len() >= SAME_ERROR_THRESHOLD {
                    let first = &last_errors[0];
                    if last_errors.iter().all(|e| e == first) {
                        append_log(Some(project_root), "agent.log", "guardrail: repeated_errors");
                        emit_session_end(project_root_opt, &session_id, "aborted");
                        emitter(AgentProgress::Done {
                            message: "Агент остановлен: повторяющиеся ошибки. Попробуй переформулировать задачу.".to_string(),
                        });
                        break Ok(String::new());
                    }
                }
            } else {
                last_errors.clear();
            }

            append_audit_event(
                project_root_opt,
                &session_id,
                &AuditEvent::ToolResult {
                    success,
                    output_len: Some(output.len()),
                },
            );
            emitter(AgentProgress::ToolResult {
                success,
                output: output.clone(),
            });
            conversation.push_str(&response);
            conversation.push_str("\n\nTool result: ");
            conversation.push_str(if success { "OK. " } else { "ERROR. " });
            conversation.push_str(&output);
            if success {
                if call.name == "create_project" {
                    conversation.push_str("\n\n[System: Project skeleton created. The user asked: \"");
                    conversation.push_str(user_message.trim());
                    conversation.push_str("\". You MUST now implement using create_file or apply_patch. Do NOT stop.]");
                } else if call.name == "list_files" {
                    conversation.push_str("\n\n[System: You listed files. The user asked: \"");
                    conversation.push_str(user_message.trim());
                    conversation.push_str("\". You MUST now read_file the relevant files and then use create_file or apply_patch to implement. Do NOT stop with just listing.]");
                } else if call.name == "read_file" {
                    conversation.push_str("\n\n[System: You have the file content. Now use apply_patch to modify it according to the user's request, or create_file for new files. Do NOT stop.]");
                }
            }
            conversation.push_str("\n\nAssistant: ");
        } else {
            let looks_like_completion = response.len() < 150
                || response.to_lowercase().contains("done")
                || response.to_lowercase().contains("готово")
                || response.to_lowercase().contains("complete")
                || response.to_lowercase().contains("finished")
                || response.to_lowercase().contains("завершено");

            if !looks_like_completion && response.len() > 200 {
                conversation.push_str(&response);
                conversation.push_str(
                    "\n\n[System: You must either call a tool or explicitly finish. Do not output long explanations without taking action.]\n\nAssistant: ",
                );
                tool_calls_in_run = 0;
                continue;
            }

            let final_message = response;
            append_log(Some(project_root), "agent.log", &format!("done msg_len={}", final_message.len()));
            emit_session_end(project_root_opt, &session_id, "done");
            emitter(AgentProgress::Done {
                message: final_message.clone(),
            });
            break Ok(final_message);
        }
    }
}
