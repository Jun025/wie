// /api/reports — rights-holder takedown intake (compliance, B안 전제).
//
// Anonymous-allowed: a rights holder need not have an account. This is an INTAKE
// channel only — it grants NO file access and exposes NO file listing (S5 holds).
// An operator reviews `file_reports` and, when a notice is upheld, disables the
// target file (user_files.disabled=1) and escalates the owner's strikes/status
// per the repeat-infringer policy (see docs/COMPLIANCE.md). Rate-limited.

import { ok, handleError, readJson, str, HttpError } from "../../_lib/http.js";
import { uuid } from "../../_lib/crypto.js";
import { rateLimit } from "../../_lib/ratelimit.js";

// Evidence attachment (7번): operator-only, never exposed by any URL. Allow image
// / PDF / plain-text proof; reject game/exec/archive payloads even if mislabeled.
const EVIDENCE_MIME_OK = /^(image\/(png|jpeg|gif|webp|bmp)|application\/pdf|text\/plain)$/;
const MAX_EVIDENCE_BYTES = 256 * 1024;
const BLOCKED_MAGICS = [
  [0x50, 0x4b, 0x03, 0x04], // PK.. ZIP/JAR
  [0xca, 0xfe, 0xba, 0xbe], // Java .class / Mach-O fat
  [0x7f, 0x45, 0x4c, 0x46], // ELF
  [0x4d, 0x5a], // MZ (PE/exe)
  [0x1f, 0x8b], // gzip
  [0x52, 0x61, 0x72, 0x21], // Rar!
  [0x37, 0x7a, 0xbc, 0xaf], // 7z
];

// Validate the optional evidence attachment ({ name, mime, data:base64 }). Returns
// a normalized record or null. Throws HttpError(415/413/400) on policy violations.
function checkEvidence(att) {
  if (att == null) return null;
  if (typeof att !== "object") throw new HttpError("첨부 형식이 올바르지 않습니다", 400);
  const name = str(att.name || "evidence", { name: "attachment_name", max: 200 });
  const mime = str(att.mime || "", { name: "attachment_mime", max: 100 });
  const data = str(att.data || "", { name: "attachment_data", min: 1, max: Math.ceil((MAX_EVIDENCE_BYTES * 4) / 3) + 8 });
  if (!EVIDENCE_MIME_OK.test(mime)) throw new HttpError("증빙은 이미지·PDF·텍스트만 첨부할 수 있습니다 (게임/실행 파일 불가)", 415, "bad_evidence_mime");
  let bytes;
  try {
    const bin = atob(data);
    bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  } catch {
    throw new HttpError("첨부 데이터가 올바른 base64가 아닙니다", 400);
  }
  if (bytes.byteLength === 0) throw new HttpError("빈 첨부는 보낼 수 없습니다", 400);
  if (bytes.byteLength > MAX_EVIDENCE_BYTES) throw new HttpError(`증빙 파일이 너무 큽니다 (최대 ${MAX_EVIDENCE_BYTES / 1024}KB)`, 413, "evidence_too_large");
  if (BLOCKED_MAGICS.some((m) => m.every((b, i) => bytes[i] === b))) throw new HttpError("게임/실행/압축 파일은 증빙으로 첨부할 수 없습니다", 415, "blocked_evidence");
  return { name, mime, data };
}

export async function onRequestPost(context) {
  try {
    await rateLimit(context, "report", { limit: 10, windowSec: 600 });
    const body = await readJson(context.request, 512 * 1024);

    // Core sworn statement (required) + standard takedown identity/standing fields.
    const statement = str(body.statement, { name: "statement", min: 10, max: 8000 });
    const reporterName = body.reporter_name == null ? "" : str(body.reporter_name, { name: "reporter_name", max: 200 });
    const reporterContact = body.reporter_contact == null ? "" : str(body.reporter_contact, { name: "reporter_contact", max: 200 });
    const workTitle = body.work_title == null ? "" : str(body.work_title, { name: "work_title", max: 300 });
    const targetHint = body.target_hint == null ? "" : str(body.target_hint, { name: "target_hint", max: 1000 });
    let reporterType = body.reporter_type == null ? "" : str(body.reporter_type, { name: "reporter_type", max: 16 }).toLowerCase();
    if (reporterType && reporterType !== "owner" && reporterType !== "agent") reporterType = "";
    const rightBasis = body.right_basis == null ? "" : str(body.right_basis, { name: "right_basis", max: 2000 });
    const goodFaith = body.good_faith ? 1 : 0;
    const envInfo = body.env_info == null ? "" : str(body.env_info, { name: "env_info", max: 8000 });
    const evidence = checkEvidence(body.attachment);

    // Enough-to-judge gate: a real takedown needs the sworn good-faith statement and
    // a way to reach the reporter (회신/허위 책임). 권리 근거는 본문(statement)으로도
    // 받을 수 있으나 최소한의 신원·연락·선의 진술은 요구한다.
    if (!goodFaith) throw new HttpError("선의·정확성에 대한 진술에 동의해야 신고를 접수할 수 있습니다", 400, "good_faith_required");
    if (!reporterName) throw new HttpError("신고자 성명/법인명을 입력해 주세요", 400, "reporter_name_required");
    if (!reporterContact) throw new HttpError("회신용 연락처(이메일)를 입력해 주세요", 400, "reporter_contact_required");

    const id = uuid();
    const now = Date.now();
    try {
      await context.env.DB.prepare(
        `INSERT INTO file_reports
           (id, reporter_name, reporter_contact, work_title, statement, target_hint,
            reporter_type, right_basis, good_faith, env_info,
            attachment_name, attachment_mime, attachment_data,
            status, action_log, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'open', '', ?, ?)`,
      )
        .bind(
          id, reporterName, reporterContact, workTitle, statement, targetHint,
          reporterType, rightBasis, goodFaith, envInfo,
          evidence ? evidence.name : "", evidence ? evidence.mime : "", evidence ? evidence.data : "",
          now, now,
        )
        .run();
    } catch (schemaErr) {
      // GRACEFUL: file_reports / the new columns don't exist until 0003+0007 apply.
      console.error("report insert (pre-migration?):", schemaErr && schemaErr.message);
      throw new HttpError("신고 접수 기능이 아직 활성화되지 않았습니다", 503, "reports_not_ready");
    }
    return ok({ report: { id, status: "open", created_at: now } });
  } catch (e) {
    return handleError(e);
  }
}
