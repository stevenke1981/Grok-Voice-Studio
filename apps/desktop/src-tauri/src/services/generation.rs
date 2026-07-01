use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use grok_voice_audio::{probe_duration_ms, FfmpegConfig};
use grok_voice_core::{
    default_split, save_project, AppError, Character, ScriptSegment, SegmentKind, SegmentStatus,
    TtsOutputFormat, TtsRequest,
};
use grok_voice_storage::{load_api_key, new_cache_entry, AudioCache, SettingsStore, SfxStore};
use grok_voice_xai::XaiTtsProvider;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use std::sync::Mutex;
use uuid::Uuid;

use crate::log_store::LogStore;

use super::project_service::ProjectService;

#[derive(Clone, Default)]
pub struct GenerationControls {
    pub cancel: Arc<AtomicBool>,
    pub pause: Arc<AtomicBool>,
}

#[derive(Clone, Serialize)]
pub struct GenerateProgressEvent {
    pub job_id: String,
    pub current: usize,
    pub total: usize,
    pub segment_id: Option<String>,
    pub segment: Option<ScriptSegment>,
    pub status: String,
    pub error: Option<String>,
    pub cached: bool,
}

pub struct GenerationService;

impl GenerationService {
    fn with_cache<T>(cache: &Arc<Mutex<Option<AudioCache>>>, f: impl FnOnce(Option<&AudioCache>) -> T) -> T {
        let guard = cache.lock().ok();
        f(guard.as_ref().and_then(|g| g.as_ref()))
    }

    pub async fn generate_one(
        project_svc: &ProjectService,
        cache: &Arc<Mutex<Option<AudioCache>>>,
        segment_id: &str,
        force: bool,
    ) -> Result<ScriptSegment, AppError> {
        let segment = {
            let project = project_svc.get().await.ok_or(AppError::Other("尚未開啟專案".into()))?;
            project
                .segments
                .iter()
                .find(|s| s.id == segment_id)
                .ok_or(AppError::SegmentNotFound(segment_id.into()))?
                .clone()
        };

        if segment.segment_kind == SegmentKind::Sfx {
            return Self::resolve_sfx_segment(project_svc, &segment, force).await;
        }

        let api_key = load_api_key()?.ok_or(AppError::MissingApiKey)?;
        let provider = XaiTtsProvider::new(api_key);

        let (character, paths, output_format) = {
            let project = project_svc.get().await.ok_or(AppError::Other("尚未開啟專案".into()))?;
            let paths = project_svc.paths().await.ok_or(AppError::Other("專案路徑不存在".into()))?;
            let character = project
                .characters
                .iter()
                .find(|c| c.id == segment.character_id)
                .ok_or(AppError::CharacterNotFound(segment.character_id.clone()))?
                .clone();
            (character, paths, TtsOutputFormat::default())
        };

        let text = build_tts_text(&segment, &character);
        if text.chars().count() > 15_000 {
            return Err(AppError::TextTooLong {
                chars: text.chars().count(),
                max: 15_000,
            });
        }

        let cache_key = AudioCache::compute_cache_key(
            "xai",
            &character.voice_profile.voice_id,
            &segment.language,
            &text,
            &output_format,
        );

        if !force {
            let cache_hit: Option<grok_voice_storage::CacheEntry> = Self::with_cache(cache, |c| {
                c.and_then(|cache| cache.lookup(&cache_key).ok().flatten())
            });
            if let Some(entry) = cache_hit {
                let updated = ScriptSegment {
                    audio_path: Some(entry.file_path),
                    duration_ms: entry.duration_ms,
                    status: SegmentStatus::Cached,
                    error_message: None,
                    ..segment
                };
                project_svc.update_segment(segment_id, |s| *s = updated.clone()).await;
                return Ok(updated);
            }
        }

        project_svc
            .update_segment(segment_id, |s| {
                s.status = SegmentStatus::Generating;
                s.error_message = None;
            })
            .await;

        let use_streaming = SettingsStore::open()
            .ok()
            .and_then(|s| s.load().ok())
            .map(|s| s.use_streaming_tts)
            .unwrap_or(true);

        tracing::info!(
            target: "generate",
            "TTS segment {segment_id} (streaming={use_streaming})"
        );

        let req = TtsRequest {
            text,
            voice_id: character.voice_profile.voice_id.clone(),
            language: segment.language.clone(),
            output_format: output_format.clone(),
        };

        let result = provider.synthesize_preferred(req, use_streaming).await?;
        let ext = if result.content_type.contains("wav") {
            "wav"
        } else {
            "mp3"
        };
        let rel_path = ProjectService::rel_audio_path(&paths, segment_id, ext);
        let file_path = paths.root.join(&rel_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file_path, &result.audio_bytes)?;

        let ffmpeg = FfmpegConfig::resolve();
        let duration_ms = probe_duration_ms(&ffmpeg, &file_path).ok();

        Self::with_cache(cache, |c| {
            if let Some(cache) = c {
                let entry = new_cache_entry(
                    cache_key,
                    "xai",
                    &character.voice_profile.voice_id,
                    &segment.language,
                    &segment.text,
                    rel_path.clone(),
                    duration_ms,
                );
                let _ = cache.insert(&entry);
            }
        });

        let updated = ScriptSegment {
            audio_path: Some(rel_path),
            duration_ms,
            status: SegmentStatus::Done,
            error_message: None,
            ..segment
        };

        project_svc.update_segment(segment_id, |s| *s = updated.clone()).await;
        if let (Some(mut project), Some(paths)) = (project_svc.get().await, project_svc.paths().await) {
            ProjectService::normalize_segment_paths(&mut project, &paths);
            project_svc.set(project.clone()).await;
            let _ = save_project(&project, &paths);
        }

        Ok(updated)
    }

