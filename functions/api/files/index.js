// /api/files — the logged-in user's PRIVATE game-file vault (B안).
//
// GET  : list THIS user's files (metadata only) + quota usage.
// POST : upload one file (raw application/octet-stream body + metadata headers).
//
// ★S5 (격리): every row is owner-scoped (`WHERE user_id = ?`). There is no path
// that lists, searches, or returns another user's files, and no global content-
// hash lookup. Dedup is strictly per-user. Bytes live in a PRIVATE R2 bucket and
// are only ever streamed back by the owner-checked download endpoint.

import { ok, err, handleError, str, HttpError } from "../../_lib/http.js";
import { uuid, sha256Hex } from "../../_lib/crypto.js";
import { requireUser } from "../../_lib/session.js";
import { rateLimit } from "../../_lib/ratelimit.js";
import { FILE_QUOTA_BYTES, PER_FILE_MAX_BYTES, filesEnabled, makeR2Key, usedBytes, looksDisallowed, looksBlockedExtension, isMissingTable } from "../../_lib/files.js";

// GET — list the user's files (no bytes) + quota. When the R2 binding is not yet
// provisioned (S8), report `enabled:false` so the UI can hide the feature without
// erroring.
export async function onRequestGet(context) {
  try {
    const user = await requireUser(context);
    if (!filesEnabled(context.env)) {
      return ok({ enabled: false, files: [], usage: { used: 0, quota: FILE_QUOTA_BYTES } });
    }
    try {
      const { results } = await context.env.DB.prepare(
        `SELECT id, file_name, kind, content_hash, size, created_at, last_seen_at
           FROM user_files
          WHERE user_id = ? AND disabled = 0
          ORDER BY created_at DESC`,
      )
        .bind(user.id)
        .all();
      const used = await usedBytes(context.env, user.id);
      return ok({ enabled: true, files: results || [], usage: { used, quota: FILE_QUOTA_BYTES } });
    } catch (schemaErr) {
      // GRACEFUL: R2 binding is live but migration 0003 hasn't been applied yet —
      // report the vault as not-enabled (UI hides it) instead of 500ing.
      if (isMissingTable(schemaErr)) {
        console.error("files list (pre-migration 0003?):", schemaErr && schemaErr.message);
        return ok({ enabled: false, files: [], usage: { used: 0, quota: FILE_QUOTA_BYTES }, migration_pending: true });
      }
      throw schemaErr;
    }
  } catch (e) {
    return handleError(e);
  }
}

