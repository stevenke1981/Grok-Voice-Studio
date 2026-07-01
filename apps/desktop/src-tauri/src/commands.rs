use grok_voice_audio::{
    concat_segments, extension_for_codec, write_subtitle, FfmpegConfig,
};
use grok_voice_core::{
    apply_parsed_script, apply_story_script, build_timeline_from_segments, parse_script,
    AppError, Character, ExportOptions, ExportResult, Project, RoleType, ScriptSegment,
    SegmentKind, SegmentStatus, SfxCategory, SoundEffect, SubtitleFormat, VoiceInfo,
};
use grok_voice_storage::{has_api_key, load_api_key, save_api_key, SettingsStore, SfxStore};
use std::collections::HashMap;
use grok_voice_xai::{
    fallback_voices, story_to_script_with_retry, CreateCustomVoiceRequest, TtsProvider,
    XaiChatClient, XaiTtsProvider,
};
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::log_store::{log_error, log_info, log_warn, LogEntry};
use crate::services::{GenerationService, ProjectService};
use crate::state::AppState;

fn err(e: AppError) -> String {
    e.to_string()
}

#[tauri::command]
pub async fn get_settings() -> Result<serde_json::Value, String> {
    let store = SettingsStore::open().map_err(err)?;
    let settings = store.load().map_err(err)?;
    Ok(serde_json::json!({
        "has_api_key": has_api_key(),
        "ffmpeg_path": settings.ffmpeg_path,
        "default_language": settings.default_language.unwrap_or_else(|| "zh".into()),
        "auto_save": settings.auto_save,
        "generation_concurrency": settings.generation_concurrency,
        "cost_per_1k_chars": settings.cost_per_1k_chars,
        "onboarding_done": settings.onboarding_done,
        "ui_language": settings.ui_language,
    }))
}

#[tauri::command]
pub async fn save_settings(
    api_key: Option<String>,
    ffmpeg_path: Option<String>,
    default_language: Option<String>,
    auto_save: bool,
    generation_concurrency: Option<u32>,
    cost_per_1k_chars: Option<f64>,
    onboarding_done: Option<bool>,
    ui_language: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let Some(key) = api_key.filter(|k| !k.is_empty()) {
        save_api_key(&key).map_err(err)?;
        log_info(&state.logs, "settings", "API Key 已儲存至系統金鑰庫");
    }
    let store = SettingsStore::open().map_err(err)?;
    let mut settings = store.load().map_err(err)?;
    settings.ffmpeg_path = ffmpeg_path;
    settings.default_language = default_language;
    settings.auto_save = auto_save;
    if let Some(c) = generation_concurrency {
        settings.generation_concurrency = c.max(1);
    }
    settings.cost_per_1k_chars = cost_per_1k_chars;
    if let Some(o) = onboarding_done {
        settings.onboarding_done = o;
    }
    if let Some(lang) = ui_language {
        settings.ui_language = lang;
    }
    store.save(&settings).map_err(err)?;
    log_info(&state.logs, "settings", "設定已儲存");
    Ok(())
}

#[tauri::command]
pub async fn create_new_project(
    title: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let project = state.project_svc.create(&title, &path).await.map_err(err)?;
    Ok(project)
}

#[tauri::command]
pub async fn open_project(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    let mut project = state.project_svc.open(&path).await.map_err(err)?;
    if let Ok(guard) = state.cache.lock() {
        if let Some(c) = guard.as_ref() {
            let _ = c.register_recent_project(&project.id, &project.title, &path);
        }
    }
    if let Some(paths) = state.project_svc.paths().await {
        ProjectService::normalize_segment_paths(&mut project, &paths);
        state.project_svc.set(project.clone()).await;
    }
    Ok(project)
}

