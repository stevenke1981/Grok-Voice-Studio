mod commands;
mod log_store;
mod services;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::create_new_project,
            commands::open_project,
            commands::save_current_project,
            commands::autosave_project,
            commands::list_recent_projects,
            commands::parse_script_command,
            commands::convert_story,
            commands::update_project,
            commands::get_project,
            commands::get_project_stats,
            commands::sync_voices,
            commands::generate_segment,
            commands::start_generate_job,
            commands::cancel_generate_job,
            commands::pause_generate_job,
            commands::resume_generate_job,
            commands::generate_all,
            commands::split_segment,
            commands::add_character,
            commands::delete_character,
            commands::preview_voice,
            commands::get_audio_src,
            commands::export_mixdown,
            commands::cleanup_cache,
            commands::export_debug_bundle,
            commands::get_logs,
            commands::clear_logs,
            commands::verify_api_key,
            commands::list_sfx_library,
            commands::preview_sfx,
            commands::import_sfx_file,
            commands::list_custom_voices,
            commands::create_custom_voice,
            commands::delete_custom_voice,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}