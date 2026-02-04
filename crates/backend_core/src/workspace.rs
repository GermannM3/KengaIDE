//! Workspace & Project System (BLOCK 14).
//!
//! Создание проектов из шаблонов, `.kengaide/config.json`.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

const KENGAIDE_DIR: &str = ".kengaide";
const CONFIG_FILE: &str = "config.json";

/// Поддерживаемые шаблоны проектов.
pub const TEMPLATES: &[&str] = &["empty", "rust", "python", "node"];

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("Invalid template: {0}. Use: empty, rust, python, node")]
    InvalidTemplate(String),
    #[error("Invalid name: {0}")]
    InvalidName(String),
    #[error("Path already exists: {0}")]
    PathExists(PathBuf),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Конфигурация проекта в `.kengaide/config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub template: String,
    pub created: String,
}

impl ProjectConfig {
    pub fn new(name: impl Into<String>, template: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            template: template.into(),
            created: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Создаёт проект из шаблона в `parent_path`.
/// Возвращает путь к созданной папке проекта.
///
/// `name` — имя папки проекта. Если None — используется `{template}-project`.
pub fn create_project_from_template(
    template: &str,
    parent_path: &Path,
    name: Option<&str>,
) -> Result<PathBuf, WorkspaceError> {
    if !TEMPLATES.contains(&template) {
        return Err(WorkspaceError::InvalidTemplate(template.to_string()));
    }

    let project_name = name
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| format!("{}-project", template));

    if project_name.contains(std::path::MAIN_SEPARATOR)
        || project_name.contains('/')
        || project_name == "."
        || project_name == ".."
    {
        return Err(WorkspaceError::InvalidName(project_name));
    }

    let project_path = parent_path.join(&project_name);
    if project_path.exists() {
        return Err(WorkspaceError::PathExists(project_path));
    }

    std::fs::create_dir_all(&project_path)?;

    let kengaide_dir = project_path.join(KENGAIDE_DIR);
    std::fs::create_dir_all(&kengaide_dir)?;

    let config = ProjectConfig::new(&project_name, template);
    let config_json = serde_json::to_string_pretty(&config).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;
    std::fs::write(kengaide_dir.join(CONFIG_FILE), config_json)?;

    match template {
        "empty" => {}
        "rust" => create_rust_template(&project_path)?,
        "python" => create_python_template(&project_path)?,
        "node" => create_node_template(&project_path)?,
        _ => {}
    }

    Ok(project_path)
}

fn create_rust_template(root: &Path) -> std::io::Result<()> {
    let src = root.join("src");
    std::fs::create_dir_all(&src)?;

    std::fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )?;

    std::fs::write(
        src.join("main.rs"),
        r#"fn main() {
    println!("Hello, world!");
}
"#,
    )?;

    Ok(())
}

fn create_python_template(root: &Path) -> std::io::Result<()> {
    std::fs::write(root.join("requirements.txt"), "")?;

    std::fs::write(
        root.join("main.py"),
        r#"def main():
    print("Hello, world!")


if __name__ == "__main__":
    main()
"#,
    )?;

    Ok(())
}

fn create_node_template(root: &Path) -> std::io::Result<()> {
    std::fs::write(
        root.join("package.json"),
        r#"{
  "name": "app",
  "version": "1.0.0",
  "main": "index.js"
}
"#,
    )?;

    std::fs::write(
        root.join("index.js"),
        r#"console.log("Hello, world!");
"#,
    )?;

    Ok(())
}

/// Проверяет/создаёт `.kengaide/` в корне проекта при открытии.
/// Если config.json отсутствует — создаёт минимальный.
pub fn ensure_workspace_dir(project_root: &Path) -> std::io::Result<()> {
    let kengaide = project_root.join(KENGAIDE_DIR);
    std::fs::create_dir_all(&kengaide)?;

    let config_path = kengaide.join(CONFIG_FILE);
    if !config_path.exists() {
        let name = project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string();
        let config = ProjectConfig::new(&name, "empty");
        let json = serde_json::to_string_pretty(&config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(config_path, json)?;
    }

    Ok(())
}
