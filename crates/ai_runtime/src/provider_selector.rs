//! ProviderSelector — выбор провайдера по роли (E5) или режиму.
//!
//! Task → Role → Model → Provider. Модель НЕ выбирает себя сама.

use ai_providers::{AiMode, AiProvider};
use std::path::Path;
use std::sync::Arc;

use crate::error::AiRuntimeError;
use crate::orchestration::{load_model_roles, RoleResolver, TaskClassifier, TaskRole};

/// Результат выбора провайдера с метаданными для UI и логов.
#[derive(Clone)]
pub struct ProviderSelection {
    pub provider: Arc<dyn AiProvider>,
    pub role: TaskRole,
    pub model_id: String,
    pub policy_source: Option<String>,
}

/// Выбирает провайдера по role-based orchestration (E5).
pub struct ProviderSelector;

impl ProviderSelector {
    /// Выбирает провайдера: TaskClassifier → RoleResolver → provider по model_id.
    /// preferred_id — fallback при недоступности целевой модели.
    pub async fn select(
        providers: &[Arc<dyn AiProvider>],
        mode: AiMode,
        user_message: &str,
        preferred_id: Option<&str>,
        project_root: Option<&Path>,
    ) -> Result<ProviderSelection, AiRuntimeError> {
        let role = TaskClassifier::classify(mode, user_message);
        let model_roles = load_model_roles(project_root);
        let target_model_id = RoleResolver::resolve(role, &model_roles);

        let mut order: Vec<usize> = (0..providers.len()).collect();
        if let Some(pid) = preferred_id {
            if let Some(idx) = providers.iter().position(|p| p.id() == pid) {
                if let Some(pos) = order.iter().position(|&i| i == idx) {
                    order.remove(pos);
                    order.insert(0, idx);
                }
            }
        }

        for &idx in &order {
            let provider = &providers[idx];
            if !provider.capabilities().modes.contains(&mode) {
                continue;
            }
            let available = provider
                .is_available()
                .await
                .map_err(AiRuntimeError::Provider)?;
            if !available {
                continue;
            }
            if let Some(mid) = provider.model_id() {
                if mid == target_model_id {
                    return Ok(ProviderSelection {
                        provider: Arc::clone(provider),
                        role,
                        model_id: target_model_id,
                        policy_source: Some("model_roles".to_string()),
                    });
                }
            }
        }

        for &idx in &order {
            let provider = &providers[idx];
            if !provider.capabilities().modes.contains(&mode) {
                continue;
            }
            let available = provider
                .is_available()
                .await
                .map_err(AiRuntimeError::Provider)?;
            if available {
                let model_id = provider
                    .model_id()
                    .map(String::from)
                    .unwrap_or_else(|| provider.name().to_string());
                return Ok(ProviderSelection {
                    provider: Arc::clone(provider),
                    role,
                    model_id,
                    policy_source: Some("fallback".to_string()),
                });
            }
        }

        Err(AiRuntimeError::NoProvider)
    }

    /// Legacy: выбор без orchestration (для обратной совместимости).
    #[allow(dead_code)]
    pub async fn select_legacy(
        providers: &[Arc<dyn AiProvider>],
        mode: AiMode,
        preferred_id: Option<&str>,
    ) -> Result<Arc<dyn AiProvider>, AiRuntimeError> {
        let mut order: Vec<usize> = (0..providers.len()).collect();
        if let Some(pid) = preferred_id {
            if let Some(idx) = providers.iter().position(|p| p.id() == pid) {
                if let Some(pos) = order.iter().position(|&i| i == idx) {
                    order.remove(pos);
                    order.insert(0, idx);
                }
            }
        }
        for &idx in &order {
            let provider = &providers[idx];
            if !provider.capabilities().modes.contains(&mode) {
                continue;
            }
            let available = provider
                .is_available()
                .await
                .map_err(AiRuntimeError::Provider)?;
            if available {
                return Ok(Arc::clone(provider));
            }
        }
        Err(AiRuntimeError::NoProvider)
    }
}
