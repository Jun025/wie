// Inquiries: text only. JAR/ROM/.mod and any binary attachment is impossible
// here — readJson() rejects anything that is not application/json, and we
// additionally scan every text field for executable/archive magic numbers so a
// base64-smuggled game binary is refused (the inquiry channel must never become
// a file-transfer path to other users — the "웹하드 경계").
//
// The game_* fields are VOLUNTARY free text the user types into a support
// ticket. They are never auto-filled from the device game library, so this is
// not a "game ownership manifest" — it is user-authored bug-report content,
// which 3-3 explicitly permits.

import { ok, readJson, handleError, str, HttpError } from "../../_lib/http.js";
import { uuid } from "../../_lib/crypto.js";
import { requireUser } from "../../_lib/session.js";
import { rateLimit } from "../../_lib/ratelimit.js";

const CATEGORIES = new Set(["question", "suggestion", "proposal", "rights_report"]);

// Magic-number prefixes for common game binaries / archives. If any field
// contains these (raw or base64-decoded), the submission is rejected.
const BINARY_MAGICS = [
  [0x50, 0x4b, 0x03, 0x04], // PK.. ZIP/JAR
  [0xca, 0xfe, 0xba, 0xbe], // Java .class / Mach-O fat
  [0x7f, 0x45, 0x4c, 0x46], // ELF
  [0x4d, 0x5a], // MZ (PE/exe)
  [0xde, 0xad, 0xbe, 0xef], // common ROM sentinel
];

function looksBinary(s) {
  // NUL or a high density of control bytes ⇒ not legitimate ticket text.
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

function rejectBinary(value, name) {
  if (looksBinary(value)) throw new HttpError(`${name}: binary content is not allowed`, 415, "binary_rejected");
  // Try to catch base64-smuggled binaries.
  const compact = value.replace(/\s+/g, "");
  if (compact.length >= 8 && /^[A-Za-z0-9+/=]+$/.test(compact)) {
    try {
      const bin = atob(compact.slice(0, 64));
      const bytes = new Uint8Array([...bin].map((c) => c.charCodeAt(0)));
      if (hasMagic(bytes)) throw new HttpError(`${name}: encoded game/binary content is not allowed`, 415, "binary_rejected");
    } catch (e) {
      if (e instanceof HttpError) throw e;
      // not valid base64 → fine, it is just text
    }
  }
}

export async function onRequestPost(context) {
  try {
    const user = await requireUser(context);
    await rateLimit(context, "inquiry", { limit: 20, windowSec: 600 });
    const body = await readJson(context.request, 128 * 1024);

    const category = str(body.category, { name: "category", min: 1, max: 32 });
    if (!CATEGORIES.has(category)) throw new HttpError("Unknown category", 400);

    const fields = {
      title: str(body.title, { name: "title", min: 1, max: 200 }),
      body: str(body.body, { name: "body", min: 1, max: 8000 }),
      game_title: body.game_title == null ? "" : str(body.game_title, { name: "game_title", max: 200 }),
      game_vendor: body.game_vendor == null ? "" : str(body.game_vendor, { name: "game_vendor", max: 200 }),
      device_model: body.device_model == null ? "" : str(body.device_model, { name: "device_model", max: 200 }),
      symptom: body.symptom == null ? "" : str(body.symptom, { name: "symptom", max: 2000 }),
    };
    for (const [name, value] of Object.entries(fields)) rejectBinary(value, name);

    const id = uuid();
    const now = Date.now();
    await context.env.DB.prepare(
      `INSERT INTO inquiries (id, user_id, category, title, body, game_title, game_vendor, device_model, symptom, status, created_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'open', ?)`,
    )
      .bind(id, user.id, category, fields.title, fields.body, fields.game_title, fields.game_vendor, fields.device_model, fields.symptom, now)
      .run();

    return ok({ inquiry: { id, category, title: fields.title, created_at: now } });
  } catch (e) {
    return handleError(e);
  }
}

export async function onRequestGet(context) {
  try {
    const user = await requireUser(context);
    const { results } = await context.env.DB.prepare(
      "SELECT id, category, title, body, game_title, game_vendor, device_model, symptom, status, created_at FROM inquiries WHERE user_id = ? ORDER BY created_at DESC",
    )
      .bind(user.id)
      .all();
    return ok({ inquiries: results || [] });
  } catch (e) {
    return handleError(e);
  }
}
