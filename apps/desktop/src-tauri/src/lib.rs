mod commands;
mod events;
mod log_store;
mod services;
mod state;

use grok_voice_core::{install_retry_hook, install_story_convert_hook};
use state::AppState;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<AppState>();
            let logs = state.logs.clone();
            let active_job = state.active_job_id.clone();
            let batch_tts_retries = state.batch_tts_retries.clone();

            install_story_convert_hook(Box::new({
                let handle = handle.clone();
                move |progress| {
                    let _ = handle.emit(
                        "story-convert-progress",
                        events::StoryConvertProgressEvent {
                            attempt: progress.attempt,
                            phase: progress.phase.to_string(),
                            message: progress.message,
                        },
                    );
                }
            }));

            install_retry_hook(Box::new(move |notification| {
                let subject = events::retry_subject(
                    notification.context.as_deref(),
                    notification.category,
                );
                let message = format!(
                    "{subject} 重試 {}/{}，等待 {}s: {}",
                    notification.attempt,
                    notification.max_retries,
                    notification.delay_secs,
                    notification.error
                );
                logs.append_with_emit(&handle, "warn", "xai_retry", message.clone());

                let _ = handle.emit(
                    "api-retry",
                    events::ApiRetryEvent {
                        category: notification.category.to_string(),
                        context: notification.context.clone(),
                        message: message.clone(),
                        attempt: notification.attempt,
                        max_retries: notification.max_retries,
                        delay_secs: notification.delay_secs,
                    },
                );

                if notification.category == "chat"
                    && notification.context.as_deref() == Some("story")
                {
                    let _ = handle.emit(
                        "story-convert-progress",
                        events::StoryConvertProgressEvent {
                            attempt: notification.attempt + 1,
                            phase: "api_retry".into(),
                            message: Some(message.clone()),
                        },
                    );
                }

                if notification.category == "tts" {
                    batch_tts_retries.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if let Ok(guard) = active_job.lock() {
                        if let Some(job_id) = guard.as_deref() {
                            let _ = handle.emit(
                                "generate-progress",
                                services::GenerateProgressEvent {
                                    job_id: job_id.to_string(),
                                    current: 0,
                                    total: 0,
                                    segment_id: notification.context.clone(),
                                    segment: None,
                                    status: "retrying".into(),
                                    error: Some(message),
                                    cached: false,
                                    retry_count: None,
                                    suggested_concurrency: None,
                                },
                            );
                        }
                    }
                }
            }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::apply_concurrency_suggestion,
            commands::dismiss_concurrency_suggestion,
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