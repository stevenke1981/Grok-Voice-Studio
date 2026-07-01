use std::fs;
use std::path::{Path, PathBuf};

use crate::models::Project;
use crate::AppError;

#[derive(Clone)]
pub struct ProjectPaths {
    pub root: PathBuf,
    pub project_file: PathBuf,
    pub segments_dir: PathBuf,
    pub mixdown_dir: PathBuf,
    pub exports_dir: PathBuf,
}

impl ProjectPaths {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        Self {
            project_file: root.join("project.json"),
            segments_dir: root.join("assets/audio/segments"),
            mixdown_dir: root.join("assets/audio/mixdown"),
            exports_dir: root.join("exports"),
            root,
        }
    }

    pub fn ensure_dirs(&self) -> Result<(), AppError> {
        fs::create_dir_all(&self.segments_dir)?;
        fs::create_dir_all(&self.mixdown_dir)?;
        fs::create_dir_all(&self.exports_dir)?;
        Ok(())
    }

    pub fn segment_audio_path(&self, segment_id: &str, ext: &str) -> PathBuf {
        self.segments_dir.join(format!("{segment_id}.{ext}"))
    }
}

pub fn save_project(project: &Project, paths: &ProjectPaths) -> Result<(), AppError> {
    paths.ensure_dirs()?;
    let json = serde_json::to_string_pretty(project)?;
    fs::write(&paths.project_file, json)?;
    Ok(())
}

pub fn load_project(paths: &ProjectPaths) -> Result<Project, AppError> {
    if !paths.project_file.exists() {
        return Err(AppError::ProjectNotFound(
            paths.project_file.display().to_string(),
        ));
    }
    let content = fs::read_to_string(&paths.project_file)?;
    let project: Project = serde_json::from_str(&content)?;
    Ok(project)
}

pub fn create_project(title: &str, root: impl AsRef<Path>) -> Result<(Project, ProjectPaths), AppError> {
    let paths = ProjectPaths::new(root);
    paths.ensure_dirs()?;
    let project = Project::new(title);
    save_project(&project, &paths)?;
    Ok((project, paths))
}