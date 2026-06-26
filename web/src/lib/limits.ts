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

// One-decimal byte formatter used everywhere usage is shown (e.g. "2.4 MB",
// "1.0 GB"). Always one decimal place from KB up.
export function fmtBytes1(n: number): string {
  if (!n || n < 0) return "0.0 B";
  if (n < 1024) return `${n.toFixed(0)} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} MB`;
  return `${(n / 1024 / 1024 / 1024).toFixed(1)} GB`;
}
