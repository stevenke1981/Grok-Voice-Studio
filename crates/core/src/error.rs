use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Parse error at line {line}: {message}")]
    Parse { line: u32, message: String },

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Missing API key")]
    MissingApiKey,

    #[error("Authentication failed")]
    AuthFailed,

    #[error("Rate limited")]
    RateLimited,

    #[error("Quota exceeded")]
    QuotaExceeded,

    #[error("Text too long: {chars} characters (max {max})")]
    TextTooLong { chars: usize, max: usize },

    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),

    #[error("FFmpeg missing: {0}")]
    FfmpegMissing(String),

    #[error("Export failed: {0}")]
    ExportFailed(String),

    #[error("Segment not found: {0}")]
    SegmentNotFound(String),

    #[error("Character not found: {0}")]
    CharacterNotFound(String),

    #[error("{0}")]
    Other(String),
}