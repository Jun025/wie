import { useCallback, useEffect, useRef, useState } from "react";
import * as lib from "../lib/library";
import type { LoadableGame } from "../lib/emulator";

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
}

export function GameLibrary({ onRun, toast }: Props) {
  const [games, setGames] = useState<lib.GameMeta[]>([]);
  const [saves, setSaves] = useState<Record<string, lib.LocalSave>>({});
  const [used, setUsed] = useState(0);
  const [cap, setCap] = useState(lib.DEFAULT_CAPACITY_MB);
  const [drag, setDrag] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const refresh = useCallback(async () => {
    const [list, total, capMb, localSaves] = await Promise.all([
      lib.listGames(),
      lib.totalGameBytes(),
      lib.capacityMB(),
      lib.listLocalSaves(),
    ]);
    setGames(list);
    setUsed(total);
    setCap(capMb);
    setSaves(Object.fromEntries(localSaves.map((s) => [s.hash, s])));
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const addFiles = useCallback(
    async (fileList: FileList | File[]) => {
      const files = [...fileList];
      const byBase = new Map<string, File[]>();
      for (const f of files) {
        const base = f.name.replace(/\.[^.]+$/, "");
        byBase.set(base, [...(byBase.get(base) ?? []), f]);
      }

      const capBytes = (await lib.capacityMB()) * 1024 * 1024;
      let usedBytes = await lib.totalGameBytes();

      for (const f of files) {
        const ext = extOf(f.name);
        const base = f.name.replace(/\.[^.]+$/, "");
        // a .jad is handled as the companion of its sibling .jar
        if (ext === "jar" && (byBase.get(base) ?? []).some((x) => extOf(x.name) === "jad")) continue;
        if (!KNOWN_EXTS.includes(ext)) {
          toast(`지원하지 않는 형식: ${f.name}`, "err");
          continue;
        }

        if (ext === "jad") {
          const jar = (byBase.get(base) ?? []).find((x) => extOf(x.name) === "jar");
          if (!jar) {
            toast(`${f.name} 에 대응하는 .jar 도 함께 선택하세요`, "err");
            continue;
          }
          const jarBytes = await jar.arrayBuffer();
          const jadBytes = await f.arrayBuffer();
          const total = jarBytes.byteLength + jadBytes.byteLength;
          if (usedBytes + total > capBytes) return toast("용량 상한 초과 — 상한을 늘리거나 게임을 삭제하세요", "err");
          const hash = await lib.sha256Hex(jarBytes);
          await lib.putGame({ hash, name: jar.name, kind: "jad", bytes: jarBytes, jadBytes, size: total, addedAt: Date.now() });
          usedBytes += total;
          continue;
        }

        const bytes = await f.arrayBuffer();
        if (usedBytes + bytes.byteLength > capBytes) return toast("용량 상한 초과 — 상한을 늘리거나 게임을 삭제하세요", "err");
        const hash = await lib.sha256Hex(bytes);
        await lib.putGame({ hash, name: f.name, kind: ext, bytes, size: bytes.byteLength, addedAt: Date.now() });
        usedBytes += bytes.byteLength;
      }
      toast("라이브러리에 추가됨 (이 기기에만 저장)", "ok");
      await refresh();
    },
    [refresh, toast],
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
    const mb = Math.max(16, Math.min(8192, parseInt(v, 10) || cur));
    await lib.setCapacityMB(mb);
    await refresh();
    toast(`용량 상한 ${mb}MB`);
  }, [refresh, toast]);

  return (
    <section className="w-full max-w-xl flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-slate-100">내 게임 라이브러리</h2>
        <button type="button" onClick={editCapacity} className="text-xs text-slate-400 hover:text-slate-200">
          {fmtBytes(used)} / {cap} MB · 용량 상한
        </button>
      </div>

      <label
        className={
          "cursor-pointer rounded-lg border-2 border-dashed px-4 py-6 text-center transition-colors " +
          (drag ? "border-sky-500 bg-sky-900/20" : "border-slate-600 bg-slate-800/40 hover:border-sky-500")
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
        <input
          ref={inputRef}
          type="file"
          multiple
          accept=".jar,.jad,.zip,.kdf,.skm"
          className="hidden"
          data-testid="file-input"
          onChange={(e) => e.target.files && void addFiles(e.target.files)}
        />
        <div className="text-slate-200 font-medium">게임 파일 추가 (BYOF)</div>
        <div className="text-xs text-slate-500 mt-1">.jar / .jad+.jar / .zip · 끌어다 놓거나 클릭</div>
        <div className="text-[11px] text-slate-600 mt-2">파일은 브라우저(IndexedDB)에만 저장되며 서버로 전송되지 않습니다.</div>
      </label>

      <ul className="flex flex-col gap-2">
        {games.length === 0 && <li className="text-sm text-slate-500 text-center py-4">아직 추가된 게임이 없습니다.</li>}
        {games.map((g) => {
          const sv = saves[g.hash];
          return (
            <li key={g.hash} className="flex items-center gap-3 rounded-lg border border-slate-700 bg-slate-800/40 px-3 py-2">
              <div className="min-w-0 flex-1">
                <div className="font-medium text-slate-100 truncate">{g.name}</div>
                <div className="text-xs text-slate-500">
                  {g.kind.toUpperCase()} · {fmtBytes(g.size)}
                  {sv && ` · 세이브 ${sv.serverId ? "동기화됨" : "로컬"}`}
                  {g.lastPlayedAt && ` · 마지막 실행 ${new Date(g.lastPlayedAt).toLocaleDateString()}`}
                </div>
              </div>
              <button
                type="button"
                data-testid="run-game"
                onClick={() => void run(g.hash)}
                className="rounded-md bg-sky-600 hover:bg-sky-500 px-3 py-1.5 text-sm font-medium text-white"
              >
                실행
              </button>
              <button
                type="button"
                onClick={() => void remove(g.hash)}
                aria-label="삭제"
                className="rounded-md bg-slate-700 hover:bg-red-700/70 px-2 py-1.5 text-sm text-red-300"
              >
                삭제
              </button>
            </li>
          );
        })}
      </ul>

      {games.length > 0 && (
        <button type="button" onClick={clearAll} className="self-end text-xs text-red-400 hover:text-red-300">
          전체 삭제
        </button>
      )}
    </section>
  );
}
