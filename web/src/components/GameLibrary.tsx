import { useCallback, useEffect, useRef, useState } from "react";
import * as lib from "../lib/library";
import { validateGame, type LoadableGame } from "../lib/emulator";
import type { User } from "../lib/api";

const KNOWN_EXTS = ["jar", "jad", "zip", "kdf", "skm"];

function extOf(name: string): string {
  const i = name.lastIndexOf(".");
  return i >= 0 ? name.slice(i + 1).toLowerCase() : "";
}

function fmtBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
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
  const [cap, setCap] = useState(lib.DEFAULT_CAPACITY_MB);
  const [drag, setDrag] = useState(false);
  const [rejected, setRejected] = useState<Rejection | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const refresh = useCallback(async () => {
    const [list, total, capMb, localSaves] = await Promise.all([lib.listGames(), lib.totalGameBytes(), lib.capacityMB(), lib.listLocalSaves()]);
    setGames(list);
    setUsed(total);
    setCap(capMb);
    setSaves(Object.fromEntries(localSaves.map((s) => [s.hash, s])));
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

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

      const capBytes = (await lib.capacityMB()) * 1024 * 1024;
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
            toast(`용량 상한(${cap}MB) 초과 — 게임을 삭제하거나 상한을 조정하세요`, "err");
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
          toast(`용량 상한(${cap}MB) 초과 — 게임을 삭제하거나 상한을 조정하세요`, "err");
          return;
        }
        if (await tryStore(f.name, ext, bytes)) {
          usedBytes += bytes.byteLength;
          added++;
        }
      }
      if (added > 0) toast(`${added}개 라이브러리에 추가됨 (이 기기에만 저장)`, "ok");
      await refresh();
    },
    [refresh, toast, tryStore, cap],
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
      await refresh();
      toast("삭제됨");
    },
    [refresh, toast],
  );

  const clearAll = useCallback(async () => {
    if (!confirm("이 기기의 게임을 모두 삭제할까요?")) return;
    await lib.clearGames();
    await refresh();
    toast("전체 삭제됨");
  }, [refresh, toast]);

  const editCapacity = useCallback(async () => {
    const cur = await lib.capacityMB();
    const v = prompt("게임 라이브러리 용량 상한(MB):", String(cur));
    if (v == null) return;
    const mb = Math.max(1, Math.min(512, parseInt(v, 10) || cur));
    await lib.setCapacityMB(mb);
    await refresh();
    toast(`용량 상한 ${mb}MB`);
  }, [refresh, toast]);

  const usedPct = Math.min(100, Math.round((used / (cap * 1024 * 1024)) * 100));
  const barColor = usedPct >= 90 ? "bg-red-500" : usedPct >= 70 ? "bg-amber-500" : "bg-accent";

  return (
    <section className="w-full max-w-xl flex flex-col gap-4">
      <h2 className="text-lg font-semibold text-fg">내 게임 라이브러리</h2>

      {/* capacity visualization (only here) */}
      <div className="rounded-lg border border-edge bg-surface2 px-3 py-2">
        <div className="mb-1 flex items-center justify-between text-xs">
          <span className="text-fg-dim">
            <span className="font-medium text-fg">{fmtBytes(used)}</span> / {cap} MB · 사용량
          </span>
          <button type="button" onClick={editCapacity} className="text-fg-dim hover:text-fg">용량 상한 변경</button>
        </div>
        <div className="h-2 w-full overflow-hidden rounded-full bg-surface" role="progressbar" aria-valuenow={usedPct} aria-valuemin={0} aria-valuemax={100} aria-label="라이브러리 용량 사용량">
          <div className={`h-full ${barColor} transition-all`} style={{ width: `${usedPct}%` }} />
        </div>
      </div>

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
        <div className="mt-2 text-[11px] text-fg-dim">파일은 브라우저(IndexedDB)에만 저장되며 서버로 전송되지 않습니다.</div>
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
          <div className="mt-1 text-[11px] text-fg-dim">※ 문의는 텍스트만 가능 — 게임/실행 파일은 첨부할 수 없습니다(서버가 거부). 거부된 파일은 서버로 전송되지 않습니다.</div>
        </div>
      )}

      <ul className="flex flex-col gap-2">
        {games.length === 0 && <li className="py-4 text-center text-sm text-fg-dim">아직 추가된 게임이 없습니다.</li>}
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
        <button type="button" onClick={clearAll} className="self-end text-xs text-red-500 hover:text-red-400">전체 삭제</button>
      )}
    </section>
  );
}
