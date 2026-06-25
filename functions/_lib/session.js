// Session lifecycle + the requireUser() gate used by every owner-scoped route.

import { signSessionId, verifySessionCookie, randomHex } from "./crypto.js";
import { HttpError } from "./http.js";

export const COOKIE_NAME = "wie_session";
const SESSION_TTL_MS = 30 * 24 * 60 * 60 * 1000; // 30 days

function sessionSecret(env) {
  const s = env.SESSION_SECRET;
  if (!s || s.length < 16) {
    // Fail closed: refuse to mint/verify sessions without a real secret.
    throw new HttpError("Server session secret is not configured", 503, "no_session_secret");
  }
  return s;
}

export async function createSession(env, userId) {
  const id = randomHex(32);
  const now = Date.now();
  const expires = now + SESSION_TTL_MS;
  await env.DB.prepare(
    "INSERT INTO sessions (id, user_id, created_at, expires_at, revoked) VALUES (?, ?, ?, ?, 0)",
  )
    .bind(id, userId, now, expires)
    .run();
  const cookieValue = await signSessionId(id, sessionSecret(env));
  return { cookieValue, expires };
}

export function sessionCookie(value, maxAgeSeconds) {
  // SameSite=Lax so the cookie still rides top-level navigations; HttpOnly so JS
  // can never read it; Secure so it is HTTPS-only.
  const parts = [
    `${COOKIE_NAME}=${value}`,
    "Path=/",
    "HttpOnly",
    "Secure",
    "SameSite=Lax",
    `Max-Age=${maxAgeSeconds}`,
  ];
  return parts.join("; ");
}

export function clearCookie() {
  return `${COOKIE_NAME}=; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=0`;
}

function readCookie(request, name) {
  const header = request.headers.get("cookie") || "";
  for (const part of header.split(";")) {
    const [k, ...v] = part.trim().split("=");
    if (k === name) return v.join("=");
  }
  return null;
}

// Returns { id, login_id, email } for the authenticated user, or null.
export async function getUser(context) {
  const { request, env } = context;
  const raw = readCookie(request, COOKIE_NAME);
  if (!raw) return null;
  let secret;
  try {
    secret = sessionSecret(env);
  } catch {
    return null;
  }
  const sessionId = await verifySessionCookie(raw, secret);
  if (!sessionId) return null;

  const row = await env.DB.prepare(
    `SELECT s.id AS sid, s.expires_at, s.revoked, u.id AS uid, u.login_id, u.email, u.email_verified, u.status
       FROM sessions s JOIN users u ON u.id = s.user_id
      WHERE s.id = ?`,
  )
    .bind(sessionId)
    .first();

  if (!row) return null;
  if (row.revoked) return null;
  if (row.expires_at < Date.now()) return null;
  return { id: row.uid, login_id: row.login_id, email: row.email, email_verified: !!row.email_verified, status: row.status, sessionId: row.sid };
}

export async function requireUser(context) {
  const user = await getUser(context);
  if (!user) throw new HttpError("Authentication required", 401, "unauthenticated");
  return user;
}

export async function revokeSession(env, sessionId) {
  await env.DB.prepare("UPDATE sessions SET revoked = 1 WHERE id = ?").bind(sessionId).run();
}
