import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import type { LogEntry } from "../types";

export function useLogs(enabled: boolean) {
  const [logs, setLogs] = useState<LogEntry[]>([]);

  const refresh = useCallback(async () => {
    try {
      const entries = await invoke<LogEntry[]>("get_logs", { limit: 500 });
      setLogs(entries);
    } catch {
      /* ignore */
    }
  }, []);

  useEffect(() => {
    if (!enabled) return;
    refresh();
  }, [enabled, refresh]);

  useEffect(() => {
    if (!enabled) return;
    const unlisten = listen<LogEntry>("app-log", (event) => {
      setLogs((prev) => [...prev.slice(-499), event.payload]);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, [enabled]);

  const clear = useCallback(async () => {
    await invoke("clear_logs");
    await refresh();
  }, [refresh]);

  return { logs, refresh, clear };
}