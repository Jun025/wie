import { useCallback, useEffect, useState } from "react";
import { devices as devicesApi, type Device } from "../lib/api";
import { deviceId, localDeviceSummary, sendHeartbeat } from "../lib/device";
import { deviceName, setDeviceName } from "../lib/saveSync";

function fmtBytes(n: number): string {
  if (!n) return "0 B";
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}
function fmtTime(t?: number): string {
  return t ? new Date(t).toLocaleString() : "—";
}

type LocalSummary = Awaited<ReturnType<typeof localDeviceSummary>>;

// Device status. The CURRENT device shows full local detail (filenames, sizes)
// read straight from IndexedDB — those NEVER leave the browser. OTHER devices
// show only the server's ANONYMOUS aggregate (count · MB · timestamps); the
// server has no filenames/titles, so it cannot show "which games".
export function DeviceStatus({ toast }: { toast: (msg: string, kind?: "ok" | "err") => void }) {
  const [devices, setDevices] = useState<Device[]>([]);
  const [local, setLocal] = useState<LocalSummary | null>(null);
  const [name, setName] = useState(deviceName());
  const [showLocalGames, setShowLocalGames] = useState(false);
  const thisId = deviceId();

  const refresh = useCallback(async () => {
    setLocal(await localDeviceSummary());
    try {
      await sendHeartbeat(); // make sure this device exists/updated before listing
      setDevices((await devicesApi.list()).devices);
    } catch (e) {
      toast(`기기 정보를 불러오지 못했습니다: ${(e as Error).message}`, "err");
    }
  }, [toast]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const saveName = useCallback(async () => {
    setDeviceName(name.trim());
    await sendHeartbeat();
    await refresh();
    toast("기기 별칭 저장됨");
  }, [name, refresh, toast]);

  const others = devices.filter((d) => d.device_id !== thisId);
  const mine = devices.find((d) => d.device_id === thisId);

  return (
    <section className="flex flex-col gap-4">
      <div>
        <h3 className="text-sm font-semibold text-fg">내 기기 현황</h3>
        <p className="mt-1 text-xs text-fg-dim">
          이 기기의 게임 목록은 브라우저에서만 읽어 표시됩니다. 다른 기기는 서버에 파일명이 없으므로 <strong>개수·용량·시각</strong>만 표시됩니다.
        </p>
      </div>

      <label className="flex flex-col gap-1 text-sm text-fg-dim">
        이 기기 별칭
        <div className="flex gap-2">
          <input
            value={name}
            maxLength={60}
            onChange={(e) => setName(e.target.value)}
            className="flex-1 rounded-md border border-edge bg-surface2 px-3 py-2 text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
          />
          <button type="button" onClick={() => void saveName()} className="rounded-md border border-edge bg-surface2 px-3 py-2 text-sm text-fg hover:border-accent">
            저장
          </button>
        </div>
      </label>

      {/* THIS device — full local detail */}
      <div className="rounded-lg border border-accent/40 bg-accent/5 p-3">
        <div className="flex items-center justify-between">
          <div className="font-medium text-fg">{name || "이 기기"} <span className="ml-1 rounded bg-accent/20 px-1.5 py-0.5 text-[10px] text-accent">이 기기</span></div>
          <div className="text-xs text-fg-dim">{local ? `${local.itemCount}개 · ${fmtBytes(local.totalBytes)}` : "…"}</div>
        </div>
        {local && (
          <div className="mt-1 text-xs text-fg-dim">
            최근 실행 {fmtTime(local.lastRun)} · 최근 저장 {fmtTime(local.lastSave)}
          </div>
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
                      {fmtBytes(g.size)}
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

      {/* OTHER devices — anonymous aggregate only */}
      <div>
        <h4 className="text-xs font-semibold text-fg-dim">다른 기기 ({others.length})</h4>
        <ul className="mt-1 flex flex-col gap-2">
          {others.length === 0 && <li className="text-xs text-fg-dim">다른 기기에서 로그인한 기록이 없습니다.</li>}
          {others.map((d) => (
            <li key={d.device_id} className="rounded-lg border border-edge bg-surface2 px-3 py-2">
              <div className="font-medium text-fg">{d.label || "(이름 없는 기기)"}</div>
              <div className="text-xs text-fg-dim">
                게임 {d.item_count}개 · {fmtBytes(d.total_bytes)} · 세이브 슬롯 {d.slot_count}개
              </div>
              <div className="text-xs text-fg-dim">
                최근 로그인 {fmtTime(d.last_login_at)} · 최근 접속 {fmtTime(d.last_seen_at)}
              </div>
              <div className="text-[10px] text-fg-dim">최근 실행 {fmtTime(d.last_run_at)} · 최근 저장 {fmtTime(d.last_save_at)}</div>
            </li>
          ))}
        </ul>
        {mine && <p className="mt-2 text-[10px] text-fg-dim">서버에는 기기별 게임 <em>개수·용량</em>과 접속 시각만 저장되며, 어떤 게임인지(파일명/제목)는 저장되지 않습니다.</p>}
      </div>
    </section>
  );
}
