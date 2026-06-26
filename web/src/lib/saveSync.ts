// Save synchronization, keyed by ROM CONTENT HASH.
//
// ┌─ BASELINE ──────────────────────────────────────────────────────────────────┐
// │ A save is identified by the ROM's content hash (sha-256), NOT by a storage   │
// │ location / path / upload-session — so the SAME ROM always maps to the SAME   │
// │ save, whether the ROM lives in IndexedDB (local) or R2 (server), and no      │
// │ matter its filename. NOT logged in → saves live ONLY in IndexedDB (never     │
// │ sent to the server). Logged in → the server is authoritative and the local   │
// │ store is a write-through cache. Server saves are scoped by user_id (per-     │
// │ owner); a not-logged-in save is never sent.                                  │
// └─────────────────────────────────────────────────────────────────────────────┘

import * as lib from "./library";
import { saves as savesApi } from "./api";
import { SAVE_LOCAL_CAP_BYTES } from "./limits";

export function bytesToB64(bytes: Uint8Array): string {
  let bin = "";
  const CHUNK = 0x8000;
  for (let i = 0; i < bytes.length; i += CHUNK) bin += String.fromCharCode(...bytes.subarray(i, i + CHUNK));
  return btoa(bin);
}

export function b64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

// Total bytes of all local saves (used for the not-logged-in 1MB cap).
export async function localSaveBytes(): Promise<number> {
  const all = await lib.listLocalSaves();
  return all.reduce((s, r) => s + (r.blob?.byteLength ?? 0), 0);
}

interface SyncOpts {
  loggedIn: boolean;
  deviceLabel?: string;
}

// Write the autosave locally (keyed by ROM hash) and, when logged in, write
// through to the server. Returns ok:false (without losing the old save) if a
// not-logged-in write would exceed the local 1 MB cap. Server push is best-effort
// (offline-safe) — it never blocks local persistence.
export async function persistSnapshot(romHash: string, blob: Uint8Array, opts: SyncOpts): Promise<{ ok: boolean; reason?: string }> {
  if (!blob || blob.length === 0) return { ok: true };
  const existing = await lib.getLocalSave(romHash);

  if (!opts.loggedIn) {
    const others = (await localSaveBytes()) - (existing?.blob?.byteLength ?? 0);
    if (others + blob.length > SAVE_LOCAL_CAP_BYTES) {
      return { ok: false, reason: "이 기기 세이브 한도(1MB)를 초과해 저장하지 못했습니다 (이전 세이브는 유지됨)" };
    }
  }

  await lib.putLocalSave({
    hash: romHash,
    blob: blob.slice().buffer as ArrayBuffer,
    updatedAt: Date.now(),
    slotLabel: existing?.slotLabel,
    serverId: existing?.serverId,
    syncedAt: existing?.syncedAt,
  });

  if (opts.loggedIn) {
    try {
      await pushSaveToServer(romHash, opts.deviceLabel || deviceName());
    } catch {
      /* offline / quota / transient — local copy is safe, will sync later */
    }
  }
  return { ok: true };
}

// Push the local save for a ROM to the server (write-through). Throws on failure
// (callers treat it as best-effort).
export async function pushSaveToServer(romHash: string, deviceLabel: string): Promise<void> {
  const local = await lib.getLocalSave(romHash);
  if (!local?.blob) return;
  const b64 = bytesToB64(new Uint8Array(local.blob));
  const res = await savesApi.upsert(romHash, b64, local.slotLabel || "자동저장", deviceLabel);
  await lib.putLocalSave({ ...local, serverId: res.save.id, syncedAt: Date.now() });
}

// Load the best snapshot for a ROM. Not logged in → local only. Logged in → take
// the server save if it is newer than the local cache (and cache it locally).
export async function loadSnapshot(romHash: string, opts: SyncOpts): Promise<Uint8Array | null> {
  const local = await lib.getLocalSave(romHash);
  if (!opts.loggedIn) return local?.blob ? new Uint8Array(local.blob) : null;

  let server = null;
  try {
    server = await savesApi.getByRom(romHash);
  } catch {
    /* offline / best-effort: fall back to local */
  }
  if (server?.payload && server.updated_at > (local?.updatedAt ?? 0)) {
    const bytes = b64ToBytes(server.payload);
    await lib.putLocalSave({ hash: romHash, blob: bytes.buffer as ArrayBuffer, updatedAt: server.updated_at, slotLabel: server.slot_label, serverId: server.id, syncedAt: Date.now() });
    return bytes;
  }
  return local?.blob ? new Uint8Array(local.blob) : null;
}

// On login: merge device-local saves into the server, keyed by ROM hash.
// Last-write-wins by timestamp; NEVER deletes the local copy, so no version is
// lost (conflict copies are preserved locally). Best-effort per save.
export async function mergeLocalSavesToServer(deviceLabel?: string): Promise<{ pushed: number; kept: number }> {
  const all = await lib.listLocalSaves();
  let pushed = 0;
  let kept = 0;
  for (const s of all) {
    if (!s.blob) continue;
    try {
      const server = await savesApi.getByRom(s.hash);
      if (!server || (s.updatedAt ?? 0) > (server.updated_at ?? 0)) {
        await pushSaveToServer(s.hash, deviceLabel || deviceName());
        pushed++;
      } else {
        if (server.payload) {
          await lib.putLocalSave({ hash: s.hash, blob: b64ToBytes(server.payload).buffer as ArrayBuffer, updatedAt: server.updated_at, slotLabel: server.slot_label, serverId: server.id, syncedAt: Date.now() });
        }
        kept++;
      }
    } catch {
      /* best-effort */
    }
  }
  return { pushed, kept };
}

export async function listCloud() {
  return (await savesApi.list()).saves;
}

export async function deleteCloud(slotId: string): Promise<void> {
  await savesApi.remove(slotId);
}

// A friendly, user-editable per-device alias (NOT a hardware identifier).
export function deviceName(): string {
  let n = localStorage.getItem("wie-device-name");
  if (!n) {
    n = recommendedDeviceName();
    localStorage.setItem("wie-device-name", n);
  }
  return n;
}

export function setDeviceName(name: string): void {
  localStorage.setItem("wie-device-name", name || recommendedDeviceName());
}

// Recommended default alias from the environment (browser/OS) — e.g. "Chrome on
// macOS". No game identity; the same browser/OS strings shown in 서비스 정보. Used
// to pre-fill the alias on a new device.
export function recommendedDeviceName(): string {
  const ua = typeof navigator !== "undefined" ? navigator.userAgent : "";
  const os = /iPhone/.test(ua) ? "iPhone" : /iPad/.test(ua) ? "iPad" : /Android/.test(ua) ? "Android" : /Mac OS X|Macintosh/.test(ua) ? "macOS" : /Windows/.test(ua) ? "Windows" : /Linux/.test(ua) ? "Linux" : "기타";
  const br = /Edg\//.test(ua) ? "Edge" : /CriOS|Chrome\//.test(ua) ? "Chrome" : /FxiOS|Firefox\//.test(ua) ? "Firefox" : /Safari\//.test(ua) ? "Safari" : "브라우저";
  return `${br} on ${os}`;
}
