import type { Character, ScriptSegment } from "../types";
import { t, type Lang } from "../i18n";

interface Props {
  lang: Lang;
  character: Character | null;
  segment: ScriptSegment | null;
  onGenerate: () => void;
  onForceGenerate: () => void;
  onSplit: () => void;
  onPlay: () => void;
  onStyleChange: (style: string) => void;
}

export function InspectorPanel({
  lang,
  character,
  segment,
  onGenerate,
  onForceGenerate,
  onSplit,
  onPlay,
  onStyleChange,
}: Props) {
  return (
    <div className="panel">
      <div className="panel-header">{t(lang, "inspector")}</div>
      <div className="panel-body">
        {character && (
          <>
            <div className="inspector-field">
              <label>角色</label>
              <input value={character.name} readOnly />
            </div>
            <div className="inspector-field">
              <label>Style Prompt</label>
              <input
                value={character.voice_profile.style_prompt ?? ""}
                placeholder="溫柔、緊張、低沉..."
                onChange={(e) => onStyleChange(e.target.value)}
              />
            </div>
          </>
        )}
        {segment && (
          <>
            {segment.segment_kind === "sfx" && (
              <div className="inspector-field">
                <label>{t(lang, "sfxSegment")}</label>
                <input value={segment.text} readOnly />
              </div>
            )}
            {segment.sfx_cues && segment.sfx_cues.length > 0 && (
              <div className="inspector-field">
                <label>{t(lang, "sfxCues")}</label>
                <input
                  readOnly
                  value={segment.sfx_cues.map((c) => c.label).join(", ")}
                />
              </div>
            )}
            <div className="inspector-field">
              <label>台詞</label>
              <textarea
                readOnly
                value={segment.text}
                style={{ width: "100%", minHeight: 60, background: "var(--bg)", color: "var(--text)", border: "1px solid var(--border)", borderRadius: 8, padding: 6 }}
              />
            </div>
            {segment.emotion_hint && (
              <div className="inspector-field">
                <label>情緒</label>
                <input value={segment.emotion_hint} readOnly />
              </div>
            )}
            {segment.error_message && (
              <div className="error-banner" style={{ marginBottom: 8, fontSize: 12 }}>
                {segment.error_message}
              </div>
            )}
            <div style={{ display: "flex", gap: 4, flexWrap: "wrap" }}>
              <button className="btn btn-sm btn-primary" onClick={onGenerate}>
                {segment.segment_kind === "sfx" ? t(lang, "sfxSegment") : "生成"}
              </button>
              <button className="btn btn-sm" onClick={onForceGenerate}>
                {t(lang, "forceRegenerate")}
              </button>
              <button className="btn btn-sm" onClick={onSplit}>
                {t(lang, "splitLong")}
              </button>
              {segment.audio_path && (
                <button className="btn btn-sm" onClick={onPlay}>
                  {t(lang, "play")}
                </button>
              )}
            </div>
          </>
        )}
        {!character && !segment && (
          <p style={{ color: "var(--muted)", fontSize: 13 }}>選擇角色或台詞</p>
        )}
      </div>
    </div>
  );
}