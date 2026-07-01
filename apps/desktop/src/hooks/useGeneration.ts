import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import type { GenerateProgressEvent, Project, ScriptSegment } from "../types";

export function useGeneration(setProject: (p: Project) => void) {
  const [generating, setGenerating] = useState(false);
  const [paused, setPaused] = useState(false);
  const [progress, setProgress] = useState({ current: 0, total: 0 });

  const reloadProject = useCallback(async () => {
    const proj = await invoke<Project | null>("get_project");
    if (proj) setProject(proj);
    return proj;
  }, [setProject]);

  useEffect(() => {
    const unlisten = listen<GenerateProgressEvent>("generate-progress", async (event) => {
      const p = event.payload;
      setProgress({ current: p.current, total: p.total });

      await reloadProject();

      if (p.status === "completed" || p.status === "cancelled" || p.status === "failed") {
        setGenerating(false);
        setPaused(false);
      }
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, [reloadProject]);

  const startAll = useCallback(
    async (onlyFailed = false, force = false) => {
      const proj = await reloadProject();
      if (!proj || proj.segments.length === 0) {
        throw new Error("尚無台詞段落，請先點擊「解析劇本」");
      }

      const pending = proj.segments.filter((s) =>
        onlyFailed
          ? s.status === "failed"
          : s.status !== "done" && s.status !== "cached",
      ).length;

      setGenerating(true);
      setProgress({ current: 0, total: pending || proj.segments.length });
      try {
        await invoke("start_generate_job", { onlyFailed, force });
      } catch (e) {
        setGenerating(false);
        throw e;
      }
    },
    [reloadProject],
  );

  const cancel = useCallback(async () => {
    await invoke("cancel_generate_job");
    setGenerating(false);
  }, []);

  const pause = useCallback(async () => {
    await invoke("pause_generate_job");
    setPaused(true);
  }, []);

  const resume = useCallback(async () => {
    await invoke("resume_generate_job");
    setPaused(false);
  }, []);

  const generateOne = useCallback(
    async (segmentId: string, force = false): Promise<ScriptSegment> => {
      const seg = await invoke<ScriptSegment>("generate_segment", {
        segmentId,
        force,
      });
      await reloadProject();
      return seg;
    },
    [reloadProject],
  );

  return { generating, paused, progress, startAll, cancel, pause, resume, generateOne };
}