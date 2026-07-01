use std::fs;
use std::path::Path;
use std::process::Command;

use std::collections::HashMap;

use grok_voice_core::{AppError, AudioCodec, ExportPreset, ScriptSegment, SegmentKind};

#[derive(Debug, Clone)]
pub struct FfmpegConfig {
    pub ffmpeg_path: String,
}

impl Default for FfmpegConfig {
    fn default() -> Self {
        Self {
            ffmpeg_path: "ffmpeg".to_string(),
        }
    }
}

impl FfmpegConfig {
    pub fn resolve() -> Self {
        if let Ok(path) = std::env::var("FFMPEG_PATH") {
            return Self {
                ffmpeg_path: path,
            };
        }
        Self::default()
    }

    fn check_available(&self) -> Result<(), AppError> {
        let output = Command::new(&self.ffmpeg_path)
            .arg("-version")
            .output();
        match output {
            Ok(o) if o.status.success() => Ok(()),
            _ => Err(AppError::FfmpegMissing(format!(
                "找不到 FFmpeg: {}",
                self.ffmpeg_path
            ))),
        }
    }
}

pub fn probe_duration_ms(config: &FfmpegConfig, path: &Path) -> Result<u64, AppError> {
    config.check_available()?;
    let ffprobe = config.ffmpeg_path.replace("ffmpeg", "ffprobe");
    let ffprobe_path = if Path::new(&ffprobe).exists() || which_ffprobe(&ffprobe) {
        ffprobe
    } else {
        "ffprobe".to_string()
    };

    let output = Command::new(&ffprobe_path)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            &path.display().to_string(),
        ])
        .output()
        .map_err(|e| AppError::ExportFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(AppError::ExportFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let secs: f64 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|_| AppError::ExportFailed("無法解析音訊長度".into()))?;

    Ok((secs * 1000.0).round() as u64)
}

fn which_ffprobe(path: &str) -> bool {
    Command::new(path)
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn concat_segments(
    config: &FfmpegConfig,
    segments: &[ScriptSegment],
    output_path: &Path,
    preset: &ExportPreset,
    sfx_paths: &HashMap<String, String>,
) -> Result<u64, AppError> {
    config.check_available()?;

    let has_audio = segments.iter().any(|s| {
        s.audio_path.is_some()
            || (s.segment_kind == SegmentKind::Sfx && s.sfx_id.is_some())
            || !s.sfx_cues.is_empty()
    });

    if !has_audio {
        return Err(AppError::ExportFailed("沒有可合併的音訊檔案".into()));
    }

    let temp_dir = output_path.parent().unwrap_or(Path::new("."));
    let list_file = temp_dir.join("concat_list.txt");
    let mut list_content = String::new();

    for (i, seg) in segments.iter().enumerate() {
        if seg.segment_kind == SegmentKind::Sfx {
            if let Some(sfx_id) = &seg.sfx_id {
                if let Some(path) = seg.audio_path.as_deref().or_else(|| sfx_paths.get(sfx_id).map(|s| s.as_str())) {
                    append_file_line(&mut list_content, path);
                }
            }
        } else if let Some(path) = &seg.audio_path {
            append_file_line(&mut list_content, path);
            for cue in &seg.sfx_cues {
                if let Some(path) = sfx_paths.get(&cue.sfx_id) {
                    append_file_line(&mut list_content, path);
                }
            }
        } else {
            for cue in &seg.sfx_cues {
                if let Some(path) = sfx_paths.get(&cue.sfx_id) {
                    append_file_line(&mut list_content, path);
                }
            }
        }

        if seg.pause_after_ms > 0 {
            let silence_path = temp_dir.join(format!("silence_{i}.wav"));
            generate_silence(config, seg.pause_after_ms, &silence_path)?;
            let escaped_silence = silence_path.display().to_string().replace('\\', "/");
            list_content.push_str(&format!("file '{escaped_silence}'\n"));
        }
    }

    fs::write(&list_file, &list_content)?;

    let list_path = list_file.display().to_string();
    let sample_rate = preset.sample_rate.to_string();
    let output_str = output_path.display().to_string();

    let mut cmd = Command::new(&config.ffmpeg_path);
    cmd.args([
        "-y",
        "-f",
        "concat",
        "-safe",
        "0",
        "-i",
        &list_path,
        "-ar",
        &sample_rate,
    ]);

    match preset.codec {
        AudioCodec::Mp3 => {
            cmd.args(["-c:a", "libmp3lame", "-b:a", "128k"]);
        }
        AudioCodec::Wav | AudioCodec::Pcm => {
            cmd.args(["-c:a", "pcm_s16le"]);
        }
        AudioCodec::Flac => {
            cmd.arg("-c:a").arg("flac");
        }
    }

    if preset.normalize {
        cmd.args(["-af", "loudnorm"]);
    }
    cmd.arg(&output_str);

    let output = cmd
        .output()
        .map_err(|e| AppError::ExportFailed(e.to_string()))?;

    let _ = fs::remove_file(&list_file);

    if !output.status.success() {
        return Err(AppError::ExportFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    probe_duration_ms(config, output_path)
}

fn append_file_line(list: &mut String, path: &str) {
    let escaped = path.replace('\\', "/").replace('\'', "'\\''");
    list.push_str(&format!("file '{escaped}'\n"));
}

fn generate_silence(config: &FfmpegConfig, duration_ms: u32, output: &Path) -> Result<(), AppError> {
    let duration_sec = format!("{:.3}", duration_ms as f64 / 1000.0);
    let output_str = output.display().to_string();
    let args = [
        "-y",
        "-f",
        "lavfi",
        "-i",
        "anullsrc=r=24000:cl=mono",
        "-t",
        &duration_sec,
        "-c:a",
        "pcm_s16le",
        &output_str,
    ];

    let result = Command::new(&config.ffmpeg_path)
        .args(args)
        .output()
        .map_err(|e| AppError::ExportFailed(e.to_string()))?;

    if !result.status.success() {
        return Err(AppError::ExportFailed(
            String::from_utf8_lossy(&result.stderr).to_string(),
        ));
    }
    Ok(())
}

pub fn extension_for_codec(codec: &AudioCodec) -> &'static str {
    match codec {
        AudioCodec::Mp3 => "mp3",
        AudioCodec::Wav => "wav",
        AudioCodec::Flac => "flac",
        AudioCodec::Pcm => "wav",
    }
}