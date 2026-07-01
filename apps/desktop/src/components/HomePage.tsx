import type { RecentProject } from "../types";
import { DIALOGUE_TEMPLATES, STORY_TEMPLATES } from "../templates";
import { t, type Lang } from "../i18n";

interface Props {
  lang: Lang;
  recentProjects: RecentProject[];
  error: string | null;
  onNewProject: () => void;
  onOpenProject: () => void;
  onOpenRecent: (path: string) => void;
  onSettings: () => void;
}

export function HomePage({
  lang,
  recentProjects,
  error,
  onNewProject,
  onOpenProject,
  onOpenRecent,
  onSettings,
}: Props) {
  return (
    <div className="home-screen">
      <h1>{t(lang, "appTitle")}</h1>
      <p style={{ color: "var(--muted)" }}>{t(lang, "subtitle")}</p>
      <div className="home-actions">
        <button className="btn btn-primary" onClick={onNewProject}>
          {t(lang, "newProject")}
        </button>
        <button className="btn" onClick={onOpenProject}>
          {t(lang, "openProject")}
        </button>
        <button className="btn" onClick={onSettings}>
          {t(lang, "settings")}
        </button>
      </div>
      <div className="template-gallery">
        <div className="panel-header">{t(lang, "templateSection")}</div>
        <div className="template-grid">
          {DIALOGUE_TEMPLATES.slice(0, 4).map((tpl) => (
            <div key={tpl.id} className="template-card" title={lang === "en" ? tpl.descriptionEn : tpl.description}>
              <div className="template-card-name">{lang === "en" ? tpl.nameEn : tpl.name}</div>
              <div className="template-card-desc">{lang === "en" ? tpl.descriptionEn : tpl.description}</div>
            </div>
          ))}
        </div>
        <div className="template-hint" style={{ fontSize: 11, color: "var(--muted)", marginTop: 6 }}>
          {DIALOGUE_TEMPLATES.length} {t(lang, "loadTemplate")} + {STORY_TEMPLATES.length} Story
        </div>
      </div>
      {recentProjects.length > 0 && (
        <div className="recent-list">
          <div className="panel-header">{t(lang, "recentProjects")}</div>
          {recentProjects.map((rp) => (
            <div key={rp.id} className="recent-item" onClick={() => onOpenRecent(rp.path)}>
              <div>{rp.title}</div>
              <div style={{ fontSize: 11, color: "var(--muted)" }}>{rp.path}</div>
            </div>
          ))}
        </div>
      )}
      {error && <div className="error-banner">{error}</div>}
    </div>
  );
}