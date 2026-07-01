import type { ExportOptions } from "../types";
import { t, type Lang } from "../i18n";

interface Props {
  lang: Lang;
  options: ExportOptions;
  onChange: (o: ExportOptions) => void;
  onExport: () => void;
  disabled: boolean;
}

export function ExportPanel({ lang, options, onChange, onExport, disabled }: Props) {
  return (
    <div className="export-bar">
      <select
        value={options.codec}
        onChange={(e) => onChange({ ...options, codec: e.target.value as ExportOptions["codec"] })}
      >
        <option value="mp3">MP3</option>
        <option value="wav">WAV</option>
        <option value="flac">FLAC</option>
      </select>
      <select
        value={options.subtitle_format}
        onChange={(e) =>
          onChange({ ...options, subtitle_format: e.target.value as ExportOptions["subtitle_format"] })
        }
      >
        <option value="srt">SRT</option>
        <option value="vtt">VTT</option>
        <option value="ass">ASS</option>
      </select>
      <label style={{ display: "flex", alignItems: "center", gap: 4, fontSize: 12 }}>
        <input
          type="checkbox"
          checked={options.show_character_in_subtitle}
          onChange={(e) => onChange({ ...options, show_character_in_subtitle: e.target.checked })}
        />
        角色名
      </label>
      <label style={{ display: "flex", alignItems: "center", gap: 4, fontSize: 12 }}>
        <input
          type="checkbox"
          checked={options.export_stems}
          onChange={(e) => onChange({ ...options, export_stems: e.target.checked })}
        />
        Stems
      </label>
      <button className="btn btn-primary btn-sm" onClick={onExport} disabled={disabled}>
        {t(lang, "export")}
      </button>
    </div>
  );
}