import { useCallback, useEffect, useRef, useState } from "react";
import { EmulatorSession, type EmuError, type LoadableGame, normalizeError } from "../lib/emulator";
import { type EmuKey, loadKeymap } from "../lib/keymap";
import { GameButton } from "./GameButton";
import { Joystick } from "./Joystick";
import { KeyRemap } from "./KeyRemap";
import { Overlay } from "./Overlay";
import { deviceName } from "../lib/saveSync";
import { useTheme } from "../hooks/useTheme";
import { audioState, getAudioContext, unlockAudio } from "../lib/audio";
import type { User } from "../lib/api";

// Materials for a one-click crash report (5번): the crashing game (so it can be
// referenced from the user's vault) + the pre-filled error/repro text. The save
// rides along on the server, keyed by the game's ROM hash (see App.resolveCrashReport).
export interface CrashReport {
  game: LoadableGame;
  title: string;
  body: string;
}

interface Props {
  game: LoadableGame;
  user: User | null;
  onExit: () => void;
  onMenu: () => void;
  toast: (msg: string, kind?: "ok" | "err") => void;
  onReportCrash: (payload: CrashReport) => void;
}

// Keyset mode = onscreen control LAYOUT preset (NOT key remapping — that lives in
// keymap.ts/localStorage independently and still applies in every mode).
type Keyset = 1 | 2 | 3;
const KEYSET_KEY = "wie-keyset";
function loadKeyset(): Keyset {
  const v = Number(localStorage.getItem(KEYSET_KEY));
  return v === 2 ? 2 : v === 3 ? 3 : 1;
}

// Hard ceiling on how long the player waits for a game to boot before it gives
// up and shows the error/restart panel. Covers wasm-fetch stalls and hung loads
// so "loading" can never persist indefinitely. (A synchronous infinite loop
// inside wasm construction can't be interrupted from JS — that's a separate,
// title-level render-wall issue, not a boot hang.)
const BOOT_TIMEOUT_MS = 20_000;

// Per-mode sizing for the control blocks below the display.
//   1 = balanced · 2 = joystick emphasized · 3 = numpad emphasized
//   joy = joystick pad · ok = CLR/OK column button · num/ngrid = numpad
const MODE = {
  1: { joy: "h-36 w-36", ok: "h-12 text-base", num: "h-10 text-base", ngrid: "w-36" },
  2: { joy: "h-44 w-44", ok: "h-14 text-lg", num: "h-8 text-xs", ngrid: "w-28" },
  3: { joy: "h-28 w-28", ok: "h-10 text-sm", num: "h-12 text-xl", ngrid: "w-48" },
} as const;

