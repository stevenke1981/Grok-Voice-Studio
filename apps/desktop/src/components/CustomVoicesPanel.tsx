import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { VoiceInfo } from "../types";
import { t, type Lang } from "../i18n";

const CONSOLE_URL =
  "https://console.x.ai/team/default/voice/voice-library?campaign=voice-docs-custom-voices";

interface Props {
  lang: Lang;
  onVoicesChanged?: () => void;
}

export function CustomVoicesPanel({ lang, onVoicesChanged }: Props) {
  const [customVoices, setCustomVoices] = useState<VoiceInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [name, setName] = useState("");
  const [language, setLanguage] = useState("zh-CN");
  const [tone, setTone] = useState("warm");

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await invoke<VoiceInfo[]>("list_custom_voices");
      setCustomVoices(list);
    } catch (e) {
      setError(String(e));
      setCustomVoices([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const handleCreate = async () => {
    const file = await open({
      multiple: false,
      title: t(lang, "selectReferenceAudio"),
      filters: [{ name: "Audio", extensions: ["wav", "mp3", "flac", "ogg", "m4a"] }],
    });
    if (!file) return;

    setCreating(true);
    setError(null);
    try {
      await invoke("create_custom_voice", {
        filePath: file,
        name: name || null,
        description: null,
        gender: null,
        accent: null,
        age: null,
        language: language || null,
        useCase: "narration",
        tone: tone || null,
      });
      setName("");
      await load();
      onVoicesChanged?.();
    } catch (e) {
      setError(String(e));
    } finally {
      setCreating(false);
    }
  };

  const handleDelete = async (voiceId: string) => {
    if (!confirm(t(lang, "confirmDeleteVoice"))) return;
    try {
      await invoke("delete_custom_voice", { voiceId });
      await load();
      onVoicesChanged?.();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="custom-voices-panel">
      <div className="custom-voices-header">
        <h3>{t(lang, "customVoices")}</h3>
        <a className="custom-voices-link" href={CONSOLE_URL} target="_blank" rel="noreferrer">
          {t(lang, "openVoiceConsole")}
        </a>
      </div>
      <p className="custom-voices-hint">{t(lang, "customVoicesHint")}</p>

      <div className="custom-voices-create">
        <input
          placeholder={t(lang, "voiceName")}
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <select value={language} onChange={(e) => setLanguage(e.target.value)}>
          <option value="zh-CN">zh-CN</option>
          <option value="en">en</option>
          <option value="ja">ja</option>
        </select>
        <select value={tone} onChange={(e) => setTone(e.target.value)}>
          <option value="warm">warm</option>
          <option value="casual">casual</option>
          <option value="professional">professional</option>
          <option value="expressive">expressive</option>
          <option value="calm">calm</option>
        </select>
        <button className="btn btn-sm btn-primary" onClick={handleCreate} disabled={creating}>
          {creating ? "..." : t(lang, "cloneVoice")}
        </button>
        <button className="btn btn-sm" onClick={load} disabled={loading}>
          {t(lang, "refreshLogs")}
        </button>
      </div>

      {error && <div className="verify-msg verify-msg-error">{error}</div>}

      <div className="custom-voices-list">
        {customVoices.length === 0 && !loading && (
          <div className="custom-voices-empty">{t(lang, "noCustomVoices")}</div>
        )}
        {customVoices.map((v) => (
          <div key={v.voice_id} className="custom-voice-row">
            <div>
              <div className="custom-voice-name">{v.name}</div>
              <div className="custom-voice-id">
                ID: <code>{v.voice_id}</code>
              </div>
              {v.description && (
                <div className="custom-voice-desc">{v.description}</div>
              )}
            </div>
            <button className="btn btn-sm" onClick={() => handleDelete(v.voice_id)}>
              {t(lang, "delete")}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}