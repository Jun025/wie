// Shared HTTP helpers for wie Pages Functions.
// Files/dirs under functions/ whose name starts with "_" are NOT routed by
// Cloudflare Pages, so this module is import-only.

// `no-store`: API responses can carry session-scoped data (account, saves), so
// they must never be cached by the browser or any intermediary.
export const JSON_HEADERS = {
  "content-type": "application/json; charset=utf-8",
  "cache-control": "no-store",
};

export function json(data, status = 200, extraHeaders = {}) {
  return new Response(JSON.stringify(data), {
    status,
    headers: { ...JSON_HEADERS, ...extraHeaders },
  });
}

export function ok(data = {}, extraHeaders = {}) {
  return json({ ok: true, ...data }, 200, extraHeaders);
}

export function err(message, status = 400, code = undefined) {
  return json({ ok: false, error: message, code }, status);
}

// Parse a JSON body, rejecting anything that is not application/json. This is
// the first line of the "no game binaries via the inquiry/save endpoints"
// defense: multipart/form-data and octet-stream uploads never get parsed.
export async function readJson(request, maxBytes = 256 * 1024) {
  const ct = (request.headers.get("content-type") || "").toLowerCase();
  if (!ct.includes("application/json")) {
    throw new HttpError("Only application/json is accepted (file uploads are rejected)", 415);
  }
  const raw = await request.text();
  if (raw.length > maxBytes) {
    throw new HttpError("Payload too large", 413);
  }
  try {
    return JSON.parse(raw);
  } catch {
    throw new HttpError("Malformed JSON", 400);
  }
}

export class HttpError extends Error {
  constructor(message, status = 400, code = undefined) {
    super(message);
    this.status = status;
    this.code = code;
  }
}

export function handleError(e) {
  if (e instanceof HttpError) {
    return err(e.message, e.status, e.code);
  }
  // Never leak internals (and never log secrets / save payloads).
  console.error("unhandled:", e && e.message);
  return err("Internal error", 500);
}

export function str(v, { min = 0, max = 4096, name = "field", trim = true } = {}) {
  if (typeof v !== "string") throw new HttpError(`${name} must be a string`, 400);
  const s = trim ? v.trim() : v;
  if (s.length < min) throw new HttpError(`${name} is too short`, 400);
  if (s.length > max) throw new HttpError(`${name} is too long`, 400);
  return s;
}
