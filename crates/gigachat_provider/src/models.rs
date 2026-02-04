//! Модели GigaChat API.


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GigaChatModel {
    GigaChatUltra,
}

impl GigaChatModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            GigaChatModel::GigaChatUltra => "GigaChat-Ultra",
        }
    }
}
