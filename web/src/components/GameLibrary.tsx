import { useCallback, useEffect, useRef, useState } from "react";
import * as lib from "../lib/library";
import { validateGame, type LoadableGame } from "../lib/emulator";
import { files as filesApi, type ServerFile, type FilesUsage, type User } from "../lib/api";
import { migrateLocalToServer, fmtBytes } from "../lib/serverLibrary";

const KNOWN_EXTS = ["jar", "jad", "zip", "kdf", "skm"];

function extOf(name: string): string {
  const i = name.lastIndexOf(".");
  return i >= 0 ? name.slice(i + 1).toLowerCase() : "";
}

interface Props {
  onRun: (game: LoadableGame) => void;
  toast: (msg: string, kind?: "ok" | "err") => void;
  user: User | null;
  onReport: () => void; // navigate to inquiry (or login) to report a bad file
}

interface Rejection {
  name: string;
  reason: string;
}

export function GameLibrary({ onRun, toast, user, onReport }: Props) {
  const [games, setGames] = useState<lib.GameMeta[]>([]);
  const [saves, setSaves] = useState<Record<string, lib.LocalSave>>({});
  const [used, setUsed] = useState(0);
  const [drag, setDrag] = useState(false);
  const [rejected, setRejected] = useState<Rejection | null>(null);
  // server vault state (only meaningful when logged in + provisioned)
  const [serverEnabled, setServerEnabled] = useState(false);
  const [serverFiles, setServerFiles] = useState<ServerFile[]>([]);
  const [serverUsage, setServerUsage] = useState<FilesUsage>({ used: 0, quota: 1024 * 1024 * 1024 });
  const [migrating, setMigrating] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const refreshLocal = useCallback(async () => {
    const [list, total, localSaves] = await Promise.all([lib.listGames(), lib.totalGameBytes(), lib.listLocalSaves()]);
    setGames(list);
    setUsed(total);
    setSaves(Object.fromEntries(localSaves.map((s) => [s.hash, s])));
  }, []);

  const refreshServer = useCallback(async () => {
    if (!user) {
      setServerEnabled(false);
      setServerFiles([]);
      return;
    }
    try {
      const res = await filesApi.list();
      setServerEnabled(res.enabled);
      setServerFiles(res.files);
      setServerUsage(res.usage);
    } catch {
      setServerEnabled(false); // not provisioned / offline — hide the section
    }
  }, [user]);

  useEffect(() => {
    void refreshLocal();
    void refreshServer();
  }, [refreshLocal, refreshServer]);

  // Store a candidate game ONLY after it validates as a loadable format. Invalid
  // files are never written to IndexedDB and are surfaced for reporting.
  const tryStore = useCallback(
    async (name: string, kind: string, bytes: ArrayBuffer, jadBytes?: ArrayBuffer): Promise<boolean> => {
      const reason = await validateGame(name, bytes);
      if (reason) {
        setRejected({ name, reason });
        return false;
      }
      const hash = await lib.sha256Hex(bytes);
      await lib.putGame({ hash, name, kind, bytes, jadBytes, size: bytes.byteLength + (jadBytes?.byteLength ?? 0), addedAt: Date.now() });
      return true;
    },
    [],
  );

  const addFiles = useCallback(
    async (fileList: FileList | File[]) => {
      setRejected(null);
      const files = [...fileList];
      const byBase = new Map<string, File[]>();
      for (const f of files) {
        const base = f.name.replace(/\.[^.]+$/, "");
        byBase.set(base, [...(byBase.get(base) ?? []), f]);
      }

      // Device-local store is capped at a FIXED 10 MB (not user-editable). Logged-in
      // users with a server vault can store far more there (1 GB) — see below.
      const capBytes = lib.LOCAL_CAP_BYTES;
      let usedBytes = await lib.totalGameBytes();
      let added = 0;

      for (const f of files) {
        const ext = extOf(f.name);
        const base = f.name.replace(/\.[^.]+$/, "");
        if (ext === "jar" && (byBase.get(base) ?? []).some((x) => extOf(x.name) === "jad")) continue; // companion of a .jad
        if (!KNOWN_EXTS.includes(ext)) {
          setRejected({ name: f.name, reason: `지원하지 않는 확장자입니다 (.jar / .jad+.jar / .zip 만 가능).` });
          continue;
        }

        if (ext === "jad") {
          const jar = (byBase.get(base) ?? []).find((x) => extOf(x.name) === "jar");
          if (!jar) {
            setRejected({ name: f.name, reason: ".jad 는 짝이 되는 .jar 파일과 함께 선택해야 합니다." });
            continue;
          }
          const jarBytes = await jar.arrayBuffer();
          const jadBytes = await f.arrayBuffer();
          if (usedBytes + jarBytes.byteLength + jadBytes.byteLength > capBytes) {
            toast(`이 기기 보관 한도(${lib.LOCAL_CAP_MB}MB) 초과 — 로그인 후 서버 보관함(1GB)에 올리거나 게임을 삭제하세요`, "err");
            return;
          }
          if (await tryStore(jar.name, "jad", jarBytes, jadBytes)) {
            usedBytes += jarBytes.byteLength + jadBytes.byteLength;
            added++;
          }
          continue;
        }

        const bytes = await f.arrayBuffer();
        if (usedBytes + bytes.byteLength > capBytes) {
          toast(`이 기기 보관 한도(${lib.LOCAL_CAP_MB}MB) 초과 — 로그인 후 서버 보관함(1GB)에 올리거나 게임을 삭제하세요`, "err");
          return;
        }
        if (await tryStore(f.name, ext, bytes)) {
          usedBytes += bytes.byteLength;
          added++;
        }
      }
      if (added > 0) toast(`${added}개 라이브러리에 추가됨 (이 기기에만 저장)`, "ok");
      await refreshLocal();
    },
    [refreshLocal, toast, tryStore],
  );

  const run = useCallback(
    async (hash: string) => {
      const g = await lib.getGame(hash);
      if (!g) return toast("게임을 찾을 수 없습니다", "err");
      onRun({ hash: g.hash, name: g.name, bytes: g.bytes });
    },
    [onRun, toast],
  );

  const remove = useCallback(
    async (hash: string) => {
      if (!confirm("이 게임을 이 기기에서 삭제할까요? (세이브 로컬 캐시도 함께 삭제)")) return;
      await lib.deleteGame(hash);
      await refreshLocal();
      toast("삭제됨");
    },
    [refreshLocal, toast],
  );

  const clearAll = useCallback(async () => {
    if (!confirm("이 기기의 게임을 모두 삭제할까요?")) return;
    await lib.clearGames();
    await refreshLocal();
    toast("전체 삭제됨");
  }, [refreshLocal, toast]);

  // ── server vault actions ────────────────────────────────────────────────────
  const migrate = useCallback(async () => {
    if (games.length === 0) return;
    if (!confirm(`이 기기의 게임 ${games.length}개를 내 서버 보관함(1GB, 본인만 접근)으로 올립니다.\n업로드된 게임은 이 기기에서 삭제됩니다(서버에 보관됨). 계속할까요?`)) return;
    setMigrating(true);
    try {
      const r = await migrateLocalToServer();
      await refreshLocal();
      await refreshServer();
      const parts = [`업로드 ${r.uploaded}`, r.deduped ? `중복 ${r.deduped}(이미 보관함)` : null, r.failed ? `실패 ${r.failed}` : null].filter(Boolean);
      toast(r.message ?? `서버 보관함에 반영됨 — ${parts.join(" · ")}`, r.stopped || r.failed ? "err" : "ok");
    } finally {
      setMigrating(false);
    }
  }, [games.length, refreshLocal, refreshServer, toast]);

  const runServer = useCallback(
    async (f: ServerFile) => {
      setBusyId(f.id);
      try {
        const bytes = await filesApi.download(f.id);
        onRun({ hash: f.content_hash, name: f.file_name, bytes });
      } catch (e) {
        toast(`서버 게임을 불러오지 못했습니다: ${(e as Error).message}`, "err");
      } finally {
        setBusyId(null);
      }
    },
    [onRun, toast],
  );

  const removeServer = useCallback(
    async (f: ServerFile) => {
      if (!confirm(`서버 보관함에서 "${f.file_name}"을(를) 삭제할까요?`)) return;
      setBusyId(f.id);
      try {
        await filesApi.remove(f.id);
        await refreshServer();
        toast("서버 보관함에서 삭제됨");
      } catch (e) {
        toast(`삭제 실패: ${(e as Error).message}`, "err");
      } finally {
        setBusyId(null);
      }
    },
    [refreshServer, toast],
  );

  const localPct = Math.min(100, Math.round((used / lib.LOCAL_CAP_BYTES) * 100));
  const localBar = localPct >= 90 ? "bg-red-500" : localPct >= 70 ? "bg-amber-500" : "bg-accent";
  const srvPct = serverUsage.quota ? Math.min(100, Math.round((serverUsage.used / serverUsage.quota) * 100)) : 0;
  const srvBar = srvPct >= 90 ? "bg-red-500" : srvPct >= 70 ? "bg-amber-500" : "bg-emerald-500";
  const showServer = !!user && serverEnabled;

  return (
    <section className="w-full max-w-xl flex flex-col gap-4">
      <h2 className="text-lg font-semibold text-fg">내 게임 라이브러리</h2>

      {/* device-local capacity (FIXED 10MB — no edit control) */}
      <div className="rounded-lg border border-edge bg-surface2 px-3 py-2">
        <div className="mb-1 flex items-center justify-between text-xs">
          <span className="text-fg-dim">
            <span className="font-medium text-fg">{fmtBytes(used)}</span> / {lib.LOCAL_CAP_MB} MB · 이 기기(브라우저) 사용량
          </span>
          <span className="text-fg-dim">한도 고정</span>
        </div>
        <div className="h-2 w-full overflow-hidden rounded-full bg-surface" role="progressbar" aria-valuenow={localPct} aria-valuemin={0} aria-valuemax={100} aria-label="이 기기 라이브러리 사용량">
          <div className={`h-full ${localBar} transition-all`} style={{ width: `${localPct}%` }} />
        </div>
      </div>

      {/* server vault capacity (1GB, owner-only) — only when logged in + provisioned */}
      {showServer && (
        <div className="rounded-lg border border-emerald-500/40 bg-emerald-500/10 px-3 py-2">
          <div className="mb-1 flex items-center justify-between text-xs">
            <span className="text-emerald-700 dark:text-emerald-200">
              <span className="font-medium">{fmtBytes(serverUsage.used)}</span> / {fmtBytes(serverUsage.quota)} · 내 서버 보관함 (본인만 접근)
            </span>
            <span className="text-emerald-700/80 dark:text-emerald-200/80">한도 고정</span>
          </div>
          <div className="h-2 w-full overflow-hidden rounded-full bg-surface" role="progressbar" aria-valuenow={srvPct} aria-valuemin={0} aria-valuemax={100} aria-label="서버 보관함 사용량">
            <div className={`h-full ${srvBar} transition-all`} style={{ width: `${srvPct}%` }} />
          </div>
        </div>
      )}

      <label
        className={
          "cursor-pointer rounded-lg border-2 border-dashed px-4 py-6 text-center transition-colors " +
          (drag ? "border-accent bg-accent/10" : "border-edge bg-surface2 hover:border-accent")
        }
        onDragOver={(e) => {
          e.preventDefault();
          setDrag(true);
        }}
        onDragLeave={() => setDrag(false)}
        onDrop={(e) => {
          e.preventDefault();
          setDrag(false);
          if (e.dataTransfer.files.length) void addFiles(e.dataTransfer.files);
        }}
      >
        <input ref={inputRef} type="file" multiple accept=".jar,.jad,.zip,.kdf,.skm" className="hidden" data-testid="file-input" onChange={(e) => e.target.files && void addFiles(e.target.files)} />
        <div className="font-medium text-fg">게임 파일 추가 (BYOF)</div>
        <div className="mt-1 text-xs text-fg-dim">.jar / .jad+.jar / .zip · 끌어다 놓거나 클릭</div>
        <div className="mt-2 text-[11px] text-fg-dim">
          {showServer
            ? "추가하면 먼저 이 기기에 저장됩니다. 아래 “서버 보관함에 올리기”로 본인 전용 서버(1GB)에 보관할 수 있어요."
            : "파일은 브라우저(IndexedDB)에만 저장되며 서버로 전송되지 않습니다."}
        </div>
      </label>

      {/* rejected file notice + report/login guidance */}
      {rejected && (
        <div className="rounded-lg border border-amber-500/50 bg-amber-500/10 px-3 py-3 text-sm">
          <div className="font-medium text-amber-700 dark:text-amber-200">불러올 수 없는 파일: {rejected.name}</div>
          <div className="mt-1 break-words text-xs text-amber-700/90 dark:text-amber-200/90">사유: {rejected.reason}</div>
          <div className="mt-1 text-xs text-fg-dim">이 파일은 라이브러리에 저장되지 않았습니다. 정상적인 WIPI/SKVM/J2ME 게임 파일인지 확인해 주세요.</div>
          <div className="mt-2 flex flex-wrap items-center gap-2">
            <button type="button" onClick={onReport} className="rounded-md bg-accent px-3 py-1.5 text-xs font-medium text-accent-fg hover:bg-accent-hover">
              {user ? "문의·건의로 신고" : "로그인하고 문의하기"}
            </button>
            <button type="button" onClick={() => setRejected(null)} className="text-xs text-fg-dim hover:text-fg">닫기</button>
          </div>
        </div>
      )}

      {/* ── server vault list (owner-only) ───────────────────────────────────── */}
      {showServer && (
        <div className="flex flex-col gap-2">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold text-emerald-700 dark:text-emerald-300">서버 보관함 (본인만 접근)</h3>
            {games.length > 0 && (
              <button
                type="button"
                onClick={() => void migrate()}
                disabled={migrating}
                className="rounded-md bg-emerald-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-emerald-500 disabled:opacity-60"
              >
                {migrating ? "올리는 중…" : `이 기기 게임 ${games.length}개 서버에 올리기`}
              </button>
            )}
          </div>
          <ul className="flex flex-col gap-2">
            {serverFiles.length === 0 && <li className="py-3 text-center text-xs text-fg-dim">서버 보관함이 비어 있습니다.</li>}
            {serverFiles.map((f) => (
              <li key={f.id} className="flex items-center gap-3 rounded-lg border border-emerald-500/30 bg-surface2 px-3 py-2">
                <div className="min-w-0 flex-1">
                  <div className="truncate font-medium text-fg">{f.file_name}</div>
                  <div className="text-xs text-fg-dim">
                    {f.kind.toUpperCase()} · {fmtBytes(f.size)} · ☁ 서버
                  </div>
                </div>
                <button type="button" onClick={() => void runServer(f)} disabled={busyId === f.id} className="rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-fg hover:bg-accent-hover disabled:opacity-60">
                  {busyId === f.id ? "…" : "실행"}
                </button>
                <button type="button" onClick={() => void removeServer(f)} disabled={busyId === f.id} aria-label="서버에서 삭제" className="rounded-md bg-surface px-2 py-1.5 text-sm text-red-500 hover:bg-red-500/15 disabled:opacity-60">
                  삭제
                </button>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* ── device-local list ────────────────────────────────────────────────── */}
      {showServer && <h3 className="text-sm font-semibold text-fg">이 기기 (브라우저)</h3>}
      <ul className="flex flex-col gap-2">
        {games.length === 0 && <li className="py-4 text-center text-sm text-fg-dim">이 기기에 추가된 게임이 없습니다.</li>}
        {games.map((g) => {
          const sv = saves[g.hash];
          return (
            <li key={g.hash} className="flex items-center gap-3 rounded-lg border border-edge bg-surface2 px-3 py-2">
              <div className="min-w-0 flex-1">
                <div className="truncate font-medium text-fg">{g.name}</div>
                <div className="text-xs text-fg-dim">
                  {g.kind.toUpperCase()} · {fmtBytes(g.size)}
                  {sv && ` · 세이브 ${sv.serverId ? "동기화됨" : "로컬"}`}
                  {g.lastPlayedAt && ` · 마지막 실행 ${new Date(g.lastPlayedAt).toLocaleDateString()}`}
                </div>
              </div>
              <button type="button" data-testid="run-game" onClick={() => void run(g.hash)} className="rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-fg hover:bg-accent-hover">
                실행
              </button>
              <button type="button" onClick={() => void remove(g.hash)} aria-label="삭제" className="rounded-md bg-surface px-2 py-1.5 text-sm text-red-500 hover:bg-red-500/15">
                삭제
              </button>
            </li>
          );
        })}
      </ul>

      {games.length > 0 && (
        <button type="button" onClick={clearAll} className="self-end text-xs text-red-500 hover:text-red-400">이 기기 전체 삭제</button>
      )}
    </section>
  );
}
