use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use grok_voice_core::{AppError, AudioCodec, TtsOutputFormat, TtsRequest, TtsResult};
use serde::Deserialize;
use serde_json::json;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        client::IntoClientRequest,
        http::header::AUTHORIZATION,
        Message,
    },
};

use crate::provider::{codec_to_str, XaiTtsProvider};
use crate::retry::{is_rate_limit_message, with_retry_category};

pub const XAI_TTS_WS_URL: &str = "wss://api.x.ai/v1/tts";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamEventKind {
    AudioDelta,
    AudioDone,
    Error,
    Other,
}

#[derive(Debug, Clone)]
pub struct StreamEvent {
    pub kind: StreamEventKind,
    pub audio_chunk: Option<Vec<u8>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WsEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

pub fn build_streaming_url(req: &TtsRequest) -> String {
    let codec = codec_to_str(&req.output_format.codec);
    let mut url = format!(
        "{XAI_TTS_WS_URL}?language={}&voice={}&codec={}",
        urlencoding::encode(&req.language),
        urlencoding::encode(&req.voice_id),
        urlencoding::encode(codec),
    );
    url.push_str(&format!("&sample_rate={}", req.output_format.sample_rate));
    if let Some(bit_rate) = req.output_format.bit_rate {
        if matches!(req.output_format.codec, AudioCodec::Mp3) {
            url.push_str(&format!("&bit_rate={bit_rate}"));
        }
    }
    url
}

pub fn content_type_for_format(fmt: &TtsOutputFormat) -> &'static str {
    match fmt.codec {
        AudioCodec::Mp3 => "audio/mpeg",
        AudioCodec::Wav => "audio/wav",
        AudioCodec::Flac => "audio/flac",
        AudioCodec::Pcm => "audio/pcm",
    }
}

pub fn parse_stream_event(raw: &str) -> Result<StreamEvent, AppError> {
    let event: WsEvent = serde_json::from_str(raw)
        .map_err(|e| AppError::ProviderUnavailable(format!("invalid WS event: {e}")))?;

    match event.event_type.as_str() {
        "audio.delta" => {
            let delta = event.delta.ok_or_else(|| {
                AppError::ProviderUnavailable("audio.delta missing delta field".into())
            })?;
            let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &delta)
                .map_err(|e| AppError::ProviderUnavailable(format!("base64 decode failed: {e}")))?;
            Ok(StreamEvent {
                kind: StreamEventKind::AudioDelta,
                audio_chunk: Some(bytes),
                error_message: None,
            })
        }
        "audio.done" => Ok(StreamEvent {
            kind: StreamEventKind::AudioDone,
            audio_chunk: None,
            error_message: None,
        }),
        "error" => {
            let msg = event
                .message
                .or(event.error)
                .unwrap_or_else(|| "streaming TTS error".into());
            Ok(StreamEvent {
                kind: StreamEventKind::Error,
                audio_chunk: None,
                error_message: Some(msg),
            })
        }
        other => Ok(StreamEvent {
            kind: StreamEventKind::Other,
            audio_chunk: None,
            error_message: Some(other.to_string()),
        }),
    }
}

