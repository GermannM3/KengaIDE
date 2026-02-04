//! Токенизация через модель (llama.cpp).

/// Грубая оценка токенов: ~4 символа на токен.
pub fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}
