// Save-sync endpoint, keyed by ROM CONTENT HASH (per-owner).
//
// ★S5 (격리): every query is scoped to the authenticated user — there is no code
// path that reads or writes another user's saves, and no global rom_hash lookup.
// `rom_hash` is the OWNER's own ROM content hash (per-owner, like
// user_files.content_hash). The `payload` is an OPAQUE base64 save snapshot the
// server never inspects. NOT-logged-in users never reach this endpoint (their
// saves stay in the browser).

import { ok, err, readJson, handleError, str, HttpError } from "../../_lib/http.js";
import { uuid, sha256Hex, b64decode } from "../../_lib/crypto.js";
import { requireUser } from "../../_lib/session.js";

const MAX_PAYLOAD_B64 = 4 * 1024 * 1024; // ~3MB per save — generous for RMS
const SAVE_SERVER_CAP_BYTES = 100 * 1024 * 1024; // 100 MB total per user (SERVER-FIXED)
const ROM_HASH_RE = /^[0-9a-f]{64}$/;

// GET — `?rom=<hash>` returns the single save (with payload) for that ROM; with no
// `rom` it lists the user's saves (metadata only) for the UI / device slot count.
export async function onRequestGet(context) {
  try {
    const user = await requireUser(context);
    const url = new URL(context.request.url);
    const rom = (url.searchParams.get("rom") || "").toLowerCase();

    if (rom) {
      if (!ROM_HASH_RE.test(rom)) throw new HttpError("bad rom hash", 400);
      const row = await context.env.DB.prepare(
        "SELECT id, rom_hash, slot_label, device_label, payload, payload_bytes, checksum, updated_at, created_at FROM saves WHERE user_id = ? AND rom_hash = ?",
      )
        .bind(user.id, rom)
        .first();
      if (!row) return err("Not found", 404);
      return ok({ save: row });
    }

    const { results } = await context.env.DB.prepare(
      "SELECT id, rom_hash, slot_label, device_label, payload_bytes, checksum, updated_at, created_at FROM saves WHERE user_id = ? ORDER BY updated_at DESC",
    )
      .bind(user.id)
      .all();
    const usedRow = await context.env.DB.prepare("SELECT COALESCE(SUM(payload_bytes),0) AS used FROM saves WHERE user_id = ?").bind(user.id).first();
    return ok({ saves: results || [], usage: { used: usedRow ? Number(usedRow.used) : 0, quota: SAVE_SERVER_CAP_BYTES } });
  } catch (e) {
    return handleError(e);
  }
}

// POST { rom_hash, payload, slot_label?, device_label? } — upsert the save for a
// ROM. Last write wins (server clock). Enforces the 100 MB per-user save quota.
export async function onRequestPost(context) {
  try {
    const user = await requireUser(context);
    const body = await readJson(context.request, MAX_PAYLOAD_B64 + 64 * 1024);

    const romHash = str(body.rom_hash, { name: "rom_hash", min: 64, max: 64 }).toLowerCase();
    if (!ROM_HASH_RE.test(romHash)) throw new HttpError("rom_hash must be sha-256 hex", 400);
    const slotLabel = body.slot_label == null ? "" : str(body.slot_label, { name: "slot_label", max: 120 });
    const deviceLabel = body.device_label == null ? "" : str(body.device_label, { name: "device_label", max: 120 });
    const payload = str(body.payload, { name: "payload", min: 0, max: MAX_PAYLOAD_B64, trim: false });

    let bytes;
    try {
      bytes = b64decode(payload);
    } catch {
      throw new HttpError("payload must be base64", 400);
    }
    const checksum = await sha256Hex(bytes);
    const now = Date.now();

    // Upsert keyed on (user_id, rom_hash): the SAME ROM always maps to one save.
    const existing = await context.env.DB.prepare("SELECT id, payload_bytes FROM saves WHERE user_id = ? AND rom_hash = ?").bind(user.id, romHash).first();

    // Quota: total of all OTHER saves + this new payload must fit 100 MB.
    const sumRow = await context.env.DB.prepare("SELECT COALESCE(SUM(payload_bytes),0) AS used FROM saves WHERE user_id = ?").bind(user.id).first();
    const otherBytes = (sumRow ? Number(sumRow.used) : 0) - (existing ? Number(existing.payload_bytes) : 0);
    if (otherBytes + bytes.length > SAVE_SERVER_CAP_BYTES) {
      return err("세이브 보관 용량(100MB)을 초과합니다", 413, "save_quota_exceeded");
    }

    let id;
    if (existing) {
      id = existing.id;
      await context.env.DB.prepare(
        "UPDATE saves SET slot_label = ?, device_label = ?, payload = ?, payload_bytes = ?, checksum = ?, updated_at = ? WHERE id = ? AND user_id = ?",
      )
        .bind(slotLabel, deviceLabel, payload, bytes.length, checksum, now, id, user.id)
        .run();
    } else {
      id = uuid();
      await context.env.DB.prepare(
        "INSERT INTO saves (id, user_id, rom_hash, slot_label, device_label, payload, payload_bytes, checksum, updated_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
      )
        .bind(id, user.id, romHash, slotLabel, deviceLabel, payload, bytes.length, checksum, now, now)
        .run();
    }

    const newUsed = otherBytes + bytes.length;
    return ok({ save: { id, rom_hash: romHash, slot_label: slotLabel, device_label: deviceLabel, payload_bytes: bytes.length, checksum, updated_at: now }, usage: { used: newUsed, quota: SAVE_SERVER_CAP_BYTES } });
  } catch (e) {
    return handleError(e);
  }
}
