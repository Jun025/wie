import { useCallback, useEffect, useRef, useState } from "react";
import { useAuth, type AuthState } from "./hooks/useAuth";
import { installAudioUnlock } from "./lib/audio";
import { useTheme } from "./hooks/useTheme";
import { GameLibrary } from "./components/GameLibrary";
import { Player } from "./components/Player";
import { AuthPanel } from "./components/AuthPanel";
import { CloudSaves } from "./components/CloudSaves";
import { InquiryForm } from "./components/InquiryForm";
import { Help } from "./components/Help";
import { ServiceInfo } from "./components/ServiceInfo";
import { Overlay } from "./components/Overlay";
import type { LoadableGame } from "./lib/emulator";
import type { CrashReport } from "./components/Player";
import * as lib from "./lib/library";
import { files as filesApi } from "./lib/api";
import { extOf } from "./lib/limits";
import { migrateLocalToServer } from "./lib/serverLibrary";
import { deviceName, pushSaveToServer, syncPendingSaves } from "./lib/saveSync";

type View = "library" | "cloud" | "inquiry" | "help" | "info";
type ToastKind = "ok" | "err" | "";
interface Toast {
  id: number;
  msg: string;
  kind: ToastKind;
}

// 계정(account) is intentionally NOT a tab anymore — it lives behind the
// top-right profile entry (opened as an overlay so the game never tears down).
const TABS: { id: View; label: string }[] = [
  { id: "library", label: "라이브러리" },
  { id: "cloud", label: "세이브 동기화" },
  { id: "inquiry", label: "문의·신고" },
  { id: "help", label: "도움말" },
  { id: "info", label: "서비스 정보" },
];

function ViewPanel({
  view,
  authState,
  onRun,
  toast,
  onReport,
  reloadKey,
  onInquireWithFiles,
  inquiryFileRefs,
  inquiryInitial,
}: {
  view: View;
  authState: AuthState;
  onRun: (g: LoadableGame) => void;
  toast: (m: string, k?: "ok" | "err") => void;
  onReport: () => void;
  reloadKey: number;
  onInquireWithFiles: (refs: { id: string; name: string }[]) => void;
  inquiryFileRefs: { id: string; name: string }[];
  inquiryInitial: { title?: string; body?: string };
}) {
  switch (view) {
    case "library":
      return <GameLibrary onRun={onRun} toast={toast} user={authState.user} onReport={onReport} reloadKey={reloadKey} onInquireWithFiles={onInquireWithFiles} />;
    case "cloud":
      return <CloudSaves user={authState.user} toast={toast} />;
    case "inquiry":
      return <InquiryForm user={authState.user} toast={toast} initialFileRefs={inquiryFileRefs} initialTitle={inquiryInitial.title} initialBody={inquiryInitial.body} />;
    case "help":
      return <Help />;
    case "info":
      return <ServiceInfo />;
  }
}

function ProfileButton({ user, onClick }: { user: AuthState["user"]; onClick: () => void }) {
  const initials = user ? user.email.slice(0, 2).toUpperCase() : "👤";
  return (
    <button
      type="button"
      onClick={onClick}
      title={user ? `계정 (${user.email})` : "로그인 / 가입"}
      aria-label={user ? `계정 ${user.email}` : "로그인 / 가입"}
      className="flex items-center gap-2 rounded-full border border-edge bg-surface2 py-1 pl-1 pr-2.5 text-xs text-fg-dim hover:text-fg hover:border-accent focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
    >
      <span className="flex h-6 w-6 items-center justify-center rounded-full bg-accent text-[11px] font-bold text-accent-fg">{initials}</span>
      <span className="max-w-[10rem] truncate">{user ? user.email : "로그인"}</span>
    </button>
  );
}

