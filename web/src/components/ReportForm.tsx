import { useState } from "react";
import { reports, ApiError } from "../lib/api";

// Rights-holder takedown / infringement notice. Intentionally usable WITHOUT an
// account (a rights holder need not register). It is an intake channel only — it
// grants no access to anyone's files and exposes no listing (S5 holds). An
// operator reviews submissions and, when a notice is upheld, disables the target
// file and applies the repeat-infringer policy (see docs/COMPLIANCE.md).
export function ReportForm({ toast }: { toast: (msg: string, kind?: "ok" | "err") => void }) {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [contact, setContact] = useState("");
  const [work, setWork] = useState("");
  const [statement, setStatement] = useState("");
  const [hint, setHint] = useState("");
  const [busy, setBusy] = useState(false);
  const [done, setDone] = useState(false);

  const submit = async () => {
    if (statement.trim().length < 10) return toast("침해 내용을 조금 더 자세히 적어 주세요", "err");
    setBusy(true);
    try {
      await reports.create({ reporter_name: name, reporter_contact: contact, work_title: work, statement, target_hint: hint });
      setDone(true);
      toast("신고가 접수되었습니다. 검토 후 조치하겠습니다.", "ok");
    } catch (e) {
      const err = e as ApiError;
      toast(err.code === "reports_not_ready" ? "신고 기능이 아직 활성화되지 않았습니다" : `접수 실패: ${err.message}`, "err");
    } finally {
      setBusy(false);
    }
  };

  if (!open) {
    return (
      <button type="button" onClick={() => setOpen(true)} className="text-accent underline hover:opacity-80">
        권리 침해 신고 (삭제 요청)
      </button>
    );
  }

  if (done) {
    return <p className="text-sm text-emerald-600 dark:text-emerald-300">신고가 접수되었습니다. 검토 후 대상 파일을 비활성화하는 등 조치하겠습니다. 추가 확인이 필요하면 남겨 주신 연락처로 연락드립니다.</p>;
  }

  const input = "w-full rounded-md border border-edge bg-surface px-3 py-2 text-sm text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent";
  return (
    <div className="flex flex-col gap-2">
      <p className="text-xs text-fg-dim">
        권리자(또는 대리인)께서는 아래에 침해 내용을 적어 주세요. 접수된 신고는 검토 후 대상 파일을 즉시 비활성화하며, 반복 침해 계정은 이용이 제한·해지될 수 있습니다.
        본 서비스는 게임 파일을 공개·공유·배포하지 않으며, 업로드 파일은 업로더 본인만 접근할 수 있습니다.
      </p>
      <input className={input} placeholder="신고자 이름/단체 (선택)" value={name} onChange={(e) => setName(e.target.value)} maxLength={200} />
      <input className={input} placeholder="연락처 이메일 (선택, 회신용)" value={contact} onChange={(e) => setContact(e.target.value)} maxLength={200} />
      <input className={input} placeholder="침해된 저작물 제목 (선택)" value={work} onChange={(e) => setWork(e.target.value)} maxLength={300} />
      <textarea className={`${input} min-h-24`} placeholder="침해 내용 / 권리 보유에 대한 선의의 진술 (필수)" value={statement} onChange={(e) => setStatement(e.target.value)} maxLength={8000} />
      <input className={input} placeholder="식별에 도움이 될 정보 (파일명 등, 선택)" value={hint} onChange={(e) => setHint(e.target.value)} maxLength={1000} />
      <div className="flex gap-2">
        <button type="button" onClick={() => void submit()} disabled={busy} className="rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-fg hover:bg-accent-hover disabled:opacity-60">
          {busy ? "접수 중…" : "신고 접수"}
        </button>
        <button type="button" onClick={() => setOpen(false)} className="text-sm text-fg-dim hover:text-fg">취소</button>
      </div>
    </div>
  );
}
