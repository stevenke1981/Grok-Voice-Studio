use grok_voice_core::{AppError, StoryScript, STORY_SYSTEM_PROMPT};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

const XAI_CHAT_URL: &str = "https://api.x.ai/v1/chat/completions";

pub struct XaiChatClient {
    client: reqwest::Client,
    api_key: String,
}

impl XaiChatClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
        }
    }

    pub async fn story_to_script(
        &self,
        story: &str,
        style: Option<&str>,
    ) -> Result<StoryScript, AppError> {
        let style_hint = style.unwrap_or("一般敘事");
        let user_prompt = format!(
            "風格：{style_hint}\n\n故事內容：\n{story}\n\n請輸出 JSON 格式的多角色配音劇本。"
        );

        let body = ChatRequest {
            model: "grok-3-mini".to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: STORY_SYSTEM_PROMPT.into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: user_prompt,
                },
            ],
            temperature: 0.3,
        };

        let resp = self
            .client
            .post(XAI_CHAT_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::ProviderUnavailable(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(match status {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => AppError::AuthFailed,
                StatusCode::TOO_MANY_REQUESTS => AppError::RateLimited,
                _ => AppError::ProviderUnavailable(format!("HTTP {status}: {text}")),
            });
        }

        let data: ChatResponse = resp
            .json()
            .await
            .map_err(|e| AppError::ProviderUnavailable(e.to_string()))?;

        let content = data
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AppError::ProviderUnavailable("LLM 回傳為空".into()))?;

        parse_story_json(&content)
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

fn parse_story_json(content: &str) -> Result<StoryScript, AppError> {
    let trimmed = content.trim();
    let json_str = if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            &trimmed[start..=end]
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    serde_json::from_str(json_str).map_err(|e| {
        // Try repair: ask to fix would need second call; for now return parse error
        AppError::Other(format!("Story JSON 解析失敗: {e}"))
    })
}

pub async fn story_to_script_with_retry(
    client: &XaiChatClient,
    story: &str,
    style: Option<&str>,
) -> Result<StoryScript, AppError> {
    match client.story_to_script(story, style).await {
        Ok(s) => Ok(s),
        Err(_) => {
            // One retry with explicit instruction
            let retry_story = format!("{story}\n\n請確保輸出有效 JSON。");
            client.story_to_script(&retry_story, style).await
        }
    }
}