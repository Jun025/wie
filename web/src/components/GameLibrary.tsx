import { useCallback, useEffect, useRef, useState } from "react";
import * as lib from "../lib/library";
import { validateGame, type LoadableGame } from "../lib/emulator";
import { files as filesApi, type ServerFile, type FilesUsage, type User } from "../lib/api";
import { migrateLocalToServer, fmtBytes, type FileUploadEvent } from "../lib/serverLibrary";
import { extOf, isBlockedUploadExt, UPLOAD_BATCH_MAX_FILES, UPLOAD_PER_FILE_MAX_BYTES, fmtBytes1 } from "../lib/limits";

// Game container formats — used to gate the server-vault "실행" button (the server
// stores any file as a private archive, but only game-format kinds are loadable).
// The device-local list uses the validated `runnable` flag instead.
const GAME_KINDS = new Set(["jar", "jad", "zip", "kdf", "skm"]);

interface Props {
  onRun: (game: LoadableGame) => void;
  toast: (msg: string, kind?: "ok" | "err") => void;
  user: User | null;
  onReport: () => void; // navigate to inquiry (or login) to report a bad file
  reloadKey?: number; // bump to force a re-read (e.g. after login auto-upload)
  onInquireWithFiles?: (refs: { id: string; name: string }[]) => void; // 6번: 선택 보관 파일로 문의 전환
}

interface Rejection {
  name: string;
  reason: string;
}

