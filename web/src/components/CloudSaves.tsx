import { useCallback, useEffect, useState } from "react";
import { type User, type CloudSave, type SavesUsage, saves as savesApi } from "../lib/api";
import * as sync from "../lib/saveSync";
import { SAVE_SERVER_CAP_BYTES, fmtBytes1 } from "../lib/limits";

interface Props {
  user: User | null;
  toast: (msg: string, kind?: "ok" | "err") => void;
}

export function CloudSaves({ user, toast }: Props) {
  const [slots, setSlots] = useState<CloudSave[]>([]);
  const [usage, setUsage] = useState<SavesUsage>({ used: 0, quota: SAVE_SERVER_CAP_BYTES });

  const refresh = useCallback(async () => {
    if (!user) return;
    try {
      const res = await savesApi.list();
      setSlots(res.saves);
      setUsage(res.usage);
    } catch (e) {
      toast(`동기화 정보를 불러오지 못했습니다: ${(e as Error).message}`, "err");
    }
  }, [user, toast]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const del = useCallback(
    async (slotId: string) => {
      if (!confirm("이 세이브를 서버에서 삭제할까요? (이 기기의 로컬 세이브는 남습니다)")) return;
      await sync.deleteCloud(slotId);
      await refresh();
      toast("서버 세이브 삭제됨");
    },
    [refresh, toast],
  );

  // F1: revert one step to the retained previous version (recovery safety net).
  const revert = useCallback(
    async (romHash: string) => {
      if (!confirm("이 세이브를 직전 버전으로 되돌릴까요? (현재 버전은 안전망에 보관됩니다)")) return;
      try {
        await savesApi.revert(romHash);
        await refresh();
        toast("직전 버전으로 되돌렸습니다", "ok");
      } catch (e) {
        toast(`되돌리기 실패: ${(e as Error).message}`, "err");
      }
    },
    [refresh, toast],
  );

  if (!user) {
    return (
      <section className="w-full max-w-xl">
        <h2 className="mb-2 text-lg font-semibold text-fg">세이브 동기화</h2>
        <div className="rounded-lg border border-dashed border-edge bg-surface2 p-4 text-fg-dim">로그인하면 세이브가 서버에 자동 저장되고 다른 기기와 동기화됩니다. 미로그인 상태의 세이브는 이 기기에만 보관됩니다.</div>
      </section>
    );
  }

  const pct = usage.quota ? Math.min(100, Math.round((usage.used / usage.quota) * 100)) : 0;
  const bar = pct >= 90 ? "bg-red-500" : pct >= 70 ? "bg-amber-500" : "bg-emerald-500";

  return (
    <section className="w-full max-w-xl flex flex-col gap-4">
      <h2 className="text-lg font-semibold text-fg">세이브 동기화</h2>
      <p className="text-sm text-fg-dim">
        세이브는 <strong>게임(롬) 단위로 자동 저장·동기화</strong>됩니다 — 같은 롬이면 이 기기든 다른 기기든, 로컬이든 서버든 항상 같은 세이브를 이어서 합니다. 슬롯을 직접 고를 필요가 없습니다.
      </p>

      {/* save capacity (server, 100MB, fixed) */}
      <div className="rounded-lg border border-emerald-500/40 bg-emerald-500/10 px-3 py-2">
        <div className="mb-1 flex items-center justify-between text-xs">
          <span className="text-emerald-700 dark:text-emerald-200">
            <span className="font-medium">{fmtBytes1(usage.used)}</span> / {fmtBytes1(usage.quota)} · 서버 세이브 사용량
          </span>
          <span className="text-emerald-700/80 dark:text-emerald-200/80 tabular-nums">{pct}%</span>
        </div>
        <div className="h-2 w-full overflow-hidden rounded-full bg-surface" role="progressbar" aria-valuenow={pct} aria-valuemin={0} aria-valuemax={100} aria-label="서버 세이브 사용량">
          <div className={`h-full ${bar} transition-all`} style={{ width: `${pct}%` }} />
        </div>
      </div>

      <h3 className="text-sm font-semibold text-fg-dim">서버에 저장된 세이브</h3>
      <ul className="flex flex-col gap-2">
        {slots.length === 0 && <li className="text-sm text-fg-dim">아직 서버에 저장된 세이브가 없습니다. 로그인 상태로 게임을 플레이하면 자동 저장됩니다.</li>}
        {slots.map((s) => (
          <li key={s.id} className="flex items-center gap-3 rounded-lg border border-edge bg-surface2 px-3 py-2">
            <div className="min-w-0 flex-1">
              <div className="truncate font-medium text-fg">{s.slot_label || "자동저장"}</div>
              <div className="text-xs text-fg-dim">
                {s.device_label ? `${s.device_label} · ` : ""}
                {fmtBytes1(s.payload_bytes)} · {new Date(s.updated_at).toLocaleString()}
              </div>
            </div>
            {(s.prev_updated_at ?? 0) > 0 && (
              <button type="button" onClick={() => void revert(s.rom_hash)} title={`직전 버전(${new Date(s.prev_updated_at as number).toLocaleString()})으로 되돌리기`} className="rounded-md border border-edge bg-surface px-2 py-1 text-xs text-fg-dim hover:text-fg hover:border-accent">
                ↺ 되돌리기
              </button>
            )}
            <button type="button" onClick={() => void del(s.id)} className="rounded-md border border-edge bg-surface px-2 py-1 text-xs text-red-500 hover:bg-red-500/15">
              삭제
            </button>
          </li>
        ))}
      </ul>
      <p className="text-xs text-fg-dim">기기별 현황(개수·용량·접속 시각)은 우측 상단 프로필 → 계정 화면에서 볼 수 있습니다.</p>
    </section>
  );
}