impl XaiTtsProvider {
    pub async fn synthesize_streaming(&self, req: TtsRequest) -> Result<TtsResult, AppError> {
        if self.api_key.is_empty() {
            return Err(AppError::MissingApiKey);
        }

        let url = build_streaming_url(&req);
        let auth = self.auth_header();

        let mut request = url
            .into_client_request()
            .map_err(|e| AppError::ProviderUnavailable(e.to_string()))?;
        request
            .headers_mut()
            .insert(AUTHORIZATION, auth.parse().map_err(|e| {
                AppError::ProviderUnavailable(format!("invalid auth header: {e}"))
            })?);

        let connect = tokio::time::timeout(Duration::from_secs(30), connect_async(request)).await;
        let (mut ws, _) = connect
            .map_err(|_| AppError::ProviderUnavailable("WebSocket connect timeout".into()))?
            .map_err(|e| AppError::ProviderUnavailable(format!("WebSocket connect failed: {e}")))?;

        ws.send(Message::Text(
            json!({ "type": "text.delta", "delta": req.text }).to_string().into(),
        ))
        .await
        .map_err(|e| AppError::ProviderUnavailable(format!("WebSocket send failed: {e}")))?;

        ws.send(Message::Text(json!({ "type": "text.done" }).to_string().into()))
            .await
            .map_err(|e| AppError::ProviderUnavailable(format!("WebSocket send failed: {e}")))?;

        let mut audio = Vec::new();
        let read = tokio::time::timeout(Duration::from_secs(120), async {
            while let Some(msg) = ws.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let event = parse_stream_event(&text)?;
                        match event.kind {
                            StreamEventKind::AudioDelta => {
                                if let Some(chunk) = event.audio_chunk {
                                    audio.extend(chunk);
                                }
                            }
                            StreamEventKind::AudioDone => break,
                            StreamEventKind::Error => {
                                let msg = event
                                    .error_message
                                    .unwrap_or_else(|| "streaming TTS error".into());
                                let lower = msg.to_ascii_lowercase();
                                if lower.contains("quota exceeded") || lower.contains("payment required")
                                {
                                    return Err(AppError::QuotaExceeded);
                                }
                                if is_rate_limit_message(&msg) {
                                    return Err(AppError::RateLimited {
                                        retry_after_secs: None,
                                    });
                                }
                                return Err(AppError::ProviderUnavailable(msg));
                            }
                            StreamEventKind::Other => {}
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Ok(Message::Ping(payload)) => {
                        ws.send(Message::Pong(payload)).await.ok();
                    }
                    Ok(_) => {}
                    Err(e) => {
                        return Err(AppError::ProviderUnavailable(format!(
                            "WebSocket read failed: {e}"
                        )));
                    }
                }
            }
            Ok(())
        })
        .await;

        read.map_err(|_| AppError::ProviderUnavailable("WebSocket read timeout".into()))??;

        if audio.is_empty() {
            return Err(AppError::ProviderUnavailable(
                "streaming TTS returned no audio".into(),
            ));
        }

        Ok(TtsResult {
            audio_bytes: audio,
            content_type: content_type_for_format(&req.output_format).to_string(),
        })
    }

    pub async fn synthesize_preferred(
        &self,
        req: TtsRequest,
        use_streaming: bool,
    ) -> Result<TtsResult, AppError> {
        with_retry_category(
            || async { self.synthesize_preferred_once(req.clone(), use_streaming).await },
            "tts",
        )
        .await
    }

    async fn synthesize_preferred_once(
        &self,
        req: TtsRequest,
        use_streaming: bool,
    ) -> Result<TtsResult, AppError> {
        if use_streaming {
            match self.synthesize_streaming(req.clone()).await {
                Ok(result) => Ok(result),
                Err(e) => {
                    if matches!(e, AppError::RateLimited { .. }) {
                        return Err(e);
                    }
                    tracing::warn!(target: "xai_tts", "streaming TTS failed, falling back to REST: {e}");
                    self.synthesize_once(req).await
                }
            }
        } else {
            self.synthesize_once(req).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    #[test]
    fn build_streaming_url_includes_voice_language_codec() {
        let req = TtsRequest {
            text: "hello".into(),
            voice_id: "eve".into(),
            language: "zh".into(),
            output_format: TtsOutputFormat::default(),
        };
        let url = build_streaming_url(&req);
        assert!(url.starts_with("wss://api.x.ai/v1/tts?"));
        assert!(url.contains("language=zh"));
        assert!(url.contains("voice=eve"));
        assert!(url.contains("codec=mp3"));
        assert!(url.contains("sample_rate=24000"));
        assert!(url.contains("bit_rate=128000"));
    }

    #[test]
    fn parse_audio_delta_decodes_base64() {
        let payload = base64::engine::general_purpose::STANDARD.encode([1, 2, 3]);
        let raw = format!(r#"{{"type":"audio.delta","delta":"{payload}"}}"#);
        let event = parse_stream_event(&raw).unwrap();
        assert_eq!(event.kind, StreamEventKind::AudioDelta);
        assert_eq!(event.audio_chunk, Some(vec![1, 2, 3]));
    }

    #[test]
    fn parse_audio_done() {
        let event = parse_stream_event(r#"{"type":"audio.done"}"#).unwrap();
        assert_eq!(event.kind, StreamEventKind::AudioDone);
    }

    #[test]
    fn parse_error_event() {
        let event = parse_stream_event(r#"{"type":"error","message":"quota exceeded"}"#).unwrap();
        assert_eq!(event.kind, StreamEventKind::Error);
        assert_eq!(event.error_message.as_deref(), Some("quota exceeded"));
    }
}