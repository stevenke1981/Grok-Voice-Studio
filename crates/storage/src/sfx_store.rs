use std::fs;
use std::path::{Path, PathBuf};

use grok_voice_core::{builtin_catalog, AppError, SoundEffect, SfxCategory};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::sfx_wav::{freq_for_sfx_id, generate_placeholder_wav};
use crate::SettingsStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SfxCatalogFile {
    sounds: Vec<SoundEffect>,
}

pub struct SfxStore {
    dir: PathBuf,
    catalog_path: PathBuf,
}

impl SfxStore {
    pub fn open() -> Result<Self, AppError> {
        let base = SettingsStore::app_data_dir()?;
        let dir = base.join("sfx");
        fs::create_dir_all(&dir)?;
        let store = Self {
            catalog_path: base.join("sfx_catalog.json"),
            dir,
        };
        store.ensure_initialized()?;
        Ok(store)
    }

    fn ensure_initialized(&self) -> Result<(), AppError> {
        if !self.catalog_path.exists() {
            let catalog = SfxCatalogFile {
                sounds: builtin_catalog(),
            };
            self.write_catalog(&catalog)?;
        }
        for sfx in self.load_catalog()? {
            self.ensure_file(&sfx)?;
        }
        Ok(())
    }

    fn write_catalog(&self, catalog: &SfxCatalogFile) -> Result<(), AppError> {
        let json = serde_json::to_string_pretty(catalog).map_err(AppError::Json)?;
        fs::write(&self.catalog_path, json).map_err(AppError::Io)?;
        Ok(())
    }

    pub fn load_catalog(&self) -> Result<Vec<SoundEffect>, AppError> {
        let content = fs::read_to_string(&self.catalog_path).map_err(AppError::Io)?;
        let file: SfxCatalogFile = serde_json::from_str(&content).map_err(AppError::Json)?;
        Ok(file.sounds)
    }

    pub fn list(&self) -> Result<Vec<SoundEffect>, AppError> {
        self.load_catalog()
    }

    pub fn resolve_path(&self, sfx_id: &str) -> Result<PathBuf, AppError> {
        let catalog = self.load_catalog()?;
        let sfx = catalog
            .iter()
            .find(|s| s.id == sfx_id)
            .ok_or_else(|| AppError::Other(format!("音效不存在: {sfx_id}")))?;
        self.ensure_file(sfx)
    }

    pub fn ensure_file(&self, sfx: &SoundEffect) -> Result<PathBuf, AppError> {
        let path = self.dir.join(&sfx.file_name);
        if path.exists() {
            return Ok(path);
        }
        let wav = generate_placeholder_wav(sfx.duration_ms, freq_for_sfx_id(&sfx.id));
        fs::write(&path, wav).map_err(AppError::Io)?;
        Ok(path)
    }

    pub fn import_file(
        &self,
        source: &Path,
        name: &str,
        category: SfxCategory,
    ) -> Result<SoundEffect, AppError> {
        let id = format!("custom_{}", &Uuid::new_v4().to_string()[..8]);
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("wav");
        let file_name = format!("{id}.{ext}");
        let dest = self.dir.join(&file_name);
        fs::copy(source, &dest).map_err(AppError::Io)?;

        let entry = SoundEffect {
            id: id.clone(),
            name: name.to_string(),
            name_en: name.to_string(),
            category,
            duration_ms: 1500,
            tags: vec!["custom".into()],
            file_name,
            builtin: false,
        };

        let mut sounds = self.load_catalog()?;
        sounds.push(entry.clone());
        self.write_catalog(&SfxCatalogFile { sounds })?;
        Ok(entry)
    }
}