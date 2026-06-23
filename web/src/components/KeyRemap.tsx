import { useEffect, useState } from "react";
import { ALL_KEYS, type EmuKey, loadKeymap, resetKeymap, saveKeymap } from "../lib/keymap";

interface KeyRemapProps {
  onChange: (map: Record<string, EmuKey>) => void;
}

// A small panel to rebind keyboard keys to emulator keys. Bindings are stored
// in localStorage only.
export function KeyRemap({ onChange }: KeyRemapProps) {
  const [map, setMap] = useState<Record<string, EmuKey>>(() => loadKeymap());
  const [listeningFor, setListeningFor] = useState<EmuKey | null>(null);

  useEffect(() => {
    if (!listeningFor) return;
    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      setMap((prev) => {
        // remove any existing binding to this physical key, then assign
        const next: Record<string, EmuKey> = {};
        for (const [code, key] of Object.entries(prev)) {
          if (code !== e.code) next[code] = key;
        }
        next[e.code] = listeningFor;
        saveKeymap(next);
        onChange(next);
        return next;
      });
      setListeningFor(null);
    };
    window.addEventListener("keydown", handler, { once: true });
    return () => window.removeEventListener("keydown", handler);
  }, [listeningFor, onChange]);

  const bindingsFor = (key: EmuKey): string[] =>
    Object.entries(map)
      .filter(([, v]) => v === key)
      .map(([code]) => code);

  return (
    <div className="text-sm">
      <div className="flex items-center justify-between mb-2">
        <h3 className="font-semibold text-slate-200">키 리매핑</h3>
        <button
          type="button"
          className="text-xs px-2 py-1 rounded bg-slate-700 hover:bg-slate-600"
          onClick={() => {
            const def = resetKeymap();
            setMap(def);
            onChange(def);
          }}
        >
          기본값 복원
        </button>
      </div>
      <div className="grid grid-cols-2 gap-x-4 gap-y-1">
        {ALL_KEYS.map((key) => (
          <div key={key} className="flex items-center justify-between gap-2 py-0.5">
            <span className="text-slate-400">{key}</span>
            <button
              type="button"
              className={
                "px-2 py-0.5 rounded text-xs min-w-24 text-right " +
                (listeningFor === key ? "bg-sky-600 text-white" : "bg-slate-800 text-slate-300 hover:bg-slate-700")
              }
              onClick={() => setListeningFor(key)}
            >
              {listeningFor === key ? "키 입력 대기…" : bindingsFor(key).join(", ") || "—"}
            </button>
          </div>
        ))}
      </div>
      <p className="text-xs text-slate-500 mt-2">버튼을 누른 뒤 원하는 키보드 키를 누르면 재할당됩니다.</p>
    </div>
  );
}
