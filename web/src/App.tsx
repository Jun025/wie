import { useCallback, useEffect, useRef, useState } from "react";
import { EmulatorSession } from "./lib/emulator";
import { type EmuKey, loadKeymap } from "./lib/keymap";
import { VirtualPad } from "./components/VirtualPad";
import { KeyRemap } from "./components/KeyRemap";

type Status = "idle" | "loading" | "running" | "error";

export default function App() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sessionRef = useRef<EmulatorSession | null>(null);
  const keymapRef = useRef<Record<string, EmuKey>>(loadKeymap());

  const [status, setStatus] = useState<Status>("idle");
  const [error, setError] = useState<string>("");
  const [fileName, setFileName] = useState<string>("");
  const [showRemap, setShowRemap] = useState(false);

  // Forward keyboard input while a game is running.
  useEffect(() => {
    if (status !== "running") return;
    const down = (e: KeyboardEvent) => {
      const emuKey = keymapRef.current[e.code];
      if (!emuKey) return;
      e.preventDefault();
      if (e.repeat) sessionRef.current?.keyRepeat(emuKey);
      else sessionRef.current?.keyDown(emuKey);
    };
    const up = (e: KeyboardEvent) => {
      const emuKey = keymapRef.current[e.code];
      if (!emuKey) return;
      e.preventDefault();
      sessionRef.current?.keyUp(emuKey);
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
    };
  }, [status]);

  // Persist saves when the tab is hidden.
  useEffect(() => {
    const onHide = () => {
      if (document.visibilityState === "hidden") void sessionRef.current?.persist();
    };
    document.addEventListener("visibilitychange", onHide);
    return () => document.removeEventListener("visibilitychange", onHide);
  }, []);

  const handleFile = useCallback(async (file: File) => {
    setError("");
    setStatus("loading");
    setFileName(file.name);

    // Tear down any previous session.
    sessionRef.current?.stop();

    // AudioContext must be created from this user gesture (autoplay policy).
    let audioCtx: AudioContext | null = null;
    try {
      audioCtx = new AudioContext();
      await audioCtx.resume();
    } catch {
      audioCtx = null;
    }

    // Wait a tick so the canvas is mounted.
    await new Promise((r) => requestAnimationFrame(r));
    const canvas = canvasRef.current;
    if (!canvas) {
      setError("캔버스를 찾을 수 없습니다.");
      setStatus("error");
      return;
    }

    const session = new EmulatorSession();
    session.onError = (msg) => {
      setError(msg);
      setStatus("error");
    };
    sessionRef.current = session;

    try {
      await session.start(file, canvas, audioCtx);
      setStatus("running");
    } catch (e) {
      setError(typeof e === "string" ? e : (e as Error)?.message ?? "로드 실패");
      setStatus("error");
    }
  }, []);

  const onInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) void handleFile(file);
  };

  const press = useCallback((key: EmuKey) => sessionRef.current?.keyDown(key), []);
  const release = useCallback((key: EmuKey) => sessionRef.current?.keyUp(key), []);

  return (
    <div className="min-h-full flex flex-col items-center px-4 py-6 gap-6">
      <header className="text-center">
        <h1 className="text-2xl font-bold text-slate-100">WIE — 브라우저 피처폰 에뮬레이터</h1>
        <p className="text-sm text-slate-400 mt-1">WIPI · SKVM · J2ME 앱을 브라우저에서 실행합니다.</p>
      </header>

      {/* Privacy notice — always visible */}
      <div className="w-full max-w-xl rounded-lg border border-emerald-700/50 bg-emerald-900/20 px-4 py-3 text-sm text-emerald-200">
        🔒 게임 파일은 <strong>당신의 브라우저에서만</strong> 처리되며 서버로 전송되지 않습니다. 업로드한 파일은 메모리 안에서만 실행되고, 세이브
        데이터만 이 브라우저(IndexedDB)에 저장됩니다.
      </div>

      {/* Upload */}
      <label className="w-full max-w-xl cursor-pointer rounded-lg border-2 border-dashed border-slate-600 bg-slate-800/40 px-4 py-6 text-center hover:border-sky-500 transition-colors">
        <input type="file" accept=".jar,.zip,.kjx,.jad" className="hidden" onChange={onInputChange} />
        <div className="text-slate-200 font-medium">게임 파일 선택 (.jar / .zip)</div>
        <div className="text-xs text-slate-500 mt-1">{fileName ? `선택됨: ${fileName}` : "클릭하여 로컬 파일을 선택하세요"}</div>
      </label>

      {status === "error" && (
        <div className="w-full max-w-xl rounded-lg border border-red-700/50 bg-red-900/20 px-4 py-3 text-sm text-red-200 whitespace-pre-wrap">
          ⚠ {error}
        </div>
      )}

      {/* Screen */}
      {(status === "running" || status === "loading") && (
        <canvas
          ref={canvasRef}
          width={240}
          height={320}
          className="emulator-canvas rounded-md border border-slate-700"
          style={{ width: "min(90vw, 360px)", aspectRatio: "240 / 320" }}
        />
      )}

      {/* Controls */}
      {status === "running" && (
        <>
          <VirtualPad onPress={press} onRelease={release} />
          <div className="w-full max-w-md">
            <button
              type="button"
              className="text-xs px-3 py-1 rounded bg-slate-700 hover:bg-slate-600 text-slate-200"
              onClick={() => setShowRemap((v) => !v)}
            >
              {showRemap ? "키 설정 닫기" : "⌨ 키 설정"}
            </button>
            {showRemap && (
              <div className="mt-3 rounded-lg bg-slate-900/60 border border-slate-700 p-4">
                <KeyRemap onChange={(m) => (keymapRef.current = m)} />
              </div>
            )}
          </div>
        </>
      )}

      <footer className="mt-auto pt-8 text-center text-xs text-slate-500 max-w-xl">
        <p>
          이 프로젝트는 디지털 보존 및 교육·연구 목적의 비영리 서비스입니다 (digital preservation / educational research).
        </p>
        <p className="mt-1">
          에뮬레이터 코어: MIT 라이선스, © 2020 Inseok Lee ·{" "}
          <a className="underline hover:text-slate-300" href="https://github.com/dlunch/wie" target="_blank" rel="noreferrer noopener">
            upstream: dlunch/wie
          </a>
        </p>
      </footer>
    </div>
  );
}
