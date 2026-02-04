//! Логирование в .kengaide/logs/ для отладки.
//!
//! Файлы: agent.log, mcp.log, runtime.log

use std::path::Path;

/// Возвращает путь к директории логов. Приоритет: project_root/.kengaide/logs, иначе ~/.kengaide/logs.
pub fn logs_dir(project_root: Option<&Path>) -> std::path::PathBuf {
    if let Some(root) = project_root {
        if root.exists() {
            return root.join(".kengaide").join("logs");
        }
    }
    dirs::home_dir()
        .map(|h| h.join(".kengaide").join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from(".kengaide").join("logs"))
}

/// Создаёт директорию логов при необходимости.
pub fn ensure_logs_dir(project_root: Option<&Path>) -> std::io::Result<std::path::PathBuf> {
    let dir = logs_dir(project_root);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Записывает строку в файл лога (append).
pub fn append_log(project_root: Option<&Path>, filename: &str, line: &str) {
    let dir = match ensure_logs_dir(project_root) {
        Ok(d) => d,
        Err(_) => return,
    };
    let path = dir.join(filename);
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let full_line = format!("[{}] {}\n", timestamp, line);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, full_line.as_bytes()));
}
