// Save-sync collection endpoint. Every query is scoped to the authenticated
// user — there is no code path that reads or writes another user's saves.
//
// The `payload` is an OPAQUE base64 save snapshot produced by the emulator in
// the browser (RMS records + per-app save files). The server never inspects it,
// never associates it with a game title/filename/hash, and only stores a
// user-chosen `slot_label`/`device_label` alias alongside it.

import { ok, err, readJson, handleError, str, HttpError } from "../../_lib/http.js";
import { uuid, sha256Hex, b64decode } from "../../_lib/crypto.js";
import { requireUser } from "../../_lib/session.js";

const MAX_PAYLOAD_B64 = 4 * 1024 * 1024; // ~3MB of save data — generous for RMS

export async function onRequestGet(context) {
  try {
    const user = await requireUser(context);
    const url = new URL(context.request.url);
    const includePayload = url.searchParams.get("include") === "payload";

    const cols = includePayload
      ? "id, slot_label, device_label, payload, payload_bytes, checksum, updated_at, created_at"
      : "id, slot_label, device_label, payload_bytes, checksum, updated_at, created_at";

    const { results } = await context.env.DB.prepare(
      `SELECT ${cols} FROM saves WHERE user_id = ? ORDER BY updated_at DESC`,
    )
      .bind(user.id)
      .all();

    return ok({ saves: results || [] });
  } catch (e) {
    return handleError(e);
  }
}

export async function onRequestPost(context) {
  try {
    const user = await requireUser(context);
    const body = await readJson(context.request, MAX_PAYLOAD_B64 + 64 * 1024);

    const slotLabel = str(body.slot_label, { name: "slot_label", min: 1, max: 120 });
    const deviceLabel = body.device_label == null ? "" : str(body.device_label, { name: "device_label", max: 120 });
    const payload = str(body.payload, { name: "payload", min: 0, max: MAX_PAYLOAD_B64, trim: false });

    // payload must be valid base64; reject anything else.
    let bytes;
    try {
      bytes = b64decode(payload);
    } catch {
      throw new HttpError("payload must be base64", 400);
    }
    const checksum = await sha256Hex(bytes);
    const now = Date.now();

    // Upsert keyed on (user_id, slot_label). Last write wins (server clock).
    const existing = await context.env.DB.prepare(
      "SELECT id FROM saves WHERE user_id = ? AND slot_label = ?",
    )
      .bind(user.id, slotLabel)
      .first();

    let id;
    if (existing) {
      id = existing.id;
      await context.env.DB.prepare(
        "UPDATE saves SET device_label = ?, payload = ?, payload_bytes = ?, checksum = ?, updated_at = ? WHERE id = ? AND user_id = ?",
      )
        .bind(deviceLabel, payload, bytes.length, checksum, now, id, user.id)
        .run();
    } else {
      id = uuid();
      await context.env.DB.prepare(
        "INSERT INTO saves (id, user_id, slot_label, device_label, payload, payload_bytes, checksum, updated_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
      )
        .bind(id, user.id, slotLabel, deviceLabel, payload, bytes.length, checksum, now, now)
        .run();
    }

    return ok({ save: { id, slot_label: slotLabel, device_label: deviceLabel, payload_bytes: bytes.length, checksum, updated_at: now } });
  } catch (e) {
    return handleError(e);
  }
}
