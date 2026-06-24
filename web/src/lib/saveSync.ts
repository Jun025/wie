// Save synchronization: local autosave (IndexedDB, per game-hash) + opaque
// cloud sync under a user-chosen slot alias.
//
// GUARDRAIL: a cloud slot is identified by a user alias (slotLabel), never by
// the game's hash/filename/title. The hash↔slot mapping is stored locally only,
// so the server can never learn which game a save belongs to.

import * as lib from "./library";
import { saves as savesApi } from "./api";

export function bytesToB64(bytes: Uint8Array): string {
  let bin = "";
  const CHUNK = 0x8000;
  for (let i = 0; i < bytes.length; i += CHUNK) {
    bin += String.fromCharCode(...bytes.subarray(i, i + CHUNK));
  }
  return btoa(bin);
}

export function b64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

// Device-only autosave of an opaque snapshot.
export async function autosaveLocal(gameHash: string, blob: Uint8Array): Promise<void> {
  if (!blob || blob.length === 0) return;
  const existing = await lib.getLocalSave(gameHash);
  await lib.putLocalSave({
    hash: gameHash,
    blob: blob.slice().buffer as ArrayBuffer,
    updatedAt: Date.now(),
    slotLabel: existing?.slotLabel,
    serverId: existing?.serverId,
    syncedAt: existing?.syncedAt,
  });
}

export async function getLocalSnapshot(gameHash: string): Promise<Uint8Array | null> {
  const s = await lib.getLocalSave(gameHash);
  return s?.blob ? new Uint8Array(s.blob) : null;
}

// Push a game's local save to the cloud under a user alias (opaque payload only).
export async function pushToCloud(gameHash: string, slotLabel: string, deviceLabel: string) {
  const local = await lib.getLocalSave(gameHash);
  if (!local?.blob) throw new Error("업로드할 로컬 세이브가 없습니다");
  const payloadB64 = bytesToB64(new Uint8Array(local.blob));
  const res = await savesApi.upsert(slotLabel, deviceLabel || deviceName(), payloadB64);
  await lib.putLocalSave({ ...local, slotLabel, serverId: res.save.id, syncedAt: Date.now() });
  return res.save;
}

export async function listCloud() {
  return (await savesApi.list(false)).saves;
}

// Download a cloud slot's opaque payload and attach it to a chosen LOCAL game.
export async function attachCloudToGame(slotId: string, gameHash: string): Promise<void> {
  const res = await savesApi.get(slotId);
  const bytes = b64ToBytes(res.save.payload!);
  await lib.putLocalSave({
    hash: gameHash,
    blob: bytes.buffer as ArrayBuffer,
    updatedAt: res.save.updated_at,
    slotLabel: res.save.slot_label,
    serverId: res.save.id,
    syncedAt: Date.now(),
  });
}

export async function deleteCloud(slotId: string): Promise<void> {
  await savesApi.remove(slotId);
}

// A friendly, user-editable per-device alias (NOT a hardware identifier).
export function deviceName(): string {
  let n = localStorage.getItem("wie-device-name");
  if (!n) {
    n = "이 기기";
    localStorage.setItem("wie-device-name", n);
  }
  return n;
}

export function setDeviceName(name: string): void {
  localStorage.setItem("wie-device-name", name || "이 기기");
}
