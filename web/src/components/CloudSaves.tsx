import { useCallback, useEffect, useState } from "react";
import type { User, CloudSave, DeviceSlot } from "../lib/api";
import * as sync from "../lib/saveSync";
import * as lib from "../lib/library";

interface Props {
  user: User | null;
  toast: (msg: string, kind?: "ok" | "err") => void;
}

function fmtBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

export function CloudSaves({ user, toast }: Props) {
  const [slots, setSlots] = useState<CloudSave[]>([]);
  const [devices, setDevices] = useState<DeviceSlot[]>([]);

  const refresh = useCallback(async () => {
    if (!user) return;
    try {
      const [s, d] = await Promise.all([sync.listCloud(), sync.listCloudDevices()]);
      setSlots(s);
      setDevices(d);
    } catch (e) {
      toast(`동기화 정보를 불러오지 못했습니다: ${(e as Error).message}`, "err");
    }
  }, [user, toast]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const attach = useCallback(
    async (slotId: string) => {
      const games = await lib.listGames();
      if (!games.length) return toast("먼저 이 기기에 게임을 추가하세요", "err");
      const choices = games.map((g, i) => `${i + 1}. ${g.name}`).join("\n");
      const pick = prompt(`이 세이브를 적용할 로컬 게임 번호:\n${choices}`);
      const idx = parseInt(pick ?? "", 10) - 1;
      if (isNaN(idx) || !games[idx]) return;
      try {
        await sync.attachCloudToGame(slotId, games[idx].hash);
        toast("로컬 게임에 세이브 적용됨", "ok");
      } catch (e) {
        toast(`적용 실패: ${(e as Error).message}`, "err");
      }
    },
    [toast],
  );

  const del = useCallback(
    async (slotId: string) => {
      if (!confirm("이 클라우드 슬롯을 삭제할까요?")) return;
      await sync.deleteCloud(slotId);
      await refresh();
      toast("삭제됨");
    },
    [refresh, toast],
  );

  if (!user) {
    return (
      <section className="w-full max-w-xl">
        <h2 className="text-lg font-semibold text-slate-100 mb-2">세이브 동기화</h2>
        <div className="rounded-lg border border-dashed border-slate-700 bg-slate-800/40 p-4 text-slate-400">
          로그인하면 다른 기기와 세이브를 동기화할 수 있습니다.
        </div>
      </section>
    );
  }

  return (
    <section className="w-full max-w-xl flex flex-col gap-4">
      <h2 className="text-lg font-semibold text-slate-100">세이브 동기화</h2>
      <p className="text-sm text-slate-400">
        세이브 데이터만 서버에 저장됩니다. 어떤 게임의 세이브인지 서버는 알 수 없으며, 슬롯은 사용자가 정한 별칭으로만 식별됩니다.
      </p>

      <h3 className="text-sm font-semibold text-slate-300">내 클라우드 슬롯</h3>
      <ul className="flex flex-col gap-2">
        {slots.length === 0 && <li className="text-sm text-slate-500">아직 업로드한 세이브가 없습니다.</li>}
        {slots.map((s) => (
          <li key={s.id} className="flex items-center gap-3 rounded-lg border border-slate-700 bg-slate-800/40 px-3 py-2">
            <div className="min-w-0 flex-1">
              <div className="font-medium text-slate-100 truncate">{s.slot_label}</div>
              <div className="text-xs text-slate-500">
                {s.device_label} · {fmtBytes(s.payload_bytes)} · {new Date(s.updated_at).toLocaleString()}
              </div>
            </div>
            <button type="button" onClick={() => void attach(s.id)} className="rounded-md bg-slate-700 hover:bg-slate-600 px-2 py-1 text-xs text-slate-200">
              로컬 게임에 적용
            </button>
            <button type="button" onClick={() => void del(s.id)} className="rounded-md bg-slate-700 hover:bg-red-700/70 px-2 py-1 text-xs text-red-300">
              삭제
            </button>
          </li>
        ))}
      </ul>

      <h3 className="text-sm font-semibold text-slate-300">다른 기기 현황</h3>
      <p className="text-xs text-slate-500">게임 정보 없이, 세이브 슬롯 개수와 갱신 시각만 표시됩니다.</p>
      <ul className="flex flex-col gap-2">
        {devices.length === 0 && <li className="text-sm text-slate-500">없음</li>}
        {devices.map((d) => (
          <li key={d.device_label} className="rounded-lg border border-slate-700 bg-slate-800/40 px-3 py-2">
            <div className="font-medium text-slate-100">{d.device_label}</div>
            <div className="text-xs text-slate-500">
              세이브 슬롯 {d.slot_count}개 · 갱신 {new Date(d.last_updated).toLocaleString()}
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
