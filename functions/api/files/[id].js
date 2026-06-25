// /api/files/:id — download or delete ONE of the user's files.
//
// ★S5 (격리): the path id is ALWAYS combined with `user_id = ?`, so a user can
// only ever reach their own file. A taken-down (disabled) file returns 404 even
// to its owner. Bytes are streamed straight from the PRIVATE R2 bucket through
// this authenticated endpoint — there is no public/presigned URL, and the response
// is marked private + no-store so no shared cache can retain it.

import { ok, err, handleError } from "../../_lib/http.js";
import { requireUser } from "../../_lib/session.js";
import { filesEnabled, usedBytes, FILE_QUOTA_BYTES } from "../../_lib/files.js";

export async function onRequestGet(context) {
  const { env, params } = context;
  try {
    const user = await requireUser(context);
    if (!filesEnabled(env)) return err("서버 보관함이 아직 설정되지 않았습니다", 503, "files_not_configured");

    const row = await env.DB.prepare(
      "SELECT id, file_name, kind, r2_key, disabled FROM user_files WHERE id = ? AND user_id = ?",
    )
      .bind(params.id, user.id)
      .first();
    if (!row || row.disabled) return err("Not found", 404);

    const obj = await env.GAMES.get(row.r2_key);
    if (!obj) return err("Not found", 404);

    // best-effort: stamp last access (never blocks the download).
    context.waitUntil?.(
      env.DB.prepare("UPDATE user_files SET last_seen_at = ? WHERE id = ? AND user_id = ?").bind(Date.now(), row.id, user.id).run().catch(() => {}),
    );

    return new Response(obj.body, {
      status: 200,
      headers: {
        "content-type": "application/octet-stream",
        // Owner-only payload: never store in any shared/intermediary cache.
        "cache-control": "private, no-store",
        "x-content-type-options": "nosniff",
        "content-disposition": `attachment; filename*=UTF-8''${encodeURIComponent(row.file_name)}`,
      },
    });
  } catch (e) {
    return handleError(e);
  }
}

export async function onRequestDelete(context) {
  const { env, params } = context;
  try {
    const user = await requireUser(context);
    if (!filesEnabled(env)) return err("서버 보관함이 아직 설정되지 않았습니다", 503, "files_not_configured");

    const row = await env.DB.prepare("SELECT r2_key FROM user_files WHERE id = ? AND user_id = ?").bind(params.id, user.id).first();
    if (!row) return err("Not found", 404);

    await env.GAMES.delete(row.r2_key).catch(() => {});
    await env.DB.prepare("DELETE FROM user_files WHERE id = ? AND user_id = ?").bind(params.id, user.id).run();

    const used = await usedBytes(env, user.id);
    return ok({ deleted: params.id, usage: { used, quota: FILE_QUOTA_BYTES } });
  } catch (e) {
    return handleError(e);
  }
}
