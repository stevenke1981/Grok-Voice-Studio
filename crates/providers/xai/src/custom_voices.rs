use grok_voice_core::{AppError, VoiceInfo};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::provider::{map_status_error, TtsProvider, XaiTtsProvider};

pub const XAI_CUSTOM_VOICES_URL: &str = "https://api.x.ai/v1/custom-voices";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomVoiceDetails {
    pub voice_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub gender: Option<String>,
    pub accent: Option<String>,
    pub age: Option<String>,
    pub language: Option<String>,
    pub use_case: Option<String>,
    pub tone: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateCustomVoiceRequest {
    pub file_path: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub gender: Option<String>,
    pub accent: Option<String>,
    pub age: Option<String>,
    pub language: Option<String>,
    pub use_case: Option<String>,
    pub tone: Option<String>,
}

#[derive(Deserialize)]
struct CustomVoicesPage {
    voices: Vec<CustomVoiceDetails>,
    pagination_token: Option<String>,
}

impl XaiTtsProvider {
    pub async fn list_custom_voices(&self) -> Result<Vec<VoiceInfo>, AppError> {
        if self.api_key.is_empty() {
            return Err(AppError::MissingApiKey);
        }

        let mut all = Vec::new();
        let mut token: Option<String> = None;

        loop {
            let mut req = self
                .client
                .get(XAI_CUSTOM_VOICES_URL)
                .header("Authorization", self.auth_header())
                .query(&[("limit", "100")]);

            if let Some(t) = &token {
                req = req.query(&[("pagination_token", t.as_str())]);
            }

            let resp = req
                .send()
                .await
                .map_err(|e| AppError::ProviderUnavailable(e.to_string()))?;

            if resp.status() == StatusCode::FORBIDDEN {
                tracing::info!("custom voices API not enabled for this team");
                return Ok(vec![]);
            }

            if !resp.status().is_success() {
                let status = resp.status();
                let retry_after_secs = XaiTtsProvider::parse_retry_after(resp.headers());
                let body = resp.text().await.unwrap_or_default();
                return Err(map_status_error(status, &body, retry_after_secs));
            }

            let page: CustomVoicesPage = resp
                .json()
                .await
                .map_err(|e| AppError::ProviderUnavailable(e.to_string()))?;

            for v in page.voices {
                all.push(custom_to_voice_info(v));
            }

            token = page.pagination_token;
            if token.is_none() {
                break;
            }
        }

        Ok(all)
    }

    pub async fn list_all_voices(&self) -> Result<Vec<VoiceInfo>, AppError> {
        let builtin = self.list_voices().await.unwrap_or_default();
        let custom = self.list_custom_voices().await.unwrap_or_default();

        let mut merged = builtin;
        let existing: std::collections::HashSet<_> =
            merged.iter().map(|v| v.voice_id.clone()).collect();
        for v in custom {
            if !existing.contains(&v.voice_id) {
                merged.push(v);
            }
        }
        Ok(merged)
    }

    pub async fn create_custom_voice(
        &self,
        req: CreateCustomVoiceRequest,
    ) -> Result<CustomVoiceDetails, AppError> {
        if self.api_key.is_empty() {
            return Err(AppError::MissingApiKey);
        }

        let path = std::path::Path::new(&req.file_path);
        if !path.exists() {
            return Err(AppError::Other(format!("找不到參考音訊: {}", req.file_path)));
        }

        let bytes = std::fs::read(path).map_err(AppError::Io)?;
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("reference.wav");
        let mime = match path.extension().and_then(|e| e.to_str()) {
            Some("mp3") => "audio/mpeg",
            Some("flac") => "audio/flac",
            Some("ogg") => "audio/ogg",
            _ => "audio/wav",
        };
        let file_part = reqwest::multipart::Part::bytes(bytes)
            .file_name(file_name.to_string())
            .mime_str(mime)
            .map_err(|e| AppError::Other(format!("multipart mime: {e}")))?;
        let mut form = reqwest::multipart::Form::new().part("file", file_part);

        if let Some(name) = req.name.filter(|s| !s.is_empty()) {
            form = form.text("name", name);
        }
        if let Some(v) = req.description.filter(|s| !s.is_empty()) {
            form = form.text("description", v);
        }
        if let Some(v) = req.gender.filter(|s| !s.is_empty()) {
            form = form.text("gender", v);
        }
        if let Some(v) = req.accent.filter(|s| !s.is_empty()) {
            form = form.text("accent", v);
        }
        if let Some(v) = req.age.filter(|s| !s.is_empty()) {
            form = form.text("age", v);
        }
        if let Some(v) = req.language.filter(|s| !s.is_empty()) {
            form = form.text("language", v);
        }
        if let Some(v) = req.use_case.filter(|s| !s.is_empty()) {
            form = form.text("use_case", v);
        }
        if let Some(v) = req.tone.filter(|s| !s.is_empty()) {
            form = form.text("tone", v);
        }

        let resp = self
            .client
            .post(XAI_CUSTOM_VOICES_URL)
            .header("Authorization", self.auth_header())
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::ProviderUnavailable(e.to_string()))?;

        if resp.status() == StatusCode::FORBIDDEN {
            return Err(AppError::Other(
                "API 建立自訂語音需要 Enterprise 方案，請至 xAI Console 建立後複製 Voice ID".into(),
            ));
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let retry_after_secs = XaiTtsProvider::parse_retry_after(resp.headers());
            let body = resp.text().await.unwrap_or_default();
            return Err(map_status_error(status, &body, retry_after_secs));
        }

        resp.json()
            .await
            .map_err(|e| AppError::ProviderUnavailable(e.to_string()))
    }

    pub async fn delete_custom_voice(&self, voice_id: &str) -> Result<bool, AppError> {
        if self.api_key.is_empty() {
            return Err(AppError::MissingApiKey);
        }

        let url = format!("{XAI_CUSTOM_VOICES_URL}/{voice_id}");
        let resp = self
            .client
            .delete(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| AppError::ProviderUnavailable(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let retry_after_secs = XaiTtsProvider::parse_retry_after(resp.headers());
            let body = resp.text().await.unwrap_or_default();
            return Err(map_status_error(status, &body, retry_after_secs));
        }

        Ok(true)
    }
}

fn custom_to_voice_info(v: CustomVoiceDetails) -> VoiceInfo {
    let name = v
        .name
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| v.voice_id.clone());
    let mut description = v.description.unwrap_or_default();
    let meta: Vec<String> = [
        v.tone.as_deref(),
        v.use_case.as_deref(),
        v.gender.as_deref(),
        v.accent.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::to_string)
    .collect();
    if !meta.is_empty() {
        if !description.is_empty() {
            description.push_str(" · ");
        }
        description.push_str(&meta.join(", "));
    }

    VoiceInfo {
        voice_id: v.voice_id,
        name: format!("{name} (自訂)"),
        language: v.language,
        description: if description.is_empty() {
            None
        } else {
            Some(description)
        },
        is_custom: true,
        tone: v.tone,
        use_case: v.use_case,
        gender: v.gender,
    }
}