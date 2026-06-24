import { useCallback, useEffect, useRef, useState } from "react";
import { EmulatorSession, type LoadableGame } from "../lib/emulator";
import { type EmuKey, loadKeymap } from "../lib/keymap";
import { VirtualPad } from "./VirtualPad";
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

const VOL_STEP = 0.08;

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

  // Boot the session once the canvas is mounted. Keyed on `game`, so it runs ONCE
  // per game and is NOT disturbed by opening overlays — switching tabs never ends
  // the game.
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

  // Route a logical key: VOLUME_UP/DOWN drive OUR output volume (so the gauge
  // moves), everything else goes to the game.
  const press = useCallback((k: EmuKey) => {
    if (k === "VOLUME_UP") sessionRef.current?.stepVolume(VOL_STEP);
    else if (k === "VOLUME_DOWN") sessionRef.current?.stepVolume(-VOL_STEP);
    else sessionRef.current?.keyDown(k);
  }, []);
  const release = useCallback((k: EmuKey) => {
    if (k !== "VOLUME_UP" && k !== "VOLUME_DOWN") sessionRef.current?.keyUp(k);
  }, []);

  // Keyboard input while running.
  useEffect(() => {
    if (status !== "running") return;
    const down = (e: KeyboardEvent) => {
      const k = keymapRef.current[e.code];
      if (!k) return;
      e.preventDefault();
      if (k === "VOLUME_UP" || k === "VOLUME_DOWN") {
        if (!e.repeat) press(k);
        return;
      }
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
  }, [status, press]);

  const saveLocal = useCallback(async () => {
    const blob = sessionRef.current?.exportBlob();
    if (!blob) return toast("저장할 세이브가 없습니다");
    await autosaveLocal(game.hash, blob);
    toast("세이브 로컬 저장됨", "ok");
  }, [game.hash, toast]);

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

  const railBtn = "rounded-md bg-surface2 border border-edge px-2.5 py-1.5 text-sm text-fg-dim hover:text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent";
  const pct = Math.round((muted ? 0 : volume) * 100);
  const volIcon = muted || pct === 0 ? "🔇" : pct < 50 ? "🔈" : "🔊";

  return (
    <section className="fixed inset-0 z-20 flex flex-col bg-surface landscape:flex-row">
      {/* control rail — top (portrait) / left side (landscape), leaving the canvas
          area free. game name lives in the menu, not over the screen. */}
      <div className="flex shrink-0 items-center gap-1.5 overflow-x-auto border-b border-edge px-2 py-1.5 landscape:flex-col landscape:items-center landscape:gap-2 landscape:overflow-x-visible landscape:overflow-y-auto landscape:border-b-0 landscape:border-r landscape:px-1.5">
        <button type="button" onClick={onExit} className={railBtn} aria-label="게임 종료" title="종료">← 종료</button>
        <button type="button" onClick={onMenu} className={railBtn} aria-label="메뉴 (라이브러리·세이브·문의·계정·도움말)" title="메뉴">☰</button>
        <div className="ml-auto flex items-center gap-1.5 landscape:ml-0 landscape:flex-col landscape:items-stretch">
          {/* volume gauge — single source of truth (session) */}
          <div className="flex items-center gap-1.5 rounded-md border border-edge bg-surface2 px-2 py-1">
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
              aria-valuenow={pct}
              className="w-20 accent-accent landscape:w-16"
            />
            <span className="w-8 text-right text-xs tabular-nums text-fg-dim">{pct}%</span>
          </div>
          <button type="button" onClick={toggleTheme} className={railBtn} aria-label={theme === "dark" ? "라이트 모드" : "다크 모드"} title={theme === "dark" ? "라이트 모드" : "다크 모드"}>
            {theme === "dark" ? "☀️" : "🌙"}
          </button>
        </div>
      </div>

      {error && (
        <div className="m-2 rounded-lg border border-red-500 bg-red-500/10 px-4 py-3 text-sm text-red-700 dark:text-red-200 whitespace-pre-wrap">⚠ {error}</div>
      )}

      {/* play area — canvas + pad coexist; column portrait, row landscape */}
      <div className="flex min-h-0 flex-1 flex-col items-center justify-start gap-2 overflow-y-auto p-2 landscape:flex-row landscape:items-center landscape:justify-center">
        <canvas
          ref={canvasRef}
          width={240}
          height={320}
          data-testid="screen"
          role="img"
          aria-label={`${game.name} 게임 화면`}
          className="emulator-canvas shrink-0 rounded-md border border-edge max-h-[52vh] landscape:max-h-[88vh] w-auto"
          style={{ aspectRatio: "240 / 320" }}
        />
        {status === "running" && (
          <div className="w-full max-w-md shrink-0 landscape:w-auto">
            <VirtualPad onPress={press} onRelease={release} />
            <div className="mt-2 flex flex-wrap items-center justify-center gap-2">
              <button type="button" onClick={() => void saveLocal()} className={railBtn}>세이브 저장</button>
              <button type="button" onClick={() => void syncCloud()} className={railBtn}>클라우드 업로드</button>
              <button type="button" onClick={() => setShowRemap(true)} className={railBtn}>⌨ 키 설정</button>
            </div>
          </div>
        )}
      </div>

      {showRemap && (
        <Overlay title="키 설정" onClose={() => setShowRemap(false)}>
          <KeyRemap onChange={(m) => (keymapRef.current = m)} />
        </Overlay>
      )}
    </section>
  );
}
