// Inquiries: title + body + auto-collected environment + an OPTIONAL small
// attachment (image / log / text only). The channel must never become a
// file-distribution path ("웹하드 경계"), so:
//   • game/executable/archive files are rejected (magic-number + MIME + 415);
//   • attachments are owner-scoped and never exposed on a public URL;
//   • game identity (filename/hash/title) is never required or auto-collected
//     (1번 기준선 / S5) — env_info is built client-side with game info excluded.

import { ok, readJson, handleError, str, HttpError } from "../../_lib/http.js";
import { uuid } from "../../_lib/crypto.js";
import { requireUser } from "../../_lib/session.js";
import { rateLimit } from "../../_lib/ratelimit.js";

// Magic-number prefixes for game binaries / archives / executables. Rejected
// whether raw, base64-smuggled in text, or uploaded as an attachment.
const BINARY_MAGICS = [
  [0x50, 0x4b, 0x03, 0x04], // PK.. ZIP/JAR
  [0xca, 0xfe, 0xba, 0xbe], // Java .class / Mach-O fat
  [0x7f, 0x45, 0x4c, 0x46], // ELF
  [0x4d, 0x5a], // MZ (PE/exe)
  [0xde, 0xad, 0xbe, 0xef], // common ROM sentinel
  [0x1f, 0x8b], // gzip
  [0x52, 0x61, 0x72, 0x21], // Rar!
  [0x37, 0x7a, 0xbc, 0xaf], // 7z
];

// Attachments are limited to these MIME families: images, plain text, logs.
const ATTACH_MIME_OK = /^(image\/(png|jpeg|gif|webp|bmp)|text\/plain)$/;
const MAX_ATTACH_BYTES = 96 * 1024; // keep the whole JSON body under readJson's cap

function looksBinary(s) {
  let control = 0;
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (c === 0) return true;
    if (c < 9 || (c > 13 && c < 32)) control++;
  }
  return s.length > 0 && control / s.length > 0.1;
}

function hasMagic(bytes) {
  return BINARY_MAGICS.some((m) => m.every((b, i) => bytes[i] === b));
}

function rejectBinaryText(value, name) {
  if (looksBinary(value)) throw new HttpError(`${name}: binary content is not allowed`, 415, "binary_rejected");
  const compact = value.replace(/\s+/g, "");
  if (compact.length >= 8 && /^[A-Za-z0-9+/=]+$/.test(compact)) {
    try {
      const bin = atob(compact.slice(0, 64));
      const bytes = new Uint8Array([...bin].map((c) => c.charCodeAt(0)));
      if (hasMagic(bytes)) throw new HttpError(`${name}: encoded game/binary content is not allowed`, 415, "binary_rejected");
    } catch (e) {
      if (e instanceof HttpError) throw e;
    }
  }
}

// Validate an optional attachment {name, mime, data(base64)}. Returns the
// normalized record or null. Rejects (415) anything that is not a small
// image/text/log, or whose bytes look like a game/executable/archive.
function validateAttachment(att) {
  if (att == null) return null;
  if (typeof att !== "object") throw new HttpError("attachment must be an object", 400);
  const name = str(att.name, { name: "attachment.name", min: 1, max: 200 });
  const mime = str(att.mime, { name: "attachment.mime", min: 1, max: 100 }).toLowerCase();
  const data = str(att.data, { name: "attachment.data", min: 1, max: Math.ceil(MAX_ATTACH_BYTES * 1.4), trim: false });

  if (!ATTACH_MIME_OK.test(mime)) {
    throw new HttpError("이미지·텍스트·로그 파일만 첨부할 수 있습니다 (게임/실행 파일 불가)", 415, "attach_type_rejected");
  }
  // Block game/exec/archive by extension regardless of declared MIME.
  if (/\.(jar|jad|zip|kdf|skm|mod|smc|gba|nes|class|exe|dll|so|bin|apk|7z|rar|gz)$/i.test(name)) {
    throw new HttpError("게임/실행 파일은 첨부할 수 없습니다", 415, "attach_ext_rejected");
  }
  let bytes;
  try {
    const bin = atob(data.replace(/\s+/g, ""));
    if (bin.length > MAX_ATTACH_BYTES) throw new HttpError("첨부 파일이 너무 큽니다 (최대 96KB)", 413, "attach_too_large");
    bytes = new Uint8Array([...bin].map((c) => c.charCodeAt(0)));
  } catch (e) {
    if (e instanceof HttpError) throw e;
    throw new HttpError("attachment.data must be base64", 400);
  }
  // Final magic-number gate: a game/executable mislabeled as image/text is refused.
  if (hasMagic(bytes)) throw new HttpError("게임/실행 파일은 첨부할 수 없습니다", 415, "attach_magic_rejected");
  return { name, mime, data };
}

