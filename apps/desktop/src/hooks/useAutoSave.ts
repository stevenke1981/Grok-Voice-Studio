import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";

export function useAutoSave(enabled: boolean, deps: unknown[]) {
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (!enabled) return;
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      invoke("autosave_project").catch(console.error);
    }, 2500);
    return () => {
      if (timer.current) clearTimeout(timer.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);
}