import { useRef, useState } from "react";
import type { EmuKey } from "../lib/keymap";

interface Props {
  press: (k: EmuKey) => void;
  release: (k: EmuKey) => void;
  repeat?: (k: EmuKey) => void; // hold-to-repeat (core Keyrepeat), like the old D-pad
  className?: string; // sizes the pad (e.g. "h-40 w-40")
}

function haptic() {
  // Same defensive guard as GameButton: navigator.vibrate is absent / throws on
  // iOS + insecure contexts.
  try {
    navigator.vibrate?.(8);
  } catch {
    /* unsupported */
  }
}

// Hold-and-slide direction pad (2번). A single pointer is captured on press; as it
// slides, the NEAREST of ↑↓←→ becomes the held key (4-direction swipe — no
// diagonals), switching live as the finger moves. Exactly one direction key is
// held at a time: changing direction releases the previous and presses the next;
// lifting releases the active key. Modern mobile-game feel vs. four fixed squares.
export function Joystick({ press, release, repeat, className = "" }: Props) {
  const padRef = useRef<HTMLDivElement>(null);
  const pointer = useRef<number | null>(null);
  const center = useRef<{ x: number; y: number; r: number }>({ x: 0, y: 0, r: 1 });
  const active = useRef<EmuKey | null>(null);
  const timers = useRef<{ delay?: number; interval?: number }>({});
  const [knob, setKnob] = useState<{ x: number; y: number }>({ x: 0, y: 0 });
  const [held, setHeld] = useState<EmuKey | null>(null); // for the knob highlight

  const clearRepeat = () => {
    if (timers.current.delay) clearTimeout(timers.current.delay);
    if (timers.current.interval) clearInterval(timers.current.interval);
    timers.current = {};
  };

  const setDir = (dir: EmuKey | null) => {
    if (dir === active.current) return;
    clearRepeat();
    if (active.current) release(active.current);
    active.current = dir;
    setHeld(dir);
    if (dir) {
      press(dir);
      haptic();
      if (repeat) {
        // Same cadence as GameButton's hold-to-repeat: 350ms delay, then 90ms.
        timers.current.delay = window.setTimeout(() => {
          timers.current.interval = window.setInterval(() => repeat(dir), 90);
        }, 350);
      }
    }
  };

  const update = (cx: number, cy: number) => {
    const { x, y, r } = center.current;
    const dx = cx - x;
    const dy = cy - y;
    const mag = Math.hypot(dx, dy);
    // Knob follows the finger, clamped to the pad radius (visual feedback).
    const clamp = mag > r ? r / mag : 1;
    setKnob({ x: dx * clamp, y: dy * clamp });
    // Inside the deadzone → no direction (neutral).
    if (mag < r * 0.35) return setDir(null);
    const dir: EmuKey = Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? "RIGHT" : "LEFT") : dy > 0 ? "DOWN" : "UP";
    setDir(dir);
  };

  const begin = (e: React.PointerEvent) => {
    const el = padRef.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    center.current = { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2, r: rect.width / 2 };
    pointer.current = e.pointerId;
    try {
      el.setPointerCapture(e.pointerId);
    } catch {
      /* capture not supported */
    }
    update(e.clientX, e.clientY);
  };

  const end = (pointerId?: number) => {
    if (pointerId !== undefined && pointer.current !== null && pointerId !== pointer.current) return;
    pointer.current = null;
    setDir(null);
    setKnob({ x: 0, y: 0 });
  };

  return (
    <div
      ref={padRef}
      role="group"
      aria-label="방향 조이스틱 — 누른 채 밀어 이동"
      className={`relative touch-none select-none rounded-full border border-edge bg-surface2 shadow-inner ${className}`}
      onPointerDown={(e) => {
        e.preventDefault();
        begin(e);
      }}
      onPointerMove={(e) => {
        if (pointer.current === null) return;
        e.preventDefault();
        update(e.clientX, e.clientY);
      }}
      onPointerUp={(e) => {
        e.preventDefault();
        end(e.pointerId);
      }}
      onPointerCancel={(e) => end(e.pointerId)}
      onLostPointerCapture={() => end()}
      onContextMenu={(e) => e.preventDefault()}
    >
      {/* direction hints around the rim */}
      <span className="pointer-events-none absolute inset-x-0 top-1 text-center text-xs text-fg-dim">▲</span>
      <span className="pointer-events-none absolute inset-x-0 bottom-1 text-center text-xs text-fg-dim">▼</span>
      <span className="pointer-events-none absolute inset-y-0 left-1.5 flex items-center text-xs text-fg-dim">◀</span>
      <span className="pointer-events-none absolute inset-y-0 right-1.5 flex items-center text-xs text-fg-dim">▶</span>
      {/* knob — follows the finger, highlights while a direction is held */}
      <div
        className={`pointer-events-none absolute left-1/2 top-1/2 h-1/2 w-1/2 rounded-full border shadow ${held ? "border-accent bg-accent" : "border-edge bg-surface"}`}
        style={{ transform: `translate(calc(-50% + ${knob.x}px), calc(-50% + ${knob.y}px))` }}
      />
    </div>
  );
}
