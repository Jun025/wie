import { useRef } from "react";
import type { EmuKey } from "../lib/keymap";

interface PadButtonProps {
  label: string;
  emuKey: EmuKey;
  onPress: (key: EmuKey) => void;
  onRelease: (key: EmuKey) => void;
  className?: string;
  title?: string;
  repeat?: boolean;
}

// Light haptic tap on press where supported (mobile). No-op elsewhere.
function haptic() {
  try {
    navigator.vibrate?.(8);
  } catch {
    /* unsupported */
  }
}

// Pointer events unify touch + mouse. We release on up/cancel so a finger
// sliding off never leaves a key stuck down. `repeat` auto-fires the key while
// held (useful for D-pad / number entry). All buttons meet a >=48px target.
function PadButton({ label, emuKey, onPress, onRelease, className = "", title, repeat }: PadButtonProps) {
  const timers = useRef<{ delay?: number; interval?: number }>({});

  const start = (el: HTMLElement, pointerId: number) => {
    el.setPointerCapture?.(pointerId);
    haptic();
    onPress(emuKey);
    if (repeat) {
      timers.current.delay = window.setTimeout(() => {
        timers.current.interval = window.setInterval(() => onPress(emuKey), 90);
      }, 350);
    }
  };
  const stop = () => {
    if (timers.current.delay) clearTimeout(timers.current.delay);
    if (timers.current.interval) clearInterval(timers.current.interval);
    timers.current = {};
    onRelease(emuKey);
  };

  return (
    <button
      type="button"
      aria-label={title ?? emuKey}
      title={title ?? emuKey}
      className={
        "no-select select-none rounded-xl bg-surface2 text-fg active:bg-accent active:text-accent-fg font-semibold " +
        "border border-edge shadow-sm flex items-center justify-center leading-none min-h-12 min-w-12 " +
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent " +
        className
      }
      onPointerDown={(e) => {
        e.preventDefault();
        start(e.target as HTMLElement, e.pointerId);
      }}
      onPointerUp={(e) => {
        e.preventDefault();
        stop();
      }}
      onPointerCancel={stop}
      onLostPointerCapture={stop}
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
    <div className="w-full max-w-md mx-auto select-none flex flex-col gap-4" aria-label="가상 키패드" role="group">
      {/* soft keys */}
      <div className="flex justify-between gap-3">
        <PadButton label="◀ L" emuKey="LEFT_SOFT_KEY" title="왼쪽 소프트키" {...common} className="h-12 flex-1 text-base" />
        <PadButton label="R ▶" emuKey="RIGHT_SOFT_KEY" title="오른쪽 소프트키" {...common} className="h-12 flex-1 text-base" />
      </div>

      <div className="flex items-center justify-between gap-5">
        {/* D-pad — large central control, auto-repeat while held */}
        <div className="grid grid-cols-3 grid-rows-3 gap-1.5 w-52 h-52 shrink-0">
          <span />
          <PadButton label="▲" emuKey="UP" repeat {...common} className="h-full text-xl" />
          <span />
          <PadButton label="◀" emuKey="LEFT" repeat {...common} className="h-full text-xl" />
          <PadButton label="OK" emuKey="OK" {...common} className="h-full bg-accent text-accent-fg text-lg" />
          <PadButton label="▶" emuKey="RIGHT" repeat {...common} className="h-full text-xl" />
          <span />
          <PadButton label="▼" emuKey="DOWN" repeat {...common} className="h-full text-xl" />
          <span />
        </div>

        {/* phone keys */}
        <div className="grid grid-cols-2 gap-2">
          <PadButton label="📞" emuKey="CALL" title="통화" {...common} className="h-14 w-16 bg-green-600 text-white border-green-700 text-lg" />
          <PadButton label="⛔" emuKey="HANGUP" title="종료" {...common} className="h-14 w-16 bg-red-600 text-white border-red-700 text-lg" />
          <PadButton label="🔉" emuKey="VOLUME_DOWN" title="볼륨 -" {...common} className="h-14 w-16 text-lg" />
          <PadButton label="🔊" emuKey="VOLUME_UP" title="볼륨 +" {...common} className="h-14 w-16 text-lg" />
        </div>
      </div>

      {/* numeric keypad + * 0 # CLR */}
      <div className="grid grid-cols-3 gap-2 max-w-xs mx-auto w-full">
        {(["NUM1", "NUM2", "NUM3", "NUM4", "NUM5", "NUM6", "NUM7", "NUM8", "NUM9"] as EmuKey[]).map((k) => (
          <PadButton key={k} label={k.replace("NUM", "")} emuKey={k} repeat {...common} className="h-14 text-lg" />
        ))}
        <PadButton label="✳" emuKey="STAR" {...common} className="h-14 text-lg" />
        <PadButton label="0" emuKey="NUM0" repeat {...common} className="h-14 text-lg" />
        <PadButton label="#" emuKey="HASH" {...common} className="h-14 text-lg" />
        <PadButton label="CLR" emuKey="CLEAR" repeat title="지우기" {...common} className="h-12 text-sm col-span-3" />
      </div>
    </div>
  );
}