export async function onRequestPost(context) {
  try {
    const user = await requireUser(context);
    await rateLimit(context, "inquiry", { limit: 20, windowSec: 600 });
    const body = await readJson(context.request, 192 * 1024);

    const title = str(body.title, { name: "title", min: 1, max: 200 });
    const text = str(body.body, { name: "body", min: 1, max: 8000 });
    // env_info is auto-collected client-side; treat as untrusted text + scan it.
    const env = body.env_info == null ? "" : str(body.env_info, { name: "env_info", max: 4000 });
    rejectBinaryText(title, "title");
    rejectBinaryText(text, "body");
    if (env) rejectBinaryText(env, "env_info");

    const att = validateAttachment(body.attachment ?? null);

    // 6번: optional references to the user's OWN vault files (by id — never bytes).
    // Bounded list; each id is validated for ownership below before any ref is stored.
    let fileIds = [];
    if (Array.isArray(body.file_ids)) {
      fileIds = [...new Set(body.file_ids.filter((x) => typeof x === "string" && /^[0-9a-f-]{1,40}$/i.test(x)))].slice(0, 20);
    }

    const id = uuid();
    const now = Date.now();
    try {
      await context.env.DB.prepare(
        `INSERT INTO inquiries (id, user_id, category, title, body, game_title, game_vendor, device_model, symptom, env_info, attachment_name, attachment_mime, attachment_data, status, created_at)
         VALUES (?, ?, 'question', ?, ?, '', '', '', '', ?, ?, ?, ?, 'open', ?)`,
      )
        .bind(id, user.id, title, text, env, att?.name ?? "", att?.mime ?? "", att?.data ?? "", now)
        .run();

      // Attach vault-file references (ownership-checked: only the user's OWN files).
      // Best-effort + graceful: a missing inquiry_file_refs table (pre-migration 0008)
      // must not fail the inquiry itself.
      let refs = 0;
      if (fileIds.length) {
        try {
          for (const fid of fileIds) {
            const owned = await context.env.DB.prepare("SELECT id FROM user_files WHERE id = ? AND user_id = ?").bind(fid, user.id).first();
            if (!owned) continue; // not the user's file → silently skip (never reference others')
            await context.env.DB.prepare("INSERT OR IGNORE INTO inquiry_file_refs (inquiry_id, file_id, user_id, created_at) VALUES (?, ?, ?, ?)").bind(id, fid, user.id, now).run();
            refs++;
          }
        } catch (refErr) {
          console.error("inquiry file refs (pre-migration 0008?):", refErr && refErr.message);
        }
      }
      return ok({ inquiry: { id, title, created_at: now, has_attachment: !!att, file_refs: refs } });
    } catch (schemaErr) {
      // GRACEFUL: env_info/attachment columns don't exist until migration 0002
      // is applied to the remote D1. Fall back to the legacy column set (the
      // env/attachment are dropped) so inquiries keep working before migration.
      console.error("inquiry insert (pre-migration?):", schemaErr && schemaErr.message);
      await context.env.DB.prepare(
        `INSERT INTO inquiries (id, user_id, category, title, body, game_title, game_vendor, device_model, symptom, status, created_at)
         VALUES (?, ?, 'question', ?, ?, '', '', '', '', 'open', ?)`,
      )
        .bind(id, user.id, title, text, now)
        .run();
      return ok({ inquiry: { id, title, created_at: now, has_attachment: false }, migration_pending: true });
    }
  } catch (e) {
    return handleError(e);
  }
}

export async function onRequestGet(context) {
  try {
    const user = await requireUser(context);
    // Do NOT return attachment_data in the list (no bulk re-download path); only
    // a flag + metadata, owner-scoped.
    try {
      const { results } = await context.env.DB.prepare(
        `SELECT id, title, body, env_info, attachment_name, attachment_mime,
                (CASE WHEN attachment_data != '' THEN 1 ELSE 0 END) AS has_attachment,
                status, created_at
           FROM inquiries WHERE user_id = ? ORDER BY created_at DESC`,
      )
        .bind(user.id)
        .all();
      return ok({ inquiries: results || [] });
    } catch (schemaErr) {
      // GRACEFUL: pre-migration schema lacks env_info/attachment_* — read the
      // legacy columns so the history still renders.
      console.error("inquiry list (pre-migration?):", schemaErr && schemaErr.message);
      const { results } = await context.env.DB.prepare(
        "SELECT id, title, body, status, created_at FROM inquiries WHERE user_id = ? ORDER BY created_at DESC",
      )
        .bind(user.id)
        .all();
      return ok({ inquiries: results || [] });
    }
  } catch (e) {
    return handleError(e);
  }
}
