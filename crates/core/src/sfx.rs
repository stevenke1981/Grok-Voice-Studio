use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SfxCategory {
    Ambient,
    Action,
    Nature,
    Ui,
    Horror,
    Custom,
}

impl Default for SfxCategory {
    fn default() -> Self {
        Self::Ambient
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundEffect {
    pub id: String,
    pub name: String,
    pub name_en: String,
    pub category: SfxCategory,
    pub duration_ms: u32,
    pub tags: Vec<String>,
    pub file_name: String,
    pub builtin: bool,
}

pub fn builtin_catalog() -> Vec<SoundEffect> {
    vec![
        sfx("rain", "雨聲", "Rain", SfxCategory::Ambient, 3000, &["weather", "ambient"]),
        sfx("thunder", "雷聲", "Thunder", SfxCategory::Horror, 2000, &["weather", "horror"]),
        sfx("wind", "風聲", "Wind", SfxCategory::Nature, 2500, &["weather", "nature"]),
        sfx("footsteps", "腳步聲", "Footsteps", SfxCategory::Action, 1500, &["movement"]),
        sfx("door_open", "開門", "Door Open", SfxCategory::Action, 800, &["door"]),
        sfx("door_close", "關門", "Door Close", SfxCategory::Action, 700, &["door"]),
        sfx("explosion", "爆炸", "Explosion", SfxCategory::Action, 1800, &["action"]),
        sfx("birds", "鳥叫", "Birds", SfxCategory::Nature, 2000, &["nature"]),
        sfx("car", "車聲", "Car", SfxCategory::Ambient, 2200, &["traffic"]),
        sfx("keyboard", "鍵盤", "Keyboard", SfxCategory::Ui, 600, &["ui", "typing"]),
        sfx("notification", "通知", "Notification", SfxCategory::Ui, 500, &["ui", "alert"]),
        sfx("heartbeat", "心跳", "Heartbeat", SfxCategory::Horror, 2000, &["horror", "tension"]),
        sfx("glass_break", "玻璃破碎", "Glass Break", SfxCategory::Action, 900, &["action"]),
        sfx("laugh_track", "笑聲", "Laugh", SfxCategory::Ui, 1200, &["crowd"]),
        sfx("crowd", "人群喧嘩", "Crowd", SfxCategory::Ambient, 2500, &["crowd", "ambient"]),
    ]
}

fn sfx(
    id: &str,
    name: &str,
    name_en: &str,
    category: SfxCategory,
    duration_ms: u32,
    tags: &[&str],
) -> SoundEffect {
    SoundEffect {
        id: id.into(),
        name: name.into(),
        name_en: name_en.into(),
        category,
        duration_ms,
        tags: tags.iter().map(|t| t.to_string()).collect(),
        file_name: format!("{id}.wav"),
        builtin: true,
    }
}

pub fn resolve_sfx_id<'a>(name: &str, catalog: &'a [SoundEffect]) -> Option<&'a SoundEffect> {
    let key = name.trim();
    if key.is_empty() {
        return None;
    }
    let lower = key.to_lowercase();
    catalog.iter().find(|s| {
        s.id == lower
            || s.name == key
            || s.name_en.eq_ignore_ascii_case(key)
            || s.name_en.to_lowercase() == lower
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_by_chinese_name() {
        let cat = builtin_catalog();
        assert_eq!(resolve_sfx_id("雨聲", &cat).map(|s| s.id.as_str()), Some("rain"));
    }

    #[test]
    fn resolves_by_id() {
        let cat = builtin_catalog();
        assert_eq!(resolve_sfx_id("thunder", &cat).map(|s| s.id.as_str()), Some("thunder"));
    }
}