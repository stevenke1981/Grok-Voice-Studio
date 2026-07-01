import { useEffect, useRef } from "react";
import type { LogEntry } from "../types";
import { t, type Lang } from "../i18n";

interface Props {
  lang: Lang;
  logs: LogEntry[];
  onClose: () => void;
  onClear: () => void;
  onRefresh: () => void;
}

function levelClass(level: string) {
  return `log-level log-level-${level}`;
}

export function LogPanel({ lang, logs, onClose, onClear, onRefresh }: Props) {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs.length]);

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal log-panel" onClick={(e) => e.stopPropagation()}>
        <div className="log-panel-header">
          <h2>{t(lang, "logs")}</h2>
          <div className="log-panel-actions">
            <button className="btn btn-sm" onClick={onRefresh}>
              {t(lang, "refreshLogs")}
            </button>
            <button className="btn btn-sm" onClick={onClear}>
              {t(lang, "clearLogs")}
            </button>
            <button className="btn btn-sm" onClick={onClose}>
              {t(lang, "close")}
            </button>
          </div>
        </div>
        <div className="log-list">
          {logs.length === 0 ? (
            <div className="log-empty">{t(lang, "noLogs")}</div>
          ) : (
            logs.map((entry) => (
              <div key={entry.id} className="log-entry">
                <span className="log-time">{entry.timestamp}</span>
                <span className={levelClass(entry.level)}>{entry.level}</span>
                <span className="log-category">[{entry.category}]</span>
                <span className="log-message">{entry.message}</span>
              </div>
            ))
          )}
          <div ref={bottomRef} />
        </div>
      </div>
    </div>
  );
}