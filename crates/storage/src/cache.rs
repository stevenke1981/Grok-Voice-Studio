use std::path::Path;

use chrono::Utc;
use grok_voice_core::{AppError, TtsOutputFormat};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub cache_key: String,
    pub provider: String,
    pub voice_id: String,
    pub language: String,
    pub text_hash: String,
    pub file_path: String,
    pub duration_ms: Option<u64>,
    pub created_at: String,
}

pub struct AudioCache {
    conn: Connection,
}

impl AudioCache {
    pub fn open(db_path: &Path) -> Result<Self, AppError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path).map_err(|e| AppError::Other(e.to_string()))?;
        let cache = Self { conn };
        cache.migrate()?;
        Ok(cache)
    }

    fn migrate(&self) -> Result<(), AppError> {
        self.conn
            .execute_batch(
                r#"
            CREATE TABLE IF NOT EXISTS audio_cache (
                cache_key TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                voice_id TEXT NOT NULL,
                language TEXT NOT NULL,
                text_hash TEXT NOT NULL,
                file_path TEXT NOT NULL,
                duration_ms INTEGER,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                path TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                status TEXT NOT NULL,
                progress_current INTEGER NOT NULL,
                progress_total INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
            )
            .map_err(|e| AppError::Other(e.to_string()))?;
        Ok(())
    }

    pub fn compute_cache_key(
        provider: &str,
        voice_id: &str,
        language: &str,
        text: &str,
        format: &TtsOutputFormat,
    ) -> String {
        let codec = format!("{:?}", format.codec);
        let payload = format!(
            "{provider}|{voice_id}|{language}|{text}|{codec}|{}|{:?}",
            format.sample_rate, format.bit_rate
        );
        let hash = Sha256::digest(payload.as_bytes());
        hex::encode(hash)
    }

    pub fn text_hash(text: &str) -> String {
        let hash = Sha256::digest(text.as_bytes());
        hex::encode(hash)
    }

    pub fn lookup(&self, cache_key: &str) -> Result<Option<CacheEntry>, AppError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT cache_key, provider, voice_id, language, text_hash, file_path, duration_ms, created_at
             FROM audio_cache WHERE cache_key = ?1",
            )
            .map_err(|e| AppError::Other(e.to_string()))?;

        let result = stmt.query_row(params![cache_key], |row| {
            Ok(CacheEntry {
                cache_key: row.get(0)?,
                provider: row.get(1)?,
                voice_id: row.get(2)?,
                language: row.get(3)?,
                text_hash: row.get(4)?,
                file_path: row.get(5)?,
                duration_ms: row.get(6)?,
                created_at: row.get(7)?,
            })
        });

        match result {
            Ok(entry) => {
                if Path::new(&entry.file_path).exists() {
                    Ok(Some(entry))
                } else {
                    self.delete(cache_key)?;
                    Ok(None)
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Other(e.to_string())),
        }
    }

    pub fn insert(&self, entry: &CacheEntry) -> Result<(), AppError> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO audio_cache
             (cache_key, provider, voice_id, language, text_hash, file_path, duration_ms, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    entry.cache_key,
                    entry.provider,
                    entry.voice_id,
                    entry.language,
                    entry.text_hash,
                    entry.file_path,
                    entry.duration_ms,
                    entry.created_at,
                ],
            )
            .map_err(|e| AppError::Other(e.to_string()))?;
        Ok(())
    }

    pub fn delete(&self, cache_key: &str) -> Result<(), AppError> {
        self.conn
            .execute(
                "DELETE FROM audio_cache WHERE cache_key = ?1",
                params![cache_key],
            )
            .map_err(|e| AppError::Other(e.to_string()))?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<(), AppError> {
        self.conn
            .execute("DELETE FROM audio_cache", [])
            .map_err(|e| AppError::Other(e.to_string()))?;
        Ok(())
    }

    pub fn register_recent_project(
        &self,
        id: &str,
        title: &str,
        path: &str,
    ) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT OR REPLACE INTO projects (id, title, path, created_at, updated_at)
             VALUES (?1, ?2, ?3, COALESCE((SELECT created_at FROM projects WHERE id = ?1), ?4), ?4)",
                params![id, title, path, now],
            )
            .map_err(|e| AppError::Other(e.to_string()))?;
        Ok(())
    }

    pub fn upsert_job(
        &self,
        id: &str,
        project_id: &str,
        status: &str,
        progress_current: usize,
        progress_total: usize,
    ) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT OR REPLACE INTO jobs (id, project_id, status, progress_current, progress_total, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, COALESCE((SELECT created_at FROM jobs WHERE id = ?1), ?6), ?6)",
                params![id, project_id, status, progress_current as i64, progress_total as i64, now],
            )
            .map_err(|e| AppError::Other(e.to_string()))?;
        Ok(())
    }

    pub fn get_job(&self, id: &str) -> Result<Option<(String, String, i64, i64)>, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT project_id, status, progress_current, progress_total FROM jobs WHERE id = ?1")
            .map_err(|e| AppError::Other(e.to_string()))?;
        let result = stmt.query_row(params![id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        });
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Other(e.to_string())),
        }
    }

    pub fn delete_orphan_cache(&self) -> Result<usize, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT cache_key, file_path FROM audio_cache")
            .map_err(|e| AppError::Other(e.to_string()))?;
        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| AppError::Other(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        let mut deleted = 0usize;
        for (key, path) in rows {
            if !Path::new(&path).exists() {
                self.delete(&key)?;
                deleted += 1;
            }
        }
        Ok(deleted)
    }

    pub fn list_recent_projects(&self, limit: usize) -> Result<Vec<(String, String, String)>, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, path FROM projects ORDER BY updated_at DESC LIMIT ?1")
            .map_err(|e| AppError::Other(e.to_string()))?;
        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|e| AppError::Other(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Other(e.to_string()))
    }
}

pub fn new_cache_entry(
    cache_key: String,
    provider: &str,
    voice_id: &str,
    language: &str,
    text: &str,
    file_path: String,
    duration_ms: Option<u64>,
) -> CacheEntry {
    CacheEntry {
        cache_key,
        provider: provider.to_string(),
        voice_id: voice_id.to_string(),
        language: language.to_string(),
        text_hash: AudioCache::text_hash(text),
        file_path,
        duration_ms,
        created_at: Utc::now().to_rfc3339(),
    }
}