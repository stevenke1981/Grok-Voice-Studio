use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use grok_voice_storage::AudioCache;

use crate::log_store::LogStore;
use crate::services::{GenerationControls, ProjectService};

pub struct AppState {
    pub project_svc: Arc<ProjectService>,
    pub cache: Arc<Mutex<Option<AudioCache>>>,
    pub generation_controls: GenerationControls,
    pub active_job_id: Arc<Mutex<Option<String>>>,
    pub batch_tts_retries: Arc<AtomicUsize>,
    pub logs: LogStore,
}

impl AppState {
    pub fn new() -> Self {
        let db_path = grok_voice_storage::SettingsStore::app_data_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("cache.db");
        Self {
            project_svc: Arc::new(ProjectService::new()),
            cache: Arc::new(Mutex::new(AudioCache::open(&db_path).ok())),
            generation_controls: GenerationControls::default(),
            active_job_id: Arc::new(Mutex::new(None)),
            batch_tts_retries: Arc::new(AtomicUsize::new(0)),
            logs: LogStore::new(2000),
        }
    }
}