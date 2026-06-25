import { useRef } from "react";

interface Props {
  label: string;
  onDown: () => void;
  onUp?: () => void;
  onRepeat?: () => void; // hold-to-repeat — sends the core's Keyrepeat, NOT a new Keydown
  repeat?: boolean;
  className?: string;
  title: string;
}

function haptic() {
  // navigator.vibrate is absent on iOS/insecure contexts and THROWS via some
  // bindings — guard + swallow (same lesson as the platform vibrate fix).
  try {
    navigator.vibrate?.(8);
  } catch {
    /* unsupported */
  }
}

// A momentary game-key button.
//
// INPUT CORRECTNESS (입력 버그 수정):
//  • pointerdown → EXACTLY ONE keyDown; pointerup/cancel → EXACTLY ONE keyUp.
//  • A single active pointer is tracked by pointerId: a second pointerdown while
//    one is already held is ignored, and only the owning pointer's up/cancel
//    releases — so a held button can never be misread as rapid re-presses (연타).
//  • Intended hold-to-repeat sends the core's Keyrepeat (feature-phone long-press
//    semantics), never a stream of fresh keyDowns.
//  • touch-action:none + preventDefault + setPointerCapture keep a press/drag on a
//    control from scrolling or jittering the page.
export function GameButton({ label, onDown, onUp, onRepeat, repeat, className = "", title }: Props) {
  const timers = useRef<{ delay?: number; interval?: number }>({});
  const activePointer = useRef<number | null>(null);

  const start = (el: HTMLElement, pointerId: number) => {
    if (activePointer.current !== null) return; // already held — no duplicate keyDown
    activePointer.current = pointerId;
    try {
      el.setPointerCapture?.(pointerId);
    } catch {
      /* capture not supported / pointer already gone */
    }
    haptic();
    onDown();
    if (repeat) {
      const rep = onRepeat ?? onDown; // fall back to onDown only if no repeat handler given
      timers.current.delay = window.setTimeout(() => {
        timers.current.interval = window.setInterval(rep, 90);
      }, 350);
    }
  };

  const stop = (pointerId?: number) => {
    // Only the owning pointer (or an untracked cancel/lost-capture) releases.
    if (pointerId !== undefined && activePointer.current !== null && pointerId !== activePointer.current) return;
    if (activePointer.current === null) return; // not held — no spurious keyUp
    if (timers.current.delay) clearTimeout(timers.current.delay);
    if (timers.current.interval) clearInterval(timers.current.interval);
    timers.current = {};
    activePointer.current = null;
    onUp?.();
  };

  return (
    <button
      type="button"
      aria-label={title}
      title={title}
      className={
        "no-select select-none touch-none rounded-xl bg-surface2 text-fg active:bg-accent active:text-accent-fg font-semibold " +
        "border border-edge shadow-sm flex items-center justify-center leading-none " +
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent " +
        className
      }
      onPointerDown={(e) => {
        e.preventDefault();
        start(e.currentTarget, e.pointerId);
      }}
      onPointerUp={(e) => {
        e.preventDefault();
        stop(e.pointerId);
      }}
      onPointerCancel={(e) => stop(e.pointerId)}
      onLostPointerCapture={() => stop()}
      onContextMenu={(e) => e.preventDefault()}
    >
      {label}
    </button>
  );
}
