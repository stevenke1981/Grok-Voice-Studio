import { t, type Lang } from "../i18n";

interface Props {
  lang: Lang;
  onStart: () => void;
}

export function OnboardingModal({ lang, onStart }: Props) {
  return (
    <div className="modal-overlay">
      <div className="modal">
        <h2>{t(lang, "onboardingTitle")}</h2>
        <p style={{ margin: "12px 0", color: "var(--muted)", lineHeight: 1.8 }}>
          {t(lang, "onboardingStep1")}
          <br />
          {t(lang, "onboardingStep2")}
          <br />
          {t(lang, "onboardingStep3")}
        </p>
        <div className="modal-actions">
          <button className="btn btn-primary" onClick={onStart}>
            {t(lang, "start")}
          </button>
        </div>
      </div>
    </div>
  );
}