export function GameLibrary({ onRun, toast, user, onReport, reloadKey, onInquireWithFiles }: Props) {
  const [games, setGames] = useState<lib.GameMeta[]>([]);
  const [saves, setSaves] = useState<Record<string, lib.LocalSave>>({});
  const [used, setUsed] = useState(0);
  const [drag, setDrag] = useState(false);
  const [dragPlay, setDragPlay] = useState(false);
  const [rejected, setRejected] = useState<Rejection | null>(null);
  // server vault state (only meaningful when logged in + provisioned)
  const [serverEnabled, setServerEnabled] = useState(false);
  const [serverFiles, setServerFiles] = useState<ServerFile[]>([]);
  const [serverUsage, setServerUsage] = useState<FilesUsage>({ used: 0, quota: 1024 * 1024 * 1024 });
  const [uploads, setUploads] = useState<FileUploadEvent[]>([]); // live per-file upload progress (1번)
  const [selected, setSelected] = useState<Set<string>>(new Set()); // 6번: checked server files for "문의"
  const [busyId, setBusyId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const playInputRef = useRef<HTMLInputElement>(null);

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
  }, [refreshLocal, refreshServer, reloadKey]);

  // Store a file in the private device vault. ANY extension is accepted (4번:
  // 확장자 무관 저장) — it is a personal private vault. We still try to VALIDATE it
  // as a loadable game: if it boots, mark it runnable (실행 허용); otherwise keep it
  // as a stored-only file (보관만, 실행 불가). Returns the stored record's hash +
  // runnable flag, or null on a storage error.
  const tryStore = useCallback(
    async (name: string, kind: string, bytes: ArrayBuffer, jadBytes?: ArrayBuffer): Promise<{ hash: string; runnable: boolean } | null> => {
      let runnable = false;
      try {
        runnable = !(await validateGame(name, bytes));
      } catch {
        runnable = false; // unparseable as a game → store-only
      }
      const hash = await lib.sha256Hex(bytes);
      await lib.putGame({ hash, name, kind, bytes, jadBytes, size: bytes.byteLength + (jadBytes?.byteLength ?? 0), addedAt: Date.now(), runnable });
      return { hash, runnable };
    },
    [],
  );

  // Live per-file upload-status upsert for the progress list (shared by every path
  // that pushes local files to the server).
  const onUploadEvent = useCallback((e: FileUploadEvent) => {
    setUploads((prev) => {
      const next = prev.slice();
      const i = next.findIndex((x) => x.hash === e.hash);
      if (i >= 0) next[i] = e;
      else next.push(e);
      return next;
    });
  }, []);

  // 3번: logged-in users have NO local-only path — device-local files auto-upload
  // to the private server vault and the local copy is freed on ACK. No manual
  // button. Best-effort (quota/offline keeps files local, retried later).
  const autoUpload = useCallback(async () => {
    if (!user) return;
    setUploads([]);
    try {
      const r = await migrateLocalToServer(undefined, onUploadEvent);
      await refreshLocal();
      await refreshServer();
      if (r.failed || r.stopped) {
        const parts = [`업로드 ${r.uploaded}`, r.deduped ? `중복 ${r.deduped}` : null, r.failed ? `실패 ${r.failed}(로컬 보존)` : null].filter(Boolean);
        toast(r.message ?? `서버 보관함 반영 — ${parts.join(" · ")}`, "err");
      }
    } catch {
      /* best-effort — local copy is safe, retried on next add/login */
    }
  }, [user, onUploadEvent, refreshLocal, refreshServer, toast]);

  const addFiles = useCallback(
    async (fileList: FileList | File[], runFirst = false) => {
      setRejected(null);
      const incoming = [...fileList];

      // 4번: batch-count cap (100 files per add) — enforced before any read.
      if (incoming.length > UPLOAD_BATCH_MAX_FILES) {
        toast(`한 번에 최대 ${UPLOAD_BATCH_MAX_FILES}개까지 추가할 수 있습니다 (선택: ${incoming.length}개)`, "err");
        return;
      }

      // 4번: block executable/script files (원천 차단) + per-file 100MB cap. Rejected
      // files never touch storage; the first rejection surfaces in the notice box.
      const rejections: Rejection[] = [];
      const files = incoming.filter((f) => {
        if (isBlockedUploadExt(f.name)) {
          rejections.push({ name: f.name, reason: "실행 파일·스크립트는 업로드할 수 없습니다" });
          return false;
        }
        if (f.size > UPLOAD_PER_FILE_MAX_BYTES) {
          rejections.push({ name: f.name, reason: `파일이 너무 큽니다 (최대 ${fmtBytes1(UPLOAD_PER_FILE_MAX_BYTES)}, 이 파일 ${fmtBytes1(f.size)})` });
          return false;
        }
        return true;
      });
      if (rejections.length) {
        setRejected(rejections[0]);
        toast(rejections.length === 1 ? `${rejections[0].name}: ${rejections[0].reason}` : `${rejections.length}개 파일을 차단했습니다 (실행 파일·용량 초과)`, "err");
        if (files.length === 0) return;
      }

      let firstRunnable: string | null = null;
      const byBase = new Map<string, File[]>();
      for (const f of files) {
        const base = f.name.replace(/\.[^.]+$/, "");
        byBase.set(base, [...(byBase.get(base) ?? []), f]);
      }

      // Device-local store is capped at a FIXED 10 MB (not user-editable). Logged-in
      // users with a server vault can store far more there (1 GB) — see below.
      const capBytes = lib.LOCAL_CAP_BYTES;
      let usedBytes = await lib.totalGameBytes();
      let addedGames = 0;
      let addedFiles = 0;

      for (const f of files) {
        const ext = extOf(f.name);
        const base = f.name.replace(/\.[^.]+$/, "");
        if (ext === "jar" && (byBase.get(base) ?? []).some((x) => extOf(x.name) === "jad")) continue; // companion of a .jad

        // .jad + .jar pair → store together as one runnable game entry.
        if (ext === "jad") {
          const jar = (byBase.get(base) ?? []).find((x) => extOf(x.name) === "jar");
          if (jar) {
            const jarBytes = await jar.arrayBuffer();
            const jadBytes = await f.arrayBuffer();
            if (usedBytes + jarBytes.byteLength + jadBytes.byteLength > capBytes) {
              toast(`이 기기 보관 한도(${lib.LOCAL_CAP_MB}MB) 초과 — 로그인 후 서버 보관함(1GB)에 올리거나 파일을 삭제하세요`, "err");
              return;
            }
            const r = await tryStore(jar.name, "jad", jarBytes, jadBytes);
            if (r) {
              usedBytes += jarBytes.byteLength + jadBytes.byteLength;
              r.runnable ? addedGames++ : addedFiles++;
              if (r.runnable) firstRunnable = firstRunnable ?? r.hash;
            }
            continue;
          }
          // a lone .jad with no .jar — fall through and store it as a plain file.
        }

        const bytes = await f.arrayBuffer();
        if (usedBytes + bytes.byteLength > capBytes) {
          toast(`이 기기 보관 한도(${lib.LOCAL_CAP_MB}MB) 초과 — 로그인 후 서버 보관함(1GB)에 올리거나 파일을 삭제하세요`, "err");
          break;
        }
        const r = await tryStore(f.name, ext || "file", bytes);
        if (r) {
          usedBytes += bytes.byteLength;
          r.runnable ? addedGames++ : addedFiles++;
          if (r.runnable) firstRunnable = firstRunnable ?? r.hash;
        }
      }
      await refreshLocal();

      // Resolve the "추가하고 바로 실행" target NOW — read its bytes before any
      // auto-upload can free the local copy (avoids a delete/read race).
      let runTarget: LoadableGame | null = null;
      if (runFirst && firstRunnable) {
        const g = await lib.getGame(firstRunnable);
        if (g) runTarget = { hash: g.hash, name: g.name, bytes: g.bytes };
      }

      // 3번: logged-in → the just-added files auto-upload to the server vault.
      // Fire-and-forget so play starts immediately.
      if (user) void autoUpload();

      if (runTarget) {
        onRun(runTarget);
        return;
      }

      // Tailored result toast. Logged in → files are on their way to the server;
      // not logged in → stored on this device only.
      const dest = user ? "서버 보관함에 올리는 중" : "이 기기에 저장";
      if (addedGames && addedFiles) toast(`게임 ${addedGames}개 추가 · 파일 ${addedFiles}개 보관 (${dest})`, "ok");
      else if (addedGames) toast(`게임 ${addedGames}개 라이브러리에 추가됨 (${dest})`, "ok");
      else if (addedFiles) toast(`${addedFiles}개 파일을 보관했습니다 — 게임 파일이 아니라 실행하지 않고 보관만 합니다 (${dest})`, "ok");
    },
    [refreshLocal, toast, tryStore, onRun, user, autoUpload],
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
      if (!confirm("이 게임을 이 기기에서 삭제할까요? (세이브는 보존되어 다시 추가하면 이어서 할 수 있습니다)")) return;
      await lib.deleteGame(hash); // save kept (keyed by ROM hash) — no save loss
      await refreshLocal();
      toast("삭제됨 (세이브는 보존)");
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

      {/* PRIMARY drop-zone: add the file AND immediately play it — click OR drag-and-
          drop. Any file is accepted; game files run immediately, other files are kept
          in the private vault (4번). Dashed border + dragover highlight make the
          drag-and-drop affordance obvious. */}
      <input
        ref={playInputRef}
        type="file"
        multiple
        className="hidden"
        data-testid="file-input-play"
        onChange={(e) => {
          if (e.target.files?.length) void addFiles(e.target.files, true);
          e.target.value = "";
        }}
      />
      <button
        type="button"
        onClick={() => playInputRef.current?.click()}
        onDragOver={(e) => {
          e.preventDefault();
          setDragPlay(true);
        }}
        onDragLeave={() => setDragPlay(false)}
        onDrop={(e) => {
          e.preventDefault();
          setDragPlay(false);
          if (e.dataTransfer.files.length) void addFiles(e.dataTransfer.files, true);
        }}
        aria-label="파일 추가하고 바로 실행 (클릭 또는 끌어다 놓기) — 게임이면 즉시 실행, 아니면 보관"
        className={
          "flex flex-col items-center justify-center gap-1 rounded-xl border-2 border-dashed px-4 py-6 text-center transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent " +
          (dragPlay ? "border-accent bg-accent/15 ring-2 ring-accent/60" : "border-accent/60 bg-accent/5 hover:border-accent hover:bg-accent/10")
        }
      >
        <span className="text-2xl leading-none" aria-hidden="true">⬇</span>
        <span className="font-semibold text-fg">▶ 파일을 끌어다 놓거나 클릭해 추가</span>
        <span className="text-xs font-normal text-fg-dim">게임 파일은 추가 즉시 실행 · 그 외 파일은 비공개 보관함에 저장(실행 안 함)</span>
      </button>

      {/* Capacity bar. Logged in → ALWAYS show the server-vault usage (used/quota +
          %), even before the vault is provisioned, so it never silently vanishes
          (cp/5번). Not logged in → show the device-local capacity. */}
      {user ? (
        <div className="rounded-lg border border-emerald-500/40 bg-emerald-500/10 px-3 py-2">
          <div className="mb-1 flex items-center justify-between text-xs">
            <span className="text-emerald-700 dark:text-emerald-200">
              <span className="font-medium">{fmtBytes(serverUsage.used)}</span> / {fmtBytes(serverUsage.quota)} · 내 서버 보관함 (본인만 접근)
            </span>
            <span className="text-emerald-700/80 dark:text-emerald-200/80 tabular-nums">{srvPct}%</span>
          </div>
          <div className="h-2 w-full overflow-hidden rounded-full bg-surface" role="progressbar" aria-valuenow={srvPct} aria-valuemin={0} aria-valuemax={100} aria-label="서버 보관함 사용량">
            <div className={`h-full ${srvBar} transition-all`} style={{ width: `${srvPct}%` }} />
          </div>
        </div>
      ) : (
        <div className="rounded-lg border border-edge bg-surface2 px-3 py-2">
          <div className="mb-1 flex items-center justify-between text-xs">
            <span className="text-fg-dim">
              <span className="font-medium text-fg">{fmtBytes(used)}</span> / {fmtBytes(lib.LOCAL_CAP_BYTES)} · 이 기기(브라우저) 사용량
            </span>
            <span className="text-fg-dim tabular-nums">{localPct}% · 로그인 시 서버 1GB</span>
          </div>
          <div className="h-2 w-full overflow-hidden rounded-full bg-surface" role="progressbar" aria-valuenow={localPct} aria-valuemin={0} aria-valuemax={100} aria-label="이 기기 라이브러리 사용량">
            <div className={`h-full ${localBar} transition-all`} style={{ width: `${localPct}%` }} />
          </div>
        </div>
      )}

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
            {games.length > 0 && <span className="text-[11px] text-emerald-700/80 dark:text-emerald-300/80">이 기기 파일은 자동으로 서버에 올라갑니다</span>}
          </div>

          {/* 6번: select vault files → carry them (by reference) into a 문의 */}
          {onInquireWithFiles && selected.size > 0 && (
            <div className="flex items-center justify-between rounded-md border border-accent/40 bg-accent/10 px-3 py-1.5 text-xs">
              <span className="text-fg">{selected.size}개 선택됨</span>
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => {
                    const refs = serverFiles.filter((f) => selected.has(f.id)).map((f) => ({ id: f.id, name: f.file_name }));
                    if (refs.length) onInquireWithFiles(refs);
                  }}
                  className="rounded-md bg-accent px-3 py-1 font-medium text-accent-fg hover:bg-accent-hover"
                >
                  선택 파일로 문의
                </button>
                <button type="button" onClick={() => setSelected(new Set())} className="text-fg-dim hover:text-fg">선택 해제</button>
              </div>
            </div>
          )}

          {/* live per-file upload progress (1번) */}
          {uploads.length > 0 && (
            <div className="rounded-lg border border-edge bg-surface2 px-3 py-2">
              <div className="mb-1.5 flex items-center justify-between text-xs">
                <span className="font-medium text-fg">서버 업로드 진행</span>
                <span className="tabular-nums text-fg-dim">
                  완료 {uploads.filter((u) => u.status === "uploaded" || u.status === "deduped").length}
                  {uploads.some((u) => u.status === "failed") ? ` · 실패 ${uploads.filter((u) => u.status === "failed").length}` : ""} / {uploads.length}
                </span>
              </div>
              <ul className="flex flex-col gap-1">
                {uploads.map((u) => {
                  const badge =
                    u.status === "uploaded"
                      ? { t: "완료 ✓", c: "text-emerald-600 dark:text-emerald-400" }
                      : u.status === "deduped"
                        ? { t: "이미 보관함 ✓", c: "text-emerald-600 dark:text-emerald-400" }
                        : u.status === "uploading"
                          ? { t: "업로드중…", c: "text-accent" }
                          : u.status === "failed"
                            ? { t: "실패", c: "text-red-500" }
                            : { t: "대기", c: "text-fg-dim" };
                  return (
                    <li key={u.hash} className="flex items-center gap-2 text-xs">
                      <span className="min-w-0 flex-1 truncate text-fg-dim">
                        {u.name} <span className="opacity-70">· {fmtBytes(u.size)}</span>
                      </span>
                      <span className={`shrink-0 tabular-nums ${badge.c}`} title={u.reason}>
                        {u.status === "uploading" && <span className="mr-1 inline-block animate-pulse">●</span>}
                        {badge.t}
                      </span>
                    </li>
                  );
                })}
              </ul>
              {uploads.some((u) => u.status === "failed") && (
                <p className="mt-1 text-[11px] text-red-500/90">실패한 항목은 이 기기에 그대로 보존되었습니다. 다시 시도할 수 있습니다.</p>
              )}
            </div>
          )}

          <ul className="flex flex-col gap-2">
            {serverFiles.length === 0 && <li className="py-3 text-center text-xs text-fg-dim">서버 보관함이 비어 있습니다.</li>}
            {serverFiles.map((f) => (
              <li key={f.id} className="flex items-center gap-3 rounded-lg border border-emerald-500/30 bg-surface2 px-3 py-2">
                {onInquireWithFiles && (
                  <input
                    type="checkbox"
                    aria-label={`${f.file_name} 선택`}
                    checked={selected.has(f.id)}
                    onChange={(e) =>
                      setSelected((prev) => {
                        const next = new Set(prev);
                        e.target.checked ? next.add(f.id) : next.delete(f.id);
                        return next;
                      })
                    }
                    className="h-4 w-4 shrink-0 accent-accent"
                  />
                )}
                <div className="min-w-0 flex-1">
                  <div className="truncate font-medium text-fg">{f.file_name}</div>
                  <div className="text-xs text-fg-dim">
                    {f.kind.toUpperCase()} · {fmtBytes(f.size)} · ☁ 서버
                    {!GAME_KINDS.has(f.kind.toLowerCase()) && " · 보관 전용"}
                  </div>
                </div>
                {GAME_KINDS.has(f.kind.toLowerCase()) && (
                  <button type="button" onClick={() => void runServer(f)} disabled={busyId === f.id} className="rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-fg hover:bg-accent-hover disabled:opacity-60">
                    {busyId === f.id ? "…" : "실행"}
                  </button>
                )}
                <button type="button" onClick={() => void removeServer(f)} disabled={busyId === f.id} aria-label="서버에서 삭제" className="rounded-md bg-surface px-2 py-1.5 text-sm text-red-500 hover:bg-red-500/15 disabled:opacity-60">
                  삭제
                </button>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* ── device-local list ────────────────────────────────────────────────── */}
      {showServer && games.length > 0 && <h3 className="text-sm font-semibold text-fg">이 기기 (브라우저) — 서버로 올리지 않은 항목</h3>}
      <ul className="flex flex-col gap-2">
        {games.length === 0 && !showServer && <li className="py-4 text-center text-sm text-fg-dim">이 기기에 추가된 게임이 없습니다.</li>}
        {games.map((g) => {
          const sv = saves[g.hash];
          return (
            <li key={g.hash} className="flex items-center gap-3 rounded-lg border border-edge bg-surface2 px-3 py-2">
              <div className="min-w-0 flex-1">
                <div className="truncate font-medium text-fg">{g.name}</div>
                <div className="text-xs text-fg-dim">
                  {g.kind.toUpperCase()} · {fmtBytes(g.size)}
                  {!lib.isRunnable(g) && " · 보관 전용(게임 아님)"}
                  {sv && ` · 세이브 ${sv.serverId ? "동기화됨" : "로컬"}`}
                  {g.lastPlayedAt && ` · 마지막 실행 ${new Date(g.lastPlayedAt).toLocaleDateString()}`}
                </div>
              </div>
              {lib.isRunnable(g) && (
                <button type="button" data-testid="run-game" onClick={() => void run(g.hash)} className="rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-fg hover:bg-accent-hover">
                  실행
                </button>
              )}
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

      {/* ── SECONDARY: "업로드만" — smaller, at the bottom (not the main action) ──── */}
      <div className="mt-2 border-t border-edge pt-3">
        <label
          className={
            "block cursor-pointer rounded-md border border-dashed px-3 py-2 text-center text-xs transition-colors " +
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
          <input ref={inputRef} type="file" multiple className="hidden" data-testid="file-input" onChange={(e) => e.target.files && void addFiles(e.target.files)} />
          <span className="font-medium text-fg-dim">보관만 (실행하지 않고 목록에 추가)</span>
          <span className="ml-1 text-[11px] text-fg-dim">· 여러 개 끌어다 놓거나 클릭{user ? " · 서버 보관함에 자동 업로드" : " · 미로그인은 이 기기에만 저장"}</span>
        </label>
      </div>
    </section>
  );
}
