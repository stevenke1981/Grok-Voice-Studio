use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct StoryConvertProgressEvent {
    pub attempt: u32,
    pub phase: String,
    pub message: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct ApiRetryEvent {
    pub category: String,
    pub context: Option<String>,
    pub message: String,
    pub attempt: u32,
    pub max_retries: u32,
    pub delay_secs: u64,
}

pub fn retry_subject(context: Option<&str>, category: &str) -> String {
    match context {
        Some("voices") => "Voice 同步".into(),
        Some("story") => "Story 轉換".into(),
        Some(id) if id.starts_with("preview:") => {
            format!("試聽 {}", id.strip_prefix("preview:").unwrap_or(id))
        }
        Some(id) => format!("段落 {id}"),
        None => match category {
            "chat" => "Story 轉換".into(),
            "tts" => "TTS 生成".into(),
            "voices" => "Voice 同步".into(),
            _ => "xAI API".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_preview_context() {
        assert_eq!(
            retry_subject(Some("preview:ara"), "tts"),
            "試聽 ara"
        );
        assert_eq!(retry_subject(Some("voices"), "voices"), "Voice 同步");
    }
}