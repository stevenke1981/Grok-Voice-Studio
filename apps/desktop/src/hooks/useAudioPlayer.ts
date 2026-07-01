import { convertFileSrc } from "@tauri-apps/api/core";
import { invoke } from "@tauri-apps/api/core";
import { useRef, useCallback } from "react";

export function useAudioPlayer() {
  const audioRef = useRef<HTMLAudioElement | null>(null);

  const play = useCallback(async (path?: string) => {
    if (!path) return;
    try {
      const abs = await invoke<string>("get_audio_src", { path });
      const src = convertFileSrc(abs);
      if (audioRef.current) {
        audioRef.current.pause();
      }
      const audio = new Audio(src);
      audioRef.current = audio;
      await audio.play();
    } catch (e) {
      console.error("播放失敗", e);
      throw e;
    }
  }, []);

  const stop = useCallback(() => {
    audioRef.current?.pause();
  }, []);

  return { play, stop };
}