#[tauri::command]
pub async fn save_current_project(state: State<'_, AppState>) -> Result<(), String> {
    state.project_svc.save().await.map_err(err)?;
    if let (Some(project), Some(paths)) = (
        state.project_svc.get().await,
        state.project_svc.paths().await,
    ) {
        if let Ok(guard) = state.cache.lock() {
            if let Some(c) = guard.as_ref() {
                let _ = c.register_recent_project(
                    &project.id,
                    &project.title,
                    &paths.root.display().to_string(),
                );
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn autosave_project(state: State<'_, AppState>) -> Result<(), String> {
    state.project_svc.save_backup().await.map_err(err)?;
    state.project_svc.save().await.map_err(err)
}

#[tauri::command]
pub async fn list_recent_projects(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let cache = state.cache.lock().map_err(|e| e.to_string())?;
    if let Some(c) = cache.as_ref() {
        let projects = c.list_recent_projects(10).map_err(err)?;
        Ok(projects
            .into_iter()
            .map(|(id, title, path)| serde_json::json!({ "id": id, "title": title, "path": path }))
            .collect())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn parse_script_command(
    input: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    log_info(&state.logs, "parse", "開始解析劇本...");
    let parsed = parse_script(&input).map_err(|e| {
        log_error(&state.logs, "parse", format!("解析失敗: {e}"));
        err(e)
    })?;
    let mut project = state.project_svc.get().await.ok_or("尚未開啟專案")?;
    project.script_raw = input;
    apply_parsed_script(&mut project, &parsed).map_err(err)?;
    if let Some(paths) = state.project_svc.paths().await {
        project.timeline = build_timeline_from_segments(&project.segments);
        state.project_svc.set(project.clone()).await;
        grok_voice_core::save_project(&project, &paths).map_err(err)?;
    }
    log_info(
        &state.logs,
        "parse",
        format!(
            "解析完成：{} 個角色，{} 句台詞",
            project.characters.len(),
            project.segments.len()
        ),
    );
    Ok(project)
}

#[tauri::command]
pub async fn convert_story(
    story: String,
    style: Option<String>,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let api_key = load_api_key().map_err(err)?.ok_or("尚未設定 API Key")?;
    let client = XaiChatClient::new(api_key);
    let script = story_to_script_with_retry(&client, &story, style.as_deref())
        .await
        .map_err(err)?;

    let mut project = state.project_svc.get().await.ok_or("尚未開啟專案")?;
    apply_story_script(&mut project, &script).map_err(err)?;
    project.timeline = build_timeline_from_segments(&project.segments);
    state.project_svc.set(project.clone()).await;
    if let Some(paths) = state.project_svc.paths().await {
        grok_voice_core::save_project(&project, &paths).map_err(err)?;
    }
    Ok(project)
}

#[tauri::command]
pub async fn update_project(project: Project, state: State<'_, AppState>) -> Result<(), String> {
    state.project_svc.set(project).await;
    Ok(())
}

#[tauri::command]
pub async fn get_project(state: State<'_, AppState>) -> Result<Option<Project>, String> {
    Ok(state.project_svc.get().await)
}

#[tauri::command]
pub async fn get_project_stats(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let project = state.project_svc.get().await.ok_or("尚未開啟專案")?;
    let total_chars: usize = project.segments.iter().map(|s| s.text.chars().count()).sum();
    let done = project
        .segments
        .iter()
        .filter(|s| s.status == grok_voice_core::SegmentStatus::Done
            || s.status == grok_voice_core::SegmentStatus::Cached)
        .count();
    let store = SettingsStore::open().map_err(err)?;
    let settings = store.load().map_err(err)?;
    let estimated_cost = settings
        .cost_per_1k_chars
        .map(|rate| (total_chars as f64 / 1000.0) * rate);

    Ok(serde_json::json!({
        "total_segments": project.segments.len(),
        "done_segments": done,
        "total_chars": total_chars,
        "estimated_cost": estimated_cost,
    }))
}

#[tauri::command]
pub async fn sync_voices() -> Result<Vec<VoiceInfo>, String> {
    match load_api_key().map_err(err)? {
        Some(key) => {
            let provider = XaiTtsProvider::new(key);
            match provider.list_all_voices().await {
                Ok(voices) if !voices.is_empty() => Ok(voices),
                _ => Ok(fallback_voices()),
            }
        }
        None => Ok(fallback_voices()),
    }
}

#[tauri::command]
pub async fn generate_segment(
    segment_id: String,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> Result<ScriptSegment, String> {
    GenerationService::generate_one(
        &state.project_svc,
        &state.cache,
        &segment_id,
        force.unwrap_or(false),
    )
    .await
    .map_err(err)
}

#[tauri::command]
pub async fn start_generate_job(
    app: AppHandle,
    only_failed: Option<bool>,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let only_failed = only_failed.unwrap_or(false);
    let force = force.unwrap_or(false);

    let project = state.project_svc.get().await.ok_or("尚未開啟專案")?;

    let pending_dialogue = project.segments.iter().any(|s| {
        s.segment_kind != SegmentKind::Sfx
            && (if only_failed {
                s.status == SegmentStatus::Failed
            } else {
                s.status != SegmentStatus::Done && s.status != SegmentStatus::Cached
            })
    });
    if pending_dialogue && load_api_key().map_err(err)?.is_none() {
        let msg = "尚未設定 API Key，請至設定頁面輸入";
        log_warn(&state.logs, "generate", msg);
        return Err(msg.into());
    }
    if project.segments.is_empty() {
        let msg = "尚無台詞段落，請先點擊「解析劇本」";
        log_warn(&state.logs, "generate", msg);
        return Err(msg.into());
    }

    let pending_count = project
        .segments
        .iter()
        .filter(|s| {
            if only_failed {
                s.status == SegmentStatus::Failed
            } else {
                s.status != SegmentStatus::Done && s.status != SegmentStatus::Cached
            }
        })
        .count();

    if pending_count == 0 {
        let msg = if only_failed {
            "沒有失敗的句子需要重生"
        } else {
            "所有句子已生成完成"
        };
        log_warn(&state.logs, "generate", msg);
        return Err(msg.into());
    }

    state.generation_controls.cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    state.generation_controls.pause.store(false, std::sync::atomic::Ordering::Relaxed);

    let store = SettingsStore::open().map_err(err)?;
    let settings = store.load().map_err(err)?;
    let concurrency = settings.generation_concurrency as usize;

    let job_id = uuid::Uuid::new_v4().to_string();
    let project_svc = state.project_svc.clone();
    let cache = state.cache.clone();
    let controls = state.generation_controls.clone();
    let active_job = state.active_job_id.clone();
    let logs = state.logs.clone();
    let job_id_clone = job_id.clone();

    log_info(
        &state.logs,
        "generate",
        format!("開始批次生成：{pending_count} 句（job {job_id_clone}）"),
    );

    if let Ok(mut j) = active_job.lock() {
        *j = Some(job_id.clone());
    }

    tokio::spawn(async move {
        let _ = GenerationService::run_batch(
            app,
            project_svc,
            cache,
            controls,
            logs,
            only_failed,
            force,
            concurrency,
        )
        .await;
    });

    Ok(job_id_clone)
}

#[tauri::command]
pub async fn cancel_generate_job(state: State<'_, AppState>) -> Result<(), String> {
    state
        .generation_controls
        .cancel
        .store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn pause_generate_job(state: State<'_, AppState>) -> Result<(), String> {
    state
        .generation_controls
        .pause
        .store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn resume_generate_job(state: State<'_, AppState>) -> Result<(), String> {
    state
        .generation_controls
        .pause
        .store(false, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn generate_all(state: State<'_, AppState>, app: AppHandle) -> Result<String, String> {
    start_generate_job(app, Some(false), Some(false), state).await
}

#[tauri::command]
pub async fn split_segment(
    segment_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ScriptSegment>, String> {
    GenerationService::split_segment(&state.project_svc, &segment_id)
        .await
        .map_err(err)
}

#[tauri::command]
pub async fn add_character(
    name: String,
    role_type: String,
    state: State<'_, AppState>,
) -> Result<Character, String> {
    let mut project = state.project_svc.get().await.ok_or("尚未開啟專案")?;
    let rt = if role_type == "narrator" {
        RoleType::Narrator
    } else {
        RoleType::Character
    };
    let id = project.get_or_create_character(&name, rt);
    let character = project
        .characters
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .ok_or("建立角色失敗")?;
    state.project_svc.set(project).await;
    Ok(character)
}

#[tauri::command]
pub async fn delete_character(
    character_id: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let mut project = state.project_svc.get().await.ok_or("尚未開啟專案")?;
    project.characters.retain(|c| c.id != character_id);
    project.segments.retain(|s| s.character_id != character_id);
    state.project_svc.set(project.clone()).await;
    Ok(project)
}

#[tauri::command]
pub async fn preview_voice(
    voice_id: String,
    text: Option<String>,
) -> Result<String, String> {
    let api_key = load_api_key().map_err(err)?.ok_or("尚未設定 API Key")?;
    let provider = XaiTtsProvider::new(api_key);
    let sample = text.unwrap_or_else(|| "你好，這是語音試聽。".into());
    let result = provider
        .synthesize(grok_voice_core::TtsRequest {
            text: sample,
            voice_id,
            language: "zh".into(),
            output_format: grok_voice_core::TtsOutputFormat::default(),
        })
        .await
        .map_err(err)?;

    let temp = std::env::temp_dir().join(format!("gvs_preview_{}.mp3", Uuid::new_v4()));
    std::fs::write(&temp, &result.audio_bytes).map_err(|e| err(AppError::Io(e)))?;
    Ok(temp.display().to_string())
}

#[tauri::command]
pub async fn get_audio_src(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let abs = if std::path::Path::new(&path).is_absolute() {
        std::path::PathBuf::from(&path)
    } else if let Some(paths) = state.project_svc.paths().await {
        paths.root.join(&path)
    } else {
        std::path::PathBuf::from(&path)
    };
    Ok(abs.display().to_string())
}

#[tauri::command]
pub async fn export_mixdown(
    options: Option<ExportOptions>,
    state: State<'_, AppState>,
) -> Result<ExportResult, String> {
    let opts = options.unwrap_or_default();
    let project = state.project_svc.get().await.ok_or("尚未開啟專案")?;
    let paths = state.project_svc.paths().await.ok_or("專案路徑不存在")?;

    let segments: Vec<ScriptSegment> = project.segments.clone();

    let exportable = segments.iter().any(|s| {
        s.audio_path.is_some()
            || s.segment_kind == SegmentKind::Sfx
            || !s.sfx_cues.is_empty()
    });
    if !exportable {
        return Err("沒有已生成的音訊可匯出".into());
    }

    let sfx_store = SfxStore::open().map_err(err)?;
    let mut sfx_paths: HashMap<String, String> = HashMap::new();
    for sfx in sfx_store.list().map_err(err)? {
        if let Ok(path) = sfx_store.resolve_path(&sfx.id) {
            sfx_paths.insert(sfx.id, path.display().to_string());
        }
    }

    log_info(
        &state.logs,
        "export",
        format!("開始匯出 {} 段音訊", segments.len()),
    );

    let store = SettingsStore::open().map_err(err)?;
    let settings = store.load().map_err(err)?;
    let ffmpeg = FfmpegConfig {
        ffmpeg_path: settings
            .ffmpeg_path
            .unwrap_or_else(|| "ffmpeg".to_string()),
    };

    let preset = grok_voice_core::ExportPreset {
        codec: opts.codec.clone(),
        sample_rate: opts.sample_rate,
        bit_rate: opts.bit_rate,
        normalize: opts.normalize,
        ..Default::default()
    };

    let ext = extension_for_codec(&opts.codec);
    let audio_path = paths.exports_dir.join(format!("final.{ext}"));
    let duration_ms =
        concat_segments(&ffmpeg, &segments, &audio_path, &preset, &sfx_paths).map_err(err)?;

    let sub_ext = match opts.subtitle_format {
        SubtitleFormat::Vtt => "vtt",
        SubtitleFormat::Ass => "ass",
        SubtitleFormat::Srt => "srt",
    };
    let subtitle_path = paths.exports_dir.join(format!("subtitles.{sub_ext}"));
    write_subtitle(
        &project,
        &segments,
        &subtitle_path,
        sub_ext,
        opts.show_character_in_subtitle,
    )
    .map_err(err)?;

    let mut stem_paths = Vec::new();
    if opts.export_stems {
        for character in &project.characters {
            let char_segments: Vec<ScriptSegment> = segments
                .iter()
                .filter(|s| s.character_id == character.id)
                .cloned()
                .collect();
            if char_segments.is_empty() {
                continue;
            }
            let stem_path = paths
                .exports_dir
                .join(format!("stem_{}.{}", character.name, ext));
            let _ = concat_segments(&ffmpeg, &char_segments, &stem_path, &preset, &sfx_paths);
            stem_paths.push(stem_path.display().to_string());
        }
    }

    let result = ExportResult {
        audio_path: audio_path.display().to_string(),
        subtitle_path: Some(subtitle_path.display().to_string()),
        stem_paths,
        duration_ms,
    };
    log_info(
        &state.logs,
        "export",
        format!("匯出完成：{}", result.audio_path),
    );
    Ok(result)
}

#[tauri::command]
pub async fn cleanup_cache(state: State<'_, AppState>) -> Result<usize, String> {
    let cache = state.cache.lock().map_err(|e| e.to_string())?;
    if let Some(c) = cache.as_ref() {
        c.delete_orphan_cache().map_err(err)
    } else {
        Ok(0)
    }
}

#[tauri::command]
pub async fn get_logs(limit: Option<usize>, state: State<'_, AppState>) -> Result<Vec<LogEntry>, String> {
    Ok(state.logs.list(limit))
}

#[tauri::command]
pub async fn clear_logs(state: State<'_, AppState>) -> Result<(), String> {
    state.logs.clear();
    log_info(&state.logs, "system", "日誌已清除");
    Ok(())
}

#[tauri::command]
pub async fn verify_api_key(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let api_key = load_api_key().map_err(err)?.ok_or("尚未設定 API Key")?;
    let provider = XaiTtsProvider::new(api_key);
    match provider.list_all_voices().await {
        Ok(voices) => {
            let custom = voices.iter().filter(|v| v.is_custom).count();
            log_info(
                &state.logs,
                "api",
                format!(
                    "API Key 驗證成功（{} 個 voice，含 {} 個自訂）",
                    voices.len(),
                    custom
                ),
            );
            Ok(serde_json::json!({
                "ok": true,
                "voice_count": voices.len(),
                "message": "API Key 有效"
            }))
        }
        Err(e) => {
            log_error(&state.logs, "api", format!("API Key 驗證失敗: {e}"));
            Err(err(e))
        }
    }
}

#[tauri::command]
pub async fn list_custom_voices() -> Result<Vec<VoiceInfo>, String> {
    let api_key = load_api_key().map_err(err)?.ok_or("尚未設定 API Key")?;
    let provider = XaiTtsProvider::new(api_key);
    provider.list_custom_voices().await.map_err(err)
}

#[tauri::command]
pub async fn create_custom_voice(
    file_path: String,
    name: Option<String>,
    description: Option<String>,
    gender: Option<String>,
    accent: Option<String>,
    age: Option<String>,
    language: Option<String>,
    use_case: Option<String>,
    tone: Option<String>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let api_key = load_api_key().map_err(err)?.ok_or("尚未設定 API Key")?;
    let provider = XaiTtsProvider::new(api_key);
    let result = provider
        .create_custom_voice(CreateCustomVoiceRequest {
            file_path,
            name,
            description,
            gender,
            accent,
            age,
            language,
            use_case,
            tone,
        })
        .await
        .map_err(err)?;
    log_info(
        &state.logs,
        "voice",
        format!("建立自訂語音: {} ({})", result.name.as_deref().unwrap_or("?"), result.voice_id),
    );
    Ok(serde_json::json!(result))
}

#[tauri::command]
pub async fn delete_custom_voice(
    voice_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let api_key = load_api_key().map_err(err)?.ok_or("尚未設定 API Key")?;
    let provider = XaiTtsProvider::new(api_key);
    let ok = provider.delete_custom_voice(&voice_id).await.map_err(err)?;
    log_info(&state.logs, "voice", format!("刪除自訂語音: {voice_id}"));
    Ok(ok)
}

#[tauri::command]
pub async fn list_sfx_library() -> Result<Vec<SoundEffect>, String> {
    let store = SfxStore::open().map_err(err)?;
    store.list().map_err(err)
}

#[tauri::command]
pub async fn preview_sfx(sfx_id: String, state: State<'_, AppState>) -> Result<String, String> {
    let store = SfxStore::open().map_err(err)?;
    let path = store.resolve_path(&sfx_id).map_err(err)?;
    log_info(
        &state.logs,
        "sfx",
        format!("試聽音效: {sfx_id}"),
    );
    Ok(path.display().to_string())
}

#[tauri::command]
pub async fn import_sfx_file(
    path: String,
    name: String,
    category: Option<String>,
    state: State<'_, AppState>,
) -> Result<SoundEffect, String> {
    let cat = match category.as_deref() {
        Some("action") => SfxCategory::Action,
        Some("nature") => SfxCategory::Nature,
        Some("ui") => SfxCategory::Ui,
        Some("horror") => SfxCategory::Horror,
        Some("custom") => SfxCategory::Custom,
        _ => SfxCategory::Ambient,
    };
    let store = SfxStore::open().map_err(err)?;
    let entry = store
        .import_file(std::path::Path::new(&path), &name, cat)
        .map_err(err)?;
    log_info(
        &state.logs,
        "sfx",
        format!("匯入音效: {} ({})", entry.name, entry.id),
    );
    Ok(entry)
}

#[tauri::command]
pub async fn export_debug_bundle(state: State<'_, AppState>) -> Result<String, String> {
    let project = state.project_svc.get().await.ok_or("尚未開啟專案")?;
    let paths = state.project_svc.paths().await.ok_or("專案路徑不存在")?;
    let bundle_dir = paths.root.join("debug_bundle");
    std::fs::create_dir_all(&bundle_dir).map_err(|e| err(AppError::Io(e)))?;
    let project_copy = serde_json::to_string_pretty(&project).map_err(|e| err(AppError::Json(e)))?;
    std::fs::write(bundle_dir.join("project.json"), project_copy).map_err(|e| err(AppError::Io(e)))?;
    Ok(bundle_dir.display().to_string())
}