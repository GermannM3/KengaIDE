//! Ошибки GigaChat Provider.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GigaChatError {
    #[error("OAuth failed: {0}")]
    OAuth(String),

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Timeout")]
    Timeout,
}
