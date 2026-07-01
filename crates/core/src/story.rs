use serde::{Deserialize, Serialize};

use crate::models::{RoleType, ScriptSegment, SegmentKind, SegmentStatus};
use crate::Project;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryCharacter {
    pub name: String,
    pub role_type: String,
    pub voice_hint: Option<String>,
    pub personality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorySegment {
    pub character: String,
    pub text: String,
    pub emotion_hint: Option<String>,
    pub pause_after_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryScript {
    pub title: Option<String>,
    pub characters: Vec<StoryCharacter>,
    pub segments: Vec<StorySegment>,
}

pub fn apply_story_script(project: &mut Project, story: &StoryScript) -> Result<(), crate::AppError> {
    if let Some(title) = &story.title {
        if !title.is_empty() {
            project.title = title.clone();
        }
    }

    project.segments.clear();

    for (idx, seg) in story.segments.iter().enumerate() {
        let role_type = match seg.character.as_str() {
            "旁白" | "narrator" | "Narrator" => RoleType::Narrator,
            _ => RoleType::Character,
        };
        let character_id = project.get_or_create_character(&seg.character, role_type);

        if let Some(ch) = project.characters.iter_mut().find(|c| c.id == character_id) {
            if let Some(story_ch) = story.characters.iter().find(|c| c.name == seg.character) {
                if let Some(hint) = &story_ch.voice_hint {
                    if hint.contains("rex") || hint.contains("旁白") {
                        ch.voice_profile.voice_id = "rex".into();
                    } else if hint.contains("leo") || hint.contains("低沉") {
                        ch.voice_profile.voice_id = "leo".into();
                    } else if hint.contains("eve") || hint.contains("女") {
                        ch.voice_profile.voice_id = "eve".into();
                    }
                }
            }
        }

        project.segments.push(ScriptSegment {
            id: uuid::Uuid::new_v4().to_string(),
            character_id,
            text: seg.text.clone(),
            language: "zh".to_string(),
            emotion_hint: seg.emotion_hint.clone(),
            speech_tags: Vec::new(),
            segment_kind: SegmentKind::Dialogue,
            sfx_id: None,
            sfx_cues: Vec::new(),
            order: idx as u32,
            pause_after_ms: seg.pause_after_ms.unwrap_or(0),
            audio_asset_id: None,
            audio_path: None,
            duration_ms: None,
            status: SegmentStatus::Pending,
            error_message: None,
        });
    }

    project.script_raw = story_to_dialogue(story);
    project.touch();
    Ok(())
}

pub fn story_to_dialogue(story: &StoryScript) -> String {
    story
        .segments
        .iter()
        .map(|s| {
            if let Some(emotion) = &s.emotion_hint {
                format!("{}（{}）：{}", s.character, emotion, s.text)
            } else {
                format!("{}：{}", s.character, s.text)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub const STORY_SYSTEM_PROMPT: &str = r#"你是專業配音劇本編劯。將故事轉成多角色配音 JSON。
規則：
1. 必須有旁白角色處理場景描述
2. 對話與旁白分開，不要全部塞給一個角色
3. 每句適合 TTS，不要太長
4. 只回傳 JSON，不要 markdown

格式：
{
  "title": "標題",
  "characters": [{"name":"旁白","role_type":"narrator","voice_hint":"rex","personality":"沉穩"}],
  "segments": [{"character":"旁白","text":"...","emotion_hint":"","pause_after_ms":500}]
}"#;