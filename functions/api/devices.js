// "Other devices" view. Returns ONLY save-slot aggregates per user-chosen
// device alias: never a game title/filename/hash. This deliberately cannot
// answer "what games does this device have" — that information is not on the
// server by design (1번 기준선 / S5).

import { ok, handleError } from "../_lib/http.js";
import { requireUser } from "../_lib/session.js";

export async function onRequestGet(context) {
  try {
    const user = await requireUser(context);
    const { results } = await context.env.DB.prepare(
      `SELECT COALESCE(NULLIF(device_label, ''), '(unnamed)') AS device_label,
              COUNT(*) AS slot_count,
              MAX(updated_at) AS last_updated
         FROM saves
        WHERE user_id = ?
        GROUP BY device_label
        ORDER BY last_updated DESC`,
    )
      .bind(user.id)
      .all();
    return ok({ devices: results || [] });
  } catch (e) {
    return handleError(e);
  }
}
