// Single-save endpoint, owner-scoped. The id in the path is always combined
// with `user_id = ?` so one user can never read or delete another's save.

import { ok, err, handleError } from "../../_lib/http.js";
import { requireUser } from "../../_lib/session.js";

export async function onRequestGet(context) {
  try {
    const user = await requireUser(context);
    const id = context.params.id;
    const row = await context.env.DB.prepare(
      "SELECT id, slot_label, device_label, payload, payload_bytes, checksum, updated_at, created_at FROM saves WHERE id = ? AND user_id = ?",
    )
      .bind(id, user.id)
      .first();
    if (!row) return err("Not found", 404);
    return ok({ save: row });
  } catch (e) {
    return handleError(e);
  }
}

export async function onRequestDelete(context) {
  try {
    const user = await requireUser(context);
    const id = context.params.id;
    const res = await context.env.DB.prepare("DELETE FROM saves WHERE id = ? AND user_id = ?").bind(id, user.id).run();
    const deleted = res.meta && res.meta.changes ? res.meta.changes : 0;
    if (!deleted) return err("Not found", 404);
    return ok({ deleted: id });
  } catch (e) {
    return handleError(e);
  }
}
