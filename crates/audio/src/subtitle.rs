use std::fs;
use std::path::Path;

use grok_voice_core::{AppError, Project, ScriptSegment, SegmentKind};

#[derive(Debug, Clone)]
pub struct SubtitleEntry {
    pub index: u32,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

fn format_timestamp(ms: u64) -> String {
    let hours = ms / 3_600_000;
    let minutes = (ms % 3_600_000) / 60_000;
    let seconds = (ms % 60_000) / 1000;
    let millis = ms % 1000;
    format!("{hours:02}:{minutes:02}:{seconds:02},{millis:03}")
}

pub fn generate_srt(
    project: &Project,
    segments: &[ScriptSegment],
    show_character: bool,
) -> Result<String, AppError> {
    let mut entries = Vec::new();
    let mut current_ms: u64 = 0;
    let mut index = 1u32;

    for seg in segments {
        if seg.segment_kind == SegmentKind::Sfx {
            current_ms += seg.duration_ms.unwrap_or(1500) + seg.pause_after_ms as u64;
            continue;
        }

        let duration = seg.duration_ms.unwrap_or(2000);
        let character_name = project
            .characters
            .iter()
            .find(|c| c.id == seg.character_id)
            .map(|c| c.name.as_str())
            .unwrap_or("未知");

        let mut line_text = seg.text.clone();
        if !seg.sfx_cues.is_empty() {
            let cues: Vec<_> = seg.sfx_cues.iter().map(|c| format!("[{}]", c.label)).collect();
            line_text = format!("{line_text} {}", cues.join(" "));
        }

        let text = if show_character {
            format!("{character_name}：{line_text}")
        } else {
            line_text
        };

        entries.push(SubtitleEntry {
            index,
            start_ms: current_ms,
            end_ms: current_ms + duration,
            text,
        });

        current_ms += duration + seg.pause_after_ms as u64;
        index += 1;
    }

    let mut srt = String::new();
    for entry in &entries {
        srt.push_str(&format!("{}\n", entry.index));
        srt.push_str(&format!(
            "{} --> {}\n",
            format_timestamp(entry.start_ms),
            format_timestamp(entry.end_ms)
        ));
        srt.push_str(&format!("{}\n\n", entry.text));
    }

    Ok(srt)
}

pub fn write_srt(
    project: &Project,
    segments: &[ScriptSegment],
    output_path: &Path,
    show_character: bool,
) -> Result<String, AppError> {
    let content = generate_srt(project, segments, show_character)?;
    fs::write(output_path, &content)?;
    Ok(content)
}

pub fn generate_vtt(
    project: &Project,
    segments: &[ScriptSegment],
    show_character: bool,
) -> Result<String, AppError> {
    let srt = generate_srt(project, segments, show_character)?;
    let vtt = format!("WEBVTT\n\n{}", srt.replace(',', "."));
    Ok(vtt)
}

pub fn generate_ass(
    project: &Project,
    segments: &[ScriptSegment],
    show_character: bool,
) -> Result<String, AppError> {
    let mut ass = String::from(
        "[Script Info]\nTitle: Grok Voice Studio\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );

    let mut current_ms: u64 = 0;
    for seg in segments {
        let duration = seg.duration_ms.unwrap_or(2000);
        let character_name = project
            .characters
            .iter()
            .find(|c| c.id == seg.character_id)
            .map(|c| c.name.as_str())
            .unwrap_or("未知");
        let text = if show_character {
            format!("{character_name}：{}", seg.text)
        } else {
            seg.text.clone()
        };
        ass.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            ass_timestamp(current_ms),
            ass_timestamp(current_ms + duration),
            text
        ));
        current_ms += duration + seg.pause_after_ms as u64;
    }
    Ok(ass)
}

fn ass_timestamp(ms: u64) -> String {
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1000;
    let cs = (ms % 1000) / 10;
    format!("{h}:{m:02}:{s:02}.{cs:02}")
}

pub fn write_subtitle(
    project: &Project,
    segments: &[ScriptSegment],
    output_path: &Path,
    format: &str,
    show_character: bool,
) -> Result<String, AppError> {
    let content = match format {
        "vtt" => generate_vtt(project, segments, show_character)?,
        "ass" => generate_ass(project, segments, show_character)?,
        _ => generate_srt(project, segments, show_character)?,
    };
    fs::write(output_path, &content)?;
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use grok_voice_core::{Project, RoleType, SegmentKind, SegmentStatus};

    #[test]
    fn generates_srt_format() {
        let mut project = Project::new("test");
        let char_id = project.get_or_create_character("旁白", RoleType::Narrator);
        let segments = vec![ScriptSegment {
            id: "s1".into(),
            character_id: char_id,
            text: "深夜的城市。".into(),
            language: "zh".into(),
            emotion_hint: None,
            speech_tags: vec![],
            segment_kind: SegmentKind::Dialogue,
            sfx_id: None,
            sfx_cues: vec![],
            order: 0,
            pause_after_ms: 500,
            audio_asset_id: None,
            audio_path: None,
            duration_ms: Some(3200),
            status: SegmentStatus::Done,
            error_message: None,
        }];

        let srt = generate_srt(&project, &segments, true).unwrap();
        assert!(srt.contains("00:00:00,000 --> 00:00:03,200"));
        assert!(srt.contains("旁白：深夜的城市。"));
    }
}