use crate::{Character, ScriptSegment};

/// Build the text payload sent to TTS, applying per-line emotion hints and character style.
pub fn build_tts_text(segment: &ScriptSegment, character: &Character) -> String {
    let mut text = segment.text.clone();

    if let Some(emotion) = segment.emotion_hint.as_ref().filter(|e| !e.is_empty()) {
        if !text.contains('[') {
            text = format!("[{emotion}] {text}");
        }
    } else if let Some(style) = character
        .voice_profile
        .style_prompt
        .as_ref()
        .filter(|s| !s.is_empty())
    {
        text = format!("[{style}] {text}");
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RoleType, SegmentKind, SegmentStatus, VoiceProfile};

    fn sample_character(style_prompt: Option<&str>) -> Character {
        Character {
            id: "c1".into(),
            name: "阿明".into(),
            role_type: RoleType::Character,
            voice_profile: VoiceProfile {
                style_prompt: style_prompt.map(str::to_string),
                ..VoiceProfile::default()
            },
            color: "#fff".into(),
        }
    }

    fn sample_segment(text: &str, emotion: Option<&str>) -> ScriptSegment {
        ScriptSegment {
            id: "s1".into(),
            character_id: "c1".into(),
            text: text.into(),
            language: "zh".into(),
            emotion_hint: emotion.map(str::to_string),
            speech_tags: Vec::new(),
            order: 0,
            segment_kind: SegmentKind::Dialogue,
            status: SegmentStatus::Pending,
            audio_path: None,
            duration_ms: None,
            error_message: None,
            sfx_cues: Vec::new(),
            sfx_id: None,
            pause_after_ms: 0,
            audio_asset_id: None,
        }
    }

    #[test]
    fn applies_emotion_hint_when_no_speech_tags() {
        let text = build_tts_text(
            &sample_segment("你聽到了嗎？", Some("緊張")),
            &sample_character(None),
        );
        assert_eq!(text, "[緊張] 你聽到了嗎？");
    }

    #[test]
    fn skips_emotion_when_speech_tags_present() {
        let text = build_tts_text(
            &sample_segment("等等。[pause]", Some("緊張")),
            &sample_character(None),
        );
        assert_eq!(text, "等等。[pause]");
    }

    #[test]
    fn applies_character_style_when_no_emotion() {
        let text = build_tts_text(
            &sample_segment("你好。", None),
            &sample_character(Some("溫柔")),
        );
        assert_eq!(text, "[溫柔] 你好。");
    }

    #[test]
    fn emotion_takes_priority_over_style() {
        let text = build_tts_text(
            &sample_segment("快跑！", Some("害怕")),
            &sample_character(Some("低沉")),
        );
        assert_eq!(text, "[害怕] 快跑！");
    }
}