export function Player({ game, user, onExit, onMenu, toast, onReportCrash }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sessionRef = useRef<EmulatorSession | null>(null);
  const keymapRef = useRef<Record<string, EmuKey>>(loadKeymap());
  const { theme, toggle: toggleTheme } = useTheme();
  const [status, setStatus] = useState<"loading" | "running" | "error">("loading");
  const [error, setError] = useState<EmuError | null>(null);
  const [bootNonce, setBootNonce] = useState(0); // bump to re-boot the same game (restart after error)
  const [showRemap, setShowRemap] = useState(false);
  const [volume, setVolume] = useState(0.8);
  const [muted, setMuted] = useState(false);
  const [volOpen, setVolOpen] = useState(false); // compact volume popover
  const [keyset, setKeyset] = useState<Keyset>(loadKeyset);

  // Boot once per game; opening overlays / switching keyset does NOT remount the
  // player, so the game keeps running.
  useEffect(() => {
    let cancelled = false;
    let settled = false; // boot has reached running/error/timeout — ignore later transitions
    setStatus("loading");
    setError(null);
    const session = new EmulatorSession();
    sessionRef.current = session;
    session.onError = (err) => {
      if (cancelled) return;
      setError(err);
      setStatus("error");
    };
    session.onVolumeChange = (v, m) => {
      if (cancelled) return;
      setVolume(v);
      setMuted(m);
    };
    setVolume(session.getVolume());
    setMuted(session.isMuted());

    // Boot watchdog: if start() never resolves (e.g. the wasm binary stalls on a
    // slow/failed network, or a title's async load hangs), the player would sit
    // in "loading" forever with no feedback and no way out. Force a transition to
    // the error state so the "다시 시작 / 나가기" panel always appears.
    const watchdog = window.setTimeout(() => {
      if (cancelled || settled) return;
      settled = true;
      session.stop();
      setError({
        message: "게임이 제한 시간 안에 시작되지 않았습니다. 다시 시작하거나 다른 브라우저·기기에서 시도해 보세요.",
        name: "BootTimeout",
        phase: "load",
        platformKind: session.platformKind() ?? undefined,
      });
      setStatus("error");
    }, BOOT_TIMEOUT_MS);

    (async () => {
      // Reuse the single app-wide AudioContext that the global unlock listener
      // created+resumed synchronously inside the launch tap (iOS WebKit only
      // honours resume/unlock from within a user gesture). Calling unlockAudio()
      // here too is a cheap, in-case retry; getAudioContext() never makes a new
      // one if it already exists.
      unlockAudio();
      const audioCtx = getAudioContext();
      await new Promise((r) => requestAnimationFrame(r));
      const canvas = canvasRef.current;
      if (!canvas || cancelled) return;
      try {
        await session.start(game, canvas, audioCtx, { syncToServer: !!user, deviceLabel: deviceName() });
        // The watchdog may have already timed us out (or the effect was torn
        // down) while start() was in flight — start() arms the run loop right
        // before resolving, so stop it or it runs on behind the error panel.
        if (cancelled || settled) {
          session.stop();
          return;
        }
        settled = true;
        window.clearTimeout(watchdog);
        setStatus("running");
        // Diagnostic for headless/console verification of the unlock path.
        console.info("[wie audio]", audioState());
      } catch (e) {
        if (cancelled || settled) return;
        settled = true;
        window.clearTimeout(watchdog);
        setError(normalizeError(e, "load", session.platformKind() ?? undefined));
        setStatus("error");
      }
    })();

    return () => {
      cancelled = true;
      window.clearTimeout(watchdog);
      void session.persist();
      session.stop();
    };
  }, [game, bootNonce]);

  // Flush the save immediately when the tab is backgrounded / left (logged-in →
  // write-through to the server). Together with the 5s autosave + restore-newest-
  // on-open, this gives "real-time-like" cross-device saves without a socket.
  useEffect(() => {
    const flush = () => {
      if (document.visibilityState === "hidden") void sessionRef.current?.persist();
    };
    document.addEventListener("visibilitychange", flush);
    window.addEventListener("pagehide", flush);
    return () => {
      document.removeEventListener("visibilitychange", flush);
      window.removeEventListener("pagehide", flush);
    };
  }, []);

  const restart = useCallback(() => {
    setError(null);
    setStatus("loading");
    setBootNonce((n) => n + 1);
  }, []);

  // 5번: one-click 문의·제보 from the crash panel. Snapshot the crash-time save
  // (persist writes it to IndexedDB, and to the server when logged in), copy the
  // error details to the clipboard as a fallback, then hand the crashing game +
  // pre-filled body to App, which references the game from the vault and opens the
  // inquiry form ready to send.
  const reportCrash = useCallback(
    async (err: EmuError) => {
      try {
        await sessionRef.current?.persist();
      } catch {
        /* best-effort snapshot */
      }
      const body = buildCrashBody(err);
      void copyToClipboard(body);
      onReportCrash({ game, title: `[게임 오류] ${err.name}`, body });
    },
    [game, onReportCrash],
  );

  const press = useCallback((k: EmuKey) => sessionRef.current?.keyDown(k), []);
  const release = useCallback((k: EmuKey) => sessionRef.current?.keyUp(k), []);

  useEffect(() => {
    if (status !== "running") return;
    const down = (e: KeyboardEvent) => {
      const k = keymapRef.current[e.code];
      if (!k) return;
      e.preventDefault();
      if (k === "VOLUME_UP") return void (!e.repeat && sessionRef.current?.stepVolume(0.08));
      if (k === "VOLUME_DOWN") return void (!e.repeat && sessionRef.current?.stepVolume(-0.08));
      if (e.repeat) sessionRef.current?.keyRepeat(k);
      else sessionRef.current?.keyDown(k);
    };
    const up = (e: KeyboardEvent) => {
      const k = keymapRef.current[e.code];
      if (!k || k === "VOLUME_UP" || k === "VOLUME_DOWN") return;
      e.preventDefault();
      sessionRef.current?.keyUp(k);
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
    };
  }, [status]);

  const cycleKeyset = useCallback(() => {
    setKeyset((k) => {
      const n: Keyset = k === 1 ? 2 : k === 2 ? 3 : 1;
      localStorage.setItem(KEYSET_KEY, String(n));
      return n;
    });
  }, []);

  // onRepeat sends the core's Keyrepeat (feature-phone long-press), so a held key
  // is one Keydown + Keyrepeats + one Keyup — never a burst of fake re-presses.
  const gkey = (k: EmuKey) => ({ onDown: () => press(k), onUp: () => release(k), onRepeat: () => sessionRef.current?.keyRepeat(k) });
  // ALL side-rail controls share one uniform square footprint (UX consistency:
  // no rail button differs in width/height — the volume control included).
  const sideSq = "h-11 w-11 shrink-0 text-sm";
  // service buttons (ghost/outline) — visually distinct from solid game keys, but
  // the SAME size/shape as the game keys on the rail.
  const svc = `${sideSq} touch-none rounded-md border border-edge bg-transparent text-xs text-fg-dim hover:text-fg hover:border-accent focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent flex items-center justify-center`;
  const side = sideSq;
  const m = MODE[keyset];
  const pct = Math.round((muted ? 0 : volume) * 100);
  const volIcon = muted || pct === 0 ? "🔇" : pct < 50 ? "🔈" : "🔊";

  return (
    <section className="fixed inset-0 z-20 flex flex-col overflow-y-auto overscroll-contain bg-surface">
      {error && <ErrorPanel error={error} onRestart={restart} onExit={onExit} onReport={() => void reportCrash(error)} toast={toast} />}

      {/* Loading feedback: without it, a booting game shows only a blank canvas —
          indistinguishable from a frozen/running one — so a slow boot reads as a
          hang. Clears the moment status flips to "running" (or an error panel
          replaces it). The watchdog above guarantees it can't linger forever. */}
      {status === "loading" && !error && (
        <div
          className="pointer-events-none absolute inset-0 z-10 flex flex-col items-center justify-center gap-3 bg-surface/70 backdrop-blur-sm"
          role="status"
          aria-live="polite"
        >
          <span className="h-8 w-8 animate-spin rounded-full border-2 border-edge border-t-accent" aria-hidden="true" />
          <p className="text-sm text-fg-dim">게임을 불러오는 중…</p>
        </div>
      )}

      {/* display + side rails (no bottom bar; no top header). flex-1 so the canvas
          fills the available vertical space; gap-1 trims the side margins.

          Each rail is a flex column split into a TOP cluster (service buttons) and
          a BOTTOM cluster (game keys), packed tightly (gap-1.5) with no empty rows.
          justify-between pins the bottom clusters to the bottom; both rails carry
          exactly 2 equally-sized game keys there (L/R · 통화/종료), so they line up
          across the two sides. Every rail control is the SAME square size. */}
      <div className="flex min-h-0 flex-1 items-stretch justify-center gap-1 px-1 pt-1">
        {/* LEFT rail — top service cluster: exit, keyset, key-remap (3 buttons, to
            balance the right rail's exit-side count after moving volume right). */}
        <div className="flex flex-col justify-between gap-1.5">
          <div className="flex flex-col items-center gap-1.5">
            <button type="button" onClick={onExit} className={svc} aria-label="게임 나가기" title="나가기">←</button>
            <button type="button" onClick={cycleKeyset} className={svc} aria-label={`키보드 세트 모드 ${keyset} — 눌러서 전환`} title="키세트 모드 전환">{keyset}</button>
            <button type="button" onClick={() => setShowRemap(true)} className={svc} aria-label="키 설정(리매핑)" title="키 설정">⌨</button>
          </div>

          <div className="flex flex-col items-center gap-1.5">
            <GameButton label="◀L" title="왼쪽 소프트키" {...gkey("LEFT_SOFT_KEY")} className={side} />
            <GameButton label="📞" title="통화" {...gkey("CALL")} className={`${side} bg-green-600 text-white border-green-700`} />
          </div>
        </div>

        {/* canvas fills the space between the rails (object-fit contain keeps the
            aspect + pixelated dots), so the display is as large as possible. */}
        <div className="flex min-h-0 min-w-0 flex-1 items-center justify-center">
          <canvas
            ref={canvasRef}
            width={240}
            height={320}
            data-testid="screen"
            role="img"
            aria-label={`${game.name} 게임 화면`}
            className="emulator-canvas h-full w-full"
            style={{ objectFit: "contain" }}
          />
        </div>

        {/* RIGHT rail — top service cluster: volume, theme, menu (3 buttons, mirrors
            the left rail's 3 so both rails carry the same count). */}
        <div className="flex flex-col justify-between gap-1.5">
          <div className="flex flex-col items-center gap-1.5">
            {/* compact volume: same square footprint as every other rail button.
                Tap to open a slider popover; the GainNode stays the single,
                device-local, real-time source of truth. */}
            <div className="relative">
              <button
                type="button"
                onClick={() => setVolOpen((o) => !o)}
                aria-label={`음량 ${pct}% — 눌러서 조절`}
                aria-expanded={volOpen}
                title="음량"
                className={`${svc} flex-col gap-0`}
              >
                <span className="text-sm leading-none">{volIcon}</span>
                <span className="text-[9px] leading-none tabular-nums">{pct}%</span>
              </button>
              {volOpen && (
                <>
                  {/* click-away backdrop */}
                  <div className="fixed inset-0 z-20" onClick={() => setVolOpen(false)} aria-hidden="true" />
                  {/* opens to the LEFT (this is the right rail) */}
                  <div className="absolute right-full top-1/2 z-30 mr-1 flex -translate-y-1/2 flex-col items-center gap-2 rounded-md border border-edge bg-surface2 p-2 shadow-lg">
                    <button
                      type="button"
                      onClick={() => sessionRef.current?.toggleMute()}
                      aria-pressed={muted}
                      aria-label={muted ? "음소거 해제" : "음소거"}
                      title={muted ? "음소거 해제" : "음소거"}
                      className="text-base"
                    >
                      {volIcon}
                    </button>
                    <input
                      type="range"
                      min={0}
                      max={100}
                      value={pct}
                      onChange={(e) => sessionRef.current?.setVolume(Number(e.target.value) / 100)}
                      aria-label="음량"
                      aria-orientation="vertical"
                      aria-valuenow={pct}
                      className="h-24 w-5 accent-accent [direction:rtl] [writing-mode:vertical-lr]"
                    />
                    <span className="text-[10px] tabular-nums text-fg-dim">{pct}%</span>
                  </div>
                </>
              )}
            </div>
            <button type="button" onClick={toggleTheme} className={svc} aria-label={theme === "dark" ? "라이트 모드" : "다크 모드"} title={theme === "dark" ? "라이트 모드" : "다크 모드"}>
              {theme === "dark" ? "☀️" : "🌙"}
            </button>
            <button type="button" onClick={onMenu} className={svc} aria-label="메뉴 (라이브러리·세이브·문의·계정·도움말)" title="메뉴">☰</button>
          </div>

          <div className="flex flex-col items-center gap-1.5">
            <GameButton label="R▶" title="오른쪽 소프트키" {...gkey("RIGHT_SOFT_KEY")} className={side} />
            <GameButton label="⛔" title="종료(게임 키)" {...gkey("HANGUP")} className={`${side} bg-red-600 text-white border-red-700`} />
          </div>
        </div>
      </div>

      {/* below the display: [ joystick ] [ numpad ] [ CLR / OK column ]. Rendered
          from the LOADING screen on (1번: fixed layout — the controls never pop in,
          so nothing reflows when the game reaches "running"). Presses before boot
          are safe no-ops (emu is null). Block weights shift with the keyset mode;
          the player never unmounts, so switching is instant. */}
      <div className="flex shrink-0 items-center justify-center gap-3 px-1.5 pb-1.5 pt-1">
        {/* hold-and-slide joystick (2번) — replaces the four fixed arrow squares */}
        <Joystick press={press} release={release} repeat={(k) => sessionRef.current?.keyRepeat(k)} className={m.joy} />

        <div className={`grid grid-cols-3 gap-1 ${m.ngrid}`}>
          {(["NUM1", "NUM2", "NUM3", "NUM4", "NUM5", "NUM6", "NUM7", "NUM8", "NUM9"] as EmuKey[]).map((k) => (
            <GameButton key={k} label={k.replace("NUM", "")} title={k.replace("NUM", "")} repeat {...gkey(k)} className={`${m.num} w-full`} />
          ))}
          <GameButton label="✳" title="별표" {...gkey("STAR")} className={`${m.num} w-full`} />
          <GameButton label="0" title="0" repeat {...gkey("NUM0")} className={`${m.num} w-full`} />
          <GameButton label="#" title="우물정" {...gkey("HASH")} className={`${m.num} w-full`} />
        </div>

        {/* far right: CLR on top of OK (2번) — OK sits to the right of the numbers,
            CLR moved up from the side rail for reach. */}
        <div className="flex flex-col justify-center gap-1">
          <GameButton label="CLR" title="지우기" repeat {...gkey("CLEAR")} className={`${m.ok} w-14`} />
          <GameButton label="OK" title="확인" {...gkey("OK")} className={`${m.ok} w-14 bg-accent text-accent-fg`} />
        </div>
      </div>

      {showRemap && (
        <Overlay title="키 설정" onClose={() => setShowRemap(false)}>
          <KeyRemap onChange={(map) => (keymapRef.current = map)} />
        </Overlay>
      )}
    </section>
  );
}

