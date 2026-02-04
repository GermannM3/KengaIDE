//! BLOCK E5 — Enterprise Model Orchestration.
//!
//! Task → Role → Model (policy-controlled). Модель НЕ выбирает себя сама.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use ai_providers::AiMode;

/// Роль задачи для выбора модели.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskRole {
    Chat,
    Coding,
    Planning,
    Analysis,
    Documentation,
}

impl TaskRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskRole::Chat => "chat",
            TaskRole::Coding => "coding",
            TaskRole::Planning => "planning",
            TaskRole::Analysis => "analysis",
            TaskRole::Documentation => "documentation",
        }
    }
}

/// Конфигурация роли в model_roles.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConfig {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// model_roles.json — ролевая карта моделей.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRolesConfig {
    #[serde(default = "default_model")]
    pub default: String,
    pub roles: HashMap<String, RoleConfig>,
}

fn default_model() -> String {
    "gigachat3".to_string()
}

/// Детерминированный классификатор: mode + message → role.
/// Rule-based, без LLM.
pub struct TaskClassifier;

impl TaskClassifier {
    /// Определяет роль по режиму и тексту задачи.
    pub fn classify(mode: AiMode, message: &str) -> TaskRole {
        let msg_lower = message.to_lowercase();
        let msg = msg_lower.as_str();

        match mode {
            AiMode::Chat => TaskRole::Chat,
            AiMode::Explain => {
                if msg.contains("refactor") || msg.contains("рефактор") {
                    TaskRole::Coding
                } else if msg.contains("log") || msg.contains("trace") || msg.contains("stack") {
                    TaskRole::Analysis
                } else {
                    TaskRole::Chat
                }
            }
            AiMode::Refactor => TaskRole::Coding,
            AiMode::Generate => TaskRole::Coding,
            AiMode::Agent => {
                if msg.contains("refactor")
                    || msg.contains("diff")
                    || msg.contains("код")
                    || msg.contains("code")
                    || msg.contains("create")
                    || msg.contains("создай")
                    || msg.contains("добавь")
                    || msg.contains("implement")
                    || msg.contains("исправь")
                    || msg.contains("fix")
                {
                    TaskRole::Coding
                } else if msg.contains("plan")
                    || msg.contains("план")
                    || msg.contains("разбей")
                    || msg.contains("decompose")
                {
                    TaskRole::Planning
                } else if msg.contains("log") || msg.contains("trace") || msg.contains("analyze") {
                    TaskRole::Analysis
                } else if msg.contains("doc") || msg.contains("readme") || msg.contains("документ") {
                    TaskRole::Documentation
                } else {
                    TaskRole::Planning
                }
            }
        }
    }
}

/// Резолвер: role → model_id с учётом model_roles и policy.
pub struct RoleResolver;

impl RoleResolver {
    /// Возвращает model_id для роли. Использует config; при отсутствии — default.
    pub fn resolve(role: TaskRole, config: &ModelRolesConfig) -> String {
        let role_str = role.as_str();
        config
            .roles
            .get(role_str)
            .map(|r| r.model.clone())
            .unwrap_or_else(|| config.default.clone())
    }
}

/// Загружает model_roles.json из ~/.kengaide/ или project_root/.kengaide/.
pub fn load_model_roles(project_root: Option<&Path>) -> ModelRolesConfig {
    let paths = [
        project_root.map(|p| p.join(".kengaide").join("model_roles.json")),
        dirs::home_dir().map(|h| h.join(".kengaide").join("model_roles.json")),
    ];
    for path in paths.into_iter().flatten() {
        if let Ok(s) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<ModelRolesConfig>(&s) {
                return cfg;
            }
        }
    }
    default_model_roles()
}

/// Дефолтная конфигурация ролей.
pub fn default_model_roles() -> ModelRolesConfig {
    let mut roles = HashMap::new();
    roles.insert(
        "chat".to_string(),
        RoleConfig {
            model: "gigachat3".to_string(),
            reason: Some("dialog, explanations, legal-safe".to_string()),
        },
    );
    roles.insert(
        "coding".to_string(),
        RoleConfig {
            model: "deepseek-coder".to_string(),
            reason: Some("code generation, refactor, diff".to_string()),
        },
    );
    roles.insert(
        "planning".to_string(),
        RoleConfig {
            model: "gigachat3".to_string(),
            reason: Some("task decomposition, reasoning".to_string()),
        },
    );
    roles.insert(
        "analysis".to_string(),
        RoleConfig {
            model: "deepseek-coder".to_string(),
            reason: Some("log analysis, stack traces".to_string()),
        },
    );
    roles.insert(
        "documentation".to_string(),
        RoleConfig {
            model: "gigachat3".to_string(),
            reason: Some("human-readable docs".to_string()),
        },
    );
    ModelRolesConfig {
        default: "gigachat3".to_string(),
        roles,
    }
}

/// Создаёт дефолтный model_roles.json в ~/.kengaide/ при первом запуске.
pub fn ensure_model_roles_config() -> std::path::PathBuf {
    let path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".kengaide")
        .join("model_roles.json");
    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let cfg = default_model_roles();
        if let Ok(json) = serde_json::to_string_pretty(&cfg) {
            let _ = std::fs::write(&path, json);
        }
    }
    path
}
