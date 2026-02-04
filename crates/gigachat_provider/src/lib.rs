//! GigaChat Provider — официальный API Sber.

mod auth;
mod client;
mod error;
mod models;
mod provider;

pub use error::GigaChatError;
pub use provider::GigaChatProvider;
