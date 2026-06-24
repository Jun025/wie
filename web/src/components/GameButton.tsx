import { useRef } from "react";

interface Props {
  label: string;
  onDown: () => void;
  onUp?: () => void;
  repeat?: boolean;
  className?: string;
  title: string;
}

function haptic() {
  try {
    navigator.vibrate?.(8);
  } catch {
    /* unsupported */
  }
}

// A momentary game-key button: pointer (touch+mouse) press/release with a light
// haptic and optional hold-to-repeat. Releases on up/cancel/leave so a finger
// sliding off never sticks a key down. >=44px touch target.
export function GameButton({ label, onDown, onUp, repeat, className = "", title }: Props) {
  const timers = useRef<{ delay?: number; interval?: number }>({});

  const start = (el: HTMLElement, pointerId: number) => {
    el.setPointerCapture?.(pointerId);
    haptic();
    onDown();
    if (repeat) {
      timers.current.delay = window.setTimeout(() => {
        timers.current.interval = window.setInterval(onDown, 90);
      }, 350);
    }
  };
  const stop = () => {
    if (timers.current.delay) clearTimeout(timers.current.delay);
    if (timers.current.interval) clearInterval(timers.current.interval);
    timers.current = {};
    onUp?.();
  };

  return (
    <button
      type="button"
      aria-label={title}
      title={title}
      className={
        "no-select select-none rounded-xl bg-surface2 text-fg active:bg-accent active:text-accent-fg font-semibold " +
        "border border-edge shadow-sm flex items-center justify-center leading-none " +
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
