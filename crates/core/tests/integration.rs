use grok_voice_core::{
    apply_parsed_script, create_project, load_project, parse_script, save_project, Project,
};

#[test]
fn project_roundtrip_save_load() {
    let dir = std::env::temp_dir().join(format!("gvs_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();

    let (mut project, paths) = create_project("測試專案", &dir).unwrap();
    let script = "旁白：故事開始。\n小明：你好。\n小美（害怕）：等一下。";
    let parsed = parse_script(script).unwrap();
    project.script_raw = script.to_string();
    apply_parsed_script(&mut project, &parsed).unwrap();
    save_project(&project, &paths).unwrap();

    let loaded = load_project(&paths).unwrap();
    assert_eq!(loaded.title, "測試專案");
    assert_eq!(loaded.segments.len(), 3);
    assert_eq!(loaded.characters.len(), 3);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mvp_acceptance_30_segments_3_characters() {
    let mut lines = Vec::new();
    let chars = ["旁白", "小明", "小美"];
    for i in 0..30 {
        let ch = chars[i % 3];
        lines.push(format!("{ch}：這是第 {} 句台詞。", i + 1));
    }
    let script = lines.join("\n");
    let parsed = parse_script(&script).unwrap();
    assert_eq!(parsed.lines.len(), 30);

    let unique: std::collections::HashSet<_> = parsed.lines.iter().map(|l| &l.character).collect();
    assert_eq!(unique.len(), 3);

    let mut project = Project::new("30句測試");
    apply_parsed_script(&mut project, &parsed).unwrap();
    assert_eq!(project.segments.len(), 30);
    assert_eq!(project.characters.len(), 3);
}