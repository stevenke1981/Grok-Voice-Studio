pub mod error;
pub mod models;
pub mod parser;
pub mod project;
pub mod retry_notify;
pub mod sfx;
pub mod story;
pub mod text_split;
pub mod timeline;
pub mod tts_text;

pub use error::AppError;
pub use models::*;
pub use parser::{apply_parsed_script, parse_script, ParsedScript, ParseError};
pub use project::{create_project, load_project, save_project, ProjectPaths};
pub use retry_notify::{
    emit_retry, emit_story_convert, install_retry_hook, install_story_convert_hook,
    retry_context_label, suggest_concurrency, with_retry_context, RetryNotification,
    StoryConvertProgress,
};
pub use story::{apply_story_script, StoryScript, STORY_SYSTEM_PROMPT};
pub use text_split::{default_split, split_long_text};
pub use tts_text::build_tts_text;
pub use sfx::{builtin_catalog, resolve_sfx_id, SoundEffect, SfxCategory};
pub use timeline::build_timeline_from_segments;