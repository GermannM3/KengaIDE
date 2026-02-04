//! Rollback патчей: замена after → before (обратное apply_patch).

use std::path::Path;

/// Откатывает один патч: в файле заменяет `after` на `before`.
pub fn rollback_patch(project_root: &Path, path: &str, before: &str, after: &str) -> Result<(), String> {
    let full = project_root.join(path.trim_start_matches('/'));
    if !full.starts_with(project_root) {
        return Err("path outside project".to_string());
    }
    let current = std::fs::read_to_string(&full).map_err(|e| e.to_string())?;
    let count = current.matches(after).count();
    if count == 0 {
        return Err("after block not found in file (cannot rollback)".to_string());
    }
    if count > 1 {
        return Err(format!("ambiguous rollback: after occurs {} times", count));
    }
    let new_content = current.replacen(after, before, 1);
    std::fs::write(&full, &new_content).map_err(|e| e.to_string())?;
    Ok(())
}
