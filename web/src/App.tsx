import { useCallback, useEffect, useState } from "react";
import { useAuth, type AuthState } from "./hooks/useAuth";
import { installAudioUnlock } from "./lib/audio";
import { useTheme } from "./hooks/useTheme";
import { GameLibrary } from "./components/GameLibrary";
import { Player } from "./components/Player";
import { AuthPanel } from "./components/AuthPanel";
import { CloudSaves } from "./components/CloudSaves";
import { InquiryForm } from "./components/InquiryForm";
import { Help } from "./components/Help";
import { Overlay } from "./components/Overlay";
import type { LoadableGame } from "./lib/emulator";

type View = "library" | "cloud" | "inquiry" | "account" | "help";
type Toast = { msg: string; kind: "ok" | "err" | "" } | null;

const TABS: { id: View; label: string }[] = [
  { id: "library", label: "라이브러리" },
  { id: "cloud", label: "세이브 동기화" },
  { id: "inquiry", label: "문의·건의" },
  { id: "account", label: "계정" },
  { id: "help", label: "도움말" },
];

// Maps a view id to its panel. Shared by the full-page nav (not playing) and the
// slide-over overlay (while playing) so a running game is never torn down.
function ViewPanel({
  view,
  authState,
  onRun,
  toast,
  onReport,
}: {
  view: View;
  authState: AuthState;
  onRun: (g: LoadableGame) => void;
  toast: (m: string, k?: "ok" | "err") => void;
  onReport: () => void;
}) {
  switch (view) {
    case "library":
      return <GameLibrary onRun={onRun} toast={toast} user={authState.user} onReport={onReport} />;
    case "cloud":
      return <CloudSaves user={authState.user} toast={toast} />;
    case "inquiry":
      return <InquiryForm user={authState.user} toast={toast} />;
    case "account":
      return <AuthPanel authState={authState} toast={toast} />;
    case "help":
      return <Help />;
  }
}

