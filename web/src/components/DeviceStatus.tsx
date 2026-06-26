import { useCallback, useEffect, useState } from "react";
import { devices as devicesApi, type Device } from "../lib/api";
import { deviceId, localDeviceSummary, sendHeartbeat } from "../lib/device";
import { deviceName, setDeviceName, recommendedDeviceName } from "../lib/saveSync";
import { fmtBytes1 } from "../lib/limits";

function fmtTime(t?: number): string {
  return t ? new Date(t).toLocaleString() : "—";
}

type LocalSummary = Awaited<ReturnType<typeof localDeviceSummary>>;

// Inline-editable device alias row. Works for the CURRENT device (saves via the
// heartbeat) and for ANY other device (saves via the owner-scoped rename API),
// with explicit save feedback.
function AliasEditor({ value, onSave, toast }: { value: string; onSave: (label: string) => Promise<void>; toast: (m: string, k?: "ok" | "err") => void }) {
  const [editing, setEditing] = useState(false);
  const [val, setVal] = useState(value);
  const [saving, setSaving] = useState(false);
  const [savedTick, setSavedTick] = useState(false);

  const save = async () => {
    setSaving(true);
    try {
      await onSave(val.trim());
      setEditing(false);
      setSavedTick(true);
      window.setTimeout(() => setSavedTick(false), 1800);
      toast("기기 별칭이 저장되었습니다", "ok");
    } catch (e) {
      toast(`별칭 저장 실패: ${(e as Error).message}`, "err");
    } finally {
      setSaving(false);
    }
  };

  if (!editing) {
    return (
      <button type="button" onClick={() => { setVal(value); setEditing(true); }} className="inline-flex items-center gap-1 text-xs text-fg-dim hover:text-accent" aria-label="별칭 편집">
        ✎ 이름 변경{savedTick && <span className="text-emerald-500">· 저장됨 ✓</span>}
      </button>
    );
  }
  return (
    <div className="flex items-center gap-1">
      <input
        value={val}
        maxLength={60}
        autoFocus
        onChange={(e) => setVal(e.target.value)}
        onKeyDown={(e) => { if (e.key === "Enter") void save(); if (e.key === "Escape") setEditing(false); }}
        className="w-40 rounded border border-edge bg-surface px-2 py-1 text-xs text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
      />
      <button type="button" onClick={() => void save()} disabled={saving} className="rounded bg-accent px-2 py-1 text-xs font-medium text-accent-fg disabled:opacity-60">{saving ? "…" : "저장"}</button>
      <button type="button" onClick={() => setEditing(false)} className="px-1 text-xs text-fg-dim hover:text-fg">취소</button>
    </div>
  );
}

