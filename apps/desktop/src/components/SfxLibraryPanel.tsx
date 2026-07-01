import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { SoundEffect } from "../types";
import { t, type Lang } from "../i18n";

interface Props {
  lang: Lang;
  onInsert: (text: string) => void;
  onPreview: (path: string) => void;
}

const CATEGORY_LABELS: Record<string, { zh: string; en: string }> = {
  ambient: { zh: "環境", en: "Ambient" },
  action: { zh: "動作", en: "Action" },
  nature: { zh: "自然", en: "Nature" },
  ui: { zh: "介面", en: "UI" },
  horror: { zh: "恐怖", en: "Horror" },
  custom: { zh: "自訂", en: "Custom" },
};

export function SfxLibraryPanel({ lang, onInsert, onPreview }: Props) {
  const [sounds, setSounds] = useState<SoundEffect[]>([]);
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<string>("all");
  const [loading, setLoading] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      setSounds(await invoke<SoundEffect[]>("list_sfx_library"));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return sounds.filter((s) => {
      const cat = String(s.category);
      if (category !== "all" && cat !== category) return false;
      if (!q) return true;
      return (
        s.name.toLowerCase().includes(q)
        || s.name_en.toLowerCase().includes(q)
        || s.id.toLowerCase().includes(q)
        || s.tags.some((tag) => tag.toLowerCase().includes(q))
      );
    });
  }, [sounds, query, category]);

  const handlePreview = async (sfxId: string) => {
    const path = await invoke<string>("preview_sfx", { sfxId });
    onPreview(path);
  };

  const handleImport = async () => {
    const file = await open({
      multiple: false,
      title: t(lang, "importSfx"),
      filters: [{ name: "Audio", extensions: ["wav", "mp3", "ogg", "flac"] }],
    });
    if (!file) return;
    const name = prompt(t(lang, "sfxNamePrompt"), "自訂音效");
    if (!name) return;
    await invoke("import_sfx_file", { path: file, name, category: "custom" });
    await load();
  };

  const categories = useMemo(() => {
    const cats = new Set(sounds.map((s) => String(s.category)));
    return ["all", ...Array.from(cats)];
  }, [sounds]);

  return (
    <div className="sfx-library sfx-library-standalone">
      <div className="sfx-library-toolbar">
        <input
          className="sfx-search"
          placeholder={t(lang, "searchSfx")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <select
          className="sfx-category-filter"
          value={category}
          onChange={(e) => setCategory(e.target.value)}
        >
          {categories.map((cat) => (
            <option key={cat} value={cat}>
              {cat === "all"
                ? t(lang, "allCategories")
                : (CATEGORY_LABELS[cat]?.[lang] ?? cat)}
            </option>
          ))}
        </select>
        <button className="btn btn-sm" onClick={handleImport}>
          {t(lang, "importSfx")}
        </button>
        <button className="btn btn-sm" onClick={load} disabled={loading}>
          {t(lang, "refreshLogs")}
        </button>
      </div>
      <div className="sfx-grid">
        {filtered.map((sfx) => (
          <div key={sfx.id} className="sfx-card">
            <div className="sfx-card-name">{lang === "en" ? sfx.name_en : sfx.name}</div>
            <div className="sfx-card-meta">
              {(CATEGORY_LABELS[String(sfx.category)]?.[lang] ?? sfx.category)}
              {" · "}
              {(sfx.duration_ms / 1000).toFixed(1)}s
            </div>
            <div className="sfx-card-actions">
              <button className="btn btn-sm" onClick={() => handlePreview(sfx.id)}>
                {t(lang, "preview")}
              </button>
              <button
                className="btn btn-sm btn-primary"
                onClick={() => onInsert(`音效：${sfx.name}\n`)}
                title={t(lang, "insertSfxLine")}
              >
                {t(lang, "insertLine")}
              </button>
              <button
                className="btn btn-sm"
                onClick={() => onInsert(`{${sfx.name}}`)}
                title={t(lang, "insertSfxInline")}
              >
                {`{${lang === "en" ? sfx.name_en : sfx.name}}`}
              </button>
            </div>
          </div>
        ))}
        {filtered.length === 0 && !loading && (
          <div className="sfx-empty">{t(lang, "noSfxFound")}</div>
        )}
      </div>
    </div>
  );
}