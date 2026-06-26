import { useCallback, useEffect, useRef, useState } from "react";
import { inquiries, reports, ApiError, type Inquiry, type InquiryAttachment, type User } from "../lib/api";
import { envInfoText } from "../lib/device";

interface Props {
  user: User | null;
  toast: (msg: string, kind?: "ok" | "err") => void;
}

const MAX_ATTACH = 96 * 1024;
const ALLOWED_MIME = /^(image\/(png|jpeg|gif|webp|bmp)|text\/plain)$/;
const BLOCKED_EXT = /\.(jar|jad|zip|kdf|skm|mod|smc|gba|nes|class|exe|dll|so|bin|apk|7z|rar|gz)$/i;
// Magic numbers of game/exec/archive files — blocked even if mislabeled.
const BLOCKED_MAGICS: number[][] = [
  [0x50, 0x4b, 0x03, 0x04],
  [0xca, 0xfe, 0xba, 0xbe],
  [0x7f, 0x45, 0x4c, 0x46],
  [0x4d, 0x5a],
  [0x1f, 0x8b],
  [0x52, 0x61, 0x72, 0x21],
  [0x37, 0x7a, 0xbc, 0xaf],
];

function bytesToB64(bytes: Uint8Array): string {
  let bin = "";
  for (let i = 0; i < bytes.length; i += 0x8000) bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(bin);
}

// Validate + read an attachment client-side (mirrors the server's 415 policy).
async function readAttachment(file: File): Promise<InquiryAttachment> {
  const buf = new Uint8Array(await file.arrayBuffer());
  if (buf.length > MAX_ATTACH) throw new Error("첨부 파일이 너무 큽니다 (최대 96KB)");
  if (BLOCKED_EXT.test(file.name)) throw new Error("게임/실행 파일은 첨부할 수 없습니다");
  if (!ALLOWED_MIME.test(file.type || "")) throw new Error("이미지·텍스트·로그 파일만 첨부할 수 있습니다");
  if (BLOCKED_MAGICS.some((m) => m.every((b, i) => buf[i] === b))) throw new Error("게임/실행 파일은 첨부할 수 없습니다");
  return { name: file.name, mime: file.type, data: bytesToB64(buf) };
}

// Evidence for a takedown report (7번): allow image / PDF / text proof up to 256KB;
// game/exec/archive payloads are still blocked. Mirrors the server's /api/reports policy.
const MAX_EVIDENCE = 256 * 1024;
const EVIDENCE_MIME = /^(image\/(png|jpeg|gif|webp|bmp)|application\/pdf|text\/plain)$/;
async function readEvidence(file: File): Promise<InquiryAttachment> {
  const buf = new Uint8Array(await file.arrayBuffer());
  if (buf.length > MAX_EVIDENCE) throw new Error("증빙 파일이 너무 큽니다 (최대 256KB)");
  if (BLOCKED_EXT.test(file.name)) throw new Error("게임/실행/압축 파일은 증빙으로 첨부할 수 없습니다");
  if (!EVIDENCE_MIME.test(file.type || "")) throw new Error("증빙은 이미지·PDF·텍스트만 첨부할 수 있습니다");
  if (BLOCKED_MAGICS.some((m) => m.every((b, i) => buf[i] === b))) throw new Error("게임/실행/압축 파일은 증빙으로 첨부할 수 없습니다");
  return { name: file.name, mime: file.type, data: bytesToB64(buf) };
}

type Kind = "inquiry" | "report";

