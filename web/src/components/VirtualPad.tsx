import type { EmuKey } from "../lib/keymap";

interface PadButtonProps {
  label: string;
  emuKey: EmuKey;
  onPress: (key: EmuKey) => void;
  onRelease: (key: EmuKey) => void;
  className?: string;
  title?: string;
}

// Pointer events unify touch + mouse. We release on up/cancel so a finger
// sliding off the button never leaves a key stuck down. All buttons meet a
// >=48px touch target.
function PadButton({ label, emuKey, onPress, onRelease, className = "", title }: PadButtonProps) {
  return (
    <button
      type="button"
      aria-label={title ?? emuKey}
      title={title ?? emuKey}
      className={
        "no-select select-none rounded-xl bg-slate-700 active:bg-sky-500 text-white font-semibold " +
        "shadow-md flex items-center justify-center leading-none min-h-12 min-w-12 " +
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-sky-400 " +
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
    <div className="w-full max-w-md mx-auto select-none flex flex-col gap-4" aria-label="가상 키패드">
      {/* soft keys */}
      <div className="flex justify-between gap-3">
        <PadButton label="◀ L" emuKey="LEFT_SOFT_KEY" title="왼쪽 소프트키" {...common} className="h-12 flex-1 text-base" />
        <PadButton label="R ▶" emuKey="RIGHT_SOFT_KEY" title="오른쪽 소프트키" {...common} className="h-12 flex-1 text-base" />
      </div>

      <div className="flex items-center justify-between gap-5">
        {/* D-pad — large central control */}
        <div className="grid grid-cols-3 grid-rows-3 gap-1.5 w-52 h-52 shrink-0">
          <span />
          <PadButton label="▲" emuKey="UP" {...common} className="h-full text-xl" />
          <span />
          <PadButton label="◀" emuKey="LEFT" {...common} className="h-full text-xl" />
          <PadButton label="OK" emuKey="OK" {...common} className="h-full bg-sky-700 text-lg" />
          <PadButton label="▶" emuKey="RIGHT" {...common} className="h-full text-xl" />
          <span />
          <PadButton label="▼" emuKey="DOWN" {...common} className="h-full text-xl" />
          <span />
        </div>

        {/* phone keys */}
        <div className="grid grid-cols-2 gap-2">
          <PadButton label="📞" emuKey="CALL" title="통화" {...common} className="h-14 w-16 bg-green-700 text-lg" />
          <PadButton label="⛔" emuKey="HANGUP" title="종료" {...common} className="h-14 w-16 bg-red-800 text-lg" />
          <PadButton label="🔉" emuKey="VOLUME_DOWN" title="볼륨 -" {...common} className="h-14 w-16 text-lg" />
          <PadButton label="🔊" emuKey="VOLUME_UP" title="볼륨 +" {...common} className="h-14 w-16 text-lg" />
        </div>
      </div>

      {/* numeric keypad + * 0 # CLR */}
      <div className="grid grid-cols-3 gap-2 max-w-xs mx-auto w-full">
        {(["NUM1", "NUM2", "NUM3", "NUM4", "NUM5", "NUM6", "NUM7", "NUM8", "NUM9"] as EmuKey[]).map((k) => (
          <PadButton key={k} label={k.replace("NUM", "")} emuKey={k} {...common} className="h-14 text-lg" />
        ))}
        <PadButton label="✳" emuKey="STAR" {...common} className="h-14 text-lg" />
        <PadButton label="0" emuKey="NUM0" {...common} className="h-14 text-lg" />
        <PadButton label="#" emuKey="HASH" {...common} className="h-14 text-lg" />
        <PadButton label="CLR" emuKey="CLEAR" title="지우기" {...common} className="h-12 text-sm col-span-3" />
      </div>
    </div>
  );
}
