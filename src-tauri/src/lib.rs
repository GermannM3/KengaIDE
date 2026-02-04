//! KengaIDE — точка входа, инициализация модулей, Tauri commands.

mod ai_config;
mod commands;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::open_project,
            commands::open_project_dialog,
            commands::create_project,
            commands::get_project_templates,
            commands::get_system_info,
            commands::list_ai_providers,
            commands::get_active_provider,
            commands::set_active_provider,
            commands::add_openai_provider,
            commands::pick_folder,
            commands::open_mcp_config_folder,
            commands::open_logs_folder,
            commands::rollback_patches,
            commands::list_audit_sessions,
            commands::get_audit_events,
            commands::open_audit_folder,
            commands::get_project_path,
            commands::get_project_tree,
            commands::read_project_file,
            commands::write_project_file,
            commands::ai_request,
            commands::ai_request_stream,
            commands::ai_agent_request,
            commands::ai_cancel,
            commands::local_model_status,
            commands::local_model_info,
            commands::start_model_download,
            commands::start_model_download_provider,
            commands::git_status,
            commands::get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running KengaIDE");
}
