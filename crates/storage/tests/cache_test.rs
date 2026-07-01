use grok_voice_core::TtsOutputFormat;
use grok_voice_storage::{new_cache_entry, AudioCache};

#[test]
fn cache_key_is_deterministic() {
    let fmt = TtsOutputFormat::default();
    let k1 = AudioCache::compute_cache_key("xai", "ara", "zh", "你好", &fmt);
    let k2 = AudioCache::compute_cache_key("xai", "ara", "zh", "你好", &fmt);
    let k3 = AudioCache::compute_cache_key("xai", "ara", "zh", "再見", &fmt);
    assert_eq!(k1, k2);
    assert_ne!(k1, k3);
}

#[test]
fn cache_hit_when_file_exists() {
    let dir = std::env::temp_dir().join(format!("gvs_cache_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = dir.join("cache.db");
    let audio_path = dir.join("test.mp3");
    std::fs::write(&audio_path, b"fake audio").unwrap();

    let cache = AudioCache::open(&db_path).unwrap();
    let key = "test_key_123".to_string();
    let entry = new_cache_entry(
        key.clone(),
        "xai",
        "ara",
        "zh",
        "測試",
        audio_path.display().to_string(),
        Some(1500),
    );
    cache.insert(&entry).unwrap();

    let hit = cache.lookup(&key).unwrap();
    assert!(hit.is_some());
    assert_eq!(hit.unwrap().duration_ms, Some(1500));

    let _ = std::fs::remove_dir_all(&dir);
}