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

export async function onRequestPost(context) {
  try {
    await rateLimit(context, "report", { limit: 10, windowSec: 600 });
    const body = await readJson(context.request, 16 * 1024);

    const statement = str(body.statement, { name: "statement", min: 10, max: 8000 });
    const reporterName = body.reporter_name == null ? "" : str(body.reporter_name, { name: "reporter_name", max: 200 });
    const reporterContact = body.reporter_contact == null ? "" : str(body.reporter_contact, { name: "reporter_contact", max: 200 });
    const workTitle = body.work_title == null ? "" : str(body.work_title, { name: "work_title", max: 300 });
    const targetHint = body.target_hint == null ? "" : str(body.target_hint, { name: "target_hint", max: 1000 });

    const id = uuid();
    const now = Date.now();
    try {
      await context.env.DB.prepare(
        `INSERT INTO file_reports (id, reporter_name, reporter_contact, work_title, statement, target_hint, status, action_log, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, 'open', '', ?, ?)`,
      )
        .bind(id, reporterName, reporterContact, workTitle, statement, targetHint, now, now)
        .run();
    } catch (schemaErr) {
      // GRACEFUL: file_reports doesn't exist until migration 0003 is applied.
      console.error("report insert (pre-migration?):", schemaErr && schemaErr.message);
      throw new HttpError("신고 접수 기능이 아직 활성화되지 않았습니다", 503, "reports_not_ready");
    }
    return ok({ report: { id, status: "open", created_at: now } });
  } catch (e) {
    return handleError(e);
  }
}
