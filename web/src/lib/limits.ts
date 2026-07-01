// Capacity limits — SERVER/CODE-FIXED, not user-modifiable. There is intentionally
// no setter and no settings UI for any of these. Two distinct kinds are tracked
// separately: uploaded game files (ROMs) vs. save data.
//
//   uploaded files : not-logged-in → 10 MB local   |  logged-in → 1 GB server
//   save data      : not-logged-in →  1 MB local   |  logged-in → 100 MB server
//
// Save data for a NOT-logged-in user lives only in IndexedDB and is never sent to
// the server (anonymous users are not tracked/stored server-side).

// uploaded game files
export const FILE_LOCAL_CAP_BYTES = 10 * 1024 * 1024; // not logged in (IndexedDB)
export const FILE_SERVER_CAP_BYTES = 1024 * 1024 * 1024; // logged in (R2, also enforced server-side)

// save data
export const SAVE_LOCAL_CAP_BYTES = 1 * 1024 * 1024; // not logged in (IndexedDB, total)
export const SAVE_SERVER_CAP_BYTES = 100 * 1024 * 1024; // logged in (server, also enforced server-side)

// ── Upload security (4번) ─────────────────────────────────────────────────────
// A single uploaded file may be at most 100 MB, and at most 100 files may be
// added in one batch. Both are SERVER-FIXED (also enforced backend-side in
// functions/_lib/files.js) — there is no setter or settings UI.
export const UPLOAD_PER_FILE_MAX_BYTES = 100 * 1024 * 1024; // 100 MB per single file
export const UPLOAD_BATCH_MAX_FILES = 100; // at most 100 files per add

// Executable / script / web-shell extensions blocked at the source (원천 차단),
// even in the user's private vault. This is a BLOCKLIST: game containers
// (jar/jad/zip/kdf/skm) and ordinary docs/images remain allowed. Backend mirrors
// this set AND magic-number screens the bytes (functions/_lib/files.js) so a
// renamed executable (malware.exe → malware.png) is still refused.
export const BLOCKED_UPLOAD_EXT = new Set([
  // native executables / installers
  "exe", "msi", "com", "scr", "cpl", "dll", "so", "dylib", "apk", "app", "deb", "rpm", "dmg",
  // shell scripts
  "sh", "bash", "zsh", "ksh", "csh", "command",
  // windows scripts
  "bat", "cmd", "ps1", "psm1", "vbs", "vbe", "wsf", "wsh", "hta",
  // server-side web scripts / shells
  "php", "php3", "php4", "php5", "phtml", "jsp", "jspx", "asp", "aspx", "cgi", "pl", "py", "rb",
]);

// The extension of a filename, lowercased, without the dot ("" if none).
export function extOf(name: string): string {
  const i = name.lastIndexOf(".");
  return i >= 0 ? name.slice(i + 1).toLowerCase() : "";
}

// True when a filename's extension is an executable/script that must be blocked.
export function isBlockedUploadExt(name: string): boolean {
  return BLOCKED_UPLOAD_EXT.has(extOf(name));
}

// One-decimal byte formatter used everywhere usage is shown (e.g. "2.4 MB",
// "1.0 GB"). Always one decimal place from KB up.
export function fmtBytes1(n: number): string {
  if (!n || n < 0) return "0.0 B";
  if (n < 1024) return `${n.toFixed(0)} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} MB`;
  return `${(n / 1024 / 1024 / 1024).toFixed(1)} GB`;
}
