import { useCallback, useEffect, useRef, useState } from "react";
import { EmulatorSession, type LoadableGame } from "../lib/emulator";
import { type EmuKey, loadKeymap } from "../lib/keymap";
import { VirtualPad } from "./VirtualPad";
import { KeyRemap } from "./KeyRemap";
import { autosaveLocal, deviceName, pushToCloud } from "../lib/saveSync";
import type { User } from "../lib/api";

interface Props {
  game: LoadableGame;
  user: User | null;
  onExit: () => void;
  toast: (msg: string, kind?: "ok" | "err") => void;
}

export function Player({ game, user, onExit, toast }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sessionRef = useRef<EmulatorSession | null>(null);
  const keymapRef = useRef<Record<string, EmuKey>>(loadKeymap());
  const [status, setStatus] = useState<"loading" | "running" | "error">("loading");
  const [error, setError] = useState("");
  const [showRemap, setShowRemap] = useState(false);

  // Boot the session once the canvas is mounted.
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

  return (
    <section className="w-full max-w-xl flex flex-col items-center gap-4">
      <div className="w-full flex items-center justify-between">
        <h2 className="text-base font-semibold text-slate-100 truncate">{game.name}</h2>
        <button type="button" onClick={onExit} className="rounded-md bg-slate-700 hover:bg-slate-600 px-3 py-1.5 text-sm text-slate-200">
          ← 라이브러리
        </button>
      </div>

      {status === "error" && (
        <div className="w-full rounded-lg border border-red-700/50 bg-red-900/20 px-4 py-3 text-sm text-red-200 whitespace-pre-wrap">⚠ {error}</div>
      )}

      <canvas
        ref={canvasRef}
        width={240}
        height={320}
        data-testid="screen"
        className="emulator-canvas rounded-md border border-slate-700"
        style={{ width: "min(90vw, 360px)", aspectRatio: "240 / 320" }}
      />

      {status === "running" && (
        <>
          <VirtualPad onPress={press} onRelease={release} />
          <div className="w-full max-w-md flex flex-wrap items-center gap-2">
            <button type="button" onClick={() => void saveLocal()} className="rounded bg-slate-700 hover:bg-slate-600 px-3 py-1.5 text-sm text-slate-200">
              세이브 저장(로컬)
            </button>
            <button type="button" onClick={() => void syncCloud()} className="rounded bg-slate-700 hover:bg-slate-600 px-3 py-1.5 text-sm text-slate-200">
              클라우드 업로드
            </button>
            <button type="button" onClick={() => setShowRemap((v) => !v)} className="rounded bg-slate-700 hover:bg-slate-600 px-3 py-1.5 text-sm text-slate-200">
              {showRemap ? "키 설정 닫기" : "⌨ 키 설정"}
            </button>
          </div>
          {showRemap && (
            <div className="w-full max-w-md rounded-lg bg-slate-900/60 border border-slate-700 p-4">
              <KeyRemap onChange={(m) => (keymapRef.current = m)} />
            </div>
          )}
          <p className="text-xs text-slate-500 text-center">키보드: 방향키 = D-pad · Enter/Space = OK · Shift = 소프트키 · 숫자키 = 키패드</p>
        </>
      )}
    </section>
  );
}
