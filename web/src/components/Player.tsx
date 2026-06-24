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

  // Boot once per game; opening overlays does NOT remount the player, so the
  // game keeps running while other tabs are shown.
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

  // Keyboard input. Volume keys drive OUR output volume (gauge); the rest go to
  // the game.
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

  // shared button sizings
  const sideBtn = "h-12 w-12 text-sm";
  const dirBtn = "h-12 w-full text-lg";
  const numBtn = "h-10 w-full text-base";
  const barBtn = "shrink-0 rounded-md bg-surface2 border border-edge px-2.5 py-1.5 text-sm text-fg-dim hover:text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent";
  const pct = Math.round((muted ? 0 : volume) * 100);
  const volIcon = muted || pct === 0 ? "🔇" : pct < 50 ? "🔈" : "🔊";

  const gkey = (k: EmuKey) => ({ onDown: () => press(k), onUp: () => release(k) });

  return (
    <section className="fixed inset-0 z-20 flex flex-col bg-surface">
      {error && (
        <div className="m-2 rounded-lg border border-red-500 bg-red-500/10 px-4 py-3 text-sm text-red-700 dark:text-red-200 whitespace-pre-wrap">⚠ {error}</div>
      )}

      {/* play region: canvas centered, controls flanking. Portrait stacks
          (canvas → [left block | numpad]); landscape is a row
          ([left block] [L/CALL][canvas][R/HANGUP] [numpad]). */}
      <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-2 overflow-auto p-1.5 landscape:flex-row">
        {/* canvas with side rails: L+CALL (left), R+HANGUP (right) */}
        <div className="flex shrink-0 items-center justify-center gap-1.5 landscape:order-2">
          <div className="flex flex-col justify-center gap-2">
            <GameButton label="◀L" title="왼쪽 소프트키" {...gkey("LEFT_SOFT_KEY")} className={sideBtn} />
            <GameButton label="📞" title="통화" {...gkey("CALL")} className={`${sideBtn} bg-green-600 text-white border-green-700`} />
          </div>
          <canvas
            ref={canvasRef}
            width={240}
            height={320}
            data-testid="screen"
            role="img"
            aria-label={`${game.name} 게임 화면`}
            className="emulator-canvas rounded-md border border-edge max-h-[56vh] landscape:max-h-[92vh] w-auto"
            style={{ aspectRatio: "240 / 320" }}
          />
          <div className="flex flex-col justify-center gap-2">
            <GameButton label="R▶" title="오른쪽 소프트키" {...gkey("RIGHT_SOFT_KEY")} className={sideBtn} />
            <GameButton label="⛔" title="종료(게임 키)" {...gkey("HANGUP")} className={`${sideBtn} bg-red-600 text-white border-red-700`} />
          </div>
        </div>

        {/* controls under canvas (portrait) / flanking (landscape via display:contents) */}
        {status === "running" && (
          <div className="flex shrink-0 items-start justify-center gap-3 landscape:contents">
            {/* LEFT block: [CLR + 세이브] over [D-pad + OK] */}
            <div className="flex shrink-0 flex-col gap-1.5 landscape:order-1">
              <div className="flex gap-1.5">
                <GameButton label="CLR" title="지우기" repeat {...gkey("CLEAR")} className="h-11 flex-1 text-sm" />
                <button type="button" onClick={() => void saveLocal()} title="세이브 저장" aria-label="세이브 저장" className="h-11 rounded-xl border border-edge bg-surface2 px-3 text-base text-fg hover:border-accent">💾</button>
              </div>
              <div className="grid grid-cols-3 grid-rows-3 gap-1 w-40">
                <span />
                <GameButton label="▲" title="위" repeat {...gkey("UP")} className={dirBtn} />
                <span />
                <GameButton label="◀" title="왼쪽" repeat {...gkey("LEFT")} className={dirBtn} />
                <GameButton label="OK" title="확인" {...gkey("OK")} className="h-12 w-full bg-accent text-accent-fg text-base" />
                <GameButton label="▶" title="오른쪽" repeat {...gkey("RIGHT")} className={dirBtn} />
                <span />
                <GameButton label="▼" title="아래" repeat {...gkey("DOWN")} className={dirBtn} />
                <span />
              </div>
            </div>

            {/* RIGHT block: numeric keypad (compact so it all fits) */}
            <div className="grid shrink-0 grid-cols-3 gap-1 w-36 landscape:order-3">
              {(["NUM1", "NUM2", "NUM3", "NUM4", "NUM5", "NUM6", "NUM7", "NUM8", "NUM9"] as EmuKey[]).map((k) => (
                <GameButton key={k} label={k.replace("NUM", "")} title={k.replace("NUM", "")} repeat {...gkey(k)} className={numBtn} />
              ))}
              <GameButton label="✳" title="별표" {...gkey("STAR")} className={numBtn} />
              <GameButton label="0" title="0" repeat {...gkey("NUM0")} className={numBtn} />
              <GameButton label="#" title="우물정" {...gkey("HASH")} className={numBtn} />
            </div>
          </div>
        )}
      </div>

      {/* bottom bar: exit / settings / volume gauge / theme / menu (never on top).
          Icon-compact so it fits even on a ~380px portrait screen. */}
      <div className="flex shrink-0 items-center gap-1.5 border-t border-edge px-2 py-1.5 overflow-x-auto">
        <button type="button" onClick={onExit} className={barBtn} aria-label="게임 나가기" title="나가기">← 나가기</button>
        <button type="button" onClick={() => setShowRemap(true)} className={barBtn} aria-label="키 설정" title="키 설정">⌨</button>
        <button type="button" onClick={() => void syncCloud()} className={barBtn} aria-label="클라우드 업로드" title="클라우드 업로드">☁</button>
        <div className="flex shrink-0 items-center gap-1 rounded-md border border-edge bg-surface2 px-1.5 py-1">
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
            className="w-20 accent-accent"
          />
          <span className="hidden w-8 text-right text-xs tabular-nums text-fg-dim sm:inline">{pct}%</span>
        </div>
        <button type="button" onClick={toggleTheme} className={barBtn} aria-label={theme === "dark" ? "라이트 모드" : "다크 모드"} title={theme === "dark" ? "라이트 모드" : "다크 모드"}>
          {theme === "dark" ? "☀️" : "🌙"}
        </button>
        <button type="button" onClick={onMenu} className={barBtn} aria-label="메뉴 (라이브러리·세이브·문의·계정·도움말)" title="메뉴">☰</button>
      </div>

      {showRemap && (
        <Overlay title="키 설정" onClose={() => setShowRemap(false)}>
          <KeyRemap onChange={(m) => (keymapRef.current = m)} />
        </Overlay>
      )}
    </section>
  );
}
