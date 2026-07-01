import { DIALOGUE_TEMPLATES, STORY_TEMPLATES } from "../templates";
import { t, tf, type Lang } from "../i18n";

const SPEECH_TAGS = ["[pause]", "[sigh]", "[laugh]", "[breath]", "[long-pause]"];

interface Props {
  lang: Lang;
  mode: "dialogue" | "story";
  value: string;
  onChange: (v: string) => void;
  onParse: () => void;
  onLoadTemplate: (content: string, style?: string) => void;
  onConvertStory?: () => void;
  convertingStory?: boolean;
  storyRetryMessage?: string | null;
  storyAttempt?: number;
  storyPhase?: string | null;
  storyStyle?: string;
  onStoryStyleChange?: (v: string) => void;
  highlightParse?: boolean;
  sfxLibraryOpen?: boolean;
  onToggleSfxLibrary?: () => void;
}

export function ScriptEditor({
  lang,
  mode,
  value,
  onChange,
  onParse,
  onLoadTemplate,
  onConvertStory,
  convertingStory,
  storyRetryMessage,
  storyAttempt,
  storyPhase,
  storyStyle,
  onStoryStyleChange,
  highlightParse,
  sfxLibraryOpen,
  onToggleSfxLibrary,
}: Props) {
  const insertTag = (tag: string) => {
    onChange(value + (value.endsWith("\n") || !value ? "" : " ") + tag + " ");
  };

  const handleTemplateSelect = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const id = e.target.value;
    if (!id) return;
    if (mode === "dialogue") {
      const tpl = DIALOGUE_TEMPLATES.find((t) => t.id === id);
      if (tpl) onLoadTemplate(tpl.content);
    } else {
      const tpl = STORY_TEMPLATES.find((t) => t.id === id);
      if (tpl) onLoadTemplate(tpl.content, tpl.style);
    }
    e.target.value = "";
  };

  return (
    <div className="panel" style={{ display: "flex", flexDirection: "column" }}>
      <div className="panel-header" style={{ display: "flex", gap: 6, flexWrap: "wrap", alignItems: "center" }}>
        <span>{mode === "story" ? t(lang, "storyMode") : t(lang, "scriptEditor")}</span>
        <select
          className="template-select"
          defaultValue=""
          onChange={handleTemplateSelect}
          title={t(lang, "loadTemplate")}
        >
          <option value="" disabled>
            {t(lang, "loadTemplate")}
          </option>
          {mode === "dialogue"
            ? DIALOGUE_TEMPLATES.map((tpl) => (
                <option key={tpl.id} value={tpl.id}>
                  {lang === "en" ? tpl.nameEn : tpl.name}
                </option>
              ))
            : STORY_TEMPLATES.map((tpl) => (
                <option key={tpl.id} value={tpl.id}>
                  {lang === "en" ? tpl.nameEn : tpl.name}
                </option>
              ))}
        </select>
        {mode === "dialogue" ? (
          <>
            <button
              className={`btn btn-sm btn-primary highlight-parse-btn${highlightParse ? " parse-highlight" : ""}`}
              onClick={onParse}
              title={highlightParse ? t(lang, "hintNeedParse") : undefined}
            >
              {highlightParse ? `▶ ${t(lang, "parseScript")}` : t(lang, "parseScript")}
            </button>
            <button
              className={`btn btn-sm${sfxLibraryOpen ? " btn-primary" : ""}`}
              onClick={onToggleSfxLibrary}
            >
              {sfxLibraryOpen ? t(lang, "hideSfxLibrary") : t(lang, "showSfxLibrary")}
            </button>
          </>
        ) : (
          <>
            <select
              className="btn btn-sm"
              value={storyStyle ?? "一般敘事"}
              onChange={(e) => onStoryStyleChange?.(e.target.value)}
              disabled={convertingStory}
            >
              <option value="一般敘事">一般敘事</option>
              <option value="恐怖">恐怖</option>
              <option value="童話">童話</option>
              <option value="短影音旁白">短影音旁白</option>
              <option value="漫畫解說">漫畫解說</option>
            </select>
            <button
              className="btn btn-sm btn-primary"
              onClick={onConvertStory}
              disabled={convertingStory || !value.trim()}
            >
              {convertingStory ? t(lang, "convertingStory") : t(lang, "convertStory")}
            </button>
          </>
        )}
      </div>
      {mode === "story" && convertingStory && (
        <div className="story-convert-status">
          <span className="story-convert-spinner" aria-hidden />
          <span>
            {tf(lang, "storyAttempt", { n: storyAttempt ?? 1 })}
            {storyPhase === "json_repair" ? " · JSON 修復" : ""}
          </span>
          {storyRetryMessage && (
            <span className="story-retry-hint" title={storyRetryMessage}>
              {t(lang, "apiRetrying")} {storyRetryMessage}
            </span>
          )}
        </div>
      )}
      {mode === "dialogue" && (
        <div
          style={{
            padding: "4px 8px",
            display: "flex",
            gap: 4,
            flexWrap: "wrap",
            borderBottom: "1px solid var(--border)",
          }}
        >
          {SPEECH_TAGS.map((tag) => (
            <button key={tag} className="btn btn-sm" onClick={() => insertTag(tag)}>
              {tag}
            </button>
          ))}
        </div>
      )}
      <textarea
        className="script-editor"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        readOnly={mode === "story" && convertingStory}
        placeholder={
          mode === "story"
            ? "貼上完整故事文本，或從上方選擇模板..."
            : "旁白：故事開始了。\n音效：雨聲\n阿明：你聽到了嗎？{雷聲} 那不是風聲。\n\n支援：音效：名稱、{行內音效}"
        }
        spellCheck={false}
      />
    </div>
  );
}