export default function App() {
  const authState = useAuth();
  const { theme, toggle: toggleTheme } = useTheme();
  const [view, setView] = useState<View>("library");
  const [running, setRunning] = useState<LoadableGame | null>(null);
  const [overlay, setOverlay] = useState<View | "menu" | null>(null); // shown on top of the player
  const [toast, setToast] = useState<Toast>(null);

  // Install the global audio-unlock listeners once, before any game-launch tap,
  // so the first user gesture creates+resumes+unlocks the shared AudioContext
  // synchronously (required by iOS WebKit).
  useEffect(() => installAudioUnlock(), []);

  const showToast = useCallback((msg: string, kind: "ok" | "err" | "" = "") => {
    setToast({ msg, kind });
    window.setTimeout(() => setToast(null), 3200);
  }, []);

  const onRun = useCallback((game: LoadableGame) => {
    setRunning(game); // starting (or switching) a game
    setOverlay(null);
  }, []);

  const exitPlayer = useCallback(() => {
    setRunning(null);
    setOverlay(null);
  }, []);

  const tabLabel = (id: View) => TABS.find((t) => t.id === id)?.label ?? "";

  // Guide the user from a rejected upload to the inquiry form (or login first).
  const reportTarget: View = authState.user ? "inquiry" : "account";
  const onReport = useCallback(() => {
    if (running) setOverlay(reportTarget);
    else setView(reportTarget);
  }, [running, reportTarget]);

  // ── Playing: player stays mounted; tabs open as overlays (game keeps running) ─
  if (running) {
    return (
      <>
        <Player game={running} user={authState.user} onExit={exitPlayer} toast={showToast} onMenu={() => setOverlay("menu")} />

        {overlay === "menu" && (
          <Overlay title="메뉴" onClose={() => setOverlay(null)}>
            <div className="w-full max-w-sm flex flex-col gap-2">
              <p className="text-sm text-fg-dim">게임은 계속 실행 중입니다. 항목을 보고 닫으면 그대로 이어집니다.</p>
              {TABS.map((t) => (
                <button
                  key={t.id}
                  type="button"
                  onClick={() => setOverlay(t.id)}
                  className="rounded-md border border-edge bg-surface2 px-4 py-3 text-left text-fg hover:border-accent focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
                >
                  {t.label}
                </button>
              ))}
            </div>
          </Overlay>
        )}

        {overlay && overlay !== "menu" && (
          <Overlay title={tabLabel(overlay)} onClose={() => setOverlay(null)}>
            <ViewPanel view={overlay} authState={authState} onRun={onRun} toast={showToast} onReport={onReport} />
          </Overlay>
        )}

        {toast && <ToastView toast={toast} />}
      </>
    );
  }

  // ── Not playing: normal header + full-page view ───────────────────────────────
  return (
    <div className="min-h-full flex flex-col">
      <header className="sticky top-0 z-10 flex items-center gap-3 border-b border-edge bg-surface/85 px-4 py-2 backdrop-blur">
        <div className="font-extrabold tracking-wide text-fg">
          WIE<span className="font-normal text-fg-dim">/web</span>
        </div>
        <nav className="flex flex-1 gap-1 overflow-x-auto" aria-label="주요 메뉴">
          {TABS.map((t) => (
            <button
              key={t.id}
              type="button"
              aria-current={view === t.id ? "page" : undefined}
              onClick={() => setView(t.id)}
              className={
                "shrink-0 rounded-md px-2.5 py-1.5 text-sm transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent " +
                (view === t.id ? "bg-surface2 text-fg" : "text-fg-dim hover:text-fg")
              }
            >
              {t.label}
            </button>
          ))}
        </nav>
        <button
          type="button"
          onClick={toggleTheme}
          aria-label={theme === "dark" ? "라이트 모드로 전환" : "다크 모드로 전환"}
          title={theme === "dark" ? "라이트 모드" : "다크 모드"}
          className="rounded-md border border-edge bg-surface2 px-2 py-1 text-sm text-fg-dim hover:text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
        >
          {theme === "dark" ? "☀️" : "🌙"}
        </button>
        <button
          type="button"
          onClick={() => setView("account")}
          aria-current={view === "account" ? "page" : undefined}
          title={authState.user ? "계정" : "로그인 / 가입"}
          className="rounded-full border border-edge bg-surface2 px-2.5 py-1 text-xs text-fg-dim hover:text-fg hover:border-accent focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
        >
          {authState.user ? `@${authState.user.login_id}` : "로그인 안 됨"}
        </button>
      </header>

      <main className="flex flex-1 flex-col items-center gap-5 px-4 py-5">
        {view === "library" && (
          <div className="w-full max-w-xl rounded-lg border border-emerald-500/40 bg-emerald-500/10 px-4 py-3 text-sm text-emerald-700 dark:text-emerald-200">
            🔒 게임 파일은 <strong>이 기기에만</strong> 저장됩니다. 업로드한 게임의 바이트·파일명·해시·“보유 목록”은 서버로 전송되지 않으며
            브라우저(IndexedDB)에만 보관됩니다. 서버에 올라가는 것은 <em>계정 정보</em>와 <em>세이브 데이터</em>뿐입니다.
          </div>
        )}
        <ViewPanel view={view} authState={authState} onRun={onRun} toast={showToast} onReport={onReport} />
      </main>

      <footer className="mt-auto px-4 py-6 text-center text-xs text-fg-dim">
        <p>이 프로젝트는 디지털 보존 및 교육·연구 목적의 비영리 서비스입니다 (digital preservation / educational research).</p>
        <p className="mt-1">
          에뮬레이터 코어: MIT 라이선스, © 2020 Inseok Lee ·{" "}
          <a className="underline hover:text-fg" href="https://github.com/dlunch/wie" target="_blank" rel="noreferrer noopener">
            upstream: dlunch/wie
          </a>
        </p>
      </footer>

      {toast && <ToastView toast={toast} />}
    </div>
  );
}

function ToastView({ toast }: { toast: NonNullable<Toast> }) {
  return (
    <div
      role="status"
      aria-live="polite"
      className={
        "fixed bottom-5 left-1/2 z-50 -translate-x-1/2 rounded-lg border px-4 py-2 text-sm shadow-lg " +
        (toast.kind === "err"
          ? "border-red-500 bg-red-500/15 text-red-700 dark:text-red-200"
          : toast.kind === "ok"
            ? "border-emerald-500 bg-emerald-500/15 text-emerald-700 dark:text-emerald-200"
            : "border-edge bg-surface2 text-fg")
      }
    >
      {toast.msg}
    </div>
  );
}
