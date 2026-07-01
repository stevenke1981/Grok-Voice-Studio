import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";

interface Position {
  x: number;
  y: number;
}

interface Size {
  w: number;
  h: number;
}

interface Props {
  title: string;
  children: ReactNode;
  onClose: () => void;
  defaultPosition?: Position;
  defaultSize?: Size;
  minWidth?: number;
  minHeight?: number;
  storageKey?: string;
}

const DEFAULT_SIZE: Size = { w: 520, h: 440 };
const DEFAULT_POS: Position = { x: 72, y: 72 };

function loadStored<T>(key: string): T | null {
  try {
    const raw = sessionStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : null;
  } catch {
    return null;
  }
}

export function FloatingWindow({
  title,
  children,
  onClose,
  defaultPosition = DEFAULT_POS,
  defaultSize = DEFAULT_SIZE,
  minWidth = 360,
  minHeight = 280,
  storageKey,
}: Props) {
  const posKey = storageKey ? `${storageKey}-pos` : null;
  const sizeKey = storageKey ? `${storageKey}-size` : null;

  const [pos, setPos] = useState<Position>(() =>
    (posKey && loadStored<Position>(posKey)) || defaultPosition,
  );
  const [size, setSize] = useState<Size>(() =>
    (sizeKey && loadStored<Size>(sizeKey)) || defaultSize,
  );

  const dragRef = useRef<{
    pointerId: number;
    startX: number;
    startY: number;
    origX: number;
    origY: number;
  } | null>(null);

  const resizeRef = useRef<{
    pointerId: number;
    startX: number;
    startY: number;
    origW: number;
    origH: number;
  } | null>(null);

  useEffect(() => {
    if (posKey) sessionStorage.setItem(posKey, JSON.stringify(pos));
  }, [pos, posKey]);

  useEffect(() => {
    if (sizeKey) sessionStorage.setItem(sizeKey, JSON.stringify(size));
  }, [size, sizeKey]);

  const clampPosition = useCallback(
    (x: number, y: number) => {
      const maxX = Math.max(0, window.innerWidth - size.w - 8);
      const maxY = Math.max(0, window.innerHeight - 48);
      return {
        x: Math.min(Math.max(8, x), maxX),
        y: Math.min(Math.max(8, y), maxY),
      };
    },
    [size.w],
  );

  const onHeaderPointerDown = (e: React.PointerEvent) => {
    if ((e.target as HTMLElement).closest("button")) return;
    dragRef.current = {
      pointerId: e.pointerId,
      startX: e.clientX,
      startY: e.clientY,
      origX: pos.x,
      origY: pos.y,
    };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  };

  const onHeaderPointerMove = (e: React.PointerEvent) => {
    if (!dragRef.current || dragRef.current.pointerId !== e.pointerId) return;
    const dx = e.clientX - dragRef.current.startX;
    const dy = e.clientY - dragRef.current.startY;
    setPos(clampPosition(dragRef.current.origX + dx, dragRef.current.origY + dy));
  };

  const onHeaderPointerUp = (e: React.PointerEvent) => {
    if (dragRef.current?.pointerId === e.pointerId) {
      dragRef.current = null;
    }
  };

  const onResizePointerDown = (e: React.PointerEvent) => {
    e.stopPropagation();
    resizeRef.current = {
      pointerId: e.pointerId,
      startX: e.clientX,
      startY: e.clientY,
      origW: size.w,
      origH: size.h,
    };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  };

  const onResizePointerMove = (e: React.PointerEvent) => {
    if (!resizeRef.current || resizeRef.current.pointerId !== e.pointerId) return;
    const dw = e.clientX - resizeRef.current.startX;
    const dh = e.clientY - resizeRef.current.startY;
    setSize({
      w: Math.max(minWidth, resizeRef.current.origW + dw),
      h: Math.max(minHeight, resizeRef.current.origH + dh),
    });
  };

  const onResizePointerUp = (e: React.PointerEvent) => {
    if (resizeRef.current?.pointerId === e.pointerId) {
      resizeRef.current = null;
    }
  };

  return (
    <div
      className="floating-window"
      style={{ left: pos.x, top: pos.y, width: size.w, height: size.h }}
      onClick={(e) => e.stopPropagation()}
    >
      <div
        className="floating-window-header"
        onPointerDown={onHeaderPointerDown}
        onPointerMove={onHeaderPointerMove}
        onPointerUp={onHeaderPointerUp}
        onPointerCancel={onHeaderPointerUp}
      >
        <span className="floating-window-title">{title}</span>
        <button className="btn btn-sm floating-window-close" onClick={onClose} type="button">
          ×
        </button>
      </div>
      <div className="floating-window-body">{children}</div>
      <div
        className="floating-window-resize"
        onPointerDown={onResizePointerDown}
        onPointerMove={onResizePointerMove}
        onPointerUp={onResizePointerUp}
        onPointerCancel={onResizePointerUp}
        title="Resize"
      />
    </div>
  );
}