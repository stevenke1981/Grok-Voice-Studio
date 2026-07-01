import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import type { ApiRetryEvent } from "../types";

export function useApiRetry() {
  const [retryMessage, setRetryMessage] = useState<string | null>(null);
  const [retryCategory, setRetryCategory] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<ApiRetryEvent>("api-retry", (event) => {
      setRetryMessage(event.payload.message);
      setRetryCategory(event.payload.category);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const clearRetry = useCallback(() => {
    setRetryMessage(null);
    setRetryCategory(null);
  }, []);

  return { retryMessage, retryCategory, clearRetry };
}