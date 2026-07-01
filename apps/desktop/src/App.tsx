import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import "./App.css";
import { CharacterPanel } from "./components/CharacterPanel";
import { ExportPanel } from "./components/ExportPanel";
import { HomePage } from "./components/HomePage";
import { InspectorPanel } from "./components/InspectorPanel";
import { OnboardingModal } from "./components/OnboardingModal";
import { LogPanel } from "./components/LogPanel";
import { SfxLibraryWindow } from "./components/SfxLibraryWindow";
import { ScriptEditor } from "./components/ScriptEditor";
import { SegmentTable } from "./components/SegmentTable";
import { SettingsModal } from "./components/SettingsModal";
import { resolveWorkflowStep, WorkflowGuide } from "./components/WorkflowGuide";
import { useAudioPlayer } from "./hooks/useAudioPlayer";
import { useAutoSave } from "./hooks/useAutoSave";
import { useGeneration } from "./hooks/useGeneration";
import { useLogs } from "./hooks/useLogs";
import { t, type Lang } from "./i18n";
import type {
  AppSettings,
  ExportOptions,
  ExportResult,
  Project,
  ProjectStats,
  RecentProject,
  VoiceInfo,
} from "./types";

const DEFAULT_EXPORT: ExportOptions = {
  codec: "mp3",
  sample_rate: 24000,
  bit_rate: 128000,
  normalize: true,
  subtitle_format: "srt",
  show_character_in_subtitle: true,
  export_stems: false,
};

