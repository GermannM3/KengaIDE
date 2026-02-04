//! Backend Core — шина между UI и AI.
//!
//! Ответственность: FS, проекты, git, LSP (перспектива), маршрутизация команд, IPC.

pub mod audit;
pub mod command_router;
pub mod fs;
pub mod git;
pub mod logs;
pub mod patch;
pub mod project;
pub mod workspace;

pub use command_router::CommandRouter;
pub use fs::FsService;
pub use git::GitService;
pub use project::ProjectService;
pub use audit::{
    append_audit_event, audit_dir, current_environment, ensure_audit_dir, finish_session_meta,
    list_sessions, read_session_events, save_session_meta, AuditEnvironment, AuditEvent,
    AuditSessionMeta,
};
pub use logs::{append_log, ensure_logs_dir, logs_dir};
pub use patch::rollback_patch;
pub use workspace::{
    create_project_from_template, ensure_workspace_dir, ProjectConfig, WorkspaceError, TEMPLATES,
};