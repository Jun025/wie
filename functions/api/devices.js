// Device registry for the logged-in user.
//
// GUARDRAIL (1번 기준선 / S5): this endpoint stores ONLY a client-generated
// device id, a user alias, login/seen timestamps, and ANONYMOUS storage
// aggregates (item_count, total_bytes, last_run/last_save). It NEVER receives or
// stores a game filename, hash, or title — so it still cannot answer "which games
// does this device have". The reading device shows its own filenames purely from
// its local IndexedDB; other devices appear as counts/sizes only.

import { ok, readJson, handleError, str, HttpError } from "../_lib/http.js";
import { requireUser } from "../_lib/session.js";

function num(v, max = Number.MAX_SAFE_INTEGER) {
  const n = typeof v === "number" && Number.isFinite(v) ? Math.floor(v) : 0;
  return Math.max(0, Math.min(max, n));
}

// GET — list this user's devices + the per-device cloud save-slot count.
export async function onRequestGet(context) {
  try {
    const user = await requireUser(context);
    try {
      const { results } = await context.env.DB.prepare(
        `SELECT d.device_id, d.label, d.last_login_at, d.last_seen_at,
                d.item_count, d.total_bytes, d.last_run_at, d.last_save_at,
                (SELECT COUNT(*) FROM saves s WHERE s.user_id = d.user_id AND s.device_label = d.label) AS slot_count
           FROM devices d
          WHERE d.user_id = ?
          ORDER BY d.last_seen_at DESC`,
      )
        .bind(user.id)
        .all();
      return ok({ devices: results || [] });
    } catch (schemaErr) {
      // GRACEFUL: the `devices` table doesn't exist until migration 0002 is
      // applied to the remote D1. Until then, report no devices instead of 500.
      console.error("devices list (pre-migration?):", schemaErr && schemaErr.message);
      return ok({ devices: [], migration_pending: true });
    }
  } catch (e) {
    return handleError(e);
  }
}

// POST — heartbeat / upsert the current device. Body carries ONLY counts/sizes,
// never game identities. `login:true` also stamps last_login_at.
export async function onRequestPost(context) {
  try {
    const user = await requireUser(context);
    const body = await readJson(context.request, 4 * 1024);
    const deviceId = str(body.device_id, { name: "device_id", min: 8, max: 64 });
    if (!/^[A-Za-z0-9_-]+$/.test(deviceId)) throw new HttpError("bad device_id", 400);
    const label = body.label == null ? "" : str(body.label, { name: "label", max: 60 });
    const now = Date.now();
    const item_count = num(body.item_count, 100000);
    const total_bytes = num(body.total_bytes);
    const last_run_at = num(body.last_run_at);
    const last_save_at = num(body.last_save_at);
    const loginStamp = body.login ? now : 0;

    try {
      await context.env.DB.prepare(
        `INSERT INTO devices (user_id, device_id, label, last_login_at, last_seen_at, item_count, total_bytes, last_run_at, last_save_at, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id, device_id) DO UPDATE SET
           label = excluded.label,
           last_seen_at = excluded.last_seen_at,
           last_login_at = MAX(devices.last_login_at, excluded.last_login_at),
           item_count = excluded.item_count,
           total_bytes = excluded.total_bytes,
           last_run_at = excluded.last_run_at,
           last_save_at = excluded.last_save_at`,
      )
        .bind(user.id, deviceId, label, loginStamp, now, item_count, total_bytes, last_run_at, last_save_at, now)
        .run();
      return ok({});
    } catch (schemaErr) {
      // GRACEFUL: no `devices` table yet (pre-migration) — accept the heartbeat
      // as a no-op so login/usage never breaks before the remote migration.
      console.error("device heartbeat (pre-migration?):", schemaErr && schemaErr.message);
      return ok({ migration_pending: true });
    }
  } catch (e) {
    return handleError(e);
  }
}
