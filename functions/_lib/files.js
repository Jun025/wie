// B안 server-side game-file storage helpers (R2 bytes + D1 metadata).
//
// ┌─ S5 (격리) ──────────────────────────────────────────────────────────────────┐
// │ Every file access is gated by requireUser() AND a `WHERE user_id = ?` clause. │
// │ R2 object keys are owner-namespaced + random (unguessable), and bytes are     │
// │ ONLY ever served back by the authenticated, owner-checked download endpoint   │
// │ — never via a public URL, a presigned URL, a global listing, or a content-    │
// │ hash lookup. Dedup compares hashes PER-USER only.                             │
// └──────────────────────────────────────────────────────────────────────────────┘

import { randomHex } from "./crypto.js";

// ★SERVER-FIXED limits — not user-modifiable (no request field, header, or env
// can change them). Login users: 1 GiB total. Per-file cap keeps every upload
// within the Workers request-body / memory budget (real WIPI/SKVM/J2ME games are
// far smaller; the 1 GiB is a *sum* across many small files).
export const FILE_QUOTA_BYTES = 1024 * 1024 * 1024; // 1 GiB per user (total)
export const PER_FILE_MAX_BYTES = 64 * 1024 * 1024; // 64 MiB per single file

export const ALLOWED_KINDS = new Set(["jar", "jad", "zip", "kdf", "skm"]);

// The R2 binding is provisioned in the Cloudflare dashboard (a human task — S8).
// Until it exists the whole feature is dormant: endpoints report "not configured"
// and never 500, so deploying this code cannot break the live site.
export function filesEnabled(env) {
  return !!(env && env.GAMES);
}

// Owner-namespaced, unguessable key. Even if a key leaked, the bytes are only
// reachable through the owner-checked download endpoint — the bucket is private.
export function makeR2Key(userId) {
  return `u/${userId}/${randomHex(24)}`;
}

// Sum of the user's ACTIVE (non-disabled) file sizes — the live quota usage.
export async function usedBytes(env, userId) {
  const row = await env.DB.prepare(
    "SELECT COALESCE(SUM(size), 0) AS used FROM user_files WHERE user_id = ? AND disabled = 0",
  )
    .bind(userId)
    .first();
  return row ? Number(row.used) : 0;
}

// Reject obvious standalone executables / scripts / web payloads so the private
// archive cannot be repurposed as a generic file host (웹하드 경계). Proprietary
// game containers (kdf/skm/jad) have no universal magic, so this is a deny-list of
// clearly-non-game payloads, not an allow-list of magics.
const DENY_MAGICS = [
  [0x4d, 0x5a], // MZ — Windows PE/.exe/.dll
  [0x7f, 0x45, 0x4c, 0x46], // ELF — Linux/Android native exec
  [0xcf, 0xfa, 0xed, 0xfe], // Mach-O (LE)
  [0xfe, 0xed, 0xfa, 0xce], // Mach-O (BE)
  [0x23, 0x21], // "#!" shebang script
  [0x3c, 0x3f, 0x78, 0x6d, 0x6c], // "<?xml"
  [0x3c, 0x68, 0x74, 0x6d, 0x6c], // "<html"
  [0x3c, 0x21, 0x44, 0x4f, 0x43], // "<!DOC"
];

export function looksDisallowed(bytes) {
  return DENY_MAGICS.some((m) => m.every((b, i) => bytes[i] === b));
}

// True when a D1 error is "the user_files table/column doesn't exist yet" — i.e.
// the R2 binding is live but migration 0003 hasn't been applied to the remote DB
// (a human task, S8). Callers degrade gracefully (no 500) until it is applied.
export function isMissingTable(err) {
  const m = err && err.message ? err.message.toLowerCase() : "";
  return m.includes("no such table") || m.includes("no such column");
}
