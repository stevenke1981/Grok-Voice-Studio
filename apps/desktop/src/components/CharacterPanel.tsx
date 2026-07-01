import type { Character, VoiceInfo } from "../types";
import { t, type Lang } from "../i18n";

interface Props {
  lang: Lang;
  characters: Character[];
  voices: VoiceInfo[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onVoiceChange: (charId: string, voiceId: string) => void;
  onPreview: (voiceId: string) => void;
  onAdd: () => void;
  onDelete: (id: string) => void;
}

export function CharacterPanel({
  lang,
  characters,
  voices,
  selectedId,
  onSelect,
  onVoiceChange,
  onPreview,
  onAdd,
  onDelete,
}: Props) {
  return (
    <div className="panel">
      <div className="panel-header" style={{ display: "flex", gap: 6 }}>
        <span>
          {t(lang, "characters")} ({characters.length})
        </span>
        <button className="btn btn-sm" onClick={onAdd}>
          +
        </button>
      </div>
      <div className="panel-body">
        {characters.map((c) => (
          <div
            key={c.id}
            className={`character-item ${selectedId === c.id ? "active" : ""}`}
            onClick={() => onSelect(c.id)}
          >
            <div className="character-dot" style={{ background: c.color }} />
            <div style={{ flex: 1 }}>
              <div className="character-name">{c.name}</div>
              <select
                value={c.voice_profile.voice_id}
                onClick={(e) => e.stopPropagation()}
                onChange={(e) => onVoiceChange(c.id, e.target.value)}
                style={{ fontSize: 11, marginTop: 4, width: "100%" }}
              >
                <optgroup label={t(lang, "builtinVoices")}>
                  {voices
                    .filter((v) => !v.is_custom)
                    .map((v) => (
                      <option key={v.voice_id} value={v.voice_id}>
                        {v.name}
                      </option>
                    ))}
                </optgroup>
                {voices.some((v) => v.is_custom) && (
                  <optgroup label={t(lang, "customVoices")}>
                    {voices
                      .filter((v) => v.is_custom)
                      .map((v) => (
                        <option key={v.voice_id} value={v.voice_id}>
                          {v.name}
                        </option>
                      ))}
                  </optgroup>
                )}
                {!voices.some((v) => v.voice_id === c.voice_profile.voice_id) && (
                  <option value={c.voice_profile.voice_id}>
                    {c.voice_profile.voice_id} ({t(lang, "customVoices")})
                  </option>
                )}
              </select>
            </div>
            <button
              className="btn btn-sm"
              onClick={(e) => {
                e.stopPropagation();
                onPreview(c.voice_profile.voice_id);
              }}
            >
              ▶
            </button>
            {selectedId === c.id && (
              <button
                className="btn btn-sm"
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(c.id);
                }}
              >
                ×
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}