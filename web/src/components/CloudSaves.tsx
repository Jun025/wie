import { useCallback, useEffect, useState } from "react";
import type { User, CloudSave } from "../lib/api";
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

  const refresh = useCallback(async () => {
    if (!user) return;
    try {
      setSlots(await sync.listCloud());
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
        <h2 className="mb-2 text-lg font-semibold text-fg">세이브 동기화</h2>
        <div className="rounded-lg border border-dashed border-edge bg-surface2 p-4 text-fg-dim">로그인하면 다른 기기와 세이브를 동기화할 수 있습니다.</div>
      </section>
    );
  }

  return (
    <section className="w-full max-w-xl flex flex-col gap-4">
      <h2 className="text-lg font-semibold text-fg">세이브 동기화</h2>
      <p className="text-sm text-fg-dim">
        세이브 데이터만 서버에 저장됩니다. 어떤 게임의 세이브인지 서버는 알 수 없으며, 슬롯은 사용자가 정한 별칭으로만 식별됩니다.
      </p>

      <h3 className="text-sm font-semibold text-fg-dim">내 클라우드 슬롯</h3>
      <ul className="flex flex-col gap-2">
        {slots.length === 0 && <li className="text-sm text-fg-dim">아직 업로드한 세이브가 없습니다.</li>}
        {slots.map((s) => (
          <li key={s.id} className="flex items-center gap-3 rounded-lg border border-edge bg-surface2 px-3 py-2">
            <div className="min-w-0 flex-1">
              <div className="truncate font-medium text-fg">{s.slot_label}</div>
              <div className="text-xs text-fg-dim">
                {s.device_label} · {fmtBytes(s.payload_bytes)} · {new Date(s.updated_at).toLocaleString()}
              </div>
            </div>
            <button type="button" onClick={() => void attach(s.id)} className="rounded-md border border-edge bg-surface px-2 py-1 text-xs text-fg hover:border-accent">
              로컬 게임에 적용
            </button>
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
