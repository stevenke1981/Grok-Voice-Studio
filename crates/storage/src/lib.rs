mod cache;
mod keyring_store;
mod settings;
mod sfx_store;
mod sfx_wav;

pub use cache::{new_cache_entry, AudioCache, CacheEntry};
pub use keyring_store::{
    delete_api_key_secure, has_api_key, load_api_key_secure, save_api_key_secure,
};
pub use settings::{SettingsStore, SETTINGS_FILE};
pub use sfx_store::SfxStore;

/// Load API key: env → keychain → legacy settings
pub fn load_api_key() -> Result<Option<String>, AppError> {
    keyring_store::load_api_key_secure()
}

pub fn save_api_key(key: &str) -> Result<(), AppError> {
    keyring_store::save_api_key_secure(key)
}

use grok_voice_core::AppError;