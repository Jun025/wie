import { useCallback, useEffect, useRef, useState } from "react";
import { EmulatorSession, type EmuError, type LoadableGame, normalizeError } from "../lib/emulator";
import { type EmuKey, loadKeymap } from "../lib/keymap";
import { GameButton } from "./GameButton";
import { KeyRemap } from "./KeyRemap";
import { Overlay } from "./Overlay";
import { autosaveLocal, deviceName, pushToCloud } from "../lib/saveSync";
import { useTheme } from "../hooks/useTheme";
import { audioState, getAudioContext, unlockAudio } from "../lib/audio";
import type { User } from "../lib/api";

interface Props {
  game: LoadableGame;
  user: User | null;
  onExit: () => void;
  onMenu: () => void;
  toast: (msg: string, kind?: "ok" | "err") => void;
}

// Keyset mode = onscreen control LAYOUT preset (NOT key remapping — that lives in
// keymap.ts/localStorage independently and still applies in every mode).
type Keyset = 1 | 2 | 3;
const KEYSET_KEY = "wie-keyset";
function loadKeyset(): Keyset {
  const v = Number(localStorage.getItem(KEYSET_KEY));
  return v === 2 ? 2 : v === 3 ? 3 : 1;
}

// Per-mode sizing for the two control blocks below the display.
//   1 = balanced · 2 = D-pad/OK emphasized · 3 = numpad emphasized
const MODE = {
  1: { dir: "h-12 w-12 text-lg", ok: "h-12 w-12 text-base", dgrid: "w-40", num: "h-10 text-base", ngrid: "w-36" },
  2: { dir: "h-16 w-16 text-2xl", ok: "h-16 w-16 text-lg", dgrid: "w-52", num: "h-7 text-xs", ngrid: "w-28" },
  3: { dir: "h-9 w-9 text-sm", ok: "h-9 w-9 text-xs", dgrid: "w-28", num: "h-12 text-xl", ngrid: "w-48" },
} as const;