    pub async fn run_batch(
        app: AppHandle,
        project_svc: Arc<ProjectService>,
        cache: Arc<Mutex<Option<AudioCache>>>,
        controls: GenerationControls,
        logs: LogStore,
        only_failed: bool,
        force: bool,
        concurrency: usize,
    ) -> Result<String, AppError> {
        let job_id = Uuid::new_v4().to_string();
        let segment_ids: Vec<String> = {
            let project = project_svc.get().await.ok_or(AppError::Other("尚未開啟專案".into()))?;
            project
                .segments
                .iter()
                .filter(|s| {
                    if only_failed {
                        s.status == SegmentStatus::Failed
                    } else {
                        s.status != SegmentStatus::Done && s.status != SegmentStatus::Cached
                    }
                })
                .map(|s| s.id.clone())
                .collect()
        };

        let total = segment_ids.len();
        if total == 0 {
            log_warn_empty_batch(&logs, &app);
            return Ok(job_id);
        }

        logs.append_with_emit(&app, "info", "generate", format!("批次任務啟動：{total} 句待處理"));

        if let Some(project) = project_svc.get().await {
            if let Ok(guard) = cache.lock() {
                if let Some(c) = guard.as_ref() {
                    let _ = c.upsert_job(&job_id, &project.id, "running", 0, total);
                }
            }
        }

        let _concurrency = concurrency;
        let mut current = 0usize;

        for id in segment_ids {
            if controls.cancel.load(Ordering::Relaxed) {
                break;
            }

            while controls.pause.load(Ordering::Relaxed) {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }

            let step = current + 1;
            logs.append_with_emit(&app, "info", "generate", format!("[{step}/{total}] 生成段落 {id}"));
            let result = Self::generate_one(&project_svc, &cache, &id, force).await;

            current += 1;
            match result {
                Ok(seg) => {
                    let cached = seg.status == SegmentStatus::Cached;
                    logs.append_with_emit(
                        &app,
                        "info",
                        "generate",
                        format!(
                            "[{step}/{total}] 完成 {id}{}",
                            if cached { " (快取)" } else { "" }
                        ),
                    );
                    let _ = app.emit(
                        "generate-progress",
                        GenerateProgressEvent {
                            job_id: job_id.clone(),
                            current,
                            total,
                            segment_id: Some(id.clone()),
                            segment: Some(seg),
                            status: "done".into(),
                            error: None,
                            cached,
                        },
                    );
                }
                Err(e) => {
                    logs.append_with_emit(&app, "error", "generate", format!("[{step}/{total}] 失敗 {id}: {e}"));
                    project_svc
                        .update_segment(&id, |s| {
                            s.status = SegmentStatus::Failed;
                            s.error_message = Some(e.to_string());
                        })
                        .await;
                    let _ = app.emit(
                        "generate-progress",
                        GenerateProgressEvent {
                            job_id: job_id.clone(),
                            current,
                            total,
                            segment_id: Some(id),
                            segment: None,
                            status: "failed".into(),
                            error: Some(e.to_string()),
                            cached: false,
                        },
                    );
                }
            }
        }

        let cancelled = controls.cancel.load(Ordering::Relaxed);
        let status = if cancelled { "cancelled" } else { "completed" };
        logs.append_with_emit(
            &app,
            if cancelled { "warn" } else { "info" },
            "generate",
            format!(
                "批次任務{}：{current}/{total}",
                if cancelled { "已取消" } else { "完成" }
            ),
        );
        if let Some(project) = project_svc.get().await {
            if let Ok(guard) = cache.lock() {
                if let Some(c) = guard.as_ref() {
                    let _ = c.upsert_job(&job_id, &project.id, status, total, total);
                }
            }
        }

        let _ = app.emit(
            "generate-progress",
            GenerateProgressEvent {
                job_id: job_id.clone(),
                current: total,
                total,
                segment_id: None,
                segment: None,
                status: status.into(),
                error: None,
                cached: false,
            },
        );

        Ok(job_id)
    }

