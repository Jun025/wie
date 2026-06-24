import { useCallback, useEffect, useState } from "react";
import { inquiries, type Inquiry, type User } from "../lib/api";

interface Props {
  user: User | null;
  toast: (msg: string, kind?: "ok" | "err") => void;
}

const CATEGORY_LABELS: Record<string, string> = {
  question: "문의",
  suggestion: "건의",
  proposal: "제안",
  rights_report: "권리신고",
};

const EMPTY = { category: "question", title: "", body: "", game_title: "", game_vendor: "", device_model: "", symptom: "" };

export function InquiryForm({ user, toast }: Props) {
  const [form, setForm] = useState({ ...EMPTY });
  const [list, setList] = useState<Inquiry[]>([]);

  const refresh = useCallback(async () => {
    if (!user) return;
    try {
      setList((await inquiries.list()).inquiries);
    } catch {
      /* ignore */
    }
  }, [user]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const set = (k: keyof typeof EMPTY) => (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) =>
    setForm((f) => ({ ...f, [k]: e.target.value }));

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await inquiries.create(form);
      toast("문의가 접수되었습니다", "ok");
      setForm({ ...EMPTY });
      await refresh();
    } catch (err) {
      toast(`전송 실패: ${(err as Error).message}`, "err");
    }
  };

  if (!user) {
    return (
      <section className="w-full max-w-xl">
        <h2 className="text-lg font-semibold text-slate-100 mb-2">문의 · 건의 · 제안</h2>
        <div className="rounded-lg border border-dashed border-slate-700 bg-slate-800/40 p-4 text-slate-400">
          문의는 로그인 회원만 작성할 수 있습니다.
        </div>
      </section>
    );
  }

  const input = "rounded-md bg-slate-800 border border-slate-700 px-3 py-2 text-slate-100 text-sm";

  return (
    <section className="w-full max-w-xl flex flex-col gap-4">
      <h2 className="text-lg font-semibold text-slate-100">문의 · 건의 · 제안</h2>
      <form onSubmit={submit} className="flex flex-col gap-3">
        <label className="flex flex-col gap-1 text-sm text-slate-400">
          분류
          <select value={form.category} onChange={set("category")} className={input}>
            <option value="question">문의</option>
            <option value="suggestion">건의</option>
            <option value="proposal">제안</option>
            <option value="rights_report">권리자 신고·삭제 요청</option>
          </select>
        </label>
        {form.category === "rights_report" && (
          <div className="rounded-md border border-amber-700/50 bg-amber-900/20 px-3 py-2 text-xs text-amber-200">
            권리자 신고·삭제 요청 창구입니다. 침해가 의심되는 콘텐츠의 설명·근거를 본문에 적어 주세요. 본 서비스는 게임 파일을 보관·배포하지 않으며(BYOF —
            파일은 이용자 브라우저에서만 처리), 침해 자료의 다운로드 링크 게시는 금지됩니다. 신고 검토 후 적절히 조치합니다.
          </div>
        )}
        <label className="flex flex-col gap-1 text-sm text-slate-400">
          제목
          <input value={form.title} onChange={set("title")} maxLength={200} required className={input} />
        </label>
        <div className="grid grid-cols-2 gap-3">
          <label className="flex flex-col gap-1 text-sm text-slate-400">
            게임 제목 <input value={form.game_title} onChange={set("game_title")} maxLength={200} placeholder="(선택) 직접 입력" className={input} />
          </label>
          <label className="flex flex-col gap-1 text-sm text-slate-400">
            출시사 <input value={form.game_vendor} onChange={set("game_vendor")} maxLength={200} placeholder="(선택)" className={input} />
          </label>
          <label className="flex flex-col gap-1 text-sm text-slate-400">
            기종 <input value={form.device_model} onChange={set("device_model")} maxLength={200} placeholder="(선택)" className={input} />
          </label>
          <label className="flex flex-col gap-1 text-sm text-slate-400">
            증상 <input value={form.symptom} onChange={set("symptom")} maxLength={2000} placeholder="(선택)" className={input} />
          </label>
        </div>
        <label className="flex flex-col gap-1 text-sm text-slate-400">
          본문
          <textarea value={form.body} onChange={set("body")} rows={6} maxLength={8000} required className={input} />
        </label>
        <p className="text-xs text-slate-500">⚠ 텍스트만 전송됩니다. 실행파일·게임파일(JAR/ROM/.mod 등)은 첨부할 수 없으며 서버가 거부합니다.</p>
        <p className="text-xs text-slate-600">본 안내·정책 문구는 법률 자문이 아니며, 서비스 운영 전 한국 지식재산권 전문 변호사의 검토를 권장합니다.</p>
        <button type="submit" className="self-start rounded-md bg-sky-600 hover:bg-sky-500 px-4 py-2 font-medium text-white">
          보내기
        </button>
      </form>

      <h3 className="text-sm font-semibold text-slate-300">내 문의 내역</h3>
      <ul className="flex flex-col gap-2">
        {list.length === 0 && <li className="text-sm text-slate-500">내역이 없습니다.</li>}
        {list.map((q) => (
          <li key={q.id} className="rounded-lg border border-slate-700 bg-slate-800/40 px-3 py-2">
            <div className="font-medium text-slate-100">
              <span className="text-xs rounded bg-slate-700 px-1.5 py-0.5 mr-2 text-slate-300">{CATEGORY_LABELS[q.category] ?? q.category}</span>
              {q.title}
            </div>
            <div className="text-xs text-slate-500">
              {new Date(q.created_at).toLocaleString()} · {q.status}
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
