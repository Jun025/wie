import type { EmuKey } from "../lib/keymap";

interface PadButtonProps {
  label: string;
  emuKey: EmuKey;
  onPress: (key: EmuKey) => void;
  onRelease: (key: EmuKey) => void;
  className?: string;
}

function PadButton({ label, emuKey, onPress, onRelease, className = "" }: PadButtonProps) {
  // Pointer events unify touch + mouse. We also release on cancel/leave so a
  // finger sliding off the button doesn't leave the key stuck down.
  return (
    <button
      type="button"
      aria-label={emuKey}
      className={
        "no-select select-none rounded-xl bg-slate-700 active:bg-sky-500 text-white font-semibold " +
        "shadow-md flex items-center justify-center text-lg leading-none " +
        className
      }
      onPointerDown={(e) => {
        e.preventDefault();
        (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
        onPress(emuKey);
      }}
      onPointerUp={(e) => {
        e.preventDefault();
        onRelease(emuKey);
      }}
      onPointerCancel={() => onRelease(emuKey)}
      onContextMenu={(e) => e.preventDefault()}
    >
      {label}
    </button>
  );
}

interface VirtualPadProps {
  onPress: (key: EmuKey) => void;
  onRelease: (key: EmuKey) => void;
}

export function VirtualPad({ onPress, onRelease }: VirtualPadProps) {
  const common = { onPress, onRelease };
  return (
    <div className="w-full max-w-md mx-auto select-none">
      {/* soft keys */}
      <div className="flex justify-between mb-4 gap-3">
        <PadButton label="◀ L" emuKey="LEFT_SOFT_KEY" {...common} className="h-12 flex-1" />
        <PadButton label="R ▶" emuKey="RIGHT_SOFT_KEY" {...common} className="h-12 flex-1" />
      </div>

      <div className="flex items-center justify-between gap-6">
        {/* D-pad */}
        <div className="grid grid-cols-3 grid-rows-3 gap-1 w-44 h-44">
          <span />
          <PadButton label="▲" emuKey="UP" {...common} className="h-full" />
          <span />
          <PadButton label="◀" emuKey="LEFT" {...common} className="h-full" />
          <PadButton label="OK" emuKey="OK" {...common} className="h-full bg-sky-700" />
          <PadButton label="▶" emuKey="RIGHT" {...common} className="h-full" />
          <span />
          <PadButton label="▼" emuKey="DOWN" {...common} className="h-full" />
          <span />
        </div>

        {/* action keys */}
        <div className="grid grid-cols-2 gap-2">
          <PadButton label="CLR" emuKey="CLEAR" {...common} className="h-14 w-16" />
          <PadButton label="📞" emuKey="CALL" {...common} className="h-14 w-16 bg-green-700" />
          <PadButton label="✳" emuKey="STAR" {...common} className="h-14 w-16" />
          <PadButton label="#" emuKey="HASH" {...common} className="h-14 w-16" />
        </div>
      </div>

      {/* numeric keypad */}
      <div className="grid grid-cols-3 gap-2 mt-4 max-w-xs mx-auto">
        {(["NUM1", "NUM2", "NUM3", "NUM4", "NUM5", "NUM6", "NUM7", "NUM8", "NUM9", "NUM0"] as EmuKey[]).map((k) => (
          <PadButton key={k} label={k.replace("NUM", "")} emuKey={k} {...common} className="h-12" />
        ))}
      </div>
    </div>
  );
}