// Best-effort short "Browser / OS" label. The full userAgent goes in the copy
// text; this is just a glanceable summary. userAgent is environment info, not
// game identity, so it is fine to show/copy.
function envSummary(): string {
  const ua = typeof navigator !== "undefined" ? navigator.userAgent : "";
  const os = /iPhone|iPad|iPod/.test(ua) ? "iOS" : /Android/.test(ua) ? "Android" : /Mac OS X|Macintosh/.test(ua) ? "macOS" : /Windows/.test(ua) ? "Windows" : /Linux/.test(ua) ? "Linux" : "기타";
  const br = /Edg\//.test(ua) ? "Edge" : /CriOS|Chrome\//.test(ua) ? "Chrome" : /FxiOS|Firefox\//.test(ua) ? "Firefox" : /Safari\//.test(ua) ? "Safari" : "기타";
  return `${br} / ${os}`;
}

// Build the copyable diagnostic text. CONTAINS NO GAME IDENTITY — no filename,
// hash, bytes, or title — only the error and the environment (1번 기준선 / S5).
function buildErrorReport(error: EmuError): string {
  return [
    "[wie 오류 보고]",
    `유형: ${error.phase === "load" ? "로드" : "실행"}`,
    `이름: ${error.name}`,
    `메시지: ${error.message}`,
    error.platformKind ? `플랫폼: ${error.platformKind}` : null,
    `환경: ${typeof navigator !== "undefined" ? navigator.userAgent : "unknown"}`,
    `경로: ${location.origin}${location.pathname}`, // origin+path only — never query/hash
    error.stack ? `스택:\n${error.stack}` : null,
  ]
    .filter(Boolean)
    .join("\n");
}