// POST — upload one file. Body = raw bytes (application/octet-stream). Metadata in
// headers: x-file-name (percent-encoded UTF-8), x-content-hash (sha-256 hex),
// x-kind (jar|jad|zip|kdf|skm).
export async function onRequestPost(context) {
  const { env, request } = context;
  try {
    const user = await requireUser(context);
    if (!filesEnabled(env)) return err("서버 보관함이 아직 설정되지 않았습니다", 503, "files_not_configured");
    // Restricted/disabled accounts (repeat-infringer policy) cannot upload.
    if (user.status === "disabled" || user.status === "restricted") {
      return err("이 계정은 업로드가 제한되어 있습니다", 403, "account_restricted");
    }
    await rateLimit(context, "file-upload", { limit: 120, windowSec: 600 });

    const ct = (request.headers.get("content-type") || "").toLowerCase();
    if (!ct.includes("application/octet-stream")) {
      throw new HttpError("Only application/octet-stream is accepted for file upload", 415, "bad_content_type");
    }

    // Metadata (validated, length-bounded).
    let fileName;
    try {
      fileName = decodeURIComponent(request.headers.get("x-file-name") || "");
    } catch {
      throw new HttpError("bad x-file-name", 400);
    }
    fileName = str(fileName, { name: "file_name", min: 1, max: 200 });
    const clientHash = str(request.headers.get("x-content-hash") || "", { name: "content_hash", min: 64, max: 64 });
    if (!/^[0-9a-f]{64}$/.test(clientHash)) throw new HttpError("content_hash must be sha-256 hex", 400);
    // 4번: the private vault stores ordinary files, but executable/script/web-shell
    // files are blocked at the source (BLOCKLIST — game containers + docs/images
    // stay allowed). Two layers: this declared-extension check, plus the byte-level
    // magic screen (`looksDisallowed`) below that defeats a renamed executable.
    const kind = str(request.headers.get("x-kind") || "file", { name: "kind", min: 1, max: 16 }).toLowerCase();
    if (looksBlockedExtension(fileName) || looksBlockedExtension(kind)) {
      throw new HttpError("실행 파일·스크립트는 업로드할 수 없습니다", 415, "blocked_extension");
    }

    // Reject oversized uploads early using the declared length, before buffering.
    const declaredLen = Number(request.headers.get("content-length") || "0");
    if (declaredLen > PER_FILE_MAX_BYTES) {
      throw new HttpError(`파일이 너무 큽니다 (단일 파일 최대 ${PER_FILE_MAX_BYTES / 1024 / 1024}MB)`, 413, "file_too_large");
    }

    // PER-USER dedup: same owner cannot store the same file twice. This is also
    // the first user_files access, so a missing-table error here means migration
    // 0003 isn't applied yet — fail gracefully (503) BEFORE writing any R2 bytes.
    let dup;
    try {
      dup = await env.DB.prepare("SELECT id FROM user_files WHERE user_id = ? AND content_hash = ?").bind(user.id, clientHash).first();
    } catch (schemaErr) {
      if (isMissingTable(schemaErr)) {
        console.error("file upload (pre-migration 0003?):", schemaErr && schemaErr.message);
        return err("서버 보관함이 아직 준비되지 않았습니다 (마이그레이션 대기)", 503, "files_not_ready");
      }
      throw schemaErr;
    }
    if (dup) return err("이미 보관함에 있는 파일입니다", 409, "duplicate");

    // Buffer (≤ per-file cap) so we can verify the hash + screen the magic bytes
    // before writing. 64 MiB fits comfortably in the Worker memory budget.
    const buf = new Uint8Array(await request.arrayBuffer());
    if (buf.byteLength === 0) throw new HttpError("빈 파일은 업로드할 수 없습니다", 400, "empty_file");
    if (buf.byteLength > PER_FILE_MAX_BYTES) {
      throw new HttpError(`파일이 너무 큽니다 (단일 파일 최대 ${PER_FILE_MAX_BYTES / 1024 / 1024}MB)`, 413, "file_too_large");
    }
    if (looksDisallowed(buf)) {
      throw new HttpError("실행 파일·스크립트·웹 문서는 보관할 수 없습니다 (게임 파일만)", 415, "disallowed_content");
    }
    // Integrity: the stored bytes must match the client's content hash (also keeps
    // dedup honest — a client can't mislabel a file's identity).
    const actualHash = await sha256Hex(buf);
    if (actualHash !== clientHash) throw new HttpError("content_hash가 본문과 일치하지 않습니다", 400, "hash_mismatch");

    // Quota: enforce the SERVER-FIXED 1 GiB ceiling on actual bytes.
    const used = await usedBytes(env, user.id);
    if (used + buf.byteLength > FILE_QUOTA_BYTES) {
      return err("보관함 용량(1GB)을 초과합니다", 413, "quota_exceeded");
    }

    const id = uuid();
    const r2Key = makeR2Key(user.id);
    const now = Date.now();

    // Write bytes to the PRIVATE bucket. customMetadata records the owner for
    // defense-in-depth/audit (access is still gated by D1 owner check, never this).
    await env.GAMES.put(r2Key, buf, {
      httpMetadata: { contentType: "application/octet-stream" },
      customMetadata: { owner: user.id, kind },
    });

    try {
      await env.DB.prepare(
        `INSERT INTO user_files (id, user_id, file_name, kind, content_hash, size, r2_key, disabled, disabled_reason, disabled_at, created_at, last_seen_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, 0, '', 0, ?, ?)`,
      )
        .bind(id, user.id, fileName, kind, clientHash, buf.byteLength, r2Key, now, now)
        .run();
    } catch (insErr) {
      // Lost a dedup race (UNIQUE owner+hash) — roll back the R2 object we wrote.
      await env.GAMES.delete(r2Key).catch(() => {});
      return err("이미 보관함에 있는 파일입니다", 409, "duplicate");
    }

    const newUsed = used + buf.byteLength;
    return ok({ file: { id, file_name: fileName, kind, content_hash: clientHash, size: buf.byteLength, created_at: now }, usage: { used: newUsed, quota: FILE_QUOTA_BYTES } });
  } catch (e) {
    return handleError(e);
  }
}
