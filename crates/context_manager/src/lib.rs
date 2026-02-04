//! Context Manager — сбор контекста для AI.
//!
//! Ответственность: текущий файл, выделение, дерево проекта, лимиты, грубая токенизация.

mod context;

pub use context::{Context, ContextBuilder, ContextError, ContextLimits};