// Body pre-filled into the 문의 form for a crash report (5번). The error details are
// auto-included; the user only needs to add the reproduction steps. The prompt
// makes clear the save reflects the LAST save point, not the crash moment, so the
// user should describe what they did after that save.
function buildCrashBody(error: EmuError): string {
  return [
    buildErrorReport(error),
    "",
    "──────────────────────────────",
    "※ 첨부된 세이브는 '오류 발생 시점'이 아니라 '마지막 저장 시점' 기준입니다.",
    "아래에 세이브 이후 어떤 행동을 했을 때 오류가 발생했는지 적어 주세요:",
    "· 재현 순서: ",
    "· 발생 빈도(항상/가끔): ",
  ].join("\n");
}

// Copy with the same defensive guard the vibrate fix taught us: navigator.clipboard
// is undefined in insecure contexts / older WebViews, so check it's a function and
// fall back to a hidden-textarea execCommand. Never throws.
async function copyToClipboard(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard && typeof navigator.clipboard.writeText === "function") {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    /* fall through to the legacy path */
  }
  try {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.style.position = "fixed";
    ta.style.left = "-9999px";
    document.body.appendChild(ta);
    ta.focus();
    ta.select();
    const ok = document.execCommand("copy");
    document.body.removeChild(ta);
    return ok;
  } catch {
    return false;
  }
}

