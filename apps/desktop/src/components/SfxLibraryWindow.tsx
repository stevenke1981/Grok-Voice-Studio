import { SfxLibraryPanel } from "./SfxLibraryPanel";
import { FloatingWindow } from "./FloatingWindow";
import { t, type Lang } from "../i18n";

interface Props {
  lang: Lang;
  open: boolean;
  onClose: () => void;
  onInsert: (text: string) => void;
  onPreview: (path: string) => void;
}

export function SfxLibraryWindow({ lang, open, onClose, onInsert, onPreview }: Props) {
  if (!open) return null;

  return (
    <div className="floating-window-layer">
      <FloatingWindow
        title={t(lang, "sfxLibrary")}
        onClose={onClose}
        storageKey="sfx-library-window"
        defaultPosition={{ x: 96, y: 96 }}
        defaultSize={{ w: 540, h: 460 }}
      >
        <SfxLibraryPanel lang={lang} onInsert={onInsert} onPreview={onPreview} />
      </FloatingWindow>
    </div>
  );
}