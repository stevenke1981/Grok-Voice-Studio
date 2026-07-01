use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
pub struct LogEntry {
    pub id: u64,
    pub timestamp: String,
    pub level: String,
    pub category: String,
    pub message: String,
}

#[derive(Clone)]
pub struct LogStore {
    inner: Arc<LogStoreInner>,
}

struct LogStoreInner {
    entries: Mutex<Vec<LogEntry>>,
    next_id: AtomicU64,
    max_entries: usize,
}

impl LogStore {
    pub fn new(max_entries: usize) -> Self {
        Self {
            inner: Arc::new(LogStoreInner {
                entries: Mutex::new(Vec::new()),
                next_id: AtomicU64::new(1),
                max_entries,
            }),
        }
    }

    pub fn append(&self, level: &str, category: &str, message: impl Into<String>) {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let entry = LogEntry {
            id,
            timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            level: level.into(),
            category: category.into(),
            message: message.into(),
        };
        if let Ok(mut entries) = self.inner.entries.lock() {
            entries.push(entry.clone());
            let excess = entries.len().saturating_sub(self.inner.max_entries);
            if excess > 0 {
                entries.drain(0..excess);
            }
        }
        tracing::info!(target: "app_log", "[{}] {}: {}", category, level, entry.message);
    }

    pub fn append_with_emit(
        &self,
        app: &AppHandle,
        level: &str,
        category: &str,
        message: impl Into<String>,
    ) {
        self.append(level, category, message);
        if let Ok(entries) = self.inner.entries.lock() {
            if let Some(entry) = entries.last() {
                let _ = app.emit("app-log", entry.clone());
            }
        }
    }

    pub fn list(&self, limit: Option<usize>) -> Vec<LogEntry> {
        let entries = self.inner.entries.lock().ok();
        let Some(entries) = entries else {
            return vec![];
        };
        let limit = limit.unwrap_or(500);
        let start = entries.len().saturating_sub(limit);
        entries[start..].to_vec()
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.inner.entries.lock() {
            entries.clear();
        }
    }
}

pub fn log_info(store: &LogStore, category: &str, message: impl Into<String>) {
    store.append("info", category, message);
}

pub fn log_warn(store: &LogStore, category: &str, message: impl Into<String>) {
    store.append("warn", category, message);
}

pub fn log_error(store: &LogStore, category: &str, message: impl Into<String>) {
    store.append("error", category, message);
}