function ErrorPanel({ error, onRestart, onExit, onReport, toast }: { error: EmuError; onRestart: () => void; onExit: () => void; onReport: () => void; toast: (msg: string, kind?: "ok" | "err") => void }) {
  const [copied, setCopied] = useState(false);
  const phaseLabel = error.phase === "load" ? "로드 중 오류" : "게임 실행 중 오류";
  const btn = "rounded-md border border-red-500/60 bg-red-500/10 px-3 py-1.5 text-xs font-medium hover:bg-red-500/20 focus-visible:outline focus-visible:outline-2 focus-visible:outline-red-500";

  const onCopy = async () => {
    const ok = await copyToClipboard(buildErrorReport(error));
    setCopied(ok);
    toast(ok ? "에러 상세를 복사했습니다 (게임 정보 미포함)" : "복사에 실패했습니다", ok ? "ok" : "err");
    if (ok) window.setTimeout(() => setCopied(false), 2500);
  };

  return (
    <div role="alert" className="m-2 rounded-lg border border-red-500 bg-red-500/10 p-4 text-sm text-red-700 dark:text-red-200">
      <div className="flex items-start gap-2">
        <span aria-hidden="true">⚠</span>
        <div className="min-w-0 flex-1">
          <p className="font-semibold break-words">
            {error.name}: {error.message}
          </p>
          <p className="mt-1 text-xs opacity-80">
            {phaseLabel}
            {error.platformKind ? ` · 플랫폼 ${error.platformKind}` : ""} · {envSummary()}
          </p>
          {error.stack && (
            <details className="mt-2">
              <summary className="cursor-pointer text-xs underline">스택 트레이스 보기</summary>
              <pre className="mt-1 max-h-48 overflow-auto whitespace-pre-wrap break-words rounded bg-black/20 p-2 text-[11px] leading-snug">{error.stack}</pre>
            </details>
          )}
          <div className="mt-3 flex flex-wrap gap-2">
            <button type="button" onClick={onReport} className="rounded-md border border-accent bg-accent px-3 py-1.5 text-xs font-medium text-accent-fg hover:bg-accent-hover focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent">
              문의·제보하기
            </button>
            <button type="button" onClick={() => void onCopy()} className={btn}>
              {copied ? "복사됨 ✓" : "에러 상세 복사"}
            </button>
            <button type="button" onClick={onRestart} className={btn}>다시 시작</button>
            <button type="button" onClick={onExit} className={btn}>나가기</button>
          </div>
        </div>
      </div>
    </div>
  );
}
