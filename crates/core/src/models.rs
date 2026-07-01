use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PROJECT_VERSION: u32 = 1;
pub const MAX_TTS_CHARS: usize = 15_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoleType {
    Narrator,
    Character,
    System,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VoiceProvider {
    #[default]
    Xai,
    Openai,
    Elevenlabs,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceProfile {
    pub provider: VoiceProvider,
    pub voice_id: String,
    pub language: String,
    pub style_prompt: Option<String>,
    pub volume_db: f32,
    pub pan: f32,
}

impl Default for VoiceProfile {
    fn default() -> Self {
        Self {
            provider: VoiceProvider::Xai,
            voice_id: "ara".to_string(),
            language: "zh".to_string(),
            style_prompt: None,
            volume_db: 0.0,
            pan: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub role_type: RoleType,
    pub voice_profile: VoiceProfile,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SegmentKind {
    #[default]
    Dialogue,
    Sfx,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SegmentStatus {
    Pending,
    Cached,
    Generating,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SfxCue {
    pub sfx_id: String,
    pub label: String,
    #[serde(default)]
    pub volume_db: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptSegment {
    pub id: String,
    pub character_id: String,
    pub text: String,
    pub language: String,
    pub emotion_hint: Option<String>,
    pub speech_tags: Vec<String>,
    #[serde(default)]
    pub segment_kind: SegmentKind,
    pub sfx_id: Option<String>,
    #[serde(default)]
    pub sfx_cues: Vec<SfxCue>,
    pub order: u32,
    pub pause_after_ms: u32,
    pub audio_asset_id: Option<String>,
    pub audio_path: Option<String>,
    pub duration_ms: Option<u64>,
    pub status: SegmentStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAsset {
    pub id: String,
    pub segment_id: String,
    pub file_path: String,
    pub duration_ms: Option<u64>,
    pub cache_key: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineClip {
    pub id: String,
    pub segment_id: String,
    pub start_ms: u64,
    pub duration_ms: u64,
    pub volume_db: f32,
    pub fade_in_ms: u32,
    pub fade_out_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineTrack {
    pub id: String,
    pub name: String,
    pub track_type: String,
    pub clips: Vec<TimelineClip>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Timeline {
    pub tracks: Vec<TimelineTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioCodec {
    Mp3,
    Wav,
    Flac,
    Pcm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPreset {
    pub id: String,
    pub name: String,
    pub codec: AudioCodec,
    pub sample_rate: u32,
    pub bit_rate: Option<u32>,
    pub normalize: bool,
}

impl Default for ExportPreset {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "Default MP3".to_string(),
            codec: AudioCodec::Mp3,
            sample_rate: 24000,
            bit_rate: Some(128_000),
            normalize: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub version: u32,
    pub id: String,
    pub title: String,
    pub script_raw: String,
    pub characters: Vec<Character>,
    pub segments: Vec<ScriptSegment>,
    pub timeline: Timeline,
    pub export_presets: Vec<ExportPreset>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            version: PROJECT_VERSION,
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            script_raw: String::new(),
            characters: Vec::new(),
            segments: Vec::new(),
            timeline: Timeline::default(),
            export_presets: vec![ExportPreset::default()],
            created_at: now,
            updated_at: now,
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn find_character_by_name(&self, name: &str) -> Option<&Character> {
        self.characters.iter().find(|c| c.name == name)
    }

    pub fn find_character_mut_by_name(&mut self, name: &str) -> Option<&mut Character> {
        self.characters.iter_mut().find(|c| c.name == name)
    }

    pub fn get_or_create_character(&mut self, name: &str, role_type: RoleType) -> String {
        if let Some(c) = self.find_character_by_name(name) {
            return c.id.clone();
        }
        let id = Uuid::new_v4().to_string();
        let colors = ["#6366f1", "#ec4899", "#14b8a6", "#f59e0b", "#8b5cf6", "#ef4444"];
        let color = colors[self.characters.len() % colors.len()].to_string();
        let default_voice = match role_type {
            RoleType::Narrator => "rex",
            _ => ["eve", "ara", "leo", "sal"][self.characters.len() % 4],
        };
        self.characters.push(Character {
            id: id.clone(),
            name: name.to_string(),
            role_type,
            voice_profile: VoiceProfile {
                voice_id: default_voice.to_string(),
                ..Default::default()
            },
            color,
        });
        id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    pub voice_id: String,
    pub name: String,
    pub language: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub is_custom: bool,
    pub tone: Option<String>,
    pub use_case: Option<String>,
    pub gender: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsOutputFormat {
    pub codec: AudioCodec,
    pub sample_rate: u32,
    pub bit_rate: Option<u32>,
}

impl Default for TtsOutputFormat {
    fn default() -> Self {
        Self {
            codec: AudioCodec::Mp3,
            sample_rate: 24000,
            bit_rate: Some(128_000),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    pub voice_id: String,
    pub language: String,
    pub output_format: TtsOutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsResult {
    pub audio_bytes: Vec<u8>,
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub audio_path: String,
    pub subtitle_path: Option<String>,
    pub stem_paths: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleFormat {
    Srt,
    Vtt,
    Ass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub codec: AudioCodec,
    pub sample_rate: u32,
    pub bit_rate: Option<u32>,
    pub normalize: bool,
    pub subtitle_format: SubtitleFormat,
    pub show_character_in_subtitle: bool,
    pub export_stems: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            codec: AudioCodec::Mp3,
            sample_rate: 24000,
            bit_rate: Some(128_000),
            normalize: true,
            subtitle_format: SubtitleFormat::Srt,
            show_character_in_subtitle: true,
            export_stems: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateJob {
    pub id: String,
    pub project_id: String,
    pub status: JobStatus,
    pub progress_current: usize,
    pub progress_total: usize,
    pub failed_segment_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}