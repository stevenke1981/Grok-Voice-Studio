use std::time::Duration;

use async_trait::async_trait;
use grok_voice_core::{
    AppError, AudioCodec, TtsOutputFormat, TtsRequest, TtsResult, VoiceInfo, MAX_TTS_CHARS,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

pub const XAI_TTS_URL: &str = "https://api.x.ai/v1/tts";
pub const XAI_VOICES_URL: &str = "https://api.x.ai/v1/tts/voices";

#[async_trait]
pub trait TtsProvider: Send + Sync {
    async fn list_voices(&self) -> Result<Vec<VoiceInfo>, AppError>;
    async fn get_voice(&self, voice_id: &str) -> Result<VoiceInfo, AppError>;
    async fn synthesize(&self, req: TtsRequest) -> Result<TtsResult, AppError>;
}

#[derive(Clone)]
pub struct XaiTtsProvider {
    pub(crate) client: reqwest::Client,
    pub(crate) api_key: String,
}

impl XaiTtsProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("failed to build HTTP client");
        Self {
            client,
            api_key: api_key.into(),
        }
    }

    pub fn from_env() -> Result<Self, AppError> {
        let api_key = std::env::var("XAI_API_KEY").map_err(|_| AppError::MissingApiKey)?;
        Ok(Self::new(api_key))
    }

    pub(crate) fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    async fn request_with_retry<F, Fut, T>(&self, mut f: F) -> Result<T, AppError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, AppError>>,
    {
        let mut attempt = 0;
        loop {
            match f().await {
                Ok(v) => return Ok(v),
                Err(AppError::RateLimited) if attempt < 3 => {
                    attempt += 1;
                    let delay = Duration::from_secs(2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

#[derive(Serialize)]
struct XaiTtsRequestBody {
    text: String,
    voice_id: String,
    language: String,
    output_format: XaiOutputFormat,
}

#[derive(Serialize)]
struct XaiOutputFormat {
    codec: String,
    sample_rate: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    bit_rate: Option<u32>,
}

fn codec_to_str(codec: &AudioCodec) -> &'static str {
    match codec {
        AudioCodec::Mp3 => "mp3",
        AudioCodec::Wav => "wav",
        AudioCodec::Flac => "flac",
        AudioCodec::Pcm => "pcm",
    }
}

fn format_to_xai(fmt: &TtsOutputFormat) -> XaiOutputFormat {
    XaiOutputFormat {
        codec: codec_to_str(&fmt.codec).to_string(),
        sample_rate: fmt.sample_rate,
        bit_rate: fmt.bit_rate,
    }
}

#[derive(Deserialize)]
struct VoicesResponse {
    voices: Vec<XaiVoice>,
}

#[derive(Deserialize)]
struct XaiVoice {
    voice_id: String,
    name: Option<String>,
    language: Option<String>,
    description: Option<String>,
}

pub(crate) fn map_status_error(status: StatusCode, body: &str) -> AppError {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => AppError::AuthFailed,
        StatusCode::TOO_MANY_REQUESTS => AppError::RateLimited,
        StatusCode::PAYMENT_REQUIRED => AppError::QuotaExceeded,
        _ => AppError::ProviderUnavailable(format!("HTTP {status}: {body}")),
    }
}

#[async_trait]
impl TtsProvider for XaiTtsProvider {
    async fn list_voices(&self) -> Result<Vec<VoiceInfo>, AppError> {
        if self.api_key.is_empty() {
            return Err(AppError::MissingApiKey);
        }

        self.request_with_retry(|| async {
            let resp = self
                .client
                .get(XAI_VOICES_URL)
                .header("Authorization", self.auth_header())
                .send()
                .await
                .map_err(|e| AppError::ProviderUnavailable(e.to_string()))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(map_status_error(status, &body));
            }

            let data: VoicesResponse = resp
                .json()
                .await
                .map_err(|e| AppError::ProviderUnavailable(e.to_string()))?;

            Ok(data
                .voices
                .into_iter()
                .map(|v| VoiceInfo {
                    voice_id: v.voice_id,
                    name: v.name.unwrap_or_else(|| "Unknown".to_string()),
                    language: v.language,
                    description: v.description,
                    is_custom: false,
                    tone: None,
                    use_case: None,
                    gender: None,
                })
                .collect())
        })
        .await
    }

    async fn get_voice(&self, voice_id: &str) -> Result<VoiceInfo, AppError> {
        let voices = self.list_all_voices().await?;
        voices
            .into_iter()
            .find(|v| v.voice_id == voice_id)
            .ok_or_else(|| AppError::Other(format!("Voice not found: {voice_id}")))
    }

    async fn synthesize(&self, req: TtsRequest) -> Result<TtsResult, AppError> {
        if self.api_key.is_empty() {
            return Err(AppError::MissingApiKey);
        }
        if req.text.chars().count() > MAX_TTS_CHARS {
            return Err(AppError::TextTooLong {
                chars: req.text.chars().count(),
                max: MAX_TTS_CHARS,
            });
        }

        let body = XaiTtsRequestBody {
            text: req.text,
            voice_id: req.voice_id,
            language: req.language,
            output_format: format_to_xai(&req.output_format),
        };

        self.request_with_retry(|| async {
            let resp = self
                .client
                .post(XAI_TTS_URL)
                .header("Authorization", self.auth_header())
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::ProviderUnavailable(e.to_string()))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body_text = resp.text().await.unwrap_or_default();
                return Err(map_status_error(status, &body_text));
            }

            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("audio/mpeg")
                .to_string();

            let audio_bytes = resp
                .bytes()
                .await
                .map_err(|e| AppError::ProviderUnavailable(e.to_string()))?
                .to_vec();

            Ok(TtsResult {
                audio_bytes,
                content_type,
            })
        })
        .await
    }
}

pub fn fallback_voices() -> Vec<VoiceInfo> {
    vec![
        VoiceInfo {
            voice_id: "eve".into(),
            name: "Eve".into(),
            language: Some("multilingual".into()),
            description: Some("活潑、熱情、女性角色".into()),
            is_custom: false,
            tone: None,
            use_case: None,
            gender: None,
        },
        VoiceInfo {
            voice_id: "ara".into(),
            name: "Ara".into(),
            language: Some("multilingual".into()),
            description: Some("溫暖、親切、對話角色".into()),
            is_custom: false,
            tone: None,
            use_case: None,
            gender: None,
        },
        VoiceInfo {
            voice_id: "rex".into(),
            name: "Rex".into(),
            language: Some("multilingual".into()),
            description: Some("清晰、專業、商業旁白".into()),
            is_custom: false,
            tone: None,
            use_case: None,
            gender: None,
        },
        VoiceInfo {
            voice_id: "sal".into(),
            name: "Sal".into(),
            language: Some("multilingual".into()),
            description: Some("中性、平衡、通用旁白".into()),
            is_custom: false,
            tone: None,
            use_case: None,
            gender: None,
        },
        VoiceInfo {
            voice_id: "leo".into(),
            name: "Leo".into(),
            language: Some("multilingual".into()),
            description: Some("權威、低沉、命令感角色".into()),
            is_custom: false,
            tone: None,
            use_case: None,
            gender: None,
        },
    ]
}