// Device status. The CURRENT device shows full local detail (filenames, sizes)
// read straight from IndexedDB — those NEVER leave the browser. OTHER devices show
// only the server's ANONYMOUS aggregate (count · MB · timestamps).
export function DeviceStatus({ toast }: { toast: (msg: string, kind?: "ok" | "err") => void }) {
  const [devices, setDevices] = useState<Device[]>([]);
  const [local, setLocal] = useState<LocalSummary | null>(null);
  const [showLocalGames, setShowLocalGames] = useState(false);
  const thisId = deviceId();

  const refresh = useCallback(async () => {
    setLocal(await localDeviceSummary());
    try {
      await sendHeartbeat(); // ensure this device exists/updated before listing
      setDevices((await devicesApi.list()).devices);
    } catch (e) {
      toast(`기기 정보를 불러오지 못했습니다: ${(e as Error).message}`, "err");
    }
  }, [toast]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const renameThis = useCallback(
    async (label: string) => {
      setDeviceName(label || recommendedDeviceName());
      await sendHeartbeat(); // write-through the new label for this device
      await refresh();
    },
    [refresh],
  );

  const renameOther = useCallback(
    async (id: string, label: string) => {
      await devicesApi.rename(id, label);
      await refresh();
    },
    [refresh],
  );

  const mine = devices.find((d) => d.device_id === thisId);
  const others = devices.filter((d) => d.device_id !== thisId);
  const myLabel = mine?.label || deviceName();

  return (
    <section className="flex flex-col gap-4">
      <div>
        <h3 className="text-sm font-semibold text-fg">내 기기 현황</h3>
        <p className="mt-1 text-xs text-fg-dim">
          이 기기의 게임 목록은 브라우저에서만 읽어 표시됩니다. 다른 기기는 서버에 파일명이 없으므로 <strong>개수·용량·시각</strong>만 표시됩니다. 모든 기기의 별칭은 여기서 바꿀 수 있습니다.
        </p>
      </div>

      {/* THIS device — full local detail + inline alias edit + badge */}
      <div className="rounded-lg border border-accent/40 bg-accent/5 p-3">
        <div className="flex items-center justify-between gap-2">
          <div className="min-w-0">
            <div className="truncate font-medium text-fg">
              {myLabel} <span className="ml-1 rounded bg-accent/20 px-1.5 py-0.5 text-[10px] text-accent">이 기기</span>
            </div>
          </div>
          <AliasEditor value={myLabel} onSave={renameThis} toast={toast} />
        </div>
        <div className="mt-1 text-xs text-fg-dim">{local ? `${local.itemCount}개 · ${fmtBytes1(local.totalBytes)}` : "…"}</div>
        {local && (
          <div className="mt-0.5 text-xs text-fg-dim">최근 실행 {fmtTime(local.lastRun)} · 최근 저장 {fmtTime(local.lastSave)}</div>
        )}
        {local && local.games.length > 0 && (
          <div className="mt-2">
            <button type="button" onClick={() => setShowLocalGames((v) => !v)} className="text-xs text-accent underline">
              {showLocalGames ? "이 기기 게임 목록 숨기기" : `이 기기 게임 목록 보기 (${local.games.length})`}
            </button>
            {showLocalGames && (
              <ul className="mt-2 flex flex-col gap-1">
                {local.games.map((g) => (
                  <li key={g.hash} className="flex items-center justify-between gap-2 rounded border border-edge bg-surface2 px-2 py-1 text-xs">
                    <span className="truncate text-fg">{g.name}</span>
                    <span className="shrink-0 text-fg-dim">
                      {fmtBytes1(g.size)}
                      {g.save ? " · 세이브" : ""}
                    </span>
                  </li>
                ))}
              </ul>
            )}
            <p className="mt-1 text-[10px] text-fg-dim">※ 이 목록은 이 브라우저에만 있으며 서버로 전송되지 않습니다.</p>
          </div>
        )}
      </div>

      {/* OTHER devices — anonymous aggregate + inline alias edit */}
      <div>
        <h4 className="text-xs font-semibold text-fg-dim">다른 기기 ({others.length})</h4>
        <ul className="mt-1 flex flex-col gap-2">
          {others.length === 0 && <li className="text-xs text-fg-dim">다른 기기에서 로그인한 기록이 없습니다.</li>}
          {others.map((d) => (
            <li key={d.device_id} className="rounded-lg border border-edge bg-surface2 px-3 py-2">
              <div className="flex items-center justify-between gap-2">
                <div className="truncate font-medium text-fg">{d.label || "(이름 없는 기기)"}</div>
                <AliasEditor value={d.label || ""} onSave={(label) => renameOther(d.device_id, label)} toast={toast} />
              </div>
              <div className="text-xs text-fg-dim">게임 {d.item_count}개 · {fmtBytes1(d.total_bytes)} · 세이브 슬롯 {d.slot_count}개</div>
              <div className="text-xs text-fg-dim">최근 로그인 {fmtTime(d.last_login_at)} · 최근 접속 {fmtTime(d.last_seen_at)}</div>
              <div className="text-[10px] text-fg-dim">최근 실행 {fmtTime(d.last_run_at)} · 최근 저장 {fmtTime(d.last_save_at)}</div>
            </li>
          ))}
        </ul>
        <p className="mt-2 text-[10px] text-fg-dim">서버에는 기기별 게임 <em>개수·용량</em>과 접속 시각만 저장되며, 어떤 게임인지(파일명/제목)는 저장되지 않습니다.</p>
      </div>
    </section>
  );
}
