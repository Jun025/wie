// One-time, hashed, expiring tokens for email verification + password reset.
//
// The raw token goes only into the emailed link; the DB stores its SHA-256 hash,
// so a database read cannot mint a working link. Tokens are single-use (used_at)
// and time-boxed. Raw tokens are never logged.

import { randomHex, sha256Hex, uuid } from "./crypto.js";

export const VERIFY_TTL_MS = 24 * 60 * 60 * 1000; // 24h
export const RESET_TTL_MS = 60 * 60 * 1000; // 1h

// Create a token for (userId, purpose), store its hash, return the RAW token.
export async function createToken(env, userId, purpose, ttlMs) {
  const raw = randomHex(32);
  const tokenHash = await sha256Hex(raw);
  const now = Date.now();
  await env.DB.prepare(
    "INSERT INTO email_tokens (id, user_id, purpose, token_hash, expires_at, used_at, created_at) VALUES (?, ?, ?, ?, ?, NULL, ?)",
  )
    .bind(uuid(), userId, purpose, tokenHash, now + ttlMs, now)
    .run();
  return raw;
}

// Validate + atomically consume a raw token. Returns the user_id on success, or
// null if missing / wrong purpose / expired / already used.
export async function consumeToken(env, rawToken, purpose) {
  if (typeof rawToken !== "string" || rawToken.length < 16) return null;
  const tokenHash = await sha256Hex(rawToken);
  const row = await env.DB.prepare(
    "SELECT id, user_id, expires_at, used_at FROM email_tokens WHERE token_hash = ? AND purpose = ?",
  )
    .bind(tokenHash, purpose)
    .first();
  if (!row || row.used_at || row.expires_at < Date.now()) return null;
  // Mark used; the WHERE used_at IS NULL guard makes this a one-shot even under a race.
  const res = await env.DB.prepare("UPDATE email_tokens SET used_at = ? WHERE id = ? AND used_at IS NULL")
    .bind(Date.now(), row.id)
    .run();
  if (!res.meta || res.meta.changes !== 1) return null;
  return row.user_id;
}

// Best-effort cleanup: drop a user's outstanding tokens of a purpose before
// issuing a fresh one (so resend invalidates the previous link).
export async function invalidateTokens(env, userId, purpose) {
  await env.DB.prepare("UPDATE email_tokens SET used_at = ? WHERE user_id = ? AND purpose = ? AND used_at IS NULL")
    .bind(Date.now(), userId, purpose)
    .run();
}
