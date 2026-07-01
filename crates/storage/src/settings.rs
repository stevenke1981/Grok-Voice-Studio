use std::fs;
use std::path::PathBuf;

use grok_voice_core::AppError;
use serde::{Deserialize, Serialize};

pub const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub xai_api_key: Option<String>,
    pub ffmpeg_path: Option<String>,
    pub default_language: Option<String>,
    #[serde(default = "default_true")]
    pub auto_save: bool,
    #[serde(default = "default_concurrency")]
    pub generation_concurrency: u32,
    #[serde(default)]
    pub cost_per_1k_chars: Option<f64>,
    #[serde(default)]
    pub onboarding_done: bool,
    #[serde(default = "default_lang_setting")]
    pub ui_language: String,
}

fn default_true() -> bool {
    true
}

fn default_concurrency() -> u32 {
    2
}

fn default_lang_setting() -> String {
    "zh".into()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            xai_api_key: None,
            ffmpeg_path: None,
            default_language: Some("zh".into()),
            auto_save: true,
            generation_concurrency: 2,
            cost_per_1k_chars: None,
            onboarding_done: false,
            ui_language: "zh".into(),
        }
    }
}

pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn app_data_dir() -> Result<PathBuf, AppError> {
        let dir = dirs::data_local_dir()
            .ok_or_else(|| AppError::Other("無法取得 app data 目錄".into()))?
            .join("GrokVoiceStudio");
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    pub fn open() -> Result<Self, AppError> {
        let dir = Self::app_data_dir()?;
        Ok(Self {
            path: dir.join(SETTINGS_FILE),
        })
    }

    pub fn load(&self) -> Result<AppSettings, AppError> {
        if !self.path.exists() {
            return Ok(AppSettings::default());
        }
        let content = fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), AppError> {
        let content = serde_json::to_string_pretty(settings)?;
        fs::write(&self.path, content)?;
        Ok(())
    }
}

