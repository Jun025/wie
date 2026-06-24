import { useCallback, useState } from "react";
import { useAuth } from "./hooks/useAuth";
import { useTheme } from "./hooks/useTheme";
import { GameLibrary } from "./components/GameLibrary";
import { Player } from "./components/Player";
import { AuthPanel } from "./components/AuthPanel";
import { CloudSaves } from "./components/CloudSaves";
import { InquiryForm } from "./components/InquiryForm";
import type { LoadableGame } from "./lib/emulator";

type View = "library" | "cloud" | "inquiry" | "account";
type Toast = { msg: string; kind: "ok" | "err" | "" } | null;

const TABS: { id: View; label: string }[] = [
  { id: "library", label: "라이브러리" },
  { id: "cloud", label: "세이브 동기화" },
  { id: "inquiry", label: "문의·건의" },
  { id: "account", label: "계정" },
];

export default function App() {
  const authState = useAuth();
  const { theme, toggle: toggleTheme } = useTheme();
  const [view, setView] = useState<View>("library");
  const [running, setRunning] = useState<LoadableGame | null>(null);
  const [toast, setToast] = useState<Toast>(null);

  const showToast = useCallback((msg: string, kind: "ok" | "err" | "" = "") => {
    setToast({ msg, kind });
    window.setTimeout(() => setToast(null), 3200);
  }, []);

  const onRun = useCallback((game: LoadableGame) => setRunning(game), []);
  const exitPlayer = useCallback(() => setRunning(null), []);

  return (
    <div className="min-h-full flex flex-col">
      <header className="sticky top-0 z-10 flex items-center gap-3 border-b border-edge bg-surface/85 px-4 py-2 backdrop-blur">
        <div className="font-extrabold tracking-wide text-fg">
          WIE<span className="font-normal text-fg-dim">/web</span>
        </div>
        <nav className="flex flex-1 gap-1" aria-label="주요 메뉴">
          {TABS.map((t) => (
            <button
              key={t.id}
              type="button"
              aria-current={view === t.id && !running ? "page" : undefined}
              onClick={() => {
                setRunning(null);
                setView(t.id);
              }}
              className={
                "rounded-md px-2.5 py-1.5 text-sm transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent " +
                (view === t.id && !running ? "bg-surface2 text-fg" : "text-fg-dim hover:text-fg")
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
        <div className="rounded-full border border-edge bg-surface2 px-2.5 py-1 text-xs text-fg-dim">
          {authState.user ? `@${authState.user.login_id}` : "로그인 안 됨"}
        </div>
      </header>

      <main className="flex flex-1 flex-col items-center gap-5 px-4 py-5">
        {!running && (
          <div className="w-full max-w-xl rounded-lg border border-emerald-500/40 bg-emerald-500/10 px-4 py-3 text-sm text-emerald-700 dark:text-emerald-200">
            🔒 게임 파일은 <strong>이 기기에만</strong> 저장됩니다. 업로드한 게임의 바이트·파일명·해시·“보유 목록”은 서버로 전송되지 않으며
            브라우저(IndexedDB)에만 보관됩니다. 서버에 올라가는 것은 <em>계정 정보</em>와 <em>세이브 데이터</em>뿐이며, 라이브러리는 로그인과
            무관하게 이 기기에서 동작합니다.
          </div>
        )}

        {running ? (
          <Player game={running} user={authState.user} onExit={exitPlayer} toast={showToast} />
        ) : view === "library" ? (
          <GameLibrary onRun={onRun} toast={showToast} />
        ) : view === "cloud" ? (
          <CloudSaves user={authState.user} toast={showToast} />
        ) : view === "inquiry" ? (
          <InquiryForm user={authState.user} toast={showToast} />
        ) : (
          <AuthPanel authState={authState} toast={showToast} />
        )}
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

      {toast && (
        <div
          role="status"
          aria-live="polite"
          className={
            "fixed bottom-5 left-1/2 -translate-x-1/2 rounded-lg border px-4 py-2 text-sm shadow-lg " +
            (toast.kind === "err"
              ? "border-red-500 bg-red-500/15 text-red-700 dark:text-red-200"
              : toast.kind === "ok"
                ? "border-emerald-500 bg-emerald-500/15 text-emerald-700 dark:text-emerald-200"
                : "border-edge bg-surface2 text-fg")
          }
        >
          {toast.msg}
        </div>
      )}
    </div>
  );
}
