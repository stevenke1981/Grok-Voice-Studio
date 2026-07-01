import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { CustomVoicesPanel } from "./CustomVoicesPanel";
import type { AppSettings } from "../types";
import { t, type Lang } from "../i18n";

interface Props {
  lang: Lang;
  settings: Partial<AppSettings>;
  apiKey: string;
  onApiKeyChange: (v: string) => void;
  costPer1k: string;
  onCostChange: (v: string) => void;
  concurrency: number;
  onConcurrencyChange: (v: number) => void;
  onSave: () => void;
  onClose: () => void;
  onVerified?: () => void;
  onVoicesChanged?: () => void;
}

export function SettingsModal({
  lang,
  settings,
  apiKey,
  onApiKeyChange,
  costPer1k,
  onCostChange,
  concurrency,
  onConcurrencyChange,
  onSave,
  onClose,
  onVerified,
  onVoicesChanged,
}: Props) {
  const [verifying, setVerifying] = useState(false);
  const [verifyMsg, setVerifyMsg] = useState<string | null>(null);
  const [verifyOk, setVerifyOk] = useState(false);

  const handleVerify = async () => {
    setVerifying(true);
    setVerifyMsg(null);
    setVerifyOk(false);
    try {
      if (apiKey) {
        await invoke("save_settings", {
          apiKey,
          ffmpegPath: null,
          defaultLanguage: "zh",
          autoSave: true,
          generationConcurrency: concurrency,
          costPer1kChars: costPer1k ? Number(costPer1k) : null,
          onboardingDone: true,
          uiLanguage: lang,
        });
      }
      const result = await invoke<{ ok: boolean; message: string }>("verify_api_key");
      setVerifyMsg(result.message);
      setVerifyOk(true);
      onVerified?.();
    } catch (e) {
      setVerifyMsg(String(e));
      setVerifyOk(false);
    } finally {
      setVerifying(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal settings-modal-wide" onClick={(e) => e.stopPropagation()}>
        <h2>{t(lang, "settings")}</h2>
        <div className="inspector-field">
          <label>xAI API Key {settings.has_api_key && "(已設定 / saved)"}</label>
          <input
            type="password"
            placeholder="XAI_API_KEY"
            value={apiKey}
            onChange={(e) => onApiKeyChange(e.target.value)}
          />
        </div>
        <div className="inspector-field">
          <label>{t(lang, "estimatedCost")} (per 1k chars)</label>
          <input
            type="number"
            step="0.01"
            value={costPer1k}
            onChange={(e) => onCostChange(e.target.value)}
          />
        </div>
        <div className="inspector-field">
          <label>並行生成數 / Concurrency</label>
          <input
            type="number"
            min={1}
            max={5}
            value={concurrency}
            onChange={(e) => onConcurrencyChange(Number(e.target.value))}
          />
        </div>
        {verifyMsg && (
          <div className={`verify-msg${verifyOk ? "" : " verify-msg-error"}`}>{verifyMsg}</div>
        )}
        {settings.has_api_key && (
          <CustomVoicesPanel lang={lang} onVoicesChanged={onVoicesChanged} />
        )}
        <div className="modal-actions">
          <button className="btn" onClick={handleVerify} disabled={verifying || (!apiKey && !settings.has_api_key)}>
            {verifying ? "..." : t(lang, "verifyApiKey")}
          </button>
          <button className="btn" onClick={onClose}>
            {t(lang, "cancel")}
          </button>
          <button className="btn btn-primary" onClick={onSave}>
            {t(lang, "save")}
          </button>
        </div>
      </div>
    </div>
  );
}