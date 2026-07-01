import type { Project, ScriptSegment } from "../types";
import { t, type Lang } from "../i18n";

function statusClass(status: string) {
  return `status-badge status-${status}`;
}

function displayStatus(lang: Lang, seg: ScriptSegment) {
  if (seg.segment_kind === "sfx") return t(lang, "sfxSegment");
  if (seg.status === "pending") return t(lang, "statusPendingHint");
  return seg.status;
}

function displayText(seg: ScriptSegment) {
  if (seg.sfx_cues?.length) {
    const cues = seg.sfx_cues.map((c) => `{${c.label}}`).join(" ");
    return `${seg.text} ${cues}`.trim();
  }
  return seg.text;
}

interface Props {
  lang: Lang;
  project: Project;
  selectedId: string | null;
  progress: { current: number; total: number };
  generating: boolean;
  paused: boolean;
  onSelect: (id: string) => void;
  onGenerate: (id: string) => void;
  onPlay: (path?: string) => void;
  onGenerateAll: () => void;
  onGenerateFailed: () => void;
  onPause: () => void;
  onResume: () => void;
  onCancel: () => void;
  hasApiKey: boolean;
  segmentCount: number;
}

export function SegmentTable({
  lang,
  project,
  selectedId,
  progress,
  generating,
  paused,
  onSelect,
  onGenerate,
  onPlay,
  onGenerateAll,
  onGenerateFailed,
  onPause,
  onResume,
  onCancel,
  hasApiKey,
  segmentCount,
}: Props) {
  const doneCount = project.segments.filter(
    (s) => s.status === "done" || s.status === "cached",
  ).length;

  const hasPendingDialogue = project.segments.some(
    (s) =>
      s.segment_kind !== "sfx"
      && s.status !== "done"
      && s.status !== "cached",
  );
  const generateBlockedReason =
    segmentCount === 0
      ? t(lang, "needParseFirst")
      : hasPendingDialogue && !hasApiKey
        ? t(lang, "needApiKey")
        : null;

  return (
    <div className="bottom-panel">
      <div className="queue-header">
        <span>
          {t(lang, "generateQueue")}: {doneCount} / {project.segments.length} {t(lang, "done")}
        </span>
        {generateBlockedReason && (
          <span className="queue-hint">{generateBlockedReason}</span>
        )}
        <div className="progress-bar">
          <div
            className="progress-fill"
            style={{
              width: `${progress.total ? (progress.current / progress.total) * 100 : 0}%`,
            }}
          />
        </div>
        {!generating ? (
          <>
            <button
              className="btn btn-primary btn-sm"
              onClick={onGenerateAll}
              disabled={!project.segments.length || (!!generateBlockedReason && hasPendingDialogue)}
              title={generateBlockedReason ?? undefined}
            >
              {t(lang, "generateAll")}
            </button>
            <button
              className="btn btn-sm"
              onClick={onGenerateFailed}
              disabled={segmentCount === 0 || (!!generateBlockedReason && hasPendingDialogue)}
              title={generateBlockedReason ?? undefined}
            >
              {t(lang, "generateFailed")}
            </button>
          </>
        ) : (
          <>
            <button className="btn btn-sm" onClick={paused ? onResume : onPause}>
              {paused ? t(lang, "resume") : t(lang, "pause")}
            </button>
            <button className="btn btn-sm" onClick={onCancel}>
              {t(lang, "cancel")}
            </button>
          </>
        )}
      </div>
      <div className="segments-table">
        <table>
          <thead>
            <tr>
              <th>#</th>
              <th>角色</th>
              <th>台詞</th>
              <th>狀態</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            {project.segments.map((seg, i) => {
              const char = project.characters.find((c) => c.id === seg.character_id);
              return (
                <SegmentRow
                  key={seg.id}
                  index={i}
                  seg={seg}
                  charName={char?.name ?? "?"}
                  selected={selectedId === seg.id}
                  lang={lang}
                  onSelect={() => onSelect(seg.id)}
                  onGenerate={() => onGenerate(seg.id)}
                  onPlay={() => onPlay(seg.audio_path)}
                />
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function SegmentRow({
  index,
  seg,
  charName,
  selected,
  lang,
  onSelect,
  onGenerate,
  onPlay,
}: {
  index: number;
  seg: ScriptSegment;
  charName: string;
  selected: boolean;
  lang: Lang;
  onSelect: () => void;
  onGenerate: () => void;
  onPlay: () => void;
}) {
  return (
    <tr
      onClick={onSelect}
      style={{ cursor: "pointer", background: selected ? "var(--surface2)" : undefined }}
    >
      <td>{index + 1}</td>
      <td>{seg.segment_kind === "sfx" ? "🔊" : charName}</td>
      <td className="text-truncate">{displayText(seg)}</td>
      <td>
        <span
          className={`${statusClass(seg.status)}${seg.segment_kind === "sfx" ? " status-sfx" : ""}`}
          title={seg.error_message ?? undefined}
        >
          {displayStatus(lang, seg)}
        </span>
      </td>
      <td>
        <button
          className="btn btn-sm"
          onClick={(e) => {
            e.stopPropagation();
            onGenerate();
          }}
        >
          生成
        </button>
        {seg.audio_path && (
          <button
            className="btn btn-sm"
            style={{ marginLeft: 4 }}
            onClick={(e) => {
              e.stopPropagation();
              onPlay();
            }}
          >
            ▶
          </button>
        )}
      </td>
    </tr>
  );
}