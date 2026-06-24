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

export function Player({ game, user, onExit, onMenu, toast }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sessionRef = useRef<EmulatorSession | null>(null);
  const keymapRef = useRef<Record<string, EmuKey>>(loadKeymap());
  const { theme, toggle: toggleTheme } = useTheme();
  const [status, setStatus] = useState<"loading" | "running" | "error">("loading");
  const [error, setError] = useState("");
  const [showRemap, setShowRemap] = useState(false);
  const [muted, setMuted] = useState(false);

  // Boot the session once the canvas is mounted. This effect is keyed on `game`,
  // so it runs ONCE per game and is NOT disturbed by opening overlays elsewhere
  // (the player stays mounted) — switching tabs never ends the game.
  useEffect(() => {
    let cancelled = false;
    const session = new EmulatorSession();
    sessionRef.current = session;
    session.onError = (msg) => {
      if (cancelled) return;
      setError(msg);
      setStatus("error");
    };

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

  // Keyboard input while running.
  useEffect(() => {
    if (status !== "running") return;
    const down = (e: KeyboardEvent) => {
      const k = keymapRef.current[e.code];
      if (!k) return;
      e.preventDefault();
      if (e.repeat) sessionRef.current?.keyRepeat(k);
      else sessionRef.current?.keyDown(k);
    };
    const up = (e: KeyboardEvent) => {
      const k = keymapRef.current[e.code];
      if (!k) return;
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

  const press = useCallback((k: EmuKey) => sessionRef.current?.keyDown(k), []);
  const release = useCallback((k: EmuKey) => sessionRef.current?.keyUp(k), []);

  const toggleMute = useCallback(() => {
    setMuted((m) => {
      const next = !m;
      sessionRef.current?.setMuted(next);
      return next;
    });
  }, []);

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

  const iconBtn = "rounded-md bg-surface2 border border-edge px-2.5 py-1.5 text-sm text-fg-dim hover:text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent";

  return (
    <section className="fixed inset-0 z-20 flex flex-col bg-surface">
      {/* compact top bar — keeps the canvas unobstructed */}
      <div className="flex items-center gap-2 border-b border-edge px-3 py-1.5">
        <button type="button" onClick={onExit} className={iconBtn} aria-label="게임 종료">
          ← 종료
        </button>
        <span className="flex-1 truncate text-sm font-medium text-fg">{game.name}</span>
        <button type="button" onClick={onMenu} className={iconBtn} aria-label="메뉴 (라이브러리·세이브·문의·계정·도움말)" title="메뉴">
          ☰
        </button>
        <button type="button" onClick={toggleMute} aria-pressed={muted} aria-label={muted ? "음소거 해제" : "음소거"} title={muted ? "음소거 해제" : "음소거"} className={iconBtn}>
          {muted ? "🔇" : "🔊"}
        </button>
        <button type="button" onClick={toggleTheme} aria-label={theme === "dark" ? "라이트 모드" : "다크 모드"} title={theme === "dark" ? "라이트 모드" : "다크 모드"} className={iconBtn}>
          {theme === "dark" ? "☀️" : "🌙"}
        </button>
      </div>

      {error && (
        <div className="m-2 rounded-lg border border-red-500 bg-red-500/10 px-4 py-3 text-sm text-red-700 dark:text-red-200 whitespace-pre-wrap">⚠ {error}</div>
      )}

      {/* play area — canvas + pad coexist; column in portrait, row in landscape */}
      <div className="flex min-h-0 flex-1 flex-col items-center justify-start gap-2 overflow-y-auto p-2 landscape:flex-row landscape:items-center landscape:justify-center">
        <canvas
          ref={canvasRef}
          width={240}
          height={320}
          data-testid="screen"
          role="img"
          aria-label={`${game.name} 게임 화면`}
          className="emulator-canvas shrink-0 rounded-md border border-edge max-h-[46vh] landscape:max-h-[86vh] w-auto"
          style={{ aspectRatio: "240 / 320" }}
        />
        {status === "running" && (
          <div className="w-full max-w-md shrink-0 landscape:w-auto">
            <VirtualPad onPress={press} onRelease={release} />
          </div>
        )}
      </div>

      {/* bottom controls */}
      {status === "running" && (
        <div className="flex flex-wrap items-center justify-center gap-2 border-t border-edge px-3 py-1.5">
          <button type="button" onClick={() => void saveLocal()} className={iconBtn}>세이브 저장</button>
          <button type="button" onClick={() => void syncCloud()} className={iconBtn}>클라우드 업로드</button>
          <button type="button" onClick={() => setShowRemap(true)} className={iconBtn}>⌨ 키 설정</button>
        </div>
      )}

      {showRemap && (
        <Overlay title="키 설정" onClose={() => setShowRemap(false)}>
          <KeyRemap onChange={(m) => (keymapRef.current = m)} />
        </Overlay>
      )}
    </section>
  );
}
