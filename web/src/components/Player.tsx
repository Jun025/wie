import { useCallback, useEffect, useRef, useState } from "react";
import { EmulatorSession, type LoadableGame } from "../lib/emulator";
import { type EmuKey, loadKeymap } from "../lib/keymap";
import { GameButton } from "./GameButton";
import { KeyRemap } from "./KeyRemap";
import { Overlay } from "./Overlay";
import { autosaveLocal, deviceName, pushToCloud } from "../lib/saveSync";
import { useTheme } from "../hooks/useTheme";
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
  const [error, setError] = useState("");
  const [showRemap, setShowRemap] = useState(false);
  const [volume, setVolume] = useState(0.8);
  const [muted, setMuted] = useState(false);
  const [keyset, setKeyset] = useState<Keyset>(loadKeyset);

  // Boot once per game; opening overlays / switching keyset does NOT remount the
  // player, so the game keeps running.
  useEffect(() => {
    let cancelled = false;
    const session = new EmulatorSession();
    sessionRef.current = session;
    session.onError = (msg) => {
      if (cancelled) return;
      setError(msg);
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
      let audioCtx: AudioContext | null = null;
      try {
        audioCtx = new AudioContext();
        await audioCtx.resume();
      } catch {
        audioCtx = null;
      }
      await new Promise((r) => requestAnimationFrame(r));
      const canvas = canvasRef.current;
      if (!canvas || cancelled) return;
      try {
        await session.start(game, canvas, audioCtx);
        if (!cancelled) setStatus("running");
      } catch (e) {
        if (!cancelled) {
          setError(typeof e === "string" ? e : (e as Error)?.message ?? "로드 실패");
          setStatus("error");
        }
      }
    })();

    return () => {
      cancelled = true;
      void session.persist();
      session.stop();
    };
  }, [game]);

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
      {error && (
        <div className="m-2 rounded-lg border border-red-500 bg-red-500/10 px-4 py-3 text-sm text-red-700 dark:text-red-200 whitespace-pre-wrap">⚠ {error}</div>
      )}

      {/* display + side rails (no bottom bar; no top header) */}
      <div className="flex shrink-0 items-stretch justify-center gap-1.5 p-1.5">
        {/* LEFT rail: 나가기 · 음량(긴) · L · 통화 · 동기화 · 키세트 */}
        <div className="flex flex-col items-center justify-center gap-1.5">
          <button type="button" onClick={onExit} className={svc} aria-label="게임 나가기" title="나가기">←</button>
          {/* tall vertical volume area (~2 buttons high) */}
          <div className="flex flex-col items-center gap-1 rounded-md border border-edge bg-surface2 px-1 py-1.5">
            <button type="button" onClick={() => sessionRef.current?.toggleMute()} aria-pressed={muted} aria-label={muted ? "음소거 해제" : "음소거"} title={muted ? "음소거 해제" : "음소거"} className="text-sm">
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
          <GameButton label="◀L" title="왼쪽 소프트키" {...gkey("LEFT_SOFT_KEY")} className={side} />
          <GameButton label="📞" title="통화" {...gkey("CALL")} className={`${side} bg-green-600 text-white border-green-700`} />
          <button type="button" onClick={() => void syncCloud()} className={svc} aria-label="세이브 동기화(클라우드 업로드)" title="동기화">☁ 동기화</button>
          <button type="button" onClick={cycleKeyset} className={svc} aria-label={`키보드 세트 모드 ${keyset} — 눌러서 전환`} title="키세트 모드 전환">🎛 모드{keyset}</button>
        </div>

        <canvas
          ref={canvasRef}
          width={240}
          height={320}
          data-testid="screen"
          role="img"
          aria-label={`${game.name} 게임 화면`}
          className="emulator-canvas self-center rounded-md border border-edge max-h-[58vh] landscape:max-h-[80vh] w-auto"
          style={{ aspectRatio: "240 / 320" }}
        />

        {/* RIGHT rail: 키설정 · 테마 · 메뉴 · R · 종료 · CLR */}
        <div className="flex flex-col items-center justify-center gap-1.5">
          <button type="button" onClick={() => setShowRemap(true)} className={svc} aria-label="키 설정(리매핑)" title="키 설정">⌨</button>
          <button type="button" onClick={toggleTheme} className={svc} aria-label={theme === "dark" ? "라이트 모드" : "다크 모드"} title={theme === "dark" ? "라이트 모드" : "다크 모드"}>
            {theme === "dark" ? "☀️" : "🌙"}
          </button>
          <button type="button" onClick={onMenu} className={svc} aria-label="메뉴 (라이브러리·세이브·문의·계정·도움말)" title="메뉴">☰</button>
          <GameButton label="R▶" title="오른쪽 소프트키" {...gkey("RIGHT_SOFT_KEY")} className={side} />
          <GameButton label="⛔" title="종료(게임 키)" {...gkey("HANGUP")} className={`${side} bg-red-600 text-white border-red-700`} />
          <GameButton label="CLR" title="지우기" repeat {...gkey("CLEAR")} className={`${side} text-xs`} />
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
