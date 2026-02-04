//! ToolExecutor — выполнение вызовов инструментов в контексте проекта.
//!
//! Для изменения существующих файлов используется только apply_patch.
//! update_file оставлен для совместимости, но в режиме Agent не используется (deprecated).
//! create_project — создание проекта из шаблона (BLOCK 14).

use std::path::PathBuf;

use backend_core::{create_project_from_template, WorkspaceError};

use crate::types::{PatchError, ToolCall, ToolResult};

/// Выполняет вызовы инструментов. Все пути относительно project_root.
pub struct ToolExecutor {
    project_root: PathBuf,
}

impl ToolExecutor {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Выполняет один вызов инструмента.
    pub fn run(&self, call: &ToolCall) -> ToolResult {
        match call.name.as_str() {
            "create_file" => self.create_file(call),
            "read_file" => self.read_file(call),
            "list_files" => self.list_files(call),
            "apply_patch" => self.apply_patch(call),
            "update_file" => self.update_file(call),
            "create_project" => self.create_project(call),
            _ => ToolResult::err(format!("Unknown tool: {}", call.name)),
        }
    }

    /// Создаёт проект из шаблона в project_root/name.
    /// template: empty | rust | python | node
    /// name: опционально, иначе {template}-project
    fn create_project(&self, call: &ToolCall) -> ToolResult {
        let template = match call.arguments.get("template").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => return ToolResult::err("create_project: missing 'template'"),
        };
        let name = call.arguments.get("name").and_then(|v| v.as_str());
        match create_project_from_template(template, &self.project_root, name) {
            Ok(path) => {
                let rel = path
                    .strip_prefix(&self.project_root)
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| path.to_string_lossy().into_owned());
                ToolResult::ok(format!("Created project at {}", rel))
            }
            Err(WorkspaceError::PathExists(path)) => {
                let rel = path
                    .strip_prefix(&self.project_root)
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| path.to_string_lossy().into_owned());
                ToolResult::ok(format!(
                    "Project already exists at {}. Use list_files(\"{}\") and read_file(\"{{path}}\") to work with it.",
                    rel, rel
                ))
            }
            Err(e) => ToolResult::err(format!("create_project: {}", e)),
        }
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = path.trim_start_matches('/');
        self.project_root.join(path)
    }

    fn create_file(&self, call: &ToolCall) -> ToolResult {
        let path = match call.arguments.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::err("create_file: missing 'path'"),
        };
        let content = match call.arguments.get("content").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult::err("create_file: missing 'content'"),
        };
        let full = self.resolve_path(path);
        if let Some(parent) = full.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return ToolResult::err(format!("create_file: mkdir failed: {}", e));
            }
        }
        match std::fs::write(&full, content) {
            Ok(()) => ToolResult::ok(format!("Created {}", path)),
            Err(e) => ToolResult::err(format!("create_file: {}", e)),
        }
    }

    fn read_file(&self, call: &ToolCall) -> ToolResult {
        let path = match call.arguments.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::err("read_file: missing 'path'"),
        };
        let full = self.resolve_path(path);
        if !full.starts_with(&self.project_root) {
            return ToolResult::err("read_file: path outside project");
        }
        match std::fs::read_to_string(&full) {
            Ok(s) => ToolResult::ok(s),
            Err(e) => ToolResult::err(format!("read_file: {}", e)),
        }
    }

    fn list_files(&self, call: &ToolCall) -> ToolResult {
        let path = call
            .arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        let full = self.resolve_path(path);
        if !full.starts_with(&self.project_root) {
            return ToolResult::err("list_files: path outside project");
        }
        if !full.is_dir() {
            return ToolResult::err("list_files: not a directory");
        }
        let mut entries = Vec::new();
        match std::fs::read_dir(&full) {
            Ok(rd) => {
                for e in rd.flatten() {
                    let name = e.file_name().to_string_lossy().into_owned();
                    let kind = if e.path().is_dir() { "dir" } else { "file" };
                    entries.push(format!("{} ({})", name, kind));
                }
            }
            Err(e) => return ToolResult::err(format!("list_files: {}", e)),
        }
        entries.sort();
        ToolResult::ok(entries.join("\n"))
    }

    /// Применяет контекстный diff: ищет точное вхождение `before`, заменяет на `after`.
    /// Файл не изменяется при 0 или >1 вхождений (структурированные ошибки для агента).
    fn apply_patch(&self, call: &ToolCall) -> ToolResult {
        let path = match call.arguments.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::err("apply_patch: missing 'path'"),
        };
        let before = match call.arguments.get("before").and_then(|v| v.as_str()) {
            Some(b) => b,
            None => return ToolResult::err("apply_patch: missing 'before'"),
        };
        let after = match call.arguments.get("after").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => return ToolResult::err("apply_patch: missing 'after'"),
        };
        let full = self.resolve_path(path);
        if !full.starts_with(&self.project_root) {
            return ToolResult::err("apply_patch: path outside project");
        }

        let current = match std::fs::read_to_string(&full) {
            Ok(s) => s,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return ToolResult::err(
                        PatchError::FileNotFound {
                            path: path.to_string(),
                        }
                        .message_for_agent(),
                    );
                }
                return ToolResult::err(
                    PatchError::IoError {
                        path: path.to_string(),
                        detail: e.to_string(),
                    }
                    .message_for_agent(),
                );
            }
        };

        let count = current.matches(before).count();
        if count == 0 {
            return ToolResult::err(
                PatchError::BeforeBlockNotFound {
                    path: path.to_string(),
                    detail: "exact 'before' block not found in file".to_string(),
                }
                .message_for_agent(),
            );
        }
        if count > 1 {
            return ToolResult::err(
                PatchError::AmbiguousPatch {
                    path: path.to_string(),
                    count: count as u32,
                }
                .message_for_agent(),
            );
        }

        let new_content = current.replacen(before, after, 1);
        match std::fs::write(&full, &new_content) {
            Ok(()) => ToolResult::ok(format!("Patched {}", path)),
            Err(e) => ToolResult::err(
                PatchError::IoError {
                    path: path.to_string(),
                    detail: e.to_string(),
                }
                .message_for_agent(),
            ),
        }
    }

    /// Полная перезапись файла. В режиме Agent не используется — только apply_patch.
    fn update_file(&self, call: &ToolCall) -> ToolResult {
        let path = match call.arguments.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::err("update_file: missing 'path'"),
        };
        let content = match call.arguments.get("content").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult::err("update_file: missing 'content'"),
        };
        let full = self.resolve_path(path);
        if !full.starts_with(&self.project_root) {
            return ToolResult::err("update_file: path outside project");
        }
        match std::fs::write(&full, content) {
            Ok(()) => ToolResult::ok(format!("Updated {}", path)),
            Err(e) => ToolResult::err(format!("update_file: {}", e)),
        }
    }
}
