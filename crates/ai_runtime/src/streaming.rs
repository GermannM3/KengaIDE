//! Заглушки для streaming и tools.
//!
//! Структура подготовлена для будущей реализации. Не реализовано.

/// Конфигурация стриминга (заглушка).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StreamingConfig {
    /// Включён ли стриминг.
    pub enabled: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

/// Конфигурация tools (заглушка).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ToolsConfig {
    /// Включены ли tools.
    pub enabled: bool,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}