    pub async fn split_segment(
        project_svc: &ProjectService,
        segment_id: &str,
    ) -> Result<Vec<ScriptSegment>, AppError> {
        let mut project = project_svc.get().await.ok_or(AppError::Other("尚未開啟專案".into()))?;
        let idx = project
            .segments
            .iter()
            .position(|s| s.id == segment_id)
            .ok_or(AppError::SegmentNotFound(segment_id.into()))?;
        let seg = project.segments[idx].clone();
        let chunks = default_split(&seg.text);
        if chunks.len() <= 1 {
            return Ok(vec![seg]);
        }

        let mut new_segments = Vec::new();
        for (i, chunk) in chunks.into_iter().enumerate() {
            new_segments.push(ScriptSegment {
                id: if i == 0 {
                    seg.id.clone()
                } else {
                    Uuid::new_v4().to_string()
                },
                text: chunk,
                order: seg.order + i as u32,
                status: SegmentStatus::Pending,
                audio_path: None,
                duration_ms: None,
                error_message: None,
                sfx_cues: if i == 0 { seg.sfx_cues.clone() } else { Vec::new() },
                ..seg.clone()
            });
        }

        project.segments.splice(idx..=idx, new_segments.iter().cloned());
        for (i, s) in project.segments.iter_mut().enumerate() {
            s.order = i as u32;
        }
        project_svc.set(project.clone()).await;
        if let Some(paths) = project_svc.paths().await {
            let _ = save_project(&project, &paths);
        }
        Ok(new_segments)
    }

    async fn resolve_sfx_segment(
        project_svc: &ProjectService,
        segment: &ScriptSegment,
        _force: bool,
    ) -> Result<ScriptSegment, AppError> {
        let sfx_id = segment
            .sfx_id
            .as_deref()
            .ok_or_else(|| AppError::Other("音效段落缺少 sfx_id".into()))?;
        let store = SfxStore::open()?;
        let abs_path = store.resolve_path(sfx_id)?;
        let catalog = store.list()?;
        let duration_ms = catalog
            .iter()
            .find(|s| s.id == sfx_id)
            .map(|s| s.duration_ms as u64);

        let rel_path = abs_path.display().to_string();
        let updated = ScriptSegment {
            audio_path: Some(rel_path),
            duration_ms,
            status: SegmentStatus::Done,
            error_message: None,
            ..segment.clone()
        };
        project_svc
            .update_segment(&segment.id, |s| *s = updated.clone())
            .await;
        if let (Some(mut project), Some(paths)) = (project_svc.get().await, project_svc.paths().await) {
            ProjectService::normalize_segment_paths(&mut project, &paths);
            project_svc.set(project.clone()).await;
            let _ = save_project(&project, &paths);
        }
        Ok(updated)
    }
}

fn log_warn_empty_batch(logs: &LogStore, app: &AppHandle) {
    use crate::log_store::log_warn;
    let msg = "批次生成：沒有待處理的句子";
    log_warn(logs, "generate", msg);
    let _ = app.emit(
        "generate-progress",
        GenerateProgressEvent {
            job_id: String::new(),
            current: 0,
            total: 0,
            segment_id: None,
            segment: None,
            status: "failed".into(),
            error: Some(msg.into()),
            cached: false,
        },
    );
}

fn build_tts_text(segment: &ScriptSegment, character: &Character) -> String {
    let mut text = segment.text.clone();
    if let Some(style) = &character.voice_profile.style_prompt {
        if !style.is_empty() && segment.emotion_hint.is_none() {
            text = format!("{text}");
        }
    }
    if let Some(emotion) = &segment.emotion_hint {
        if !text.contains('[') {
            text = format!("{text}");
            let _ = emotion;
        }
    }
    text
}