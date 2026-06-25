// Migrate device-local (IndexedDB) games into the logged-in user's PRIVATE
// server vault, then free the local copy.
//
// Rules (per the B안 directive):
//   • upload succeeds  → delete the local copy (it now lives on the server).
//   • per-user dedup (409 duplicate) → the file is already in THIS user's vault,
//     so just delete the local copy (no re-upload).
//   • quota exceeded (413) → STOP; keep the remaining games on the device
//     (never silently drop local data).
//   • any other error → keep the local copy and count it as failed.
//
// All network access goes through api.ts (files.*) — this module makes no fetch
// of its own, so the no-leak audit's "network only in api.ts" invariant holds.

import * as lib from "./library";
import { files as filesApi, ApiError } from "./api";

export interface MigrateResult {
  uploaded: number;
  deduped: number;
  failed: number;
  stopped: boolean; // true if halted on a full vault
  message?: string;
}

export async function migrateLocalToServer(onProgress?: (done: number, total: number) => void): Promise<MigrateResult> {
  const metas = await lib.listGames();
  let uploaded = 0;
  let deduped = 0;
  let failed = 0;
  let stopped = false;
  let message: string | undefined;

  for (let i = 0; i < metas.length; i++) {
    const g = await lib.getGame(metas[i].hash);
    if (!g) {
      failed++;
      onProgress?.(i + 1, metas.length);
      continue;
    }
    try {
      // g.hash is sha-256 of g.bytes (set at import time) — the server re-hashes
      // the same bytes and matches it, so content_hash stays honest.
      await filesApi.upload(g.name, g.kind, g.hash, g.bytes);
      await lib.deleteGame(g.hash); // success → free local
      uploaded++;
    } catch (e) {
      const err = e as ApiError;
      if (err.code === "duplicate") {
        await lib.deleteGame(g.hash); // already in this user's vault → just free local
        deduped++;
      } else if (err.code === "quota_exceeded") {
        stopped = true;
        message = "보관함 용량(1GB)이 가득 찼습니다. 남은 게임은 이 기기에 그대로 보존했습니다.";
        break; // keep the rest local
      } else {
        failed++;
        message = err.message; // keep local on any other failure
      }
    }
    onProgress?.(i + 1, metas.length);
  }

  return { uploaded, deduped, failed, stopped, message };
}

export function fmtBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} MB`;
  return `${(n / 1024 / 1024 / 1024).toFixed(2)} GB`;
}
