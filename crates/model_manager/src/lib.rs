//! Model Manager — загрузка, хранение, версии моделей.
//!
//! Работает только через AI Runtime. Не общается с UI напрямую.

mod manager;

pub use manager::{ModelInfo, ModelManager, ModelManagerError};
