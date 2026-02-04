//! Tauri commands — IPC между UI и Backend.

use ai_providers::{ApiProvider, AiChunk, AiProvider, GenerateOptions};
use ai_runtime::{run_agent_loop, AgentProgress, AiResponse, ChunkEmitter};
use backend_core::command_router::AiRequest;
use backend_core::{
    create_project_from_template, ensure_audit_dir, ensure_logs_dir, list_sessions,
    read_session_events, rollback_patch, TEMPLATES,
};

use crate::ai_config::{load_config, save_config, ProviderEntry};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Emitter, State};

use crate::state::AppState;

/// Payload события ai_chunk: request_id + chunk для UI.
#[derive(Debug, Clone, Serialize)]
pub struct AiChunkPayload {
    pub request_id: String,
    #[serde(flatten)]
    pub chunk: AiChunk,
}

/// Payload события ai_model_selected: role + model для UI badge (E5).
#[derive(Debug, Clone, Serialize)]
pub struct AiModelSelectedPayload {
    pub request_id: String,
    pub role: String,
    pub model_id: String,
}

/// Payload события agent_progress: request_id + прогресс (tool_call, tool_result, done).
#[derive(Debug, Clone, Serialize)]
pub struct AgentProgressPayload {
    pub request_id: String,
    #[serde(flatten)]
    pub progress: AgentProgress,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum LocalModelStatus {
    NotAvailable,
    NotLoaded,
    Ready,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalModelInfo {
    pub status: LocalModelStatus,
    pub size_gb: f64,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenProjectPayload {
    pub path: String,
}

/// Узел дерева файлов для боковой панели.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectTreeNode {
    pub name: String,
    pub path: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ProjectTreeNode>>,
}

/// Директории, которые не показываем в дереве (тяжёлые/служебные).
const SKIP_DIRS: &[&str] = &[".git", "node_modules", "target", "__pycache__", ".next", "dist", "build", ".venv", "venv"];

/// Статус Git для Status Bar: ветка и количество изменений.
#[derive(Debug, Clone, Serialize)]
pub struct GitStatus {
    pub branch: String,
    pub changes: u32,
}

/// Возвращает статус Git в корне проекта (ветка, кол-во изменённых файлов).
#[tauri::command]
pub async fn git_status(state: State<'_, AppState>) -> Result<Option<GitStatus>, String> {
    let root = {
        let project = state.project.read().await;
        project.current().map(|p| p.root.clone())
    };
    let Some(root) = root else {
        return Ok(None);
    };
    let root_str = root.to_string_lossy().into_owned();
    let result = tokio::task::spawn_blocking(move || {
        let branch = std::process::Command::new("git")
            .args(["-C", &root_str, "branch", "--show-current"])
            .output();
        let changes = std::process::Command::new("git")
            .args(["-C", &root_str, "status", "--short"])
            .output();
        match (branch, changes) {
            (Ok(b), Ok(c)) if b.status.success() && c.status.success() => {
                let branch = String::from_utf8_lossy(&b.stdout).trim().to_string();
                let change_count = String::from_utf8_lossy(&c.stdout)
                    .lines()
                    .filter(|l| !l.is_empty())
                    .count() as u32;
                Some(GitStatus {
                    branch: if branch.is_empty() {
                        "detached".to_string()
                    } else {
                        branch
                    },
                    changes: change_count,
                })
            }
            _ => None,
        }
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(result)
}

/// Версия приложения для About.
#[tauri::command]
pub fn get_app_version() -> Result<(String, String), String> {
    Ok(("KengaIDE".to_string(), env!("CARGO_PKG_VERSION").to_string()))
}

const MAX_TREE_DEPTH: usize = 6;
const MAX_TREE_NODES: usize = 500;

/// Выбирает папку через диалог. Возвращает путь или None при отмене.
#[tauri::command]
pub async fn pick_folder() -> Result<Option<String>, String> {
    let path = tokio::task::spawn_blocking(|| {
        let mut dialog = rfd::FileDialog::new();
        if let Some(home) = dirs::home_dir() {
            dialog = dialog.set_directory(home);
        }
        dialog.pick_folder()
    })
    .await
    .map_err(|e| e.to_string())?
    .map(|p| p.to_string_lossy().into_owned());
    Ok(path)
}

/// Открывает папку логов (.kengaide/logs в проекте или ~/.kengaide/logs).
#[tauri::command]
pub async fn open_logs_folder(state: State<'_, AppState>) -> Result<(), String> {
    let project_root = {
        let project = state.project.read().await;
        project.current().map(|p| p.root.clone())
    };
    let dir = ensure_logs_dir(project_root.as_deref()).map_err(|e| e.to_string())?;
    let path = dir.to_string_lossy().into_owned();
    tokio::task::spawn_blocking(move || {
        #[cfg(windows)]
        let _ = std::process::Command::new("explorer").arg(&path).status();
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&path).status();
        #[cfg(not(any(windows, target_os = "macos")))]
        let _ = std::process::Command::new("xdg-open").arg(&path).status();
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Открывает папку настроек MCP (~/.kengaide) в проводнике. Создаёт папку, если её нет.
#[tauri::command]
pub async fn open_mcp_config_folder() -> Result<(), String> {
    let dir = dirs::home_dir()
        .ok_or_else(|| "Не найдена домашняя папка".to_string())
        .map(|h| h.join(".kengaide"))?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.to_string_lossy().into_owned();
    tokio::task::spawn_blocking(move || {
        #[cfg(windows)]
        let _ = std::process::Command::new("explorer").arg(&path).status();
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&path).status();
        #[cfg(not(any(windows, target_os = "macos")))]
        let _ = std::process::Command::new("xdg-open").arg(&path).status();
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Откатывает список патчей (в обратном порядке: последний первым).
#[derive(Debug, serde::Deserialize)]
pub struct PatchForRollback {
    pub path: String,
    pub before: String,
    pub after: String,
}

#[tauri::command]
pub async fn rollback_patches(
    state: State<'_, AppState>,
    patches: Vec<PatchForRollback>,
) -> Result<usize, String> {
    let root = {
        let project = state.project.read().await;
        project
            .current()
            .map(|p| p.root.clone())
            .ok_or("Нет открытого проекта")?
    };
    let mut ok_count = 0;
    for p in patches.into_iter().rev() {
        if rollback_patch(&root, &p.path, &p.before, &p.after).is_ok() {
            ok_count += 1;
        }
    }
    Ok(ok_count)
}

/// Список session_id в аудите (E6).
#[tauri::command]
pub async fn list_audit_sessions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let project_root = {
        let project = state.project.read().await;
        project.current().map(|p| p.root.clone())
    };
    Ok(list_sessions(project_root.as_deref()))
}

/// События сессии для Replay / Explain (E6).
#[tauri::command]
pub async fn get_audit_events(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let project_root = {
        let project = state.project.read().await;
        project.current().map(|p| p.root.clone())
    };
    let events = read_session_events(project_root.as_deref(), &session_id)
        .map_err(|e| e.to_string())?;
    let values: Vec<serde_json::Value> = events
        .into_iter()
        .filter_map(|e| serde_json::to_value(&e).ok())
        .collect();
    Ok(values)
}

/// Открывает папку аудита (.kengaide/audit).
#[tauri::command]
pub async fn open_audit_folder(state: State<'_, AppState>) -> Result<(), String> {
    let project_root = {
        let project = state.project.read().await;
        project.current().map(|p| p.root.clone())
    };
    let dir = ensure_audit_dir(project_root.as_deref()).map_err(|e| e.to_string())?;
    let path = dir.to_string_lossy().into_owned();
    tokio::task::spawn_blocking(move || {
        #[cfg(windows)]
        let _ = std::process::Command::new("explorer").arg(&path).status();
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&path).status();
        #[cfg(not(any(windows, target_os = "macos")))]
        let _ = std::process::Command::new("xdg-open").arg(&path).status();
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Возвращает путь текущего открытого проекта (или null).
#[tauri::command]
pub async fn get_project_path(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let project = state.project.read().await;
    Ok(project
        .current()
        .map(|p| p.root.to_string_lossy().into_owned()))
}

/// Строит дерево файлов проекта для боковой панели.
#[tauri::command]
pub async fn get_project_tree(state: State<'_, AppState>) -> Result<Option<Vec<ProjectTreeNode>>, String> {
    let root = {
        let project = state.project.read().await;
        match project.current() {
            Some(p) => p.root.clone(),
            None => return Ok(None),
        }
    };
    let tree = tokio::task::spawn_blocking(move || {
        build_tree(&root, &root, 0, &mut 0)
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(Some(tree))
}

fn build_tree(
    root: &std::path::Path,
    dir: &std::path::Path,
    depth: usize,
    count: &mut usize,
) -> Vec<ProjectTreeNode> {
    if depth >= MAX_TREE_DEPTH || *count >= MAX_TREE_NODES {
        return Vec::new();
    }
    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };
    let mut entries: Vec<_> = read_dir
        .filter_map(|e| e.ok())
        .map(|e| {
            let path = e.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let is_dir = e.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            (name, path, is_dir)
        })
        .collect();
    entries.sort_by(|a, b| {
        let a_dir = a.2;
        let b_dir = b.2;
        match (a_dir, b_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
        }
    });
    let mut out = Vec::new();
    for (name, path, is_dir) in entries {
        if *count >= MAX_TREE_NODES {
            break;
        }
        if name.starts_with('.') && name != ".cursor" && name != ".vscode" && name != ".kengaide" {
            continue;
        }
        let path_str = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string_lossy().into_owned());
        if is_dir {
            if SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            *count += 1;
            let children = build_tree(root, &path, depth + 1, count);
            out.push(ProjectTreeNode {
                name,
                path: path_str,
                kind: "dir".to_string(),
                children: if children.is_empty() { None } else { Some(children) },
            });
        } else {
            *count += 1;
            out.push(ProjectTreeNode {
                name,
                path: path_str,
                kind: "file".to_string(),
                children: None,
            });
        }
    }
    out
}

/// Читает содержимое файла проекта по относительному пути (для открытия в редакторе).
/// Путь не должен выходить за пределы корня проекта (path traversal запрещён).
#[tauri::command]
pub async fn read_project_file(
    state: State<'_, AppState>,
    relative_path: String,
) -> Result<String, String> {
    let root = {
        let project = state.project.read().await;
        match project.current() {
            Some(p) => p.root.clone(),
            None => return Err("Проект не открыт".to_string()),
        }
    };
    let rel = PathBuf::from(&relative_path);
    if rel.is_absolute() || rel.components().any(|c| c == std::path::Component::ParentDir) {
        return Err("Недопустимый путь".to_string());
    }
    let full = root.join(&rel);
    let root_canon = root
        .canonicalize()
        .map_err(|e| format!("Корень проекта: {}", e))?;
    let full_canon = full
        .canonicalize()
        .map_err(|e| format!("Файл не найден или недоступен: {}", e))?;
    if !full_canon.starts_with(&root_canon) {
        return Err("Путь вне проекта".to_string());
    }
    if !full_canon.is_file() {
        return Err("Не файл".to_string());
    }
    let content = tokio::task::spawn_blocking(move || std::fs::read_to_string(&full_canon))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| format!("Ошибка чтения: {}", e))?;
    Ok(content)
}

/// Записывает содержимое в файл проекта по относительному пути.
#[tauri::command]
pub async fn write_project_file(
    state: State<'_, AppState>,
    relative_path: String,
    content: String,
) -> Result<(), String> {
    let root = {
        let project = state.project.read().await;
        match project.current() {
            Some(p) => p.root.clone(),
            None => return Err("Проект не открыт".to_string()),
        }
    };
    let rel = PathBuf::from(&relative_path);
    if rel.is_absolute() || rel.components().any(|c| c == std::path::Component::ParentDir) {
        return Err("Недопустимый путь".to_string());
    }
    let full = root.join(&rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Ошибка создания папки: {}", e))?;
    }
    tokio::task::spawn_blocking(move || std::fs::write(&full, content))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| format!("Ошибка записи: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn open_project(state: State<'_, AppState>, payload: OpenProjectPayload) -> Result<(), String> {
    let path = PathBuf::from(&payload.path);
    let mut project = state.project.write().await;
    project.open(path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Открывает диалог выбора папки и открывает выбранную папку как проект.
/// Начальная директория — домашняя папка пользователя, чтобы диалог не ограничивался одним диском.
#[tauri::command]
pub async fn open_project_dialog(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let path = tokio::task::spawn_blocking(|| {
        let mut dialog = rfd::FileDialog::new();
        if let Some(home) = dirs::home_dir() {
            dialog = dialog.set_directory(home);
        }
        dialog.pick_folder()
    })
    .await
    .map_err(|e| e.to_string())?
    .map(|p| p.to_string_lossy().into_owned());
    let path = match path {
        Some(p) => p,
        None => return Ok(None),
    };
    let path_buf = PathBuf::from(&path);
    let mut project = state.project.write().await;
    project.open(path_buf).map_err(|e| e.to_string())?;
    Ok(Some(path))
}

/// Параметры для создания проекта из шаблона.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectPayload {
    pub template: String,
    pub parent_dir: Option<String>,
    pub name: Option<String>,
}

/// Создаёт проект из шаблона. При успехе автоматически открывает его.
/// template: empty | rust | python | node
/// parent_dir: если None и проект открыт — используется корень; иначе — домашняя папка
#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    payload: CreateProjectPayload,
) -> Result<String, String> {
    let parent = match payload.parent_dir.as_deref().filter(|s| !s.is_empty()) {
        Some(p) => PathBuf::from(p),
        None => {
            let project = state.project.read().await;
            match project.current() {
                Some(p) => p.root.clone(),
                None => dirs::home_dir().ok_or("Не найдена домашняя папка")?,
            }
        }
    };
    if !parent.exists() || !parent.is_dir() {
        return Err(format!("Папка не существует: {}", parent.display()));
    }
    let project_path = tokio::task::spawn_blocking({
        let template = payload.template.clone();
        let name = payload.name.clone();
        move || create_project_from_template(&template, &parent, name.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    let path_str = project_path.to_string_lossy().into_owned();
    let mut project = state.project.write().await;
    project.open(project_path).map_err(|e| e.to_string())?;
    Ok(path_str)
}

/// Возвращает список доступных шаблонов проектов.
#[tauri::command]
pub fn get_project_templates() -> Vec<&'static str> {
    TEMPLATES.to_vec()
}

/// Информация о провайдере для UI.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub available: bool,
}

/// Список AI-провайдеров и их доступность.
#[tauri::command]
pub async fn list_ai_providers(state: State<'_, AppState>) -> Result<Vec<ProviderInfo>, String> {
    let guard = state.ai_runtime.read().await;
    let mut out = Vec::new();
    for p in guard.providers() {
        let available = p.is_available().await.unwrap_or(false);
        out.push(ProviderInfo {
            id: p.id().to_string(),
            name: p.name().to_string(),
            available,
        });
    }
    Ok(out)
}

/// Текущий активный (предпочитаемый) провайдер.
#[tauri::command]
pub async fn get_active_provider(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let guard = state.ai_runtime.read().await;
    Ok(guard.preferred_provider_id().map(String::from))
}

/// Устанавливает активного провайдера по id.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetActiveProviderPayload {
    provider_id: Option<String>,
}

#[tauri::command]
pub async fn set_active_provider(
    state: State<'_, AppState>,
    payload: SetActiveProviderPayload,
) -> Result<(), String> {
    let provider_id = payload.provider_id;
    let mut guard = state.ai_runtime.write().await;
    guard.set_preferred_provider(provider_id.clone());
    drop(guard);

    let mut config = load_config();
    config.active_provider_id = provider_id;
    save_config(&config).map_err(|e| e.to_string())?;
    Ok(())
}

/// Добавляет OpenAI провайдера по API key.
#[tauri::command]
pub async fn add_openai_provider(
    state: State<'_, AppState>,
    api_key: String,
) -> Result<(), String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Err("API key не может быть пустым".to_string());
    }

    let mut guard = state.ai_runtime.write().await;
    guard.add_provider(Arc::new(ApiProvider::openai(Some(key.to_string()))));
    drop(guard);

    let mut config = load_config();
    let id = format!("openai-{}", config.providers.len());
    config.providers.push(ProviderEntry {
        id: id.clone(),
        provider_type: "openai".to_string(),
        api_key: Some(key.to_string()),
        client_id: None,
        client_secret: None,
    });
    config.active_provider_id = Some("cloud-openai".to_string());
    save_config(&config).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct AiRequestPayload {
    pub request: AiRequest,
    pub current_file_path: Option<String>,
    pub current_file_content: Option<String>,
    pub selection: Option<String>,
}

#[tauri::command]
pub async fn ai_request(state: State<'_, AppState>, payload: AiRequestPayload) -> Result<AiResponse, String> {
    state.router.validate(&payload.request).map_err(|e| e.to_string())?;

    let project_root = state
        .project
        .read()
        .await
        .current()
        .map(|p| p.root.clone());

    let current_file = payload
        .current_file_path
        .zip(payload.current_file_content)
        .map(|(path, content)| (PathBuf::from(path), content));

    let runtime = state.ai_runtime.read().await;
    let response = runtime
        .handle_request(
            payload.request.clone(),
            project_root.as_deref(),
            current_file,
            payload.selection.as_deref(),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(response)
}

/// Запускает streaming-генерацию. Сразу возвращает request_id; чанки приходят по событию "ai_chunk".
#[tauri::command]
pub async fn ai_request_stream(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    payload: AiRequestPayload,
) -> Result<String, String> {
    state.router.validate(&payload.request).map_err(|e| e.to_string())?;

    let project_root = state
        .project
        .read()
        .await
        .current()
        .map(|p| p.root.clone());

    let current_file = payload
        .current_file_path
        .zip(payload.current_file_content)
        .map(|(path, content)| (PathBuf::from(path), content));

    let options = GenerateOptions {
        temperature: None,
        max_tokens: None,
    };

    let emitter: ChunkEmitter = Arc::new({
        let app = app.clone();
        move |request_id, chunk| {
            let _ = app.emit(
                "ai_chunk",
                &AiChunkPayload {
                    request_id: request_id.to_string(),
                    chunk: chunk.clone(),
                },
            );
        }
    });

    let result = state
        .ai_controller
        .run_stream(
            payload.request,
            project_root.as_deref(),
            current_file,
            payload.selection.as_deref(),
            options,
            emitter,
        )
        .await
        .map_err(|e| e.to_string())?;

    let _ = app.emit(
        "ai_model_selected",
        &AiModelSelectedPayload {
            request_id: result.request_id.clone(),
            role: result.role.as_str().to_string(),
            model_id: result.model_id,
        },
    );

    Ok(result.request_id)
}

/// Загружает модель для указанного локального провайдера.
#[tauri::command]
pub async fn start_model_download_provider(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<(), String> {
    #[cfg(feature = "local")]
    {
        let provider = state
            .local_providers
            .iter()
            .find(|p| p.id() == provider_id)
            .ok_or_else(|| format!("Провайдер {} не найден", provider_id))?;
        let app = app.clone();
        provider
            .ensure_model(move |progress| {
                let _ = app.emit("model_download_progress", &progress);
            })
            .await
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(feature = "local"))]
    let _ = (app, state, provider_id);
    Ok(())
}

/// Останавливает генерацию по request_id.
#[tauri::command]
pub async fn ai_cancel(state: State<'_, AppState>, request_id: String) -> Result<(), String> {
    state.ai_controller.cancel(&request_id).await;
    Ok(())
}

/// Запускает агента (режим Agent): план → инструменты → проверка. Требует открытый проект.
/// Возвращает request_id; прогресс приходит по событию "agent_progress".
#[tauri::command]
pub async fn ai_agent_request(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    payload: AiRequestPayload,
) -> Result<String, String> {
    state.router.validate(&payload.request).map_err(|e| e.to_string())?;

    let (message, project_root) = match &payload.request {
        AiRequest::Agent { message } => {
            let root = state
                .project
                .read()
                .await
                .current()
                .map(|p| p.root.clone());
            (message.clone(), root)
        }
        _ => return Err("ai_agent_request requires request.type: agent".to_string()),
    };

    let project_root = project_root.ok_or("Open a project first (Agent needs project root)")?;

    let (providers, preferred_id): (Vec<std::sync::Arc<dyn AiProvider>>, Option<String>) = {
        let guard = state.ai_runtime.read().await;
        (
            guard.providers().iter().map(Arc::clone).collect(),
            guard.preferred_provider_id().map(String::from),
        )
    };

    let request_id = uuid::Uuid::new_v4().to_string();
    let request_id_for_spawn = request_id.clone();
    let app_emit = app.clone();
    let rid = request_id.clone();
    let emitter: std::sync::Arc<dyn Fn(AgentProgress) + Send + Sync> = Arc::new(move |progress| {
        let _ = app_emit.emit(
            "agent_progress",
            &AgentProgressPayload {
                request_id: rid.clone(),
                progress: progress.clone(),
            },
        );
    });

    let project_root_clone = project_root.clone();
    let preferred_for_spawn = preferred_id.clone();
    tokio::spawn(async move {
        if let Err(e) = run_agent_loop(
            &providers,
            &project_root_clone,
            &message,
            emitter,
            20,
            preferred_for_spawn.as_deref(),
        )
        .await
        {
            let _ = app.emit(
                "agent_progress",
                &AgentProgressPayload {
                    request_id: request_id_for_spawn.clone(),
                    progress: AgentProgress::Done {
                        message: format!("Error: {}", e),
                    },
                },
            );
        }
    });

    Ok(request_id)
}

/// Информация о системе для первого запуска.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemInfo {
    pub ram_gb: f64,
    pub cpu_cores: usize,
}

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    #[cfg(feature = "local")]
    {
        let ram = local_provider::hardware_detect::ram_gb()
            .map_err(|e| e.to_string())?;
        let cores = local_provider::hardware_detect::cpu_cores()
            .map_err(|e| e.to_string())?;
        return Ok(SystemInfo {
            ram_gb: ram,
            cpu_cores: cores,
        });
    }
    #[cfg(not(feature = "local"))]
    Ok(SystemInfo {
        ram_gb: 0.0,
        cpu_cores: 0,
    })
}

#[tauri::command]
pub async fn local_model_status(state: State<'_, AppState>) -> Result<LocalModelStatus, String> {
    #[cfg(feature = "local")]
    {
        for p in &state.local_providers {
            let ok = p.is_available().await.map_err(|e| e.to_string())?;
            if ok {
                return Ok(LocalModelStatus::Ready);
            }
        }
        if !state.local_providers.is_empty() {
            return Ok(LocalModelStatus::NotLoaded);
        }
    }
    #[cfg(not(feature = "local"))]
    let _ = &state;
    Ok(LocalModelStatus::NotAvailable)
}

#[tauri::command]
pub async fn local_model_info(state: State<'_, AppState>) -> Result<LocalModelInfo, String> {
    #[cfg(feature = "local")]
    {
        if let Some(ref p) = state.local_providers.first() {
            let ok = p.is_available().await.map_err(|e| e.to_string())?;
            let status = if ok {
                LocalModelStatus::Ready
            } else {
                LocalModelStatus::NotLoaded
            };
            return Ok(LocalModelInfo {
                status,
                size_gb: p.model_size_gb(),
                display_name: p.name().to_string(),
            });
        }
    }
    #[cfg(not(feature = "local"))]
    let _ = &state;
    Ok(LocalModelInfo {
        status: LocalModelStatus::NotAvailable,
        size_gb: 0.0,
        display_name: "GigaChat3 Ultra Preview".to_string(),
    })
}

#[tauri::command]
pub async fn start_model_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    #[cfg(feature = "local")]
    {
        if let Some(ref p) = state.local_providers.first() {
            let app = app.clone();
            p.ensure_model(move |progress| {
                let _ = app.emit("model_download_progress", &progress);
            })
            .await
            .map_err(|e| e.to_string())?;
        }
    }
    #[cfg(not(feature = "local"))]
    let _ = (app, state);
    Ok(())
}
