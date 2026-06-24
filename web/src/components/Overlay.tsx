import { useEffect } from "react";

interface Props {
  title: string;
  onClose: () => void;
  children: React.ReactNode;
}

// A slide-over panel rendered ON TOP of the running player. The player stays
// mounted and keeps ticking underneath, so opening a panel never ends the game.
export function Overlay({ title, onClose, children }: Props) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div className="fixed inset-0 z-40 flex" role="dialog" aria-modal="true" aria-label={title}>
      <button type="button" aria-label="닫기" className="absolute inset-0 bg-black/50" onClick={onClose} />
      <div className="relative ml-auto h-full w-full max-w-xl overflow-y-auto bg-surface border-l border-edge shadow-2xl">
        <div className="sticky top-0 z-10 flex items-center justify-between border-b border-edge bg-surface/95 px-4 py-2 backdrop-blur">
          <h2 className="font-semibold text-fg">{title}</h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded-md bg-surface2 px-3 py-1.5 text-sm text-fg-dim hover:text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
          >
            닫기 ✕
          </button>
        </div>
        <div className="flex flex-col items-center p-4">{children}</div>
      </div>
    </div>
  );
}
