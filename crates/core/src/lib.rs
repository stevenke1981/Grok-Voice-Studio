pub mod error;
pub mod models;
pub mod parser;
pub mod project;
pub mod sfx;
pub mod story;
pub mod text_split;
pub mod timeline;

pub use error::AppError;
pub use models::*;
pub use parser::{apply_parsed_script, parse_script, ParsedScript, ParseError};
pub use project::{create_project, load_project, save_project, ProjectPaths};
pub use story::{apply_story_script, StoryScript, STORY_SYSTEM_PROMPT};
pub use text_split::{default_split, split_long_text};
pub use sfx::{builtin_catalog, resolve_sfx_id, SoundEffect, SfxCategory};
pub use timeline::build_timeline_from_segments;