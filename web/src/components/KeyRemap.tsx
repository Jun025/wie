import { useEffect, useState } from "react";
import { ALL_KEYS, PRESETS, type EmuKey, loadKeymap, resetKeymap, saveKeymap, unboundKeys } from "../lib/keymap";

interface KeyRemapProps {
  onChange: (map: Record<string, EmuKey>) => void;
}

// Rebind keyboard keys to emulator keys. Bindings + presets are stored in
// localStorage only — never sent anywhere.
export function KeyRemap({ onChange }: KeyRemapProps) {
  const [map, setMap] = useState<Record<string, EmuKey>>(() => loadKeymap());
  const [listeningFor, setListeningFor] = useState<EmuKey | null>(null);

  useEffect(() => {
    if (!listeningFor) return;
    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      setMap((prev) => {
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

  const apply = (next: Record<string, EmuKey>) => {
    setMap(next);
    saveKeymap(next);
    onChange(next);
  };

  const bindingsFor = (key: EmuKey): string[] =>
    Object.entries(map)
      .filter(([, v]) => v === key)
      .map(([code]) => code.replace(/^(Key|Digit|Arrow)/, ""));

  const unbound = unboundKeys(map);

  return (
    <div className="text-sm">
      <div className="flex flex-wrap items-center justify-between gap-2 mb-3">
        <h3 className="font-semibold text-fg">키 리매핑</h3>
        <div className="flex flex-wrap gap-1.5">
          {PRESETS.map((p) => (
            <button
              key={p.id}
              type="button"
              className="text-xs px-2 py-1 rounded bg-surface2 border border-edge text-fg-dim hover:text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
              onClick={() => apply({ ...p.map })}
            >
              {p.label}
            </button>
          ))}
          <button
            type="button"
            className="text-xs px-2 py-1 rounded bg-surface2 border border-edge text-fg-dim hover:text-fg"
            onClick={() => apply(resetKeymap())}
          >
            기본값
          </button>
        </div>
      </div>

      {unbound.length > 0 && (
        <p className="mb-2 text-xs text-amber-600 dark:text-amber-300">
          ⚠ 바인딩이 없는 키: {unbound.join(", ")} — 필요한 키에 물리 키를 할당하세요.
        </p>
      )}

      <div className="grid grid-cols-2 gap-x-4 gap-y-1">
        {ALL_KEYS.map((key) => (
          <div key={key} className="flex items-center justify-between gap-2 py-0.5">
            <span className="text-fg-dim">{key}</span>
            <button
              type="button"
              className={
                "px-2 py-0.5 rounded text-xs min-w-24 text-right border border-edge focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent " +
                (listeningFor === key ? "bg-accent text-accent-fg" : "bg-surface2 text-fg-dim hover:text-fg")
              }
              onClick={() => setListeningFor(key)}
            >
              {listeningFor === key ? "키 입력 대기…" : bindingsFor(key).join(", ") || "—"}
            </button>
          </div>
        ))}
      </div>
      <p className="text-xs text-fg-dim mt-2">버튼을 누른 뒤 원하는 키보드 키를 누르면 재할당됩니다. 프리셋으로 한 번에 바꿀 수도 있습니다.</p>
    </div>
  );
}
