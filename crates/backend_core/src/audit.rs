//! BLOCK E6 — Audit / Replay / Explainability.
//!
//! Append-only event sourcing. session_id генерируется до первого токена.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Директория аудита: project_root/.kengaide/audit или ~/.kengaide/audit.
pub fn audit_dir(project_root: Option<&Path>) -> std::path::PathBuf {
    if let Some(root) = project_root {
        if root.exists() {
            return root.join(".kengaide").join("audit");
        }
    }
    dirs::home_dir()
        .map(|h| h.join(".kengaide").join("audit"))
        .unwrap_or_else(|| std::path::PathBuf::from(".kengaide").join("audit"))
}

/// Создаёт директорию аудита.
pub fn ensure_audit_dir(project_root: Option<&Path>) -> std::io::Result<std::path::PathBuf> {
    let dir = audit_dir(project_root);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// События аудита (append-only JSONL).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditEvent {
    SessionStart {
        session_id: String,
        mode: String,
        task: String,
        policy: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        environment: Option<AuditEnvironment>,
    },
    TaskClassified {
        role: String,
        reason: String,
    },
    ModelSelected {
        role: String,
        model: String,
        provider: String,
    },
    PromptSent {
        #[serde(skip_serializing_if = "Option::is_none")]
        tokens: Option<usize>,
    },
    StreamChunk {
        size: usize,
    },
    ToolCall {
        tool: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    ToolResult {
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        output_len: Option<usize>,
    },
    PatchApplied {
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        hash: Option<String>,
    },
    Error {
        message: String,
    },
    SessionEnd {
        status: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEnvironment {
    pub os: String,
    pub arch: String,
    pub offline: bool,
}

/// Пишет событие в audit_events.jsonl (append-only).
pub fn append_audit_event(project_root: Option<&Path>, session_id: &str, event: &AuditEvent) {
    let dir = match ensure_audit_dir(project_root) {
        Ok(d) => d,
        Err(_) => return,
    };
    let path = dir.join(format!("{}.jsonl", session_id));
    let line = match serde_json::to_string(event) {
        Ok(s) => format!("{}\n", s),
        Err(_) => return,
    };
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
}

/// Метаданные сессии (audit_session.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSessionMeta {
    pub session_id: String,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    pub mode: String,
    pub task: String,
    pub status: String,
    pub policy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<AuditEnvironment>,
}

/// Сохраняет метаданные сессии.
pub fn save_session_meta(project_root: Option<&Path>, meta: &AuditSessionMeta) {
    let dir = match ensure_audit_dir(project_root) {
        Ok(d) => d,
        Err(_) => return,
    };
    let path = dir.join(format!("{}_meta.json", meta.session_id));
    if let Ok(json) = serde_json::to_string_pretty(meta) {
        let _ = std::fs::write(&path, json);
    }
}

/// Обновляет ended_at и status в метаданных.
pub fn finish_session_meta(
    project_root: Option<&Path>,
    session_id: &str,
    status: &str,
) {
    let dir = match ensure_audit_dir(project_root) {
        Ok(d) => d,
        Err(_) => return,
    };
    let path = dir.join(format!("{}_meta.json", session_id));
    if let Ok(s) = std::fs::read_to_string(&path) {
        if let Ok(mut meta) = serde_json::from_str::<AuditSessionMeta>(&s) {
            meta.ended_at = Some(chrono::Utc::now().to_rfc3339());
            meta.status = status.to_string();
            if let Ok(json) = serde_json::to_string_pretty(&meta) {
                let _ = std::fs::write(&path, json);
            }
        }
    }
}

/// Читает события сессии для Replay.
pub fn read_session_events(
    project_root: Option<&Path>,
    session_id: &str,
) -> std::io::Result<Vec<AuditEvent>> {
    let dir = audit_dir(project_root);
    let path = dir.join(format!("{}.jsonl", session_id));
    let content = std::fs::read_to_string(&path)?;
    let mut events = Vec::new();
    for line in content.lines() {
        if line.is_empty() {
            continue;
        }
        if let Ok(ev) = serde_json::from_str::<AuditEvent>(line) {
            events.push(ev);
        }
    }
    Ok(events)
}

/// Текущая среда для audit.
pub fn current_environment() -> AuditEnvironment {
    AuditEnvironment {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        offline: false,
    }
}

/// Список session_id в директории аудита.
pub fn list_sessions(project_root: Option<&Path>) -> Vec<String> {
    let dir = audit_dir(project_root);
    let mut ids = std::collections::HashSet::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.ends_with(".jsonl") {
                if let Some(id) = name.strip_suffix(".jsonl") {
                    ids.insert(id.to_string());
                }
            }
        }
    }
    let mut list: Vec<String> = ids.into_iter().collect();
    list.sort();
    list.reverse();
    list
}
