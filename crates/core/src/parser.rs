use regex::Regex;
use uuid::Uuid;

use crate::models::{Project, RoleType, ScriptSegment, SegmentKind, SegmentStatus, SfxCue};
use crate::sfx::{builtin_catalog, resolve_sfx_id};
use crate::AppError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParsedLineKind {
    Dialogue,
    Sfx,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedLine {
    pub kind: ParsedLineKind,
    pub character: Option<String>,
    pub emotion_hint: Option<String>,
    pub text: String,
    pub speech_tags: Vec<String>,
    pub sfx_name: Option<String>,
    pub sfx_cues: Vec<SfxCue>,
    pub line_number: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedScript {
    pub lines: Vec<ParsedLine>,
    pub pause_after_line: Vec<Option<u32>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParseError {
    pub line: u32,
    pub message: String,
}

fn extract_speech_tags(text: &str) -> (String, Vec<String>) {
    let tag_re = Regex::new(r"\[([a-zA-Z][\w-]*)\]").unwrap();
    let mut tags = Vec::new();
    for cap in tag_re.captures_iter(text) {
        tags.push(cap[1].to_string());
    }
    (text.to_string(), tags)
}

fn extract_inline_sfx(text: &str) -> (String, Vec<SfxCue>) {
    let catalog = builtin_catalog();
    let re = Regex::new(r"\{([^}]+)\}").unwrap();
    let mut cues = Vec::new();
    let mut cleaned = String::new();
    let mut last = 0usize;

    for cap in re.captures_iter(text) {
        let m = cap.get(0).unwrap();
        let label = cap[1].trim().to_string();
        cleaned.push_str(&text[last..m.start()]);
        last = m.end();

        if let Some(sfx) = resolve_sfx_id(&label, &catalog) {
            cues.push(SfxCue {
                sfx_id: sfx.id.clone(),
                label: sfx.name.clone(),
                volume_db: 0.0,
            });
        } else {
            cleaned.push_str(&format!("{{{label}}}"));
        }
    }
    cleaned.push_str(&text[last..]);
    let cleaned = cleaned.replace("  ", " ").trim().to_string();
    (cleaned, cues)
}

fn parse_sfx_line(line: &str) -> Option<String> {
    let line = line.trim();
    let patterns = [
        r"^音效\s*[:：]\s*(.+)$",
        r"^【音效】\s*(.+)$",
        r"^\[SFX\]\s*(.+)$",
        r"^\{SFX\s*[:：]\s*([^}]+)\}$",
    ];
    for pat in patterns {
        if let Ok(re) = Regex::new(pat) {
            if let Some(caps) = re.captures(line) {
                return Some(caps[1].trim().to_string());
            }
        }
    }
    if let Ok(re) = Regex::new(r"^【([^】]+)】$") {
        if let Some(caps) = re.captures(line) {
            let name = caps[1].trim();
            if resolve_sfx_id(name, &builtin_catalog()).is_some() {
                return Some(name.to_string());
            }
        }
    }
    None
}

fn parse_dialogue_line(line: &str) -> Option<(String, Option<String>, String)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    if let Some(caps) = Regex::new(r"^\[([^\]]+)\]\s*(.+)$").ok()?.captures(line) {
        let name = caps[1].trim().to_string();
        let text = caps[2].trim().to_string();
        return Some((name, None, text));
    }

    let re_emotion = Regex::new(r"^(.+?)（([^）]+)）\s*[:：]\s*(.+)$").ok()?;
    if let Some(caps) = re_emotion.captures(line) {
        let name = caps[1].trim().to_string();
        let emotion = caps[2].trim().to_string();
        let text = caps[3].trim().to_string();
        return Some((name, Some(emotion), text));
    }

    let re_basic = Regex::new(r"^(.+?)\s*[:：]\s*(.+)$").ok()?;
    if let Some(caps) = re_basic.captures(line) {
        let name = caps[1].trim().to_string();
        let text = caps[2].trim().to_string();
        if !name.is_empty() && !text.is_empty() {
            return Some((name, None, text));
        }
    }

    None
}

fn is_comment(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('#') || trimmed.starts_with("//")
}

fn is_stage_direction(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('（') && trimmed.ends_with('）')
}

pub fn parse_script(input: &str) -> Result<ParsedScript, AppError> {
    let mut lines = Vec::new();
    let mut pause_after_line = Vec::new();
    let mut errors = Vec::new();
    let mut consecutive_blank = 0u32;

    for (idx, raw_line) in input.lines().enumerate() {
        let line_number = (idx + 1) as u32;
        let trimmed = raw_line.trim();

        if trimmed.is_empty() {
            consecutive_blank += 1;
            continue;
        }

        if is_comment(trimmed) {
            continue;
        }

        if is_stage_direction(trimmed) {
            continue;
        }

        let pause_ms = if consecutive_blank > 0 {
            Some(consecutive_blank * 500)
        } else {
            None
        };
        consecutive_blank = 0;

        if let Some(sfx_name) = parse_sfx_line(trimmed) {
            lines.push(ParsedLine {
                kind: ParsedLineKind::Sfx,
                character: None,
                emotion_hint: None,
                text: sfx_name.clone(),
                speech_tags: Vec::new(),
                sfx_name: Some(sfx_name),
                sfx_cues: Vec::new(),
                line_number,
            });
            pause_after_line.push(pause_ms);
            continue;
        }

        match parse_dialogue_line(raw_line) {
            Some((character, emotion_hint, text)) => {
                let (text, inline_cues) = extract_inline_sfx(&text);
                let (text, speech_tags) = extract_speech_tags(&text);
                lines.push(ParsedLine {
                    kind: ParsedLineKind::Dialogue,
                    character: Some(character),
                    emotion_hint,
                    text,
                    speech_tags,
                    sfx_name: None,
                    sfx_cues: inline_cues,
                    line_number,
                });
                pause_after_line.push(pause_ms);
            }
            None => {
                errors.push(ParseError {
                    line: line_number,
                    message: format!("無法解析此行: {trimmed}"),
                });
            }
        }
    }

    if !errors.is_empty() {
        let first = &errors[0];
        return Err(AppError::Parse {
            line: first.line,
            message: first.message.clone(),
        });
    }

    Ok(ParsedScript {
        lines,
        pause_after_line,
    })
}

fn system_sfx_character_id(project: &mut Project) -> String {
    project.get_or_create_character("音效", RoleType::System)
}

pub fn apply_parsed_script(project: &mut Project, parsed: &ParsedScript) -> Result<(), AppError> {
    let old_segments = std::mem::take(&mut project.segments);
    let catalog = builtin_catalog();

    for (idx, line) in parsed.lines.iter().enumerate() {
        let pause_after_ms = parsed.pause_after_line.get(idx).copied().flatten().unwrap_or(0);

        if line.kind == ParsedLineKind::Sfx {
            let sfx_name = line.sfx_name.as_deref().unwrap_or(&line.text);
            let sfx = resolve_sfx_id(sfx_name, &catalog).ok_or_else(|| AppError::Parse {
                line: line.line_number,
                message: format!("未知音效: {sfx_name}"),
            })?;
            let character_id = system_sfx_character_id(project);

            let preserved = old_segments.iter().find(|s| {
                s.segment_kind == SegmentKind::Sfx
                    && s.sfx_id.as_deref() == Some(sfx.id.as_str())
                    && s.order == idx as u32
            });

            if let Some(old) = preserved {
                project.segments.push(ScriptSegment {
                    id: old.id.clone(),
                    character_id,
                    text: sfx.name.clone(),
                    language: old.language.clone(),
                    emotion_hint: None,
                    speech_tags: Vec::new(),
                    segment_kind: SegmentKind::Sfx,
                    sfx_id: Some(sfx.id.clone()),
                    sfx_cues: Vec::new(),
                    order: idx as u32,
                    pause_after_ms,
                    audio_asset_id: old.audio_asset_id.clone(),
                    audio_path: old.audio_path.clone(),
                    duration_ms: old.duration_ms,
                    status: old.status.clone(),
                    error_message: old.error_message.clone(),
                });
            } else {
                project.segments.push(ScriptSegment {
                    id: Uuid::new_v4().to_string(),
                    character_id,
                    text: sfx.name.clone(),
                    language: "zh".to_string(),
                    emotion_hint: None,
                    speech_tags: Vec::new(),
                    segment_kind: SegmentKind::Sfx,
                    sfx_id: Some(sfx.id.clone()),
                    sfx_cues: Vec::new(),
                    order: idx as u32,
                    pause_after_ms,
                    audio_asset_id: None,
                    audio_path: None,
                    duration_ms: Some(sfx.duration_ms as u64),
                    status: SegmentStatus::Pending,
                    error_message: None,
                });
            }
            continue;
        }

        let character = line.character.as_deref().unwrap_or("旁白");
        let role_type = if character == "旁白" || character.to_lowercase() == "narrator" {
            RoleType::Narrator
        } else {
            RoleType::Character
        };

        let character_id = project.get_or_create_character(character, role_type);

        let preserved = old_segments.iter().find(|s| {
            s.segment_kind == SegmentKind::Dialogue
                && project
                    .characters
                    .iter()
                    .find(|c| c.id == s.character_id)
                    .map(|c| c.name.as_str())
                    == Some(character)
                && s.text == line.text
                && s.sfx_cues == line.sfx_cues
        });

        if let Some(old) = preserved {
            project.segments.push(ScriptSegment {
                id: old.id.clone(),
                character_id,
                text: line.text.clone(),
                language: old.language.clone(),
                emotion_hint: line.emotion_hint.clone(),
                speech_tags: line.speech_tags.clone(),
                segment_kind: SegmentKind::Dialogue,
                sfx_id: None,
                sfx_cues: line.sfx_cues.clone(),
                order: idx as u32,
                pause_after_ms,
                audio_asset_id: old.audio_asset_id.clone(),
                audio_path: old.audio_path.clone(),
                duration_ms: old.duration_ms,
                status: old.status.clone(),
                error_message: old.error_message.clone(),
            });
        } else {
            project.segments.push(ScriptSegment {
                id: Uuid::new_v4().to_string(),
                character_id,
                text: line.text.clone(),
                language: "zh".to_string(),
                emotion_hint: line.emotion_hint.clone(),
                speech_tags: line.speech_tags.clone(),
                segment_kind: SegmentKind::Dialogue,
                sfx_id: None,
                sfx_cues: line.sfx_cues.clone(),
                order: idx as u32,
                pause_after_ms,
                audio_asset_id: None,
                audio_path: None,
                duration_ms: None,
                status: SegmentStatus::Pending,
                error_message: None,
            });
        }
    }

    project.touch();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chinese_colon() {
        let input = "旁白：故事開始。\n小明：你好。";
        let parsed = parse_script(input).unwrap();
        assert_eq!(parsed.lines.len(), 2);
        assert_eq!(parsed.lines[0].character.as_deref(), Some("旁白"));
        assert_eq!(parsed.lines[1].character.as_deref(), Some("小明"));
    }

    #[test]
    fn parses_standalone_sfx() {
        let input = "旁白：故事開始。\n音效：雨聲\n小明：你好。";
        let parsed = parse_script(input).unwrap();
        assert_eq!(parsed.lines.len(), 3);
        assert_eq!(parsed.lines[1].kind, ParsedLineKind::Sfx);
        assert_eq!(parsed.lines[1].sfx_name.as_deref(), Some("雨聲"));
    }

    #[test]
    fn parses_inline_sfx() {
        let input = "阿明：你聽到了嗎？{雷聲} 那不是風聲。";
        let parsed = parse_script(input).unwrap();
        assert_eq!(parsed.lines[0].sfx_cues.len(), 1);
        assert_eq!(parsed.lines[0].sfx_cues[0].sfx_id, "thunder");
        assert!(!parsed.lines[0].text.contains('{'));
    }

    #[test]
    fn parses_emotion_hint() {
        let input = "小美（害怕）：你聽到了嗎？";
        let parsed = parse_script(input).unwrap();
        assert_eq!(parsed.lines[0].character.as_deref(), Some("小美"));
        assert_eq!(parsed.lines[0].emotion_hint.as_deref(), Some("害怕"));
    }

    #[test]
    fn preserves_speech_tags() {
        let input = "阿明：等一下。[pause] 我想到了。";
        let parsed = parse_script(input).unwrap();
        assert!(parsed.lines[0].text.contains("[pause]"));
        assert!(parsed.lines[0].speech_tags.contains(&"pause".to_string()));
    }

    #[test]
    fn demo_dialogue_with_sfx() {
        let input = r#"旁白：深夜的城市，只剩雨聲。
音效：雨聲
阿明（緊張）：你聽到了嗎？{雷聲} 那不是風聲。"#;
        let parsed = parse_script(input).unwrap();
        assert_eq!(parsed.lines.len(), 3);
        assert_eq!(parsed.lines[1].kind, ParsedLineKind::Sfx);
        assert_eq!(parsed.lines[2].sfx_cues[0].sfx_id, "thunder");
    }
}