use grok_voice_core::AppError;

const SERVICE: &str = "GrokVoiceStudio";
const XAI_KEY_USER: &str = "xai_api_key";

pub fn load_api_key_secure() -> Result<Option<String>, AppError> {
    if let Ok(key) = std::env::var("XAI_API_KEY") {
        if !key.is_empty() {
            return Ok(Some(key));
        }
    }

    let entry = keyring::Entry::new(SERVICE, XAI_KEY_USER)
        .map_err(|e| AppError::Other(format!("Keyring error: {e}")))?;

    match entry.get_password() {
        Ok(key) if !key.is_empty() => Ok(Some(key)),
        Ok(_) => Ok(None),
        Err(keyring::Error::NoEntry) => {
            // Migrate from legacy settings.json if present
            migrate_legacy_key()
        }
        Err(e) => Err(AppError::Other(format!("Keyring read error: {e}"))),
    }
}

pub fn save_api_key_secure(key: &str) -> Result<(), AppError> {
    let entry = keyring::Entry::new(SERVICE, XAI_KEY_USER)
        .map_err(|e| AppError::Other(format!("Keyring error: {e}")))?;
    entry
        .set_password(key)
        .map_err(|e| AppError::Other(format!("Keyring write error: {e}")))?;

    // Remove legacy plaintext key
    if let Ok(store) = super::SettingsStore::open() {
        if let Ok(mut settings) = store.load() {
            settings.xai_api_key = None;
            let _ = store.save(&settings);
        }
    }
    Ok(())
}

pub fn delete_api_key_secure() -> Result<(), AppError> {
    let entry = keyring::Entry::new(SERVICE, XAI_KEY_USER)
        .map_err(|e| AppError::Other(format!("Keyring error: {e}")))?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Other(format!("Keyring delete error: {e}"))),
    }
}

pub fn has_api_key() -> bool {
    load_api_key_secure()
        .ok()
        .flatten()
        .is_some()
}

fn migrate_legacy_key() -> Result<Option<String>, AppError> {
    let store = super::SettingsStore::open()?;
    let settings = store.load()?;
    if let Some(key) = settings.xai_api_key.filter(|k| !k.is_empty()) {
        let _ = save_api_key_secure(&key);
        return Ok(Some(key));
    }
    Ok(None)
}