export default function App() {
  const authState = useAuth();
  const { theme, toggle: toggleTheme } = useTheme();
  const [view, setView] = useState<View>("library");
  const [running, setRunning] = useState<LoadableGame | null>(null);
  const [overlay, setOverlay] = useState<View | "menu" | null>(null); // panels shown over the player
  const [accountOpen, setAccountOpen] = useState(false); // profile/account overlay (any state)
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [resetToken, setResetToken] = useState<string | null>(null);
  const [libVersion, setLibVersion] = useState(0); // bump to make GameLibrary re-read after auto-upload
  const [inquiryFileRefs, setInquiryFileRefs] = useState<{ id: string; name: string }[]>([]); // 6번: vault files carried into 문의
  const [inquiryInitial, setInquiryInitial] = useState<{ title?: string; body?: string }>({}); // 5번: crash-report pre-fill
  const [pendingCrash, setPendingCrash] = useState<CrashReport | null>(null); // 5번: awaiting login before reporting
  const autoUploadedFor = useRef<string | null>(null);

  // Install the global audio-unlock listeners once (iOS WebKit needs the first
  // gesture to create+resume+unlock the shared AudioContext synchronously).
  useEffect(() => installAudioUnlock(), []);

  // A password-reset link (`/?reset=TOKEN`) opens the account overlay in reset mode.
  useEffect(() => {
    const t = new URLSearchParams(window.location.search).get("reset");
    if (t) {
      setResetToken(t);
      setAccountOpen(true);
    }
  }, []);

  // Unified toast system: one queue, one render location, opaque + kind-colored.
  // Consecutive duplicates are collapsed so a repeated message doesn't stack.
  const showToast = useCallback((msg: string, kind: ToastKind = "") => {
    setToasts((prev) => {
      if (prev.length && prev[prev.length - 1].msg === msg && prev[prev.length - 1].kind === kind) return prev;
      const id = (prev[prev.length - 1]?.id ?? 0) + 1;
      window.setTimeout(() => setToasts((cur) => cur.filter((t) => t.id !== id)), 3600);
      return [...prev.slice(-2), { id, msg, kind }]; // keep at most 3
    });
  }, []);

  // ── Auto-upload local ROMs to the server once on login (Task 1) ───────────────
  // On login (and the vault provisioned), push device-local ROMs to the user's
  // private server vault and free the local copy — but ONLY after the server's
  // confirmed ACK (200/duplicate). Quota/errors keep the file local (no loss).
  // migrateLocalToServer() already enforces that ACK-gated deletion + the save
  // merge runs from useAuth. After this completes a logged-in device is empty.
  useEffect(() => {
    const u = authState.user;
    if (!u || !authState.filesConfigured) {
      if (!u) autoUploadedFor.current = null; // allow re-run after a re-login
      return;
    }
    if (autoUploadedFor.current === u.id) return;
    autoUploadedFor.current = u.id;
    void (async () => {
      const games = await lib.listGames();
      if (games.length === 0) return;
      showToast(`이 기기 게임 ${games.length}개를 서버 보관함에 올리는 중…`);
      const r = await migrateLocalToServer();
      setLibVersion((v) => v + 1);
      const parts = [`업로드 ${r.uploaded}`, r.deduped ? `중복 ${r.deduped}` : null, r.failed ? `실패 ${r.failed}(로컬 보존)` : null].filter(Boolean);
      showToast(r.message ?? `서버 보관함에 자동 반영됨 — ${parts.join(" · ")}`, r.stopped || r.failed ? "err" : "ok");
    })();
  }, [authState.user, authState.filesConfigured, showToast]);

  // Retry pushing offline saves to the server when the network returns (logged-in
  // only). persistSnapshot already write-throughs on every autosave; this closes
  // the gap where a save was made while offline. Also runs once on mount to flush
  // anything left pending from a previous session.
  useEffect(() => {
    if (!authState.user) return;
    const flush = () => void syncPendingSaves(deviceName());
    window.addEventListener("online", flush);
    flush();
    return () => window.removeEventListener("online", flush);
  }, [authState.user]);

  const onRun = useCallback((game: LoadableGame) => {
    setRunning(game);
    setOverlay(null);
  }, []);

  const exitPlayer = useCallback(() => {
    setRunning(null);
    setOverlay(null);
  }, []);

  const tabLabel = (id: View) => TABS.find((t) => t.id === id)?.label ?? "";

  // A rejected upload routes the user to the inquiry form, logging in first if needed.
  const onReport = useCallback(() => {
    if (!authState.user) return setAccountOpen(true);
    if (running) setOverlay("inquiry");
    else setView("inquiry");
  }, [running, authState.user]);

  // 6번: carry selected vault files (by reference) into the 문의 form + switch to it.
  const onInquireWithFiles = useCallback(
    (refs: { id: string; name: string }[]) => {
      setInquiryFileRefs(refs);
      if (!authState.user) return setAccountOpen(true);
      if (running) setOverlay("inquiry");
      else setView("inquiry");
    },
    [running, authState.user],
  );

  // 5번: turn a crash into a ready-to-send 문의. Reference the crashing game from
  // the user's private vault (uploading it if it isn't there yet) and make sure the
  // crash-time save is on the server (keyed by the ROM hash) so it rides along —
  // then open the inquiry pre-filled with the error details + repro prompt.
  const resolveCrashReport = useCallback(
    async (payload: CrashReport) => {
      const { game, title, body } = payload;
      let fileRef: { id: string; name: string } | null = null;
      try {
        const list = await filesApi.list();
        if (list.enabled) {
          const existing = list.files.find((f) => f.content_hash === game.hash);
          if (existing) fileRef = { id: existing.id, name: existing.file_name };
          else {
            const up = await filesApi.upload(game.name, extOf(game.name) || "file", game.hash, game.bytes);
            fileRef = { id: up.file.id, name: up.file.file_name };
          }
        }
      } catch {
        /* best-effort — proceed without the game reference if the vault is unavailable */
      }
      try {
        await pushSaveToServer(game.hash, deviceName());
      } catch {
        /* best-effort — the local save is still safe and syncs later */
      }
      setInquiryFileRefs(fileRef ? [fileRef] : []);
      setInquiryInitial({ title, body });
      if (running) setOverlay("inquiry");
      else setView("inquiry");
    },
    [running],
  );

  // 5번: from the crash panel. Logged in → report now; otherwise stash the payload,
  // open login, and an effect below resumes once the user is authenticated.
  const onReportCrash = useCallback(
    (payload: CrashReport) => {
      if (!authState.user) {
        setPendingCrash(payload);
        setAccountOpen(true);
        return;
      }
      void resolveCrashReport(payload);
    },
    [authState.user, resolveCrashReport],
  );

  // Resume a pending crash report after the user logs in (attachments need a session).
  useEffect(() => {
    if (!authState.user || !pendingCrash) return;
    const p = pendingCrash;
    setPendingCrash(null);
    setAccountOpen(false);
    void resolveCrashReport(p);
  }, [authState.user, pendingCrash, resolveCrashReport]);

  const closeReset = useCallback(() => {
    setResetToken(null);
    setAccountOpen(false);
    // Drop the ?reset= param so a refresh doesn't reopen it.
    window.history.replaceState(null, "", window.location.pathname);
  }, []);

  const accountOverlay = accountOpen && (
    <Overlay title={resetToken ? "비밀번호 재설정" : "계정"} onClose={resetToken ? closeReset : () => setAccountOpen(false)}>
      <AuthPanel authState={authState} toast={showToast} resetToken={resetToken} onResetDone={closeReset} />
    </Overlay>
  );

  // ── Playing: player stays mounted; everything opens as overlays ────────────────
  if (running) {
    return (
      <>
        <Player game={running} user={authState.user} onExit={exitPlayer} toast={showToast} onMenu={() => setOverlay("menu")} onReportCrash={onReportCrash} />

        {overlay === "menu" && (
          <Overlay title="메뉴" onClose={() => setOverlay(null)}>
            <div className="flex w-full max-w-sm flex-col gap-2">
              <p className="text-sm text-fg-dim">게임은 계속 실행 중입니다. 항목을 보고 닫으면 그대로 이어집니다.</p>
              <button
                type="button"
                onClick={() => {
                  setOverlay(null);
                  setAccountOpen(true);
                }}
                className="flex items-center gap-2 rounded-md border border-edge bg-surface2 px-4 py-3 text-left text-fg hover:border-accent"
              >
                <span className="flex h-6 w-6 items-center justify-center rounded-full bg-accent text-[11px] font-bold text-accent-fg">
                  {authState.user ? authState.user.email.slice(0, 2).toUpperCase() : "👤"}
                </span>
                {authState.user ? `계정 (${authState.user.email})` : "로그인 / 가입"}
              </button>
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
            <ViewPanel view={overlay} authState={authState} onRun={onRun} toast={showToast} onReport={onReport} reloadKey={libVersion} onInquireWithFiles={onInquireWithFiles} inquiryFileRefs={inquiryFileRefs} inquiryInitial={inquiryInitial} />
          </Overlay>
        )}

        {accountOverlay}
        <ToastStack toasts={toasts} />
      </>
    );
  }

  // ── Not playing: header + full-page view ──────────────────────────────────────
  return (
    <div className="flex min-h-full flex-col">
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
        <ProfileButton user={authState.user} onClick={() => setAccountOpen(true)} />
      </header>

      <main className="flex flex-1 flex-col items-center gap-5 px-4 py-5">
        {view === "library" && (
          <div className="w-full max-w-xl rounded-lg border border-emerald-500/40 bg-emerald-500/10 px-4 py-3 text-sm text-emerald-700 dark:text-emerald-200">
            {authState.user ? (
              <>
                🔒 게임 파일은 <strong>본인만 접근 가능한 서버 보관함(1GB)</strong>에 <strong>자동으로 저장</strong>됩니다 — 이 기기에
                추가하면 곧바로 서버에 올라갑니다(별도 “올리기” 버튼 없음). 회원님 파일은 <em>공유·공개되지 않으며</em> 다른
                누구도(직접 링크·검색 포함) 접근할 수 없습니다.
              </>
            ) : (
              <>
                🔒 게임 파일은 <strong>이 기기에만</strong> 저장됩니다. 업로드한 게임의 바이트·파일명·해시·“보유 목록”은 서버로 전송되지 않으며
                브라우저(IndexedDB)에만 보관됩니다(미로그인). 로그인하면 본인 전용 서버 보관함(1GB)에 올릴 수 있습니다.
              </>
            )}
          </div>
        )}
        <ViewPanel view={view} authState={authState} onRun={onRun} toast={showToast} onReport={onReport} reloadKey={libVersion} onInquireWithFiles={onInquireWithFiles} inquiryFileRefs={inquiryFileRefs} inquiryInitial={inquiryInitial} />
      </main>

      <footer className="mt-auto px-4 py-6 text-center text-xs text-fg-dim">
        <button type="button" onClick={() => setView("info")} className="underline hover:text-fg">
          서비스 정보 · 라이선스
        </button>
      </footer>

      {accountOverlay}
      <ToastStack toasts={toasts} />
    </div>
  );
}

function ToastStack({ toasts }: { toasts: Toast[] }) {
  if (!toasts.length) return null;
  return (
    <div className="fixed bottom-5 left-1/2 z-50 flex -translate-x-1/2 flex-col items-center gap-2" role="status" aria-live="polite">
      {toasts.map((t) => {
        const icon = t.kind === "err" ? "⚠️" : t.kind === "ok" ? "✓" : "•";
        const border = t.kind === "err" ? "border-red-500" : t.kind === "ok" ? "border-emerald-500" : "border-edge";
        const iconColor = t.kind === "err" ? "text-red-500" : t.kind === "ok" ? "text-emerald-500" : "text-fg-dim";
        return (
          <div key={t.id} className={`flex max-w-[90vw] items-start gap-2 rounded-lg border ${border} bg-surface2 px-4 py-2 text-sm text-fg shadow-lg`}>
            <span className={`shrink-0 ${iconColor}`} aria-hidden="true">{icon}</span>
            <span className="break-words">{t.msg}</span>
          </div>
        );
      })}
    </div>
  );
}
