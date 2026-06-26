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
import { pushSaveToServer, deviceName } from "./saveSync";

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
      // Push the ROM's save to the server too (keyed by the SAME hash) and KEEP
      // the local save — so running the now-server ROM resumes the same save.
      await pushSaveToServer(g.hash, deviceName()).catch(() => {});
      await lib.deleteGame(g.hash); // free local ROM (save kept by default)
      uploaded++;
    } catch (e) {
      const err = e as ApiError;
      if (err.code === "duplicate") {
        await pushSaveToServer(g.hash, deviceName()).catch(() => {});
        await lib.deleteGame(g.hash); // already in vault → free local ROM (save kept)
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

// 1-decimal byte formatter (re-exported from the shared limits module).
export { fmtBytes1 as fmtBytes } from "./limits";
