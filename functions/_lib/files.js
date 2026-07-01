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
export const PER_FILE_MAX_BYTES = 100 * 1024 * 1024; // 100 MB per single file (4번)

export const ALLOWED_KINDS = new Set(["jar", "jad", "zip", "kdf", "skm"]);

// 4번: executable / script / web-shell extensions blocked at the source, even in
// the private vault (BLOCKLIST — game containers + ordinary files stay allowed).
// Mirrors web/src/lib/limits.ts BLOCKED_UPLOAD_EXT. The byte-level magic screen
// (looksDisallowed) is the anti-spoof layer; this is the declared-type layer.
export const BLOCKED_UPLOAD_EXT = new Set([
  "exe", "msi", "com", "scr", "cpl", "dll", "so", "dylib", "apk", "app", "deb", "rpm", "dmg",
  "sh", "bash", "zsh", "ksh", "csh", "command",
  "bat", "cmd", "ps1", "psm1", "vbs", "vbe", "wsf", "wsh", "hta",
  "php", "php3", "php4", "php5", "phtml", "jsp", "jspx", "asp", "aspx", "cgi", "pl", "py", "rb",
]);

// True when a filename's (or kind's) extension is a blocked executable/script.
export function looksBlockedExtension(nameOrKind) {
  const s = String(nameOrKind || "").toLowerCase();
  const i = s.lastIndexOf(".");
  const ext = i >= 0 ? s.slice(i + 1) : s; // accept a bare "kind" or a "name.ext"
  return BLOCKED_UPLOAD_EXT.has(ext);
}

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
  [0xce, 0xfa, 0xed, 0xfe], // Mach-O 32-bit (LE)
  [0x23, 0x21], // "#!" shebang script
  [0x3c, 0x3f, 0x78, 0x6d, 0x6c], // "<?xml"
  [0x3c, 0x3f, 0x70, 0x68, 0x70], // "<?php"
  [0x3c, 0x25], // "<%" — JSP/ASP scriptlet
  [0x3c, 0x68, 0x74, 0x6d, 0x6c], // "<html"
  [0x3c, 0x21, 0x44, 0x4f, 0x43], // "<!DOC"
  [0xd0, 0xcf, 0x11, 0xe0], // MS CFBF (OLE) — .msi / legacy Office
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
