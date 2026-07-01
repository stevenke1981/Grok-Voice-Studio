import { t, type Lang } from "../i18n";

export type WorkflowStep = "script" | "parse" | "voices" | "generate" | "export";

interface Props {
  lang: Lang;
  currentStep: WorkflowStep;
  hasScript: boolean;
  segmentCount: number;
  pendingCount: number;
  doneCount: number;
  hasApiKey: boolean;
  onGoParse?: () => void;
  onGoSettings?: () => void;
}

const STEPS: { id: WorkflowStep; labelKey: "stepScript" | "stepParse" | "stepVoices" | "stepGenerate" | "stepExport" }[] = [
  { id: "script", labelKey: "stepScript" },
  { id: "parse", labelKey: "stepParse" },
  { id: "voices", labelKey: "stepVoices" },
  { id: "generate", labelKey: "stepGenerate" },
  { id: "export", labelKey: "stepExport" },
];

function stepIndex(step: WorkflowStep) {
  return STEPS.findIndex((s) => s.id === step);
}

export function WorkflowGuide({
  lang,
  currentStep,
  hasScript,
  segmentCount,
  pendingCount,
  doneCount,
  hasApiKey,
  onGoParse,
  onGoSettings,
}: Props) {
  const currentIdx = stepIndex(currentStep);

  const hint = (() => {
    if (!hasApiKey) {
      return { text: t(lang, "hintNeedApiKey"), action: t(lang, "goSettings"), onClick: onGoSettings };
    }
    if (hasScript && segmentCount === 0) {
      return { text: t(lang, "hintNeedParse"), action: t(lang, "goParse"), onClick: onGoParse };
    }
    if (segmentCount > 0 && pendingCount > 0) {
      return { text: t(lang, "hintNeedGenerate"), action: null, onClick: undefined };
    }
    if (segmentCount > 0 && doneCount > 0 && pendingCount === 0) {
      return { text: t(lang, "hintReadyExport"), action: null, onClick: undefined };
    }
    return { text: t(lang, "hintEnterScript"), action: null, onClick: undefined };
  })();

  return (
    <div className="workflow-guide">
      <div className="workflow-steps">
        {STEPS.map((step, i) => {
          const isDone = i < currentIdx;
          const isActive = i === currentIdx;
          const isParse = step.id === "parse";
          return (
            <div
              key={step.id}
              className={[
                "workflow-step",
                isDone ? "done" : "",
                isActive ? "active" : "",
                isActive && isParse ? "highlight-parse" : "",
              ]
                .filter(Boolean)
                .join(" ")}
            >
              <span className="workflow-step-num">{isDone ? "✓" : i + 1}</span>
              <span className="workflow-step-label">{t(lang, step.labelKey)}</span>
              {isActive && <span className="workflow-step-badge">{t(lang, "currentStep")}</span>}
            </div>
          );
        })}
      </div>
      <div className="workflow-hint">
        <span>{hint.text}</span>
        {hint.action && hint.onClick && (
          <button className="btn btn-sm btn-primary" onClick={hint.onClick}>
            {hint.action}
          </button>
        )}
      </div>
    </div>
  );
}

export function resolveWorkflowStep(
  hasScript: boolean,
  segmentCount: number,
  allVoicesAssigned: boolean,
  pendingCount: number,
  doneCount: number,
): WorkflowStep {
  if (!hasScript) return "script";
  if (segmentCount === 0) return "parse";
  if (!allVoicesAssigned) return "voices";
  if (pendingCount > 0) return "generate";
  if (doneCount > 0) return "export";
  return "generate";
}