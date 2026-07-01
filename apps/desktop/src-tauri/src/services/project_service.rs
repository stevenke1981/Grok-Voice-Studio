use std::path::Path;

use grok_voice_core::{
    create_project, load_project, save_project, AppError, Project, ProjectPaths,
};
use tokio::sync::RwLock;

pub struct ProjectService {
    pub project: RwLock<Option<Project>>,
    pub paths: RwLock<Option<ProjectPaths>>,
}

impl ProjectService {
    pub fn new() -> Self {
        Self {
            project: RwLock::new(None),
            paths: RwLock::new(None),
        }
    }

    pub async fn create(&self, title: &str, path: &str) -> Result<Project, AppError> {
        let (project, paths) = create_project(title, path)?;
        *self.project.write().await = Some(project.clone());
        *self.paths.write().await = Some(paths);
        Ok(project)
    }

    pub async fn open(&self, path: &str) -> Result<Project, AppError> {
        let paths = ProjectPaths::new(path);
        let project = load_project(&paths)?;
        *self.project.write().await = Some(project.clone());
        *self.paths.write().await = Some(paths);
        Ok(project)
    }

    pub async fn save(&self) -> Result<(), AppError> {
        let mut project = self.project.write().await;
        let paths = self.paths.read().await;
        if let (Some(p), Some(paths)) = (project.as_mut(), paths.as_ref()) {
            p.touch();
            save_project(p, paths)?;
        }
        Ok(())
    }

    pub async fn save_backup(&self) -> Result<(), AppError> {
        let project = self.project.read().await;
        let paths = self.paths.read().await;
        if let (Some(p), Some(paths)) = (project.as_ref(), paths.as_ref()) {
            let backup_dir = paths.root.join(".autosave");
            std::fs::create_dir_all(&backup_dir)?;
            let backup_file = backup_dir.join("project.autosave.json");
            let json = serde_json::to_string_pretty(p)?;
            std::fs::write(backup_file, json)?;
        }
        Ok(())
    }

    pub async fn get(&self) -> Option<Project> {
        self.project.read().await.clone()
    }

    pub async fn set(&self, project: Project) {
        *self.project.write().await = Some(project);
    }

    pub async fn paths(&self) -> Option<ProjectPaths> {
        self.paths.read().await.clone()
    }

    pub async fn update_segment(&self, segment_id: &str, update: impl FnOnce(&mut grok_voice_core::ScriptSegment)) {
        let mut project = self.project.write().await;
        if let Some(p) = project.as_mut() {
            if let Some(seg) = p.segments.iter_mut().find(|s| s.id == segment_id) {
                update(seg);
            }
        }
    }

    pub fn rel_audio_path(paths: &ProjectPaths, segment_id: &str, ext: &str) -> String {
        format!("assets/audio/segments/{segment_id}.{ext}")
    }

    pub fn resolve_audio_path(paths: &ProjectPaths, rel: &str) -> std::path::PathBuf {
        paths.root.join(rel)
    }

    pub fn normalize_segment_paths(project: &mut Project, paths: &ProjectPaths) {
        for seg in &mut project.segments {
            if let Some(ref p) = seg.audio_path {
                if Path::new(p).is_absolute() {
                    if let Ok(rel) = Path::new(p).strip_prefix(&paths.root) {
                        seg.audio_path = Some(rel.display().to_string().replace('\\', "/"));
                    }
                }
            }
        }
    }
}