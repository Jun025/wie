// Shared WebAudio context + iOS WebKit unlock.
//
// iOS (every browser is WebKit) refuses to start an AudioContext unless it is
// created/resumed *synchronously inside a user gesture*, and even then the output
// path stays silent until a buffer is played from within that same gesture. This
// module centralises the standard WebKit unlock so the whole app shares ONE
// context that is woken on the first touch/click/key and re-resumed whenever the
// page comes back to the foreground.
//
// No network is involved here — this is pure local audio plumbing (S5 / no-leak).

type AnyWindow = Window & { webkitAudioContext?: typeof AudioContext };

let ctx: AudioContext | null = null;
let unlockedOutput = false; // silent unlock buffer has been played at least once
let listenersInstalled = false;

// Lazily create the single shared AudioContext, preferring the standard
// constructor and falling back to Safari's prefixed `webkitAudioContext`.
export function getAudioContext(): AudioContext | null {
  if (ctx) return ctx;
  const Ctor = window.AudioContext || (window as AnyWindow).webkitAudioContext;
  if (!Ctor) return null;
  try {
    ctx = new Ctor();
  } catch {
    ctx = null;
  }
  return ctx;
}

// Resume + (once) play a 1-frame silent buffer to wake the iOS output path.
// MUST be called synchronously from a user-gesture handler — calling `resume()`
// or scheduling the buffer outside a gesture is ignored by WebKit. Safe and cheap
// to call on every gesture; it no-ops once the context is running and unlocked.
export function unlockAudio(): void {
  const c = getAudioContext();
  if (!c) return;

  // Resume synchronously within the gesture. iOS only honours this here.
  if (c.state !== "running") {
    // The promise is intentionally not awaited: awaiting would defer the work
    // past the gesture and iOS would drop it. Fire-and-forget keeps it in-gesture.
    void c.resume().catch(() => {});
  }

  if (unlockedOutput) return;
  try {
    const buffer = c.createBuffer(1, 1, c.sampleRate);
    const source = c.createBufferSource();
    source.buffer = buffer;
    source.connect(c.destination);
    source.start(0);
    unlockedOutput = true;
  } catch {
    /* unlock buffer failed; a later gesture retries */
  }
}

// Resume the shared context if the OS/browser suspended it (autoplay policy or
// iOS backgrounding). Safe to call from anywhere; a no-op when already running.
export function resumeAudio(): void {
  const c = ctx;
  if (c && c.state === "suspended") void c.resume().catch(() => {});
}

// Install global, idempotent listeners:
//  • first touch/click/key → create + resume + unlock synchronously in-gesture.
//  • returning to the foreground (visibility/focus/pageshow) → re-resume, since
//    iOS suspends the context when the tab/app is backgrounded.
// Listeners stay attached so a context that iOS re-suspends mid-session is woken
// again by the next interaction.
export function installAudioUnlock(): void {
  if (listenersInstalled) return;
  listenersInstalled = true;

  const onGesture = () => unlockAudio();
  // capture + passive so we run before app handlers and never block scrolling.
  const opts: AddEventListenerOptions = { capture: true, passive: true };
  window.addEventListener("touchend", onGesture, opts);
  window.addEventListener("touchstart", onGesture, opts);
  window.addEventListener("mousedown", onGesture, opts);
  window.addEventListener("click", onGesture, opts);
  window.addEventListener("keydown", onGesture, opts);

  const onResume = () => {
    if (document.visibilityState === "visible") resumeAudio();
  };
  document.addEventListener("visibilitychange", onResume);
  window.addEventListener("focus", onResume);
  window.addEventListener("pageshow", onResume);
}

// Diagnostics for headless/console verification of the unlock path.
export function audioState(): { hasCtx: boolean; state: string; sampleRate: number; unlockedOutput: boolean } {
  return {
    hasCtx: !!ctx,
    state: ctx?.state ?? "none",
    sampleRate: ctx?.sampleRate ?? 0,
    unlockedOutput,
  };
}