function App() {
  const [screen, setScreen] = useState<"home" | "editor">("home");
  const [project, setProject] = useState<Project | null>(null);
  const [recentProjects, setRecentProjects] = useState<RecentProject[]>([]);
  const [voices, setVoices] = useState<VoiceInfo[]>([]);
  const [settings, setSettings] = useState<Partial<AppSettings>>({});
  const [lang, setLang] = useState<Lang>("zh");
  const [selectedCharId, setSelectedCharId] = useState<string | null>(null);
  const [selectedSegId, setSelectedSegId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [showNewProject, setShowNewProject] = useState(false);
  const [apiKey, setApiKey] = useState("");
  const [costPer1k, setCostPer1k] = useState("");
  const [concurrency, setConcurrency] = useState(2);
  const [newProjectTitle, setNewProjectTitle] = useState("我的配音專案");
  const [editorMode, setEditorMode] = useState<"dialogue" | "story">("dialogue");
  const [storyText, setStoryText] = useState("");
  const [storyStyle, setStoryStyle] = useState("一般敘事");
  const [exportOptions, setExportOptions] = useState<ExportOptions>(DEFAULT_EXPORT);
  const [stats, setStats] = useState<ProjectStats | null>(null);
  const [showLogs, setShowLogs] = useState(false);
  const [showSfxLibrary, setShowSfxLibrary] = useState(false);

  const { play } = useAudioPlayer();
  const generation = useGeneration(setProject);
  const { logs, refresh: refreshLogs, clear: clearLogs } = useLogs(screen === "editor");

  useAutoSave(!!settings.auto_save && screen === "editor", [project?.script_raw, project?.characters, project?.segments]);

  const loadRecent = useCallback(async () => {
    try {
      const list = await invoke<RecentProject[]>("list_recent_projects");
      setRecentProjects(list);
    } catch {
      /* ignore */
    }
  }, []);

  const loadSettings = useCallback(async () => {
    try {
      const s = await invoke<AppSettings>("get_settings");
      setSettings(s);
      setLang((s.ui_language as Lang) || "zh");
      setConcurrency(s.generation_concurrency || 2);
      if (s.cost_per_1k_chars) setCostPer1k(String(s.cost_per_1k_chars));
      if (!s.onboarding_done) setShowOnboarding(true);
    } catch {
      /* ignore */
    }
  }, []);

  const loadStats = useCallback(async () => {
    try {
      const s = await invoke<ProjectStats>("get_project_stats");
      setStats(s);
    } catch {
      setStats(null);
    }
  }, []);

  const syncVoices = useCallback(async () => {
    try {
      setVoices(await invoke<VoiceInfo[]>("sync_voices"));
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    loadRecent();
    loadSettings();
    syncVoices();
  }, [loadRecent, loadSettings, syncVoices]);

  useEffect(() => {
    if (screen === "editor") loadStats();
  }, [screen, project?.segments, loadStats]);

  const selectedCharacter = useMemo(
    () => project?.characters.find((c) => c.id === selectedCharId) ?? null,
    [project, selectedCharId],
  );
  const selectedSegment = useMemo(
    () => project?.segments.find((s) => s.id === selectedSegId) ?? null,
    [project, selectedSegId],
  );
  const doneCount = useMemo(
    () => project?.segments.filter((s) => s.status === "done" || s.status === "cached").length ?? 0,
    [project],
  );
  const pendingCount = useMemo(
    () =>
      project?.segments.filter(
        (s) => s.status === "pending" || s.status === "generating" || s.status === "failed",
      ).length ?? 0,
    [project],
  );
  const allVoicesAssigned = useMemo(
    () =>
      (project?.characters.length ?? 0) > 0 &&
      (project?.characters.every((c) => c.voice_profile.voice_id) ?? false),
    [project],
  );
  const workflowStep = useMemo(() => {
    if (!project) return "script" as const;
    return resolveWorkflowStep(
      !!project.script_raw.trim(),
      project.segments.length,
      allVoicesAssigned,
      pendingCount,
      doneCount,
    );
  }, [project, allVoicesAssigned, pendingCount, doneCount]);

  const handleCreateProject = async () => {
    const folder = await open({ directory: true, multiple: false, title: "選擇專案資料夾" });
    if (!folder) return;
    try {
      const p = await invoke<Project>("create_new_project", { title: newProjectTitle, path: folder });
      setProject(p);
      setScreen("editor");
      setShowNewProject(false);
      setError(null);
      loadRecent();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleOpenProject = async (path?: string) => {
    let projectPath = path;
    if (!projectPath) {
      const selected = await open({ directory: true, multiple: false, title: "開啟專案" });
      if (!selected) return;
      projectPath = selected;
    }
    try {
      const p = await invoke<Project>("open_project", { path: projectPath });
      setProject(p);
      setScreen("editor");
      setError(null);
      loadRecent();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleParse = async () => {
    if (!project) return;
    try {
      const p = await invoke<Project>("parse_script_command", { input: project.script_raw });
      setProject(p);
      setSuccess(`已解析 ${p.segments.length} 句，下一步請指定角色語音並生成`);
      setError(null);
      refreshLogs();
    } catch (e) {
      setError(String(e));
      refreshLogs();
    }
  };

  const handleInsertSfx = (text: string) => {
    if (!project) return;
    setEditorMode("dialogue");
    const raw = project.script_raw;
    const needsSpace = raw.length > 0 && !raw.endsWith("\n") && !raw.endsWith(" ");
    setProject({ ...project, script_raw: raw + (needsSpace ? " " : "") + text });
  };

  const handleGoParse = () => {
    setEditorMode("dialogue");
    setTimeout(() => {
      document.querySelector<HTMLButtonElement>(".highlight-parse-btn")?.scrollIntoView({
        behavior: "smooth",
        block: "center",
      });
    }, 50);
  };

  const handleConvertStory = async () => {
    if (!storyText.trim()) return;
    try {
      const p = await invoke<Project>("convert_story", { story: storyText, style: storyStyle });
      setProject(p);
      setEditorMode("dialogue");
      setSuccess(`Story Mode 產出 ${p.segments.length} 句`);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleUpdateCharacterVoice = async (charId: string, voiceId: string) => {
    if (!project) return;
    const updated = {
      ...project,
      characters: project.characters.map((c) =>
        c.id === charId ? { ...c, voice_profile: { ...c.voice_profile, voice_id: voiceId } } : c,
      ),
    };
    setProject(updated);
    await invoke("update_project", { project: updated });
    await invoke("save_current_project");
  };

  const handleStyleChange = async (style: string) => {
    if (!project || !selectedCharId) return;
    const updated = {
      ...project,
      characters: project.characters.map((c) =>
        c.id === selectedCharId
          ? { ...c, voice_profile: { ...c.voice_profile, style_prompt: style } }
          : c,
      ),
    };
    setProject(updated);
    await invoke("update_project", { project: updated });
  };

  const handlePreviewVoice = async (voiceId: string) => {
    try {
      const path = await invoke<string>("preview_voice", { voiceId, text: null });
      await play(path);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleAddCharacter = async () => {
    const name = prompt("角色名稱");
    if (!name) return;
    try {
      await invoke("add_character", { name, roleType: "character" });
      const p = await invoke<Project | null>("get_project");
      if (p) setProject(p);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDeleteCharacter = async (id: string) => {
    if (!confirm("確定刪除此角色？")) return;
    try {
      const p = await invoke<Project>("delete_character", { characterId: id });
      setProject(p);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleExport = async () => {
    try {
      const result = await invoke<ExportResult>("export_mixdown", { options: exportOptions });
      setSuccess(`已匯出: ${result.audio_path}`);
      if (result.subtitle_path) {
        await revealItemInDir(result.audio_path);
      }
    } catch (e) {
      setError(String(e));
    }
  };

  const handleSaveSettings = async () => {
    try {
      await invoke("save_settings", {
        apiKey: apiKey || null,
        ffmpegPath: null,
        defaultLanguage: "zh",
        autoSave: true,
        generationConcurrency: concurrency,
        costPer1kChars: costPer1k ? Number(costPer1k) : null,
        onboardingDone: true,
        uiLanguage: lang,
      });
      await loadSettings();
      setShowSettings(false);
      setShowOnboarding(false);
      setApiKey("");
      setSuccess("設定已儲存");
      syncVoices();
      refreshLogs();
    } catch (e) {
      setError(String(e));
    }
  };

  if (screen === "home") {
    return (
      <>
        {showOnboarding && <OnboardingModal lang={lang} onStart={() => { setShowOnboarding(false); setShowSettings(true); }} />}
        {showNewProject && (
          <div className="modal-overlay" onClick={() => setShowNewProject(false)}>
            <div className="modal" onClick={(e) => e.stopPropagation()}>
              <h2>{t(lang, "newProject")}</h2>
              <div className="inspector-field">
                <label>專案名稱</label>
                <input value={newProjectTitle} onChange={(e) => setNewProjectTitle(e.target.value)} />
              </div>
              <div className="modal-actions">
                <button className="btn" onClick={() => setShowNewProject(false)}>{t(lang, "cancel")}</button>
                <button className="btn btn-primary" onClick={handleCreateProject}>建立</button>
              </div>
            </div>
          </div>
        )}
        {showSettings && (
          <SettingsModal
            lang={lang}
            settings={settings}
            apiKey={apiKey}
            onApiKeyChange={setApiKey}
            costPer1k={costPer1k}
            onCostChange={setCostPer1k}
            concurrency={concurrency}
            onConcurrencyChange={setConcurrency}
            onSave={handleSaveSettings}
            onClose={() => setShowSettings(false)}
            onVerified={loadSettings}
            onVoicesChanged={syncVoices}
          />
        )}
        <HomePage
          lang={lang}
          recentProjects={recentProjects}
          error={error}
          onNewProject={() => setShowNewProject(true)}
          onOpenProject={() => handleOpenProject()}
          onOpenRecent={handleOpenProject}
          onSettings={() => setShowSettings(true)}
        />
      </>
    );
  }

  if (!project) return null;

  return (
    <div className="app">
      <div className="topbar">
        <h1>{t(lang, "appTitle")}</h1>
        <span className="project-title">{project.title}</span>
        {stats && (
          <span className="project-stats">
            {t(lang, "totalChars")}: {stats.total_chars}
            {stats.estimated_cost != null && ` · ${t(lang, "estimatedCost")}: $${stats.estimated_cost.toFixed(2)}`}
          </span>
        )}
        <button className="btn" onClick={() => { setScreen("home"); setProject(null); }}>{t(lang, "home")}</button>
        <button className="btn" onClick={() => invoke("save_current_project")}>{t(lang, "save")}</button>
        <button className="btn" onClick={syncVoices}>{t(lang, "syncVoices")}</button>
        <button className="btn" onClick={() => setShowSettings(true)}>{t(lang, "settings")}</button>
        <button className="btn" onClick={() => invoke("cleanup_cache").then((n) => setSuccess(`清理 ${n} 筆`))}>{t(lang, "cleanupCache")}</button>
        <button className="btn" onClick={() => invoke("export_debug_bundle").then((p) => setSuccess(`Debug: ${p}`))}>{t(lang, "debugBundle")}</button>
        <button className="btn" onClick={() => { setShowLogs(true); refreshLogs(); }}>{t(lang, "logs")}</button>
        <button
          className={`btn${showSfxLibrary ? " btn-primary" : ""}`}
          onClick={() => setShowSfxLibrary((v) => !v)}
        >
          {t(lang, "sfxLibrary")}
        </button>
        <ExportPanel lang={lang} options={exportOptions} onChange={setExportOptions} onExport={handleExport} disabled={doneCount === 0} />
      </div>

      {error && <div className="error-banner">{error}<button className="btn btn-sm" style={{ marginLeft: 8 }} onClick={() => setError(null)}>×</button></div>}
      {success && <div className="success-banner">{success}<button className="btn btn-sm" style={{ marginLeft: 8 }} onClick={() => setSuccess(null)}>×</button></div>}

      <WorkflowGuide
        lang={lang}
        currentStep={workflowStep}
        hasScript={!!project.script_raw.trim()}
        segmentCount={project.segments.length}
        pendingCount={pendingCount}
        doneCount={doneCount}
        hasApiKey={!!settings.has_api_key}
        onGoParse={handleGoParse}
        onGoSettings={() => setShowSettings(true)}
      />

      <div className="mode-tabs">
        <button className={`btn btn-sm ${editorMode === "dialogue" ? "btn-primary" : ""}`} onClick={() => setEditorMode("dialogue")}>{t(lang, "dialogueMode")}</button>
        <button className={`btn btn-sm ${editorMode === "story" ? "btn-primary" : ""}`} onClick={() => setEditorMode("story")}>{t(lang, "storyModeTab")}</button>
      </div>

      <div className="main-layout">
        <CharacterPanel
          lang={lang}
          characters={project.characters}
          voices={voices}
          selectedId={selectedCharId}
          onSelect={setSelectedCharId}
          onVoiceChange={handleUpdateCharacterVoice}
          onPreview={handlePreviewVoice}
          onAdd={handleAddCharacter}
          onDelete={handleDeleteCharacter}
        />
        <ScriptEditor
          lang={lang}
          mode={editorMode}
          value={editorMode === "story" ? storyText : project.script_raw}
          onChange={(v) => editorMode === "story" ? setStoryText(v) : setProject({ ...project, script_raw: v })}
          onParse={handleParse}
          highlightParse={workflowStep === "parse"}
          sfxLibraryOpen={showSfxLibrary}
          onToggleSfxLibrary={() => setShowSfxLibrary((v) => !v)}
          onLoadTemplate={(content, style) => {
            if (editorMode === "story") {
              setStoryText(content);
              if (style) setStoryStyle(style);
            } else {
              setProject({ ...project, script_raw: content });
            }
          }}
          onConvertStory={handleConvertStory}
          storyStyle={storyStyle}
          onStoryStyleChange={setStoryStyle}
        />
        <InspectorPanel
          lang={lang}
          character={selectedCharacter}
          segment={selectedSegment}
          onGenerate={() => selectedSegId && generation.generateOne(selectedSegId).catch((e) => setError(String(e)))}
          onForceGenerate={() => selectedSegId && generation.generateOne(selectedSegId, true).catch((e) => setError(String(e)))}
          onSplit={() => selectedSegId && invoke("split_segment", { segmentId: selectedSegId }).then(() => invoke<Project | null>("get_project").then((p) => p && setProject(p))).catch((e) => setError(String(e)))}
          onPlay={() => selectedSegment?.audio_path && play(selectedSegment.audio_path).catch((e) => setError(String(e)))}
          onStyleChange={handleStyleChange}
        />
      </div>

      {project.timeline?.tracks?.length > 0 && (
        <div className="timeline-preview">
          <span className="panel-header">{t(lang, "timeline")}</span>
          <div className="timeline-tracks">
            {(project.timeline.tracks as Array<{ id: string; name: string; clips: { start_ms: number; duration_ms: number }[] }>).map((track) => (
              <div key={track.id} className="timeline-track">
                <span className="track-label">{track.name}</span>
                <div className="track-clips">
                  {track.clips.map((clip, i) => (
                    <div
                      key={i}
                      className="timeline-clip"
                      style={{ width: Math.max(clip.duration_ms / 50, 8) }}
                      title={`${clip.start_ms}ms`}
                    />
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      <SegmentTable
        lang={lang}
        project={project}
        selectedId={selectedSegId}
        progress={generation.progress}
        generating={generation.generating}
        paused={generation.paused}
        onSelect={setSelectedSegId}
        onGenerate={(id) => generation.generateOne(id).catch((e) => setError(String(e)))}
        onPlay={(path) => play(path).catch((e) => setError(String(e)))}
        onGenerateAll={() => generation.startAll(false, false).catch((e) => setError(String(e)))}
        onGenerateFailed={() => generation.startAll(true, false).catch((e) => setError(String(e)))}
        onPause={generation.pause}
        onResume={generation.resume}
        onCancel={generation.cancel}
        hasApiKey={!!settings.has_api_key}
        segmentCount={project.segments.length}
      />

      {showLogs && (
        <LogPanel
          lang={lang}
          logs={logs}
          onClose={() => setShowLogs(false)}
          onClear={clearLogs}
          onRefresh={refreshLogs}
        />
      )}

      <SfxLibraryWindow
        lang={lang}
        open={showSfxLibrary}
        onClose={() => setShowSfxLibrary(false)}
        onInsert={handleInsertSfx}
        onPreview={(path) => play(path).catch((e) => setError(String(e)))}
      />

      {showSettings && (
        <SettingsModal
          lang={lang}
          settings={settings}
          apiKey={apiKey}
          onApiKeyChange={setApiKey}
          costPer1k={costPer1k}
          onCostChange={setCostPer1k}
          concurrency={concurrency}
          onConcurrencyChange={setConcurrency}
          onSave={handleSaveSettings}
          onClose={() => setShowSettings(false)}
          onVerified={loadSettings}
          onVoicesChanged={syncVoices}
        />
      )}
    </div>
  );
}

export default App;