// Unified 문의·신고 screen — one screen, a type toggle:
//   • 문의·건의 — logged-in members; env auto-attached; image/text/log attachment
//                  (game/exec files blocked, 415). 게임 식별정보 미포함.
//   • 권리 침해 신고·삭제요청 — anonymous-allowed (no login); intake only, no file
//                  access; game/exec attachments are not part of this form.
export function InquiryForm({ user, toast }: Props) {
  const [kind, setKind] = useState<Kind>("inquiry");

  // inquiry fields
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [attach, setAttach] = useState<InquiryAttachment | null>(null);
  const [showEnv, setShowEnv] = useState(false);
  const [list, setList] = useState<Inquiry[]>([]);
  const fileRef = useRef<HTMLInputElement>(null);
  const env = envInfoText();

  // report (권리 침해 신고·삭제요청) fields — standard takedown intake
  const [rType, setRType] = useState<"owner" | "agent">("owner");
  const [rName, setRName] = useState("");
  const [rContact, setRContact] = useState("");
  const [rWork, setRWork] = useState("");
  const [rBasis, setRBasis] = useState("");
  const [rStatement, setRStatement] = useState("");
  const [rHint, setRHint] = useState("");
  const [rGoodFaith, setRGoodFaith] = useState(false);
  const [rEvidence, setREvidence] = useState<InquiryAttachment | null>(null);
  const [rShowEnv, setRShowEnv] = useState(false);
  const reFileRef = useRef<HTMLInputElement>(null);
  const [reportDone, setReportDone] = useState(false);

  const [busy, setBusy] = useState(false);

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

  const onPickFile = async (f: File | undefined) => {
    if (!f) return setAttach(null);
    try {
      setAttach(await readAttachment(f));
    } catch (e) {
      setAttach(null);
      if (fileRef.current) fileRef.current.value = "";
      toast((e as Error).message, "err");
    }
  };

  const submitInquiry = async (e: React.FormEvent) => {
    e.preventDefault();
    setBusy(true);
    try {
      await inquiries.create({ title: title.trim(), body: body.trim(), env_info: env, attachment: attach });
      toast("문의가 접수되었습니다 (환경정보 자동 첨부)", "ok");
      setTitle("");
      setBody("");
      setAttach(null);
      if (fileRef.current) fileRef.current.value = "";
      await refresh();
    } catch (err) {
      toast(`전송 실패: ${(err as Error).message}`, "err");
    } finally {
      setBusy(false);
    }
  };

  const onPickEvidence = async (f: File | undefined) => {
    if (!f) return setREvidence(null);
    try {
      setREvidence(await readEvidence(f));
    } catch (e) {
      setREvidence(null);
      if (reFileRef.current) reFileRef.current.value = "";
      toast((e as Error).message, "err");
    }
  };

  const submitReport = async (e: React.FormEvent) => {
    e.preventDefault();
    if (rStatement.trim().length < 10) return toast("침해 내용을 조금 더 자세히 적어 주세요", "err");
    if (!rName.trim()) return toast("신고자 성명/법인명을 입력해 주세요", "err");
    if (!rContact.trim()) return toast("회신용 연락처(이메일)를 입력해 주세요", "err");
    if (!rGoodFaith) return toast("선의·정확성에 대한 진술에 동의해 주세요", "err");
    setBusy(true);
    try {
      await reports.create({
        reporter_type: rType,
        reporter_name: rName,
        reporter_contact: rContact,
        work_title: rWork,
        right_basis: rBasis,
        statement: rStatement,
        target_hint: rHint,
        good_faith: rGoodFaith,
        env_info: env,
        attachment: rEvidence,
      });
      setReportDone(true);
      toast("신고가 접수되었습니다. 검토 후 조치하겠습니다.", "ok");
    } catch (err) {
      const e2 = err as ApiError;
      toast(e2.code === "reports_not_ready" ? "신고 기능이 아직 활성화되지 않았습니다" : `접수 실패: ${e2.message}`, "err");
    } finally {
      setBusy(false);
    }
  };

  const input = "rounded-md bg-surface2 border border-edge px-3 py-2 text-fg text-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent";
  const tab = (k: Kind, label: string) => (
    <button
      type="button"
      onClick={() => setKind(k)}
      aria-pressed={kind === k}
      className={"rounded-md px-3 py-1.5 text-sm " + (kind === k ? "bg-accent text-accent-fg" : "bg-surface2 text-fg-dim hover:text-fg")}
    >
      {label}
    </button>
  );

  return (
    <section className="w-full max-w-xl flex flex-col gap-4">
      <h2 className="text-lg font-semibold text-fg">문의 · 신고</h2>

      <div className="flex flex-wrap gap-2" role="tablist" aria-label="문의/신고 유형">
        {tab("inquiry", "문의 · 건의")}
        {tab("report", "권리 침해 신고 · 삭제요청")}
      </div>

      {/* ── 권리 침해 신고 (비로그인 접수 가능) ──────────────────────────────── */}
      {kind === "report" &&
        (reportDone ? (
          <p className="text-sm text-emerald-600 dark:text-emerald-300">신고가 접수되었습니다. 검토 후 대상 파일을 비활성화하는 등 조치하겠습니다. 추가 확인이 필요하면 남겨 주신 연락처로 연락드립니다.</p>
        ) : (
          <form onSubmit={submitReport} className="flex flex-col gap-2.5">
            <p className="text-xs text-fg-dim">
              권리자(또는 대리인)는 <strong>로그인 없이</strong> 신고할 수 있습니다. 접수된 신고는 검토 후 대상 파일을 비활성화하며, 반복 침해 계정은 이용이 제한·해지될 수 있습니다. 본 서비스는 게임 파일을 공개·공유·배포하지 않으며 업로드 파일은 업로더 본인만 접근합니다.
              <br />아래 정보는 <strong>신고 처리(회신·진위 확인) 목적으로만</strong> 운영자가 사용하며 외부에 공개되지 않습니다.
            </p>

            {/* 신고 자격 */}
            <fieldset className="flex flex-wrap items-center gap-3 text-sm text-fg">
              <legend className="mb-1 text-xs text-fg-dim">신고 자격 *</legend>
              <label className="flex items-center gap-1.5">
                <input type="radio" name="rtype" checked={rType === "owner"} onChange={() => setRType("owner")} className="accent-accent" /> 권리자 본인
              </label>
              <label className="flex items-center gap-1.5">
                <input type="radio" name="rtype" checked={rType === "agent"} onChange={() => setRType("agent")} className="accent-accent" /> 대리인(위임받음)
              </label>
            </fieldset>

            <input className={input} placeholder="신고자 성명 / 법인명 *" value={rName} onChange={(e) => setRName(e.target.value)} maxLength={200} required />
            <input className={input} type="email" placeholder="회신용 연락처 이메일 *" value={rContact} onChange={(e) => setRContact(e.target.value)} maxLength={200} required />
            <input className={input} placeholder="침해된 저작물 제목" value={rWork} onChange={(e) => setRWork(e.target.value)} maxLength={300} />
            <textarea className={`${input} min-h-16`} placeholder="권리 근거 / 권원 (저작권·상표권 등 보유 근거, 등록번호 등)" value={rBasis} onChange={(e) => setRBasis(e.target.value)} maxLength={2000} />
            <textarea className={`${input} min-h-24`} placeholder="침해 주장 내용 — 어떤 권리가 어떻게 침해되었는지 구체적으로 (필수)" value={rStatement} onChange={(e) => setRStatement(e.target.value)} maxLength={8000} required />
            <input className={input} placeholder="대상 식별정보 (파일명·업로더 단서 등, 알고 있으면)" value={rHint} onChange={(e) => setRHint(e.target.value)} maxLength={1000} />

            {/* 증빙 첨부 */}
            <div className="flex flex-col gap-1 text-sm text-fg-dim">
              증빙 자료 첨부 (선택 · 이미지/PDF/텍스트, 최대 256KB)
              <input
                ref={reFileRef}
                type="file"
                accept="image/png,image/jpeg,image/gif,image/webp,image/bmp,application/pdf,text/plain,.txt,.pdf"
                onChange={(e) => void onPickEvidence(e.target.files?.[0])}
                className="text-xs text-fg-dim file:mr-2 file:rounded file:border file:border-edge file:bg-surface2 file:px-2 file:py-1 file:text-fg"
              />
              {rEvidence && <span className="text-xs text-emerald-600 dark:text-emerald-300">첨부됨: {rEvidence.name} ({rEvidence.mime})</span>}
              <span className="text-[11px] text-fg-dim">게임/실행/압축 파일은 증빙으로 첨부할 수 없습니다. 첨부는 운영자만 열람합니다(공개 URL 없음).</span>
            </div>

            {/* 환경정보 자동 첨부 (문의와 동일) */}
            <div className="rounded-md border border-edge bg-surface2 px-3 py-2 text-xs text-fg-dim">
              <button type="button" onClick={() => setRShowEnv((v) => !v)} className="underline">
                {rShowEnv ? "자동 첨부 환경정보 숨기기" : "자동으로 첨부될 환경정보 보기"}
              </button>
              {rShowEnv && <pre className="mt-2 max-h-40 overflow-auto whitespace-pre-wrap break-words text-[11px] text-fg">{env}</pre>}
              <p className="mt-1">접수 시 신고자 환경정보가 자동 첨부됩니다(처리 참고용).</p>
            </div>

            {/* 선의·정확성 진술 */}
            <label className="flex items-start gap-2 text-xs text-fg">
              <input type="checkbox" checked={rGoodFaith} onChange={(e) => setRGoodFaith(e.target.checked)} className="mt-0.5 accent-accent" required />
              <span>본인은 위 신고 내용이 사실이며 정당한 권리에 근거함을 선의로 진술하고, 본인이 권리자 또는 그 적법한 대리인임을 확인합니다. (허위 신고 시 책임을 질 수 있음을 이해합니다.) *</span>
            </label>

            <button type="submit" disabled={busy} className="self-start rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-fg hover:opacity-90 disabled:opacity-60">
              {busy ? "접수 중…" : "신고 접수"}
            </button>
          </form>
        ))}

      {/* ── 문의·건의 (로그인 회원) ─────────────────────────────────────────── */}
      {kind === "inquiry" &&
        (!user ? (
          <div className="rounded-lg border border-dashed border-edge bg-surface2 p-4 text-sm text-fg-dim">
            문의·건의는 로그인 회원만 작성할 수 있습니다. (권리 침해 신고는 위 “권리 침해 신고” 탭에서 로그인 없이 접수할 수 있어요.)
          </div>
        ) : (
          <>
            <form onSubmit={submitInquiry} className="flex flex-col gap-3">
              <label className="flex flex-col gap-1 text-sm text-fg-dim">
                제목
                <input value={title} onChange={(e) => setTitle(e.target.value)} maxLength={200} required className={input} />
              </label>
              <label className="flex flex-col gap-1 text-sm text-fg-dim">
                내용
                <textarea value={body} onChange={(e) => setBody(e.target.value)} rows={6} maxLength={8000} required className={input} />
              </label>

              <div className="flex flex-col gap-1 text-sm text-fg-dim">
                파일 첨부 (선택 · 이미지/텍스트/로그만)
                <input
                  ref={fileRef}
                  type="file"
                  accept="image/png,image/jpeg,image/gif,image/webp,image/bmp,text/plain,.txt,.log"
                  onChange={(e) => void onPickFile(e.target.files?.[0])}
                  className="text-xs text-fg-dim file:mr-2 file:rounded file:border file:border-edge file:bg-surface2 file:px-2 file:py-1 file:text-fg"
                />
                {attach && <span className="text-xs text-emerald-600 dark:text-emerald-300">첨부됨: {attach.name} ({attach.mime})</span>}
                <span className="text-[11px] text-fg-dim">게임/실행 파일(JAR·ROM·ZIP 등)은 첨부할 수 없으며 서버가 거부합니다(415). 최대 96KB.</span>
              </div>

              <div className="rounded-md border border-edge bg-surface2 px-3 py-2 text-xs text-fg-dim">
                <button type="button" onClick={() => setShowEnv((v) => !v)} className="underline">
                  {showEnv ? "자동 첨부 환경정보 숨기기" : "자동으로 첨부될 환경정보 보기"}
                </button>
                {showEnv && <pre className="mt-2 max-h-40 overflow-auto whitespace-pre-wrap break-words text-[11px] text-fg">{env}</pre>}
                <p className="mt-1">접수 시 위 환경정보가 자동 첨부됩니다. 게임 파일명·제목 등 게임 식별정보는 포함되지 않습니다.</p>
              </div>

              <button type="submit" disabled={busy} className="self-start rounded-md bg-accent hover:opacity-90 disabled:opacity-60 px-4 py-2 font-medium text-accent-fg">
                {busy ? "전송 중…" : "보내기"}
              </button>
            </form>

            <h3 className="text-sm font-semibold text-fg-dim">내 문의 내역</h3>
            <ul className="flex flex-col gap-2">
              {list.length === 0 && <li className="text-sm text-fg-dim">내역이 없습니다.</li>}
              {list.map((q) => (
                <li key={q.id} className="rounded-lg border border-edge bg-surface2 px-3 py-2">
                  <div className="font-medium text-fg">
                    {q.title}
                    {q.has_attachment ? <span className="ml-2 rounded bg-surface px-1.5 py-0.5 text-[10px] text-fg-dim">📎 첨부</span> : null}
                  </div>
                  <div className="text-xs text-fg-dim">
                    {new Date(q.created_at).toLocaleString()} · {q.status}
                  </div>
                </li>
              ))}
            </ul>
          </>
        ))}
    </section>
  );
}