export function Player({ game, user, onExit, onMenu, toast }: Props) {
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
        await session.start(game, canvas, audioCtx);
        if (!cancelled) {
          setStatus("running");
          // Diagnostic for headless/console verification of the unlock path.
          console.info("[wie audio]", audioState());
        }
      } catch (e) {
        if (!cancelled) {
          setError(normalizeError(e, "load", session.platformKind() ?? undefined));
          setStatus("error");
        }
      }
    })();

    return () => {
      cancelled = true;
      void session.persist();
      session.stop();
    };
  }, [game, bootNonce]);

  const restart = useCallback(() => {
    setError(null);
    setStatus("loading");
    setBootNonce((n) => n + 1);
  }, []);

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

  const syncCloud = useCallback(async () => {
    if (!user) return toast("로그인 후 업로드할 수 있습니다", "err");
    const blob = sessionRef.current?.exportBlob();
    if (blob) await autosaveLocal(game.hash, blob);
    const label = prompt("클라우드 슬롯 별칭 (게임 이름이 아닌 사용자 별칭):", "내 세이브 1");
    if (!label) return;
    try {
      await pushToCloud(game.hash, label, deviceName());
      toast("클라우드에 업로드됨 (세이브만)", "ok");
    } catch (e) {
      toast(`업로드 실패: ${(e as Error).message}`, "err");
    }
  }, [game.hash, user, toast]);

  const gkey = (k: EmuKey) => ({ onDown: () => press(k), onUp: () => release(k) });
  // service buttons (ghost/outline) — visually distinct from solid game keys.
  const svc = "shrink-0 rounded-md border border-edge bg-transparent px-2 py-1 text-xs text-fg-dim hover:text-fg hover:border-accent focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent";
  const side = "h-11 w-11 text-sm";
  const m = MODE[keyset];
  const pct = Math.round((muted ? 0 : volume) * 100);
  const volIcon = muted || pct === 0 ? "🔇" : pct < 50 ? "🔈" : "🔊";

  return (
    <section className="fixed inset-0 z-20 flex flex-col overflow-y-auto bg-surface">
      {error && <ErrorPanel error={error} onRestart={restart} onExit={onExit} toast={toast} />}

      {/* display + side rails (no bottom bar; no top header). flex-1 so the canvas
          fills the available vertical space; gap-1 trims the side margins.

          Both rails are 7-row grids of EQUAL height (items-stretch) with equal
          1fr rows, so a given row index sits at the same y on both sides. The
          game-key pairs are pinned to matching rows — L/R on row 5, 통화/종료 on
          row 6, 동기화/CLR on row 7 (bottom) — so they line up regardless of how
          many service buttons each rail carries. */}
      <div className="flex min-h-0 flex-1 items-stretch justify-center gap-1 p-1">
        {/* LEFT rail: 나가기 · 음량(콤팩트) · … · L · 통화 · 동기화 */}
        <div className="grid grid-rows-7 gap-1.5">
          <button type="button" onClick={onExit} className={`${svc} row-start-1 place-self-center`} aria-label="게임 나가기" title="나가기">←</button>

          {/* compact volume: one button-high. Tap to open a slider popover; the
              GainNode stays the single, device-local, real-time source of truth. */}
          <div className="relative row-start-2 place-self-center">
            <button
              type="button"
              onClick={() => setVolOpen((o) => !o)}
              aria-label={`음량 ${pct}% — 눌러서 조절`}
              aria-expanded={volOpen}
              title="음량"
              className={`${svc} flex h-11 w-11 flex-col items-center justify-center gap-0 px-0 py-0`}
            >
              <span className="text-sm leading-none">{volIcon}</span>
              <span className="text-[9px] leading-none tabular-nums">{pct}%</span>
            </button>
            {volOpen && (
              <>
                {/* click-away backdrop */}
                <div className="fixed inset-0 z-20" onClick={() => setVolOpen(false)} aria-hidden="true" />
                <div className="absolute left-full top-1/2 z-30 ml-1 flex -translate-y-1/2 flex-col items-center gap-2 rounded-md border border-edge bg-surface2 p-2 shadow-lg">
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

          <GameButton label="◀L" title="왼쪽 소프트키" {...gkey("LEFT_SOFT_KEY")} className={`${side} row-start-5 place-self-center`} />
          <GameButton label="📞" title="통화" {...gkey("CALL")} className={`${side} row-start-6 place-self-center bg-green-600 text-white border-green-700`} />
          <button type="button" onClick={() => void syncCloud()} className={`${svc} row-start-7 place-self-center`} aria-label="세이브 동기화(클라우드 업로드)" title="동기화">☁ 동기화</button>
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

        {/* RIGHT rail: 키세트 · 키설정 · 테마 · 메뉴 · R · 종료 · CLR */}
        <div className="grid grid-rows-7 gap-1.5">
          <button type="button" onClick={cycleKeyset} className={`${svc} row-start-1 place-self-center`} aria-label={`키보드 세트 모드 ${keyset} — 눌러서 전환`} title="키세트 모드 전환">🎛 모드{keyset}</button>
          <button type="button" onClick={() => setShowRemap(true)} className={`${svc} row-start-2 place-self-center`} aria-label="키 설정(리매핑)" title="키 설정">⌨</button>
          <button type="button" onClick={toggleTheme} className={`${svc} row-start-3 place-self-center`} aria-label={theme === "dark" ? "라이트 모드" : "다크 모드"} title={theme === "dark" ? "라이트 모드" : "다크 모드"}>
            {theme === "dark" ? "☀️" : "🌙"}
          </button>
          <button type="button" onClick={onMenu} className={`${svc} row-start-4 place-self-center`} aria-label="메뉴 (라이브러리·세이브·문의·계정·도움말)" title="메뉴">☰</button>
          <GameButton label="R▶" title="오른쪽 소프트키" {...gkey("RIGHT_SOFT_KEY")} className={`${side} row-start-5 place-self-center`} />
          <GameButton label="⛔" title="종료(게임 키)" {...gkey("HANGUP")} className={`${side} row-start-6 place-self-center bg-red-600 text-white border-red-700`} />
          <GameButton label="CLR" title="지우기" repeat {...gkey("CLEAR")} className={`${side} row-start-7 place-self-center text-xs`} />
        </div>
      </div>

      {/* below the display: left = D-pad + OK, right = numpad. Block weights shift
          with the keyset mode (the player never unmounts, so switching is instant
          and the game keeps running). */}
      {status === "running" && (
        <div className="flex shrink-0 items-center justify-center gap-3 p-1.5">
          <div className={`grid grid-cols-3 grid-rows-3 gap-1 ${m.dgrid}`}>
            <span />
            <GameButton label="▲" title="위" repeat {...gkey("UP")} className={`${m.dir} w-full`} />
            <span />
            <GameButton label="◀" title="왼쪽" repeat {...gkey("LEFT")} className={`${m.dir} w-full`} />
            <GameButton label="OK" title="확인" {...gkey("OK")} className={`${m.ok} w-full bg-accent text-accent-fg`} />
            <GameButton label="▶" title="오른쪽" repeat {...gkey("RIGHT")} className={`${m.dir} w-full`} />
            <span />
            <GameButton label="▼" title="아래" repeat {...gkey("DOWN")} className={`${m.dir} w-full`} />
            <span />
          </div>
          <div className={`grid grid-cols-3 gap-1 ${m.ngrid}`}>
            {(["NUM1", "NUM2", "NUM3", "NUM4", "NUM5", "NUM6", "NUM7", "NUM8", "NUM9"] as EmuKey[]).map((k) => (
              <GameButton key={k} label={k.replace("NUM", "")} title={k.replace("NUM", "")} repeat {...gkey(k)} className={`${m.num} w-full`} />
            ))}
            <GameButton label="✳" title="별표" {...gkey("STAR")} className={`${m.num} w-full`} />
            <GameButton label="0" title="0" repeat {...gkey("NUM0")} className={`${m.num} w-full`} />
            <GameButton label="#" title="우물정" {...gkey("HASH")} className={`${m.num} w-full`} />
          </div>
        </div>
      )}

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

function ErrorPanel({ error, onRestart, onExit, toast }: { error: EmuError; onRestart: () => void; onExit: () => void; toast: (msg: string, kind?: "ok" | "err") => void }) {
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
