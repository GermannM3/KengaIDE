//! Agent Tools — инструменты для IDE-агента.
//!
//! Минимум: create_file, read_file, list_files, update_file.
//! Все пути относительно project_root.

mod executor;
mod types;

pub use executor::ToolExecutor;
pub use types::{PatchError, ToolCall, ToolResult};
