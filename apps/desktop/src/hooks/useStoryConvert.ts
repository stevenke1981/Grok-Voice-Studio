import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import type { StoryConvertProgressEvent } from "../types";

export function useStoryConvert() {
  const [attempt, setAttempt] = useState(1);
  const [phase, setPhase] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<StoryConvertProgressEvent>("story-convert-progress", (event) => {
      setAttempt(event.payload.attempt);
      setPhase(event.payload.phase);
      setMessage(event.payload.message ?? null);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const reset = useCallback(() => {
    setAttempt(1);
    setPhase(null);
    setMessage(null);
  }, []);

  return { attempt, phase